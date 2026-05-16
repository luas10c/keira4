// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod error;

use config::{AppConfig, AppConfigState};
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
fn load_config(
    config: tauri::State<'_, AppConfigState>,
) -> Result<AppConfig, String> {
    config
        .0
        .lock()
        .map(|config| config.clone())
        .map_err(|_| "config state lock poisoned".to_owned())
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

fn main() {
    let ctx = tauri::generate_context!();
    let identifier = ctx.config().identifier.clone();

    tauri::Builder::default()
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
            let config_path = config::config_path(app)?;
            let app_config = config::load_from_path(&config_path)?;

            log::info!("loaded config from {}", config_path.display());
            app.manage(AppConfigState(std::sync::Mutex::new(app_config)));

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
            set_config_value
        ])
        .run(ctx)
        .expect("error while running tauri application");
}
