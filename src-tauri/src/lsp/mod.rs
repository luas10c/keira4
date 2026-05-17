use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspSessionConfig {
    pub dialect: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub ssl: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspStatus {
    pub phase: String,
    pub running: bool,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub configured: bool,
}

#[derive(Debug, Clone, Default)]
struct LspRuntime {
    pub configured_session: Option<LspSessionConfig>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub running: bool,
}

#[derive(Default)]
pub struct LspRuntimeState(Mutex<LspRuntime>);

pub fn configure_session(
    state: &LspRuntimeState,
    session: LspSessionConfig,
) -> Result<LspStatus, String> {
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "lsp runtime state lock poisoned".to_owned())?;
    runtime.configured_session = Some(session);
    Ok(status_from_runtime(&runtime))
}

pub fn start(
    state: &LspRuntimeState,
    command: String,
    args: Option<Vec<String>>,
) -> Result<LspStatus, String> {
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "lsp runtime state lock poisoned".to_owned())?;

    runtime.command = Some(command);
    runtime.args = args.unwrap_or_default();
    runtime.running = true;

    Ok(status_from_runtime(&runtime))
}

pub fn stop(state: &LspRuntimeState) -> Result<LspStatus, String> {
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "lsp runtime state lock poisoned".to_owned())?;
    runtime.running = false;
    Ok(status_from_runtime(&runtime))
}

pub fn status(state: &LspRuntimeState) -> Result<LspStatus, String> {
    let runtime = state
        .0
        .lock()
        .map_err(|_| "lsp runtime state lock poisoned".to_owned())?;
    Ok(status_from_runtime(&runtime))
}

fn status_from_runtime(runtime: &LspRuntime) -> LspStatus {
    let phase = if runtime.running {
        "running"
    } else if runtime.configured_session.is_some() {
        "configured"
    } else {
        "idle"
    };

    LspStatus {
        phase: phase.to_owned(),
        running: runtime.running,
        command: runtime.command.clone(),
        args: runtime.args.clone(),
        configured: runtime.configured_session.is_some(),
    }
}
