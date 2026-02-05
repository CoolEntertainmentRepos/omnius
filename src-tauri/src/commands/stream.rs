use std::sync::Arc;

use tauri::State;
use tokio::sync::RwLock;

use crate::torrent::{StreamInfo, StreamStats, TorrentManager};
use crate::subtitles::SubtitleClient;

pub type TorrentManagerState = Arc<RwLock<Option<Arc<TorrentManager>>>>;

#[tauri::command]
pub async fn start_stream(
    manager: State<'_, TorrentManagerState>,
    torrent_hash: String,
    file_index: Option<i32>,
) -> Result<StreamInfo, String> {
    println!("[start_stream] Starting stream for hash: {}, file_index: {:?}", torrent_hash, file_index);

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

    match manager.start_stream(&magnet_uri, file_index.map(|i| i as usize)).await {
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
    println!("[get_stream_status] Getting status for: {}", info_hash);
    let guard = manager.read().await;
    let manager = guard.as_ref().ok_or("TorrentManager not initialized")?;

    let result = manager.get_stream_status(&info_hash).await;
    println!("[get_stream_status] Result: {:?}", result);
    result
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

/// Check if a stream URL is serving data (Range request bytes=0-0)
#[tauri::command]
pub async fn check_stream_ready(stream_url: String) -> Result<bool, String> {
    println!("[check_stream_ready] Checking: {}", stream_url);
    let client = reqwest::Client::new();
    match client
        .get(&stream_url)
        .header("Range", "bytes=0-0")
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status().as_u16();
            println!("[check_stream_ready] Status: {}", status);
            Ok(status == 200 || status == 206)
        }
        Err(e) => {
            println!("[check_stream_ready] Error: {}", e);
            Ok(false)
        }
    }
}

/// Download a subtitle, store it in the stream server, return its HTTP URL
#[tauri::command]
pub async fn serve_subtitle(
    manager: State<'_, TorrentManagerState>,
    subtitle_client: State<'_, SubtitleClient>,
    download_url: String,
) -> Result<String, String> {
    println!("[serve_subtitle] Downloading from: {}", download_url);

    // Download and convert to VTT
    let vtt_content = subtitle_client.download_subtitle_raw(&download_url).await?;

    // Store in the stream server
    let guard = manager.read().await;
    let mgr = guard.as_ref().ok_or("TorrentManager not initialized")?;

    // Use a hash of the URL as the ID
    let id = format!("{:x}", md5_hash(&download_url));
    let subtitle_url = mgr.store_subtitle(id, vtt_content).await;

    println!("[serve_subtitle] Serving at: {}", subtitle_url);
    Ok(subtitle_url)
}

/// Simple hash for subtitle IDs (not cryptographic, just for uniqueness)
fn md5_hash(input: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish()
}
