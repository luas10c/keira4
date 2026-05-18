// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod database;
mod extension;
mod error;
mod lsp;

use config::{AppConfig, AppConfigState};
use extension::{
    ExtensionRuntimeState, ExtensionSearchResult, InstalledExtension, InstalledTheme, LoadedExtension,
    SearchExtensionsFilter,
    ThemeColors,
};
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
fn lsp_configure_session(
    lsp: tauri::State<'_, lsp::LspRuntimeState>,
    session: lsp::LspSessionConfig,
) -> Result<lsp::LspStatus, String> {
    lsp::configure_session(&lsp, session)
}

#[tauri::command]
fn lsp_start(
    lsp: tauri::State<'_, lsp::LspRuntimeState>,
    command: String,
    args: Option<Vec<String>>,
) -> Result<lsp::LspStatus, String> {
    lsp::start(&lsp, command, args)
}

#[tauri::command]
fn lsp_stop(lsp: tauri::State<'_, lsp::LspRuntimeState>) -> Result<lsp::LspStatus, String> {
    lsp::stop(&lsp)
}

#[tauri::command]
fn lsp_status(lsp: tauri::State<'_, lsp::LspRuntimeState>) -> Result<lsp::LspStatus, String> {
    lsp::status(&lsp)
}

#[tauri::command]
fn load_config(
    app: tauri::AppHandle,
    config: tauri::State<'_, AppConfigState>,
) -> Result<AppConfig, String> {
    let config_path = config::config_path(&app).map_err(|error| error.to_string())?;
    let latest = config::load_from_path(&config_path).map_err(|error| error.to_string())?;

    let mut state = config
        .0
        .lock()
        .map_err(|_| "config state lock poisoned".to_owned())?;
    *state = latest.clone();

    Ok(latest)
}

#[tauri::command]
fn patch_config(
    app: tauri::AppHandle,
    config: tauri::State<'_, AppConfigState>,
    patches: Vec<config::ConfigPatch>,
) -> Result<AppConfig, String> {
    let config_path =
        config::config_path(&app).map_err(|error| error.to_string())?;
    let updated = config::patch_from_path(&config_path, &patches)
        .map_err(|error| error.to_string())?;

    let mut state = config
        .0
        .lock()
        .map_err(|_| "config state lock poisoned".to_owned())?;
    *state = updated.clone();

    Ok(updated)
}

#[tauri::command]
fn set_config_value(
    app: tauri::AppHandle,
    config: tauri::State<'_, AppConfigState>,
    key: String,
    value: serde_json::Value,
) -> Result<AppConfig, String> {
    patch_config(app, config, vec![config::ConfigPatch { key, value }])
}

#[tauri::command]
fn list_extensions(app: tauri::AppHandle) -> Result<Vec<InstalledExtension>, String> {
    let runtime = app.state::<ExtensionRuntimeState>();
    extension::list_extensions(&app, &runtime).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_extensions(
    app: tauri::AppHandle,
    runtime: tauri::State<'_, ExtensionRuntimeState>,
) -> Result<Vec<LoadedExtension>, String> {
    extension::load_extensions_into_state(&app, &runtime).map_err(|error| error.to_string())
}

#[tauri::command]
fn reset_extensions_runtime(
    runtime: tauri::State<'_, ExtensionRuntimeState>,
) -> Result<(), String> {
    extension::reset_runtime_state(&runtime).map_err(|error| error.to_string())
}

#[tauri::command]
fn search_extensions(
    app: tauri::AppHandle,
    filter: Option<SearchExtensionsFilter>,
) -> Result<Vec<ExtensionSearchResult>, String> {
    extension::search_extensions(&app, filter.unwrap_or_default())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_themes(app: tauri::AppHandle) -> Result<Vec<InstalledTheme>, String> {
    let config_path = config::config_path(&app).map_err(|error| error.to_string())?;
    let app_config = config::load_from_path(&config_path).map_err(|error| error.to_string())?;
    let extensions_dir = extension::extensions_dir(&app).map_err(|error| error.to_string())?;

    extension::list_themes_from_dir(&extensions_dir, &app_config.theme)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn load_theme_colors(app: tauri::AppHandle) -> Result<ThemeColors, String> {
    let config_path = config::config_path(&app).map_err(|error| error.to_string())?;
    let app_config = config::load_from_path(&config_path).map_err(|error| error.to_string())?;

    extension::load_theme_colors_for_identifier(&app, &app_config.theme)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn load_theme_colors_for_identifier(
    app: tauri::AppHandle,
    identifier: String,
) -> Result<ThemeColors, String> {
    extension::load_theme_colors_for_identifier(&app, &identifier)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn uninstall_extension(
    app: tauri::AppHandle,
    extension_id: String,
) -> Result<Vec<InstalledExtension>, String> {
    extension::uninstall_extension(&app, &extension_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_extension_enabled(
    app: tauri::AppHandle,
    extension_id: String,
    enabled: bool,
) -> Result<Vec<InstalledExtension>, String> {
    extension::set_extension_enabled(&app, &extension_id, enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn select_theme(
    app: tauri::AppHandle,
    config: tauri::State<'_, AppConfigState>,
    runtime: tauri::State<'_, ExtensionRuntimeState>,
    identifier: String,
) -> Result<AppConfig, String> {
    let resolved_theme_id = extension::resolve_theme_identifier_from_state(&runtime, &identifier)
        .or_else(|_| {
            let extensions_dir = extension::extensions_dir(&app)?;
            extension::resolve_theme_identifier_from_dir(&extensions_dir, &identifier)
        })
        .map_err(|error| error.to_string())?;
    let config_path = config::config_path(&app).map_err(|error| error.to_string())?;
    let updated = config::patch_from_path(
        &config_path,
        &[config::ConfigPatch {
            key: "theme".into(),
            value: serde_json::Value::String(resolved_theme_id),
        }],
    )
    .map_err(|error| error.to_string())?;

    let mut state = config
        .0
        .lock()
        .map_err(|_| "config state lock poisoned".to_owned())?;
    *state = updated.clone();

    Ok(updated)
}

fn main() {
    let ctx = tauri::generate_context!();
    let identifier = ctx.config().identifier.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(identifier.into()),
                    }),
                ])
                .format(|out, message, record| {
                    out.finish(format_args!("[{}] {}", record.level(), message))
                })
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_config_dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&app_config_dir)?;

            let config_path = config::config_path(app)?;
            let app_config = config::load_from_path(&config_path)?;

            app.manage(AppConfigState(std::sync::Mutex::new(app_config)));
            app.manage(database::AppState {
                database: database::Database::new(),
            });
            app.manage(ExtensionRuntimeState::default());
            app.manage(lsp::LspRuntimeState::default());

            #[cfg(desktop)]
            let _ = app.handle().plugin(tauri_plugin_positioner::init());

            #[cfg(desktop)]
            let _ = app
                .handle()
                .plugin(tauri_plugin_updater::Builder::new().build());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            patch_config,
            set_config_value,
            lsp_configure_session,
            lsp_start,
            lsp_stop,
            lsp_status,
            database::connect,
            database::disconnect,
            database::execute_query,
            database::execute_mutation,
            database::is_connected,
            database::get_databases,
            database::get_tables,
            database::connect_saved,
            database::save_connection,
            database::list_connections,
            database::delete_connection,
            list_extensions,
            load_extensions,
            reset_extensions_runtime,
            list_themes,
            load_theme_colors,
            load_theme_colors_for_identifier,
            search_extensions,
            uninstall_extension,
            set_extension_enabled,
            select_theme
        ])
        .run(ctx)
        .expect("error while running tauri application");
}
