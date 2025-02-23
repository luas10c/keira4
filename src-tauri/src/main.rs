// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_positioner::init())
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
      #[cfg(desktop)]
      let _ = app.handle().plugin(tauri_plugin_updater::Builder::new().build());

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
