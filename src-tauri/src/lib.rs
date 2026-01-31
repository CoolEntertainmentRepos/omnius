use std::sync::Arc;

use tauri::Manager;
use tokio::sync::RwLock;

mod commands;
mod omdb;
mod subtitles;
mod torrent;
mod yts;

use commands::{
    get_movie_details, get_movie_rating, get_movie_suggestions, list_movies, start_stream,
    get_stream_status, stop_stream, check_for_updates, get_app_version, search_subtitles,
    get_subtitle_languages, TorrentManagerState,
};
use omdb::OmdbClient;
use subtitles::SubtitleClient;
use torrent::TorrentManager;
use yts::YtsClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let yts_client = YtsClient::new();
    let omdb_client = OmdbClient::new();
    let subtitle_client = SubtitleClient::new();
    let torrent_manager: TorrentManagerState = Arc::new(RwLock::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(yts_client)
        .manage(omdb_client)
        .manage(subtitle_client)
        .manage(torrent_manager.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let manager_state = torrent_manager.clone();

            tauri::async_runtime::spawn(async move {
                println!("[TorrentManager] Starting initialization...");

                let data_dir = match app_handle.path().app_data_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        eprintln!("[TorrentManager] Failed to get app data dir: {}", e);
                        return;
                    }
                };

                println!("[TorrentManager] Data dir: {:?}", data_dir);

                // Create the directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(&data_dir) {
                    eprintln!("[TorrentManager] Failed to create app data dir: {}", e);
                    return;
                }

                println!("[TorrentManager] Directory created/exists, creating manager...");

                match TorrentManager::new(data_dir).await {
                    Ok(manager) => {
                        println!("[TorrentManager] Manager created, wrapping in Arc...");
                        let manager = Arc::new(manager);

                        // Start the stream server
                        println!("[TorrentManager] Starting stream server...");
                        if let Err(e) = manager.init_stream_server().await {
                            eprintln!("[TorrentManager] Failed to start stream server: {}", e);
                        } else {
                            println!("[TorrentManager] Stream server started on port {}", manager.stream_port());
                        }

                        let mut guard = manager_state.write().await;
                        *guard = Some(manager);
                        println!("[TorrentManager] Initialized successfully and stored in state");
                    }
                    Err(e) => {
                        eprintln!("[TorrentManager] Failed to initialize: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_movies,
            get_movie_details,
            get_movie_rating,
            get_movie_suggestions,
            start_stream,
            get_stream_status,
            stop_stream,
            check_for_updates,
            get_app_version,
            search_subtitles,
            get_subtitle_languages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
