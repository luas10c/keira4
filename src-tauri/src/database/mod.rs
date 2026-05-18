mod database;
mod store;

use tauri::{AppHandle, Emitter, State};

pub use database::{
    ConnectionConfig, Database, MutationResult, PendingSshHostKey, PendingSshHostKeyEvent,
    QueryArgs, QueryResult, SshConfig, SslConfig,
};
pub use store::{SaveConnectionPayload, SavedConnection};

const SSH_HOST_KEY_CONFIRMATION_REQUIRED_EVENT: &str = "ssh-host-key-confirmation-required";

pub struct AppState {
    pub database: Database,
}

#[tauri::command]
pub async fn connect(
    app: AppHandle,
    config: ConnectionConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    connect_with_notifications(&app, &state, config).await
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<(), String> {
    state.database.disconnect().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_query(
    args: QueryArgs,
    state: State<'_, AppState>,
) -> Result<QueryResult, String> {
    state
        .database
        .execute_query(args)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_mutation(
    query: String,
    state: State<'_, AppState>,
) -> Result<MutationResult, String> {
    state
        .database
        .execute_mutation(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_connected(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.database.is_connected().await)
}

#[tauri::command]
pub async fn get_databases(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .database
        .get_databases()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_tables(
    database: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    state
        .database
        .get_tables(&database)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_saved(
    name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (saved, mysql_password, ssh_password) =
        store::get_connection(&app, &name).map_err(|e| e.to_string())?;

    let ssh = saved.ssh.map(|s| SshConfig {
        host: s.host,
        port: s.port,
        username: s.username,
        private_key_path: s.private_key_path,
        password: ssh_password,
    });

    let ssl = saved.ssl.map(|s| SslConfig {
        ca_cert: s.ca_cert,
        client_cert: s.client_cert,
        client_key: s.client_key,
        accept_invalid_certs: s.accept_invalid_certs,
    });

    let config = ConnectionConfig {
        host: saved.host,
        port: saved.port,
        username: saved.username,
        password: mysql_password,
        database: saved.database,
        ssh,
        ssl,
    };

    connect_with_notifications(&app, &state, config).await
}

#[tauri::command]
pub fn get_pending_ssh_host_key_confirmation(
    state: State<'_, AppState>,
) -> Result<Option<PendingSshHostKey>, String> {
    Ok(state.database.take_pending_ssh_host_key())
}

#[tauri::command]
pub fn trust_pending_ssh_host_key(
    host: String,
    port: u16,
    state: State<'_, AppState>,
) -> Result<PendingSshHostKey, String> {
    state
        .database
        .trust_pending_ssh_host_key(&host, port)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_pending_ssh_host_key_confirmation(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.database.clear_pending_ssh_host_key();
    Ok(())
}

#[tauri::command]
pub fn save_connection(
    payload: SaveConnectionPayload,
    app: AppHandle,
) -> Result<(), String> {
    store::save_connection(&app, payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_connections(app: AppHandle) -> Result<Vec<SavedConnection>, String> {
    store::list_connections(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_connection(name: String, app: AppHandle) -> Result<(), String> {
    store::delete_connection(&app, &name).map_err(|e| e.to_string())
}

async fn connect_with_notifications(
    app: &AppHandle,
    state: &State<'_, AppState>,
    config: ConnectionConfig,
) -> Result<(), String> {
    match state.database.connect(config).await {
        Ok(()) => Ok(()),
        Err(error) => {
            if let Some(pending) = state.database.take_pending_ssh_host_key() {
                let payload = PendingSshHostKeyEvent {
                    host: pending.host,
                    port: pending.port,
                    fingerprint: pending.fingerprint,
                    known_hosts_path: pending.known_hosts_path,
                    reason: pending.reason,
                };

                app.emit(SSH_HOST_KEY_CONFIRMATION_REQUIRED_EVENT, payload)
                    .map_err(|emit_error| emit_error.to_string())?;
            }

            Err(error.to_string())
        }
    }
}
