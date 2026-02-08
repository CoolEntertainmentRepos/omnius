use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::server::OmniusClient;
use crate::server::types::*;

pub type OmniusClientState = Arc<RwLock<OmniusClient>>;

#[tauri::command]
pub async fn start_stream(
    client: State<'_, OmniusClientState>,
    torrent_hash: String,
    file_index: Option<i32>,
) -> Result<StreamInfo, String> {
    println!("[start_stream] Starting stream for hash: {}, file_index: {:?}", torrent_hash, file_index);

    let client = client.read().await;
    let mut info = client.start_stream(&torrent_hash, file_index.map(|i| i as usize)).await?;

    // Make stream_url absolute using server base URL
    if info.stream_url.starts_with('/') {
        info.stream_url = format!("{}{}", client.base_url(), info.stream_url);
    }

    println!("[start_stream] Success! Stream URL: {}", info.stream_url);
    Ok(info)
}

#[tauri::command]
pub async fn get_stream_status(
    client: State<'_, OmniusClientState>,
    info_hash: String,
) -> Result<StreamStatus, String> {
    println!("[get_stream_status] Getting status for: {}", info_hash);
    let client = client.read().await;
    client.get_stream_status(&info_hash).await
}

#[tauri::command]
pub async fn stop_stream(
    client: State<'_, OmniusClientState>,
    info_hash: String,
) -> Result<(), String> {
    let client = client.read().await;
    client.stop_stream(&info_hash).await
}

/// Check if a stream URL is serving data (Range request bytes=0-0)
#[tauri::command]
pub async fn check_stream_ready(stream_url: String) -> Result<bool, String> {
    println!("[check_stream_ready] Checking: {}", stream_url);
    let http_client = reqwest::Client::new();
    match http_client
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

/// Return the server subtitle download URL (no local download needed)
#[tauri::command]
pub async fn serve_subtitle(
    client: State<'_, OmniusClientState>,
    download_url: String,
) -> Result<String, String> {
    println!("[serve_subtitle] Building server URL for: {}", download_url);
    let client = client.read().await;
    let subtitle_url = client.subtitle_download_url(&download_url);
    println!("[serve_subtitle] Serving at: {}", subtitle_url);
    Ok(subtitle_url)
}

/// List all files in a torrent
#[tauri::command]
pub async fn list_torrent_files(
    client: State<'_, OmniusClientState>,
    info_hash: String,
) -> Result<Vec<TorrentFile>, String> {
    println!("[list_torrent_files] Listing files for: {}", info_hash);
    let client = client.read().await;
    client.list_torrent_files(&info_hash).await
}
