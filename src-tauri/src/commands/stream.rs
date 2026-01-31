use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::torrent::{StreamInfo, StreamStats, TorrentManager};

pub type TorrentManagerState = Arc<RwLock<Option<Arc<TorrentManager>>>>;

#[tauri::command]
pub async fn start_stream(
    manager: State<'_, TorrentManagerState>,
    torrent_hash: String,
) -> Result<StreamInfo, String> {
    println!("[start_stream] Starting stream for hash: {}", torrent_hash);

    let guard = manager.read().await;
    println!("[start_stream] Got manager lock, checking if initialized...");

    let manager = match guard.as_ref() {
        Some(m) => {
            println!("[start_stream] TorrentManager is available");
            m
        }
        None => {
            println!("[start_stream] ERROR: TorrentManager not initialized!");
            return Err("TorrentManager not initialized".to_string());
        }
    };

    // Build magnet URI from hash
    let magnet_uri = format!(
        "magnet:?xt=urn:btih:{}&tr=udp://open.demonii.com:1337/announce&tr=udp://tracker.openbittorrent.com:80&tr=udp://tracker.coppersurfer.tk:6969&tr=udp://glotorrents.pw:6969/announce&tr=udp://tracker.opentrackr.org:1337/announce&tr=udp://torrent.gresille.org:80/announce&tr=udp://p4p.arenabg.com:1337&tr=udp://tracker.leechers-paradise.org:6969",
        torrent_hash
    );

    println!("[start_stream] Built magnet URI, adding torrent...");

    match manager.start_stream(&magnet_uri).await {
        Ok(info) => {
            println!("[start_stream] Success! Stream URL: {}", info.stream_url);
            Ok(info)
        }
        Err(e) => {
            println!("[start_stream] Error starting stream: {}", e);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn get_stream_status(
    manager: State<'_, TorrentManagerState>,
    info_hash: String,
) -> Result<StreamStats, String> {
    let guard = manager.read().await;
    let manager = guard.as_ref().ok_or("TorrentManager not initialized")?;

    manager.get_stream_status(&info_hash).await
}

#[tauri::command]
pub async fn stop_stream(
    manager: State<'_, TorrentManagerState>,
    info_hash: String,
) -> Result<(), String> {
    let guard = manager.read().await;
    let manager = guard.as_ref().ok_or("TorrentManager not initialized")?;

    manager.stop_stream(&info_hash).await
}
