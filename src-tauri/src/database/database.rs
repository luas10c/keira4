use mysql::prelude::*;
use mysql::*;
use mysql::{PoolConstraints, OptsBuilder, SslOpts, ClientIdentity};
use russh::keys::ssh_key::HashAlg;
use std::time::Duration;
use tokio::sync::Mutex;
use log::{error, info, warn};

use std::{fs, path::{Path, PathBuf}};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
use crate::error::{DbError, SshError};

pub type DbResult<T> = Result<T, DbError>;

/// SSH tunnel configuration.
/// The password (if used) is never serialized to disk — it comes from the keychain at runtime.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    /// Path to the private key (e.g., "~/.ssh/id_rsa").
    /// Support ~ — expanded internally without external dependency.
    pub private_key_path: Option<String>,
    /// SSH password — not serialized; comes from the keychain at runtime.
    #[serde(skip_serializing)]
    pub password: Option<String>,
}

/// SSL/TLS configuration for MySQL connections.
/// Cert paths are optional — omitting them uses the system CA store.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SslConfig {
    /// Path to CA certificate file (PEM). None = trust system CAs.
    pub ca_cert: Option<String>,
    /// Path to client certificate file (PEM). Required for mutual TLS.
    pub client_cert: Option<String>,
    /// Path to client private key file (PEM). Required for mutual TLS.
    pub client_key: Option<String>,
    /// If true, skip server certificate verification (insecure — dev only).
    pub accept_invalid_certs: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    /// Optional SSH tunnel. If present, opens the tunnel before connecting to MySQL.
    pub ssh: Option<SshConfig>,
    /// Optional SSL/TLS configuration. None = plain TCP (no SSL).
    pub ssl: Option<SslConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderDir {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryArgs {
    pub query: String,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default = "default_query_limit")]
    pub limit: u64,
    #[serde(default)]
    pub order_by: Option<String>,
    #[serde(default)]
    pub order_dir: Option<OrderDir>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
    pub cursor: Option<String>,
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
    pub limit: u64,
    #[serde(rename = "hasNext")]
    pub has_next: bool,
    #[serde(rename = "hasPrev")]
    pub has_prev: bool,
}

#[derive(Debug, Serialize)]
pub struct MutationResult {
    pub affected_rows: u64,
    pub last_insert_id: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingSshHostKey {
    pub host: String,
    pub port: u16,
    pub fingerprint: String,
    pub known_hosts_path: String,
    pub known_hosts_line: Option<usize>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingSshHostKeyEvent {
    pub host: String,
    pub port: u16,
    pub fingerprint: String,
    pub known_hosts_path: String,
    pub known_hosts_line: Option<usize>,
    pub reason: String,
}

#[derive(Debug, Clone)]
struct PendingSshHostKeyApproval {
    pending: PendingSshHostKey,
    public_key: russh::keys::ssh_key::PublicKey,
}

impl OrderDir {
    pub fn as_sql(&self) -> &'static str {
        match self {
            OrderDir::Asc => "ASC",
            OrderDir::Desc => "DESC",
        }
    }
}

const MAX_PAGE_SIZE: u64 = 1_000;

fn default_query_limit() -> u64 {
    10
}

struct SshTunnel {
    _session: Arc<tokio::sync::Mutex<russh::client::Handle<SshClientHandler>>>,
    #[allow(dead_code)]
    local_port: u16,
}

pub struct Database {
    pool: Mutex<Option<Pool>>,
    ssh_tunnel: Mutex<Option<SshTunnel>>,
    pending_ssh_host_key: Arc<StdMutex<Option<PendingSshHostKeyApproval>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
            ssh_tunnel: Mutex::new(None),
            pending_ssh_host_key: Arc::new(StdMutex::new(None)),
        }
    }

    pub fn take_pending_ssh_host_key(&self) -> Option<PendingSshHostKey> {
        self.pending_ssh_host_key
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|pending| pending.pending.clone()))
    }

    pub fn trust_pending_ssh_host_key(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
    ) -> DbResult<PendingSshHostKey> {
        let pending = self
            .pending_ssh_host_key
            .lock()
            .map_err(|_| DbError::SshTunnel(SshError::Io("pending SSH host key lock poisoned".into())))?
            .clone();

        let Some(pending) = pending else {
            return Err(DbError::SshTunnel(SshError::Io(
                "No pending SSH host key confirmation request".into(),
            )));
        };

        if pending.pending.host != host || pending.pending.port != port {
            return Err(DbError::SshTunnel(SshError::Io(format!(
                "Pending SSH host key does not match {}:{}",
                host, port
            ))));
        }

        if pending.pending.fingerprint != fingerprint {
            return Err(DbError::SshTunnel(SshError::Io(
                "Pending SSH host key fingerprint does not match confirmation payload".into(),
            )));
        }

        if let Some(line) = pending.pending.known_hosts_line {
            remove_known_hosts_line(Path::new(&pending.pending.known_hosts_path), line)?;
        }

        russh::keys::known_hosts::learn_known_hosts_path(
            &pending.pending.host,
            pending.pending.port,
            &pending.public_key,
            Path::new(&pending.pending.known_hosts_path),
        )
        .map_err(|error| DbError::SshTunnel(SshError::Io(format!(
            "Failed to persist SSH host key to {}: {}",
            pending.pending.known_hosts_path, error
        ))))?;

        self.clear_pending_ssh_host_key();
        Ok(pending.pending)
    }

    pub fn clear_pending_ssh_host_key(&self) {
        if let Ok(mut guard) = self.pending_ssh_host_key.lock() {
            *guard = None;
        }
    }

    pub async fn connect(&self, config: ConnectionConfig) -> DbResult<()> {
        self.clear_pending_ssh_host_key();

        let mut pool_guard = self.pool.lock().await;

        if pool_guard.is_some() {
            return Err(DbError::AlreadyConnected);
        }

        let mut tunnel_guard = self.ssh_tunnel.lock().await;

        let (mysql_host, mysql_port) = if let Some(ref ssh_cfg) = config.ssh {
            let (handle, local_port) =
                open_ssh_tunnel(
                    ssh_cfg,
                    &config.host,
                    config.port,
                    Arc::clone(&self.pending_ssh_host_key),
                )
                .await?;

            *tunnel_guard = Some(SshTunnel { _session: handle, local_port });

            ("127.0.0.1".to_string(), local_port)
        } else {
            (config.host.clone(), config.port)
        };

        let constraints = PoolConstraints::new(1, 4).unwrap();
        let pool_opts = PoolOpts::default().with_constraints(constraints);

        if tunnel_guard.is_some() {
            let lp = tunnel_guard.as_ref().unwrap().local_port;
            info!("Connecting to MySQL via SSH tunnel: 127.0.0.1:{} (remote: {}:{})",
                lp, config.host, config.port);
        } else {
            info!("Connecting to MySQL: {}:{}", config.host, config.port);
        }

        let mut opts = OptsBuilder::new()
            .ip_or_hostname(Some(mysql_host))
            .tcp_port(mysql_port)
            .user(Some(&config.username))
            .pass(Some(&config.password))
            .db_name(config.database.as_deref().filter(|s| !s.is_empty()))
            .tcp_connect_timeout(Some(Duration::from_secs(10)))
            .pool_opts(pool_opts);

        if let Some(ref ssl) = config.ssl {
          let mut ssl_opts = SslOpts::default();

          if let Some(ref ca_path) = ssl.ca_cert {
              ssl_opts = ssl_opts.with_root_cert_path(Some(std::path::PathBuf::from(ca_path)));
          }

          if let (Some(ref cert), Some(ref key)) = (&ssl.client_cert, &ssl.client_key) {
              let identity = ClientIdentity::new(
                  std::path::PathBuf::from(cert),
                  std::path::PathBuf::from(key),
              );

              ssl_opts = ssl_opts.with_client_identity(Some(identity));
          }

          if ssl.accept_invalid_certs {
              warn!("SSL: accept_invalid_certs=true — server certificate will NOT be verified");
              ssl_opts = ssl_opts
                  .with_danger_accept_invalid_certs(true)
                  .with_danger_skip_domain_validation(true);
          }

          info!(
              "SSL enabled for {}:{} (verify={})",
              config.host,
              config.port,
              !ssl.accept_invalid_certs
          );

          opts = opts.ssl_opts(Some(ssl_opts));
        }

        // Tunnel rollback on any MySQL failure to maintain consistent state.
        let pool = Pool::new(opts).map_err(|e| {
            error!("MySQL pool creation failed ({}:{}) — {}: rolling back SSH tunnel",
                config.host, config.port, e);
            *tunnel_guard = None;
            e
        })?;

        let mut conn = pool.get_conn().map_err(|e| {
            error!("MySQL initial connection failed ({}:{}) — {}: rolling back SSH tunnel",
                config.host, config.port, e);
            *tunnel_guard = None;
            e
        })?;

        conn.query_drop("SELECT 1").map_err(|e| {
            error!("MySQL ping failed ({}:{}) — {}: rolling back SSH tunnel",
                config.host, config.port, e);
            *tunnel_guard = None;
            e
        })?;

        *pool_guard = Some(pool);
        info!("Connected to MySQL {}:{} (user: {})", config.host, config.port, config.username);
        Ok(())
    }

    pub async fn disconnect(&self) -> DbResult<()> {
        let mut pool_guard = self.pool.lock().await;
        if pool_guard.is_none() {
            return Err(DbError::NotConnected);
        }
        *pool_guard = None;

        let mut tunnel_guard = self.ssh_tunnel.lock().await;
        if tunnel_guard.is_some() {
            *tunnel_guard = None;
            info!("SSH tunnel closed");
        }

        info!("Disconnected from database");
        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.pool.lock().await.is_some()
    }

    async fn get_conn(&self) -> DbResult<PooledConn> {
        let pool = {
            let guard = self.pool.lock().await;
            guard.as_ref().ok_or(DbError::NotConnected)?.clone()
        };
        let conn = pool.get_conn()?;
        Ok(conn)
    }

    pub async fn execute_mutation(&self, query: &str) -> DbResult<MutationResult> {
        let mut conn = self.get_live_conn().await?;
        conn.query_drop(query).map_err(|e| {
            error!("Mutation failed: {} | query_bytes={}", e, query.len());
            DbError::MySql(e)
        })?;
        info!("Mutation executed (query_bytes={})", query.len());
        Ok(MutationResult {
            affected_rows: conn.affected_rows(),
            last_insert_id: conn.last_insert_id(),
        })
    }

    pub async fn execute_query(&self, args: QueryArgs) -> DbResult<QueryResult> {
        let requested_limit = if args.limit == 0 { default_query_limit() } else { args.limit };
        let limit = requested_limit.min(MAX_PAGE_SIZE).max(1);

        let base_query = args.query.trim().trim_end_matches(';');
        let order_by = sanitize_identifier(args.order_by.as_deref().unwrap_or("id"))?;
        let direction = args.order_dir.unwrap_or(OrderDir::Asc);
        let comparator = match direction {
            OrderDir::Asc => ">",
            OrderDir::Desc => "<",
        };
        let cursor_clause = args
            .cursor
            .as_deref()
            .map(|cursor| {
                format!(
                    " WHERE `{}` {} '{}'",
                    order_by,
                    comparator,
                    escape_sql_literal(cursor)
                )
            })
            .unwrap_or_default();

        let paged_sql = format!(
            "SELECT * FROM ({}) AS _keira_cursor{} ORDER BY `{}` {} LIMIT {}",
            base_query,
            cursor_clause,
            order_by,
            direction.as_sql(),
            limit + 1
        );
        info!("Query executed (query_bytes={})", paged_sql.len());

        let mut conn = self.get_live_conn().await?;

        let start = std::time::Instant::now();
        let (columns, mut rows) = fetch_rows(&mut conn, &paged_sql, limit as usize + 1)?;
        let elapsed = start.elapsed().as_millis();
        if elapsed > 500 {
            warn!("Slow query ({}ms, query_bytes={})", elapsed, args.query.len());
        }

        let has_next = rows.len() > limit as usize;
        if has_next {
            rows.pop();
        }

        let order_column_index = columns
            .iter()
            .position(|column| column == &order_by)
            .ok_or_else(|| DbError::MissingOrderByColumn(order_by.clone()))?;
        let next_cursor = if has_next {
            rows
                .last()
                .and_then(|row| row.get(order_column_index))
                .and_then(|value| value.clone())
        } else {
            None
        };

        Ok(QueryResult {
            columns,
            rows,
            cursor: args.cursor,
            next_cursor,
            limit,
            has_next,
            has_prev: false,
        })
    }

    // ─── Metadata ────────────────────────────────────────────────────────

    pub async fn get_databases(&self) -> DbResult<Vec<String>> {
        let mut conn = self.get_live_conn().await?;
        let dbs: Vec<String> = conn.query("SHOW DATABASES")?;
        Ok(dbs)
    }

    pub async fn get_tables(&self, database: &str) -> DbResult<Vec<String>> {
        let mut conn = self.get_live_conn().await?;
        let database = sanitize_identifier(database)?;
        let tables: Vec<String> = conn.query(format!("SHOW TABLES FROM `{}`", database))?;
        Ok(tables)
    }

    async fn get_live_conn(&self) -> DbResult<PooledConn> {
        let mut conn = self.get_conn().await?;
        if conn.query_drop("SELECT 1").is_err() {
            warn!("Dead connection detected, reconnecting...");
            conn = self.get_conn().await?;
        }
        Ok(conn)
    }
}

// ─── SSH Tunnel ───────────────────────────────────────────────────────────────

/// Expand `~` and `~/path` to the user's home directory.
/// Use HOME (Unix) or USERPROFILE (Windows) — no external dependencies.
/// Paths that do not begin with `~` are returned untouched.
fn expand_tilde(path: &str) -> String {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"));

    let Some(home) = home else {
        return path.to_owned();
    };

    if path == "~" {
        return home.to_string_lossy().into_owned();
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return std::path::Path::new(&home)
            .join(rest)
            .to_string_lossy()
            .into_owned();
    }

    path.to_owned()
}

/// russh client handler — accepts all host keys (first-connect trust model).
struct SshClientHandler {
    host: String,
    port: u16,
    known_hosts_path: PathBuf,
    pending_host_key: Arc<StdMutex<Option<PendingSshHostKeyApproval>>>,
}

impl russh::client::Handler for SshClientHandler {
    type Error = DbError;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        validate_server_key_against_known_hosts_path(
            &self.host,
            self.port,
            server_public_key,
            &self.known_hosts_path,
            &self.pending_host_key,
        )?;

        Ok(true)
    }
}

/// Opens a local → remote SSH tunnel using russh (pure Rust, no ssh binary required).
/// Returns (Handle, local_port). Handle must stay alive for the tunnel to remain open.
///
/// Spawns a Tokio task that accepts TCP connections on the local port and
/// forwards each one through a russh direct-tcpip channel to the remote host.
async fn open_ssh_tunnel(
    ssh_cfg: &SshConfig,
    remote_host: &str,
    remote_port: u16,
    pending_host_key: Arc<StdMutex<Option<PendingSshHostKeyApproval>>>,
) -> DbResult<(Arc<tokio::sync::Mutex<russh::client::Handle<SshClientHandler>>>, u16)> {
    // ── Validate key file ────────────────────────────────────────────────
    if let Some(ref key_path) = ssh_cfg.private_key_path {
        let expanded = expand_tilde(key_path);
        if !std::path::Path::new(&expanded).exists() {
            return Err(SshError::KeyFileNotFound { path: expanded }.into());
        }
    }

    let known_hosts_path = default_known_hosts_path()?;

    // ── Connect + SSH handshake ──────────────────────────────────────────
    info!("SSH connecting: {}@{}:{}", ssh_cfg.username, ssh_cfg.host, ssh_cfg.port);

    let config = Arc::new(russh::client::Config::default());

    let mut session = russh::client::connect(
        config,
        (ssh_cfg.host.as_str(), ssh_cfg.port),
        SshClientHandler {
            host: ssh_cfg.host.clone(),
            port: ssh_cfg.port,
            known_hosts_path: known_hosts_path.clone(),
            pending_host_key,
        },
    )
    .await
    .map_err(|e| {
        match e {
            DbError::SshTunnel(SshError::HostKeyMismatch { .. }) => e,
            other => {
                let msg = other.to_string().to_lowercase();
                if msg.contains("connection refused") || msg.contains("timed out")
                    || msg.contains("no route") || msg.contains("network unreachable")
                {
                    DbError::SshTunnel(SshError::HostUnreachable {
                        host: ssh_cfg.host.clone(),
                        port: ssh_cfg.port,
                        reason: other.to_string(),
                    })
                } else {
                    other
                }
            }
        }
    })?;

    // ── Authenticate ─────────────────────────────────────────────────────
    if let Some(ref key_path) = ssh_cfg.private_key_path {
        let expanded = expand_tilde(key_path);
        let key = russh::keys::load_secret_key(&expanded, ssh_cfg.password.as_deref())
            .map_err(|e| {
                error!("SSH Failed to load key '{}': {}", expanded, e);
                DbError::SshTunnel(SshError::KeyFileNotFound { path: expanded.clone() })
            })?;

        info!("SSH Key loaded: {} algorithm={:?}", expanded, key.algorithm());

        // RSA keys require an explicit hash algorithm — modern OpenSSH servers
        // reject RSA without SHA2. Use SHA2-256 for RSA, None for Ed25519/ECDSA.
        let hash_alg = match key.algorithm() {
            russh::keys::Algorithm::Rsa { .. } => {
                Some(russh::keys::HashAlg::Sha256)
            }
            _ => None,
        };
        let key_with_hash = russh::keys::key::PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg);

        let auth_result = session
            .authenticate_publickey(&ssh_cfg.username, key_with_hash)
            .await
            .map_err(|e| {
                error!("SSH authenticate_publickey error: {}", e);
                DbError::SshTunnel(SshError::AuthFailed {
                    user: ssh_cfg.username.clone(),
                    host: ssh_cfg.host.clone(),
                    port: ssh_cfg.port,
                })
            })?;

        info!("SSH publickey auth result: {:?}", auth_result);

        if !auth_result.success() {
            return Err(DbError::SshTunnel(SshError::AuthFailed {
                user: ssh_cfg.username.clone(),
                host: ssh_cfg.host.clone(),
                port: ssh_cfg.port,
            }));
        }
    } else if let Some(ref password) = ssh_cfg.password {
        let auth_result: russh::client::AuthResult = session
            .authenticate_password(&ssh_cfg.username, password)
            .await
            .map_err(|_| DbError::SshTunnel(SshError::AuthFailed {
                user: ssh_cfg.username.clone(),
                host: ssh_cfg.host.clone(),
                port: ssh_cfg.port,
            }))?;

        if !auth_result.success() {
            return Err(DbError::SshTunnel(SshError::AuthFailed {
                user: ssh_cfg.username.clone(),
                host: ssh_cfg.host.clone(),
                port: ssh_cfg.port,
            }));
        }
    } else {
        return Err(DbError::SshTunnel(SshError::Io(
            "No authentication method — supply private_key_path or password".into(),
        )));
    }

    info!("SSH authenticated: {}@{}:{}", ssh_cfg.username, ssh_cfg.host, ssh_cfg.port);

    // ── Bind local port ──────────────────────────────────────────────────
    let listener = TcpListener::bind("127.0.0.1:0").await
        .map_err(|e| DbError::SshTunnel(SshError::Io(format!("Failed to bind local port: {}", e))))?;
    let local_port = listener.local_addr()
        .map_err(|e| DbError::SshTunnel(SshError::Io(e.to_string())))?
        .port();

    {
        let verify_channel = session
            .channel_open_direct_tcpip(remote_host, remote_port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| DbError::SshTunnel(SshError::PortForwardFailed {
                local_port,
                remote_host: remote_host.to_owned(),
                remote_port,
                reason: e.to_string(),
            }))?;

        let _ = verify_channel.close().await;
    }

    // ── Spawn forwarding task ────────────────────────────────────────────
    // Arc<Mutex> lets the spawned task share the session handle safely.
    let session_arc = Arc::new(tokio::sync::Mutex::new(session));
    let session_task = Arc::clone(&session_arc);
    let rhost = remote_host.to_string();

    tokio::spawn(async move {
        info!("SSH Forwarding task started — listening on 127.0.0.1:{}", local_port);
        loop {
            match listener.accept().await {
                Ok((mut local_stream, peer)) => {
                    info!("SSH Accepted connection from {}", peer);
                    let sess = Arc::clone(&session_task);
                    let rh = rhost.clone();
                    tokio::spawn(async move {
                        info!("SSH Opening direct-tcpip channel to {}:{}", rh, remote_port);
                        let ch = {
                            let guard = sess.lock().await;
                            guard.channel_open_direct_tcpip(&rh, remote_port as u32, "127.0.0.1", 0).await
                        };
                        match ch {
                            Ok(channel) => {
                                info!("SSH Channel opened, proxying data");
                                let mut remote_stream = channel.into_stream();
                                match tokio::io::copy_bidirectional(
                                    &mut local_stream, &mut remote_stream,
                                ).await {
                                    Ok((tx, rx)) => info!("SSH Connection closed (tx={} rx={})", tx, rx),
                                    Err(e) => info!("SSH Connection closed: {}", e),
                                }
                            }
                            Err(e) => {
                                let forward_error = SshError::PortForwardFailed {
                                    local_port,
                                    remote_host: rh.clone(),
                                    remote_port,
                                    reason: e.to_string(),
                                };
                                error!("{}", forward_error);
                                let _ = tokio::io::AsyncWriteExt::shutdown(&mut local_stream).await;
                            }
                        }
                    });
                }
                Err(e) => {
                    info!("SSH Tunnel listener closed: {}", e);
                    break;
                }
            }
        }
    });

    info!("SSH tunnel active: 127.0.0.1:{} -> {}:{}", local_port, remote_host, remote_port);

    // Yield to let the forwarding task start before MySQL pool tries to connect.
    // Without this the pool connects before the task is scheduled and the
    // connection attempt to 127.0.0.1:PORT is refused.
    tokio::task::yield_now().await;

    Ok((session_arc, local_port))
}

fn default_known_hosts_path() -> DbResult<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join(".ssh").join("known_hosts"))
        .ok_or_else(|| {
            DbError::SshTunnel(SshError::Io(
                "Could not determine home directory for SSH known_hosts".into(),
            ))
        })
}

fn remove_known_hosts_line(path: &Path, line_to_remove: usize) -> DbResult<()> {
    if line_to_remove == 0 || !path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(path).map_err(|error| {
        DbError::SshTunnel(SshError::Io(format!(
            "Failed to read SSH known_hosts at {}: {}",
            path.display(),
            error
        )))
    })?;

    let mut rewritten = String::new();
    for (index, line) in content.lines().enumerate() {
        if index + 1 != line_to_remove {
            rewritten.push_str(line);
            rewritten.push('\n');
        }
    }

    fs::write(path, rewritten).map_err(|error| {
        DbError::SshTunnel(SshError::Io(format!(
            "Failed to update SSH known_hosts at {}: {}",
            path.display(),
            error
        )))
    })
}

fn validate_server_key_against_known_hosts_path(
    host: &str,
    port: u16,
    server_public_key: &russh::keys::ssh_key::PublicKey,
    known_hosts_path: &Path,
    pending_host_key: &Arc<StdMutex<Option<PendingSshHostKeyApproval>>>,
) -> DbResult<()> {
    let fingerprint = server_public_key.fingerprint(HashAlg::Sha256).to_string();
    let known_hosts_path_string = known_hosts_path.display().to_string();

    match russh::keys::check_known_hosts_path(host, port, server_public_key, known_hosts_path) {
        Ok(true) => {
            if let Ok(mut guard) = pending_host_key.lock() {
                *guard = None;
            }
            info!(
                "SSH host key verified for {}:{} ({}) via {}",
                host,
                port,
                fingerprint,
                known_hosts_path.display()
            );
            Ok(())
        }
        Ok(false) => {
            let pending = PendingSshHostKey {
                host: host.to_owned(),
                port,
                fingerprint,
                known_hosts_path: known_hosts_path_string,
                known_hosts_line: None,
                reason: "server key is not present in known_hosts".into(),
            };
            store_pending_host_key(pending_host_key, &pending, server_public_key)?;
            Err(DbError::SshTunnel(SshError::HostKeyMismatch {
                host: pending.host,
                port: pending.port,
                fingerprint: pending.fingerprint,
                known_hosts_path: pending.known_hosts_path,
                reason: pending.reason,
            }))
        }
        Err(russh::keys::Error::KeyChanged { line }) => {
            let pending = PendingSshHostKey {
                host: host.to_owned(),
                port,
                fingerprint,
                known_hosts_path: known_hosts_path_string,
                known_hosts_line: Some(line),
                reason: format!("known_hosts entry on line {} does not match the presented key", line),
            };
            store_pending_host_key(pending_host_key, &pending, server_public_key)?;
            Err(DbError::SshTunnel(SshError::HostKeyMismatch {
                host: pending.host,
                port: pending.port,
                fingerprint: pending.fingerprint,
                known_hosts_path: pending.known_hosts_path,
                reason: pending.reason,
            }))
        }
        Err(error) => Err(DbError::SshTunnel(SshError::Io(format!(
            "Failed to validate SSH host key using {}: {}",
            known_hosts_path.display(),
            error
        )))),
    }
}

fn store_pending_host_key(
    pending_host_key: &Arc<StdMutex<Option<PendingSshHostKeyApproval>>>,
    pending: &PendingSshHostKey,
    server_public_key: &russh::keys::ssh_key::PublicKey,
) -> DbResult<()> {
    let mut guard = pending_host_key
        .lock()
        .map_err(|_| DbError::SshTunnel(SshError::Io("pending SSH host key lock poisoned".into())))?;
    *guard = Some(PendingSshHostKeyApproval {
        pending: pending.clone(),
        public_key: server_public_key.clone(),
    });
    Ok(())
}

// ─── Utilitários ─────────────────────────────────────────────────────────────

fn fetch_rows(
    conn: &mut PooledConn,
    query: &str,
    capacity: usize,
) -> DbResult<(Vec<String>, Vec<Vec<Option<String>>>)> {
    let result = conn.query_iter(query)?;

    let cols: Vec<Column> = result.columns().as_ref().to_vec();
    let columns: Vec<String> = cols.iter().map(|c| c.name_str().to_string()).collect();
    let col_count = columns.len();

    let mut rows: Vec<Vec<Option<String>>> = Vec::with_capacity(capacity);

    for row_result in result {
        let row: Row = row_result?;
        let mut values: Vec<Option<String>> = Vec::with_capacity(col_count);

        for i in 0..col_count {
            let val = match row.get_opt::<Value, usize>(i) {
                Some(Ok(Value::NULL)) | None => None,
                Some(Ok(v)) => Some(value_to_string(v)),
                Some(Err(e)) => return Err(DbError::ValueConversion {
                    column: columns[i].clone(),
                    details: e.to_string(),
                }),
            };
            values.push(val);
        }
        rows.push(values);
    }

    Ok((columns, rows))
}

fn sanitize_identifier(name: &str) -> DbResult<String> {
    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(name.to_string())
    } else {
        Err(DbError::InvalidIdentifier(name.to_string()))
    }
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

fn value_to_string(v: Value) -> String {
    match v {
        Value::Bytes(b) => String::from_utf8(b)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()),
        Value::Int(i) => i.to_string(),
        Value::UInt(u) => u.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Double(d) => d.to_string(),
        Value::Date(y, m, d, h, min, s, ms) => {
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}", y, m, d, h, min, s, ms)
        }
        Value::Time(neg, days, h, min, s, ms) => {
            let sign = if neg { "-" } else { "" };
            format!("{}{}:{:02}:{:02}.{:06}", sign, days * 24 + h as u32, min, s, ms)
        }
        Value::NULL => String::from("NULL"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        escape_sql_literal, sanitize_identifier, validate_server_key_against_known_hosts_path,
        PendingSshHostKeyApproval,
    };
    use crate::error::{DbError, SshError};
    use russh::keys::parse_public_key_base64;
    use std::{fs, path::PathBuf, sync::{Arc, Mutex as StdMutex}, time::{SystemTime, UNIX_EPOCH}};

    fn unique_test_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("keira4-known-hosts-test-{nanos}"))
    }

    fn pending_store() -> Arc<StdMutex<Option<PendingSshHostKeyApproval>>> {
        Arc::new(StdMutex::new(None))
    }

    #[test]
    fn accepts_safe_sql_identifiers() {
        assert_eq!(sanitize_identifier("db_01").unwrap(), "db_01");
        assert_eq!(sanitize_identifier("users").unwrap(), "users");
    }

    #[test]
    fn rejects_unsafe_sql_identifiers() {
        assert!(sanitize_identifier("db-name").is_err());
        assert!(sanitize_identifier("db name").is_err());
        assert!(sanitize_identifier("").is_err());
        assert!(sanitize_identifier("db`; DROP TABLE users; --").is_err());
    }

    #[test]
    fn escapes_sql_literals_with_standard_quote_escaping() {
        assert_eq!(escape_sql_literal("O'Reilly"), "O''Reilly");
        assert_eq!(escape_sql_literal("back\\slash"), "back\\slash");
    }

    #[test]
    fn accepts_known_host_key_from_known_hosts_file() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("test directory should be created");
        let path = dir.join("known_hosts");
        fs::write(
            &path,
            "[localhost]:13265 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJdD7y3aLq454yWBdwLWbieU1ebz9/cu7/QEXn9OIeZJ\n",
        )
        .expect("known_hosts should be written");
        let key = parse_public_key_base64(
            "AAAAC3NzaC1lZDI1NTE5AAAAIJdD7y3aLq454yWBdwLWbieU1ebz9/cu7/QEXn9OIeZJ",
        )
        .expect("public key should parse");
        let pending = pending_store();

        validate_server_key_against_known_hosts_path("localhost", 13265, &key, &path, &pending)
            .expect("known host key should be accepted");

        fs::remove_dir_all(&dir).expect("test directory should be removed");
    }

    #[test]
    fn rejects_unknown_host_key_when_not_in_known_hosts_file() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("test directory should be created");
        let path = dir.join("known_hosts");
        fs::write(&path, "").expect("known_hosts should be written");
        let key = parse_public_key_base64(
            "AAAAC3NzaC1lZDI1NTE5AAAAIJdD7y3aLq454yWBdwLWbieU1ebz9/cu7/QEXn9OIeZJ",
        )
        .expect("public key should parse");
        let pending = pending_store();

        let error = validate_server_key_against_known_hosts_path(
            "localhost",
            13265,
            &key,
            &path,
            &pending,
        )
            .expect_err("unknown host key should be rejected");

        match error {
            DbError::SshTunnel(SshError::HostKeyMismatch { fingerprint, .. }) => {
                assert!(fingerprint.starts_with("SHA256:"));
            }
            other => panic!("unexpected error: {other}"),
        }
        assert!(pending.lock().unwrap().as_ref().unwrap().pending.known_hosts_line.is_none());

        fs::remove_dir_all(&dir).expect("test directory should be removed");
    }

    #[test]
    fn rejects_changed_host_key_when_known_hosts_entry_differs() {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).expect("test directory should be created");
        let path = dir.join("known_hosts");
        fs::write(
            &path,
            "[localhost]:13265 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIA6rWI3G1sz07DnfFlrouTcysQlj2P+jpNSOEWD9OJ3X\n",
        )
        .expect("known_hosts should be written");
        let presented_key = parse_public_key_base64(
            "AAAAC3NzaC1lZDI1NTE5AAAAIJdD7y3aLq454yWBdwLWbieU1ebz9/cu7/QEXn9OIeZJ",
        )
        .expect("public key should parse");
        let pending = pending_store();

        let error = validate_server_key_against_known_hosts_path(
            "localhost",
            13265,
            &presented_key,
            &path,
            &pending,
        )
        .expect_err("changed host key should be rejected");

        match error {
            DbError::SshTunnel(SshError::HostKeyMismatch { reason, .. }) => {
                assert!(reason.contains("line"));
            }
            other => panic!("unexpected error: {other}"),
        }
        assert_eq!(
            pending.lock().unwrap().as_ref().unwrap().pending.known_hosts_line,
            Some(1)
        );

        fs::remove_dir_all(&dir).expect("test directory should be removed");
    }
}
