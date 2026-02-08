use std::sync::Arc;

use tauri::Manager;
use tokio::sync::RwLock;

mod commands;
mod server;
mod config;

use commands::{
    start_stream, get_stream_status, stop_stream, check_stream_ready,
    serve_subtitle, list_torrent_files,
    search_subtitles, search_subtitles_by_filename, get_subtitle_languages, download_subtitle,
    check_for_updates, get_app_version,
    get_storage_info, clear_cache, get_download_path,
    get_server_url, set_server_url,
    OmniusClientState,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_videoplayer::init())
        .setup(|app| {
            // Load config from app data directory
            let config_dir = app.path().app_data_dir().ok();
            let settings = Arc::new(config::Settings::load(config_dir));

            // Create server client
            let client = server::OmniusClient::new(&settings.server_url());
            let client_state: OmniusClientState = Arc::new(RwLock::new(client));

            app.manage(settings);
            app.manage(client_state);

            println!("[Omnius] App initialized - pure client mode (no torrent engine)");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_stream,
            get_stream_status,
            stop_stream,
            check_stream_ready,
            serve_subtitle,
            list_torrent_files,
            search_subtitles,
            search_subtitles_by_filename,
            get_subtitle_languages,
            download_subtitle,
            check_for_updates,
            get_app_version,
            get_storage_info,
            clear_cache,
            get_download_path,
            get_server_url,
            set_server_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
