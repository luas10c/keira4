use keyring_core::Entry;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use log::{error, info, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Keychain error: {0}")]
    Keyring(String),

    #[error("Store error: {0}")]
    Store(String),

    #[error("Connection '{0}' not found")]
    ConnectionNotFound(String),
}

pub type StoreResult<T> = Result<T, StoreError>;

const STORE_FILE: &str = "connections.json";
const STORE_KEY: &str = "connections";
const KEYRING_SERVICE: &str = "keira4";

/// SSH configuration saved in JSON — no password (it's stored in the keychain).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SavedSshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    /// Path to private key (optional — if None, use SSH agent)
    pub private_key_path: Option<String>,
    /// If true, there is an SSH password saved in the keychain for this connection.
    pub has_password: bool,
}

/// SSL configuration saved in JSON — only paths, no secrets.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SavedSslConfig {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub accept_invalid_certs: bool,
}

/// Connection saved in JSON — without any password.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SavedConnection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: Option<String>,
    pub ssh: Option<SavedSshConfig>,
    pub ssl: Option<SavedSslConfig>,
}

/// SSL payload sent by the frontend upon saving.
#[derive(Debug, Deserialize)]
pub struct SaveSslPayload {
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub accept_invalid_certs: bool,
}

/// SSH payload sent by the frontend upon saving (includes password in plain text).
#[derive(Debug, Deserialize)]
pub struct SaveSshPayload {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub private_key_path: Option<String>,
    /// Optional SSH password — only used without a private key.
    pub password: Option<String>,
}

/// Full payload sent by the frontend upon saving.
#[derive(Debug, Deserialize)]
pub struct SaveConnectionPayload {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: Option<String>,
    pub ssh: Option<SaveSshPayload>,
    pub ssl: Option<SaveSslPayload>,
}

// ─── Chaves de keychain ───────────────────────────────────────────────────────
// Separador __ em vez de : — o Windows Credential Manager não suporta :

fn keyring_mysql_key(name: &str) -> String { format!("{}__mysql", name) }
fn keyring_ssh_key(name: &str)   -> String { format!("{}__ssh",   name) }


/// Save a connection:
/// - data without passwords in tauri-plugin-store
/// - MySQL password in the keychain with key `<name>__mysql`
/// - SSH password (if any) in the keychain with key `<name>__ssh`
///
/// When updating a connection that previously had SSH and now does not,
/// The `__ssh` entry is removed to prevent it from becoming orphaned in the keychain.
pub fn save_connection(app: &AppHandle, payload: SaveConnectionPayload) -> StoreResult<()> {
    // 1. MySQL Password
    keyring_entry(&keyring_mysql_key(&payload.name))?
        .set_password(&payload.password)
        .map_err(|e| {
            error!("Failed to save MySQL password to keychain for '{}': {}", payload.name, e);
            StoreError::Keyring(e.to_string())
        })?;

    // 2. SSH configuration + SSH password
    let saved_ssh = if let Some(ssh) = payload.ssh {
        let has_password = match ssh.password.as_deref() {
            Some(pwd) if !pwd.is_empty() => {
                keyring_entry(&keyring_ssh_key(&payload.name))?
                    .set_password(pwd)
                    .map_err(|e| {
                        error!("Failed to save SSH password to keychain for '{}': {}", payload.name, e);
                        StoreError::Keyring(e.to_string())
                    })?;
                true
            }
            _ => {
                delete_keychain_entry(&keyring_ssh_key(&payload.name), &payload.name, "SSH");
                false
            }
        };
        Some(SavedSshConfig {
            host: ssh.host,
            port: ssh.port,
            username: ssh.username,
            private_key_path: ssh.private_key_path,
            has_password,
        })
    } else {
        // SSH removido — limpa entrada órfã no keychain
        delete_keychain_entry(&keyring_ssh_key(&payload.name), &payload.name, "SSH");
        None
    };

    // 3. Persists in the store
    let saved_ssl = payload.ssl.map(|s| SavedSslConfig {
        ca_cert: s.ca_cert,
        client_cert: s.client_cert,
        client_key: s.client_key,
        accept_invalid_certs: s.accept_invalid_certs,
    });


    let conn = SavedConnection {
        name: payload.name.clone(),
        host: payload.host,
        port: payload.port,
        username: payload.username,
        database: payload.database,
        ssh: saved_ssh,
        ssl: saved_ssl,
    };

    let mut connections = load_connections_from_store(app)?;

    if let Some(existing) = connections.iter_mut().find(|c| c.name == conn.name) {
        *existing = conn;
    } else {
        connections.push(conn);
    }

    persist_connections(app, &connections)?;
    info!("Connection '{}' saved", payload.name);
    Ok(())
}

pub fn list_connections(app: &AppHandle) -> StoreResult<Vec<SavedConnection>> {
    load_connections_from_store(app)
}

/// Deletes the connection and its `__mysql` and `__ssh` entries from the keychain.
pub fn delete_connection(app: &AppHandle, name: &str) -> StoreResult<()> {
    delete_keychain_entry(&keyring_mysql_key(name), name, "MySQL");
    delete_keychain_entry(&keyring_ssh_key(name), name, "SSH");

    let mut connections = load_connections_from_store(app)?;
    let before = connections.len();
    connections.retain(|c| c.name != name);

    if connections.len() == before {
        return Err(StoreError::ConnectionNotFound(name.to_string()));
    }

    persist_connections(app, &connections)?;
    info!("Connection '{}' deleted", name);
    Ok(())
}

/// Returns (SavedConnection, mysql_password, ssh_password) for internal use.
pub fn get_connection(
    app: &AppHandle,
    name: &str,
) -> StoreResult<(SavedConnection, String, Option<String>)> {
    let connections = load_connections_from_store(app)?;
    let conn = connections
        .into_iter()
        .find(|c| c.name == name)
        .ok_or_else(|| StoreError::ConnectionNotFound(name.to_string()))?;

    let mysql_password = get_mysql_password(name)?;

    let ssh_password = if conn.ssh.as_ref().map(|s| s.has_password).unwrap_or(false) {
        get_ssh_password(name)?
    } else {
        None
    };

    Ok((conn, mysql_password, ssh_password))
}

// ─── Helpers privados ─────────────────────────────────────────────────────────

fn get_mysql_password(name: &str) -> StoreResult<String> {
    keyring_entry(&keyring_mysql_key(name))?
        .get_password()
        .map_err(|e| {
            error!("Failed to get MySQL password from keychain for '{}': {}", name, e);
            StoreError::Keyring(e.to_string())
        })
}

fn get_ssh_password(name: &str) -> StoreResult<Option<String>> {
    match keyring_entry(&keyring_ssh_key(name))?.get_password() {
        Ok(pwd) => Ok(Some(pwd)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => {
            error!("Failed to get SSH password from keychain for '{}': {}", name, e);
            Err(StoreError::Keyring(e.to_string()))
        }
    }
}

fn keyring_entry(key: &str) -> StoreResult<Entry> {
    Entry::new(KEYRING_SERVICE, key).map_err(|e| StoreError::Keyring(e.to_string()))
}

/// Remove entrada do keychain. Match direto em keyring::Error::NoEntry —
/// não depende de substring da mensagem de erro.
fn delete_keychain_entry(key: &str, connection_name: &str, label: &str) {
    let entry = match keyring_entry(key) {
        Ok(e) => e,
        Err(e) => {
            error!("Failed to build keychain entry for '{}' ({}): {}", connection_name, label, e);
            return;
        }
    };
    match entry.delete_credential() {
        Ok(_) => {}
        Err(keyring_core::Error::NoEntry) => {
            warn!("No {} keychain entry for '{}', skipping", label, connection_name);
        }
        Err(e) => {
            error!("Failed to delete {} keychain entry for '{}': {}", label, connection_name, e);
        }
    }
}

fn load_connections_from_store(app: &AppHandle) -> StoreResult<Vec<SavedConnection>> {
    let store = app.store(STORE_FILE).map_err(|e| StoreError::Store(e.to_string()))?;

    let connections = match store.get(STORE_KEY) {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => vec![],
    };

    Ok(connections)
}

fn persist_connections(app: &AppHandle, connections: &[SavedConnection]) -> StoreResult<()> {
    let store = app.store(STORE_FILE).map_err(|e| {
        StoreError::Store(e.to_string())
    })?;

    let value = serde_json::to_value(connections)
        .map_err(|e| StoreError::Store(e.to_string()))?;
    store.set(STORE_KEY.to_string(), value);

    store.save().map_err(|e| {
        error!("Failed to persist connections to store: {}", e);
        StoreError::Store(e.to_string())
    })?;

    Ok(())
}
