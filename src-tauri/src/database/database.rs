use mysql::prelude::*;
use mysql::*;
use mysql::{PoolConstraints, OptsBuilder, SslOpts, ClientIdentity};
use std::time::Duration;
use tokio::sync::Mutex;
use log::{error, info, warn};

use std::sync::Arc;
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
}

impl Database {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(None),
            ssh_tunnel: Mutex::new(None),
        }
    }

    pub async fn connect(&self, config: ConnectionConfig) -> DbResult<()> {
        let mut pool_guard = self.pool.lock().await;

        if pool_guard.is_some() {
            return Err(DbError::AlreadyConnected);
        }

        let mut tunnel_guard = self.ssh_tunnel.lock().await;

        let (mysql_host, mysql_port) = if let Some(ref ssh_cfg) = config.ssh {
            let (handle, local_port) =
                open_ssh_tunnel(ssh_cfg, &config.host, config.port).await?;

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
            error!("Mutation failed: {} | query: {}", e, query);
            DbError::MySql(e)
        })?;
        info!("Mutation executed: {}", query);
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
        info!("Query executed: {}", paged_sql);

        let mut conn = self.get_live_conn().await?;

        let start = std::time::Instant::now();
        let (columns, mut rows) = fetch_rows(&mut conn, &paged_sql, limit as usize + 1)?;
        let elapsed = start.elapsed().as_millis();
        if elapsed > 500 {
            warn!("Slow query ({}ms): {}", elapsed, args.query);
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
struct SshClientHandler;

impl russh::client::Handler for SshClientHandler {
    type Error = russh::Error;

    // API for russh >= 0.50: &mut self, &ssh_key::PublicKey -> Result<bool, _>
    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
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
) -> DbResult<(Arc<tokio::sync::Mutex<russh::client::Handle<SshClientHandler>>>, u16)> {
    // ── Validate key file ────────────────────────────────────────────────
    if let Some(ref key_path) = ssh_cfg.private_key_path {
        let expanded = expand_tilde(key_path);
        if !std::path::Path::new(&expanded).exists() {
            return Err(SshError::KeyFileNotFound { path: expanded }.into());
        }
    }

    // ── Connect + SSH handshake ──────────────────────────────────────────
    info!("SSH connecting: {}@{}:{}", ssh_cfg.username, ssh_cfg.host, ssh_cfg.port);

    let config = Arc::new(russh::client::Config::default());

    let mut session = russh::client::connect(
        config,
        (ssh_cfg.host.as_str(), ssh_cfg.port),
        SshClientHandler,
    )
    .await
    .map_err(|e| {
        let msg = e.to_string().to_lowercase();
        if msg.contains("connection refused") || msg.contains("timed out")
            || msg.contains("no route") || msg.contains("network unreachable")
        {
            DbError::SshTunnel(SshError::HostUnreachable {
                host: ssh_cfg.host.clone(),
                port: ssh_cfg.port,
                reason: e.to_string(),
            })
        } else {
            DbError::SshTunnel(SshError::Io(e.to_string()))
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
                                error!("SSH direct-tcpip failed to {}:{}: {}", rh, remote_port, e);
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
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(name.to_string())
    } else {
        Err(DbError::InvalidIdentifier(name.to_string()))
    }
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
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
