//! ArtEZ Notulist — Tauri-backend.

mod commands;
mod config;
mod logging;
mod pipeline;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            commands::bootstrap(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_config,
            commands::save_settings,
            commands::start_transcription,
            commands::generate_notulen,
            commands::open_output_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
