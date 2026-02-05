use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use librqbit::{
    api::TorrentIdOrHash, AddTorrent, AddTorrentOptions, ManagedTorrent, Session, SessionOptions,
};
use tokio::sync::RwLock;

use super::stream_server::StreamServer;
use super::types::{is_video_file, StreamInfo, StreamStats, TorrentError, TorrentFile};

type ManagedTorrentHandle = Arc<ManagedTorrent>;

/// Information about an active torrent
struct ActiveTorrent {
    handle: ManagedTorrentHandle,
    #[allow(dead_code)]
    video_file_index: usize,
    #[allow(dead_code)]
    video_file_name: String,
    #[allow(dead_code)]
    video_file_size: u64,
}

/// Manages torrent downloads and streaming
pub struct TorrentManager {
    /// The librqbit session
    session: Arc<Session>,
    /// Map of info_hash -> active torrent info
    active_torrents: RwLock<HashMap<String, ActiveTorrent>>,
    /// The stream server
    stream_server: Arc<StreamServer>,
    /// Download directory
    #[allow(dead_code)]
    download_dir: PathBuf,
}

impl TorrentManager {
    /// Create a new TorrentManager
    pub async fn new(download_dir: PathBuf) -> Result<Self, TorrentError> {
        println!("[TorrentManager::new] Creating download directory: {:?}", download_dir);
        // Create the download directory if it doesn't exist
        tokio::fs::create_dir_all(&download_dir).await?;
        println!("[TorrentManager::new] Directory created");

        // Pick an unused port for the stream server
        let http_port = portpicker::pick_unused_port().unwrap_or(9753);
        println!("[TorrentManager::new] Selected HTTP port: {}", http_port);

        // Create session options - disable DHT persistence for Android compatibility
        let options = SessionOptions {
            disable_dht: false,
            disable_dht_persistence: true,  // Disable persistence to avoid Android file system issues
            fastresume: false,              // Disable for simpler Android storage
            enable_upnp_port_forwarding: true,
            ..Default::default()
        };

        println!("[TorrentManager::new] Creating librqbit session...");
        // Create the librqbit session
        let session = match Session::new_with_opts(download_dir.clone(), options).await {
            Ok(s) => {
                println!("[TorrentManager::new] Session created successfully");
                s
            }
            Err(e) => {
                println!("[TorrentManager::new] Failed to create session: {}", e);
                return Err(TorrentError::LibrqbitError(e.to_string()));
            }
        };

        let manager = Self {
            session,
            active_torrents: RwLock::new(HashMap::new()),
            stream_server: Arc::new(StreamServer::new(http_port)),
            download_dir,
        };

        Ok(manager)
    }

    /// Initialize the stream server (must be called after creation when wrapped in Arc)
    pub async fn init_stream_server(self: &Arc<Self>) -> Result<(), TorrentError> {
        self.stream_server.start(self.clone()).await
    }

    /// Add a torrent from a magnet link and start streaming
    /// If file_index is provided, stream that specific file; otherwise auto-select largest video
    pub async fn start_stream(&self, magnet_uri: &str, file_index: Option<usize>) -> Result<StreamInfo, String> {
        println!("[TorrentManager::start_stream] Creating AddTorrent from URI, file_index: {:?}", file_index);
        let add_torrent = AddTorrent::from_url(magnet_uri);

        println!("[TorrentManager::start_stream] Adding torrent to session...");
        // Add the torrent to the session
        let add_result = self
            .session
            .add_torrent(
                add_torrent,
                Some(AddTorrentOptions {
                    overwrite: true,
                    ..Default::default()
                }),
            )
            .await;

        let handle = match add_result {
            Ok(r) => {
                println!("[TorrentManager::start_stream] Torrent added, getting handle...");
                match r.into_handle() {
                    Some(h) => h,
                    None => {
                        println!("[TorrentManager::start_stream] Failed to get torrent handle!");
                        return Err("Failed to get torrent handle".to_string());
                    }
                }
            }
            Err(e) => {
                println!("[TorrentManager::start_stream] Failed to add torrent: {}", e);
                return Err(format!("Failed to add torrent: {}", e));
            }
        };

        println!("[TorrentManager::start_stream] Waiting for metadata...");
        // Wait for metadata to be available
        if let Err(e) = handle.wait_until_initialized().await {
            println!("[TorrentManager::start_stream] Failed to get metadata: {}", e);
            return Err(format!("Failed to get metadata: {}", e));
        }
        println!("[TorrentManager::start_stream] Metadata received!");

        // Get the info hash (using hex crate to encode the 20-byte hash)
        let info_hash = hex::encode(handle.info_hash().0);

        // Get the video file - either by specified index or find largest
        let video_file = match file_index {
            Some(idx) => {
                println!("[TorrentManager::start_stream] Using specified file_index: {}", idx);
                self.get_file_at_index(&handle, idx)?
            }
            None => {
                println!("[TorrentManager::start_stream] Auto-selecting largest video file");
                self.find_largest_video_file(&handle)?
            }
        };

        // Store the active torrent
        {
            let mut torrents = self.active_torrents.write().await;
            torrents.insert(
                info_hash.clone(),
                ActiveTorrent {
                    handle,
                    video_file_index: video_file.index,
                    video_file_name: video_file.name.clone(),
                    video_file_size: video_file.size,
                },
            );
        }

        // Construct stream URL
        let stream_url = format!(
            "http://127.0.0.1:{}/stream/{}/{}",
            self.stream_server.port(),
            info_hash,
            video_file.index
        );

        Ok(StreamInfo {
            info_hash,
            stream_url,
            file_name: video_file.name,
            total_size: video_file.size,
            file_index: video_file.index,
        })
    }

    /// Get file info at a specific index
    fn get_file_at_index(
        &self,
        handle: &ManagedTorrentHandle,
        idx: usize,
    ) -> Result<TorrentFile, String> {
        let result = handle
            .with_metadata(|metadata| {
                metadata.file_infos.get(idx).map(|file_info| {
                    let filename = file_info.relative_filename.to_string_lossy().to_string();
                    TorrentFile {
                        index: idx,
                        name: filename.clone(),
                        size: file_info.len,
                        is_video: is_video_file(&filename),
                    }
                })
            })
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        result.ok_or_else(|| format!("File not found at index {}", idx))
    }

    /// Find the largest video file in a torrent
    fn find_largest_video_file(
        &self,
        handle: &ManagedTorrentHandle,
    ) -> Result<TorrentFile, String> {
        let result = handle
            .with_metadata(|metadata| {
                let mut largest_video: Option<TorrentFile> = None;

                for (idx, file_info) in metadata.file_infos.iter().enumerate() {
                    let filename = file_info.relative_filename.to_string_lossy().to_string();
                    let is_video = is_video_file(&filename);

                    if is_video {
                        let current = TorrentFile {
                            index: idx,
                            name: filename,
                            size: file_info.len,
                            is_video: true,
                        };

                        largest_video = match largest_video {
                            Some(existing) if existing.size >= current.size => Some(existing),
                            _ => Some(current),
                        };
                    }
                }

                largest_video
            })
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        result.ok_or_else(|| "No video file found in torrent".to_string())
    }

    /// Get statistics for a torrent stream
    pub async fn get_stream_status(&self, info_hash: &str) -> Result<StreamStats, String> {
        let torrents = self.active_torrents.read().await;

        let torrent = torrents
            .get(info_hash)
            .ok_or_else(|| format!("Torrent not found: {}", info_hash))?;

        let stats = torrent.handle.stats();

        let total_bytes = stats.total_bytes;
        let downloaded_bytes = stats.progress_bytes;
        let progress_percent = if total_bytes > 0 {
            (downloaded_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let (download_speed, upload_speed, peers_connected) = stats
            .live
            .map(|live| {
                (
                    (live.download_speed.mbps * 1_000_000.0 / 8.0) as u64,
                    (live.upload_speed.mbps * 1_000_000.0 / 8.0) as u64,
                    live.snapshot.peer_stats.live as u32,
                )
            })
            .unwrap_or((0, 0, 0));

        Ok(StreamStats {
            downloaded_bytes,
            total_bytes,
            download_speed,
            upload_speed,
            peers_connected,
            progress_percent,
            is_finished: stats.finished,
        })
    }

    /// Get a torrent handle for streaming
    pub async fn get_torrent_handle(
        &self,
        info_hash: &str,
    ) -> Result<ManagedTorrentHandle, TorrentError> {
        let torrents = self.active_torrents.read().await;
        let torrent = torrents
            .get(info_hash)
            .ok_or_else(|| TorrentError::TorrentNotFound(info_hash.to_string()))?;
        Ok(torrent.handle.clone())
    }

    /// List all files in a torrent
    #[allow(dead_code)]
    pub async fn list_files(&self, info_hash: &str) -> Result<Vec<TorrentFile>, String> {
        let torrents = self.active_torrents.read().await;
        let torrent = torrents
            .get(info_hash)
            .ok_or_else(|| format!("Torrent not found: {}", info_hash))?;

        let files = torrent
            .handle
            .with_metadata(|metadata| {
                metadata
                    .file_infos
                    .iter()
                    .enumerate()
                    .map(|(idx, file_info)| {
                        let filename = file_info.relative_filename.to_string_lossy().to_string();
                        TorrentFile {
                            index: idx,
                            name: filename.clone(),
                            size: file_info.len,
                            is_video: is_video_file(&filename),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        Ok(files)
    }

    /// Stop a torrent stream and remove it
    pub async fn stop_stream(&self, info_hash: &str) -> Result<(), String> {
        let mut torrents = self.active_torrents.write().await;

        let torrent = torrents
            .remove(info_hash)
            .ok_or_else(|| format!("Torrent not found: {}", info_hash))?;

        // Delete the torrent from the session
        let id = torrent.handle.id();
        self.session
            .delete(TorrentIdOrHash::Id(id), true)
            .await
            .map_err(|e| format!("Failed to delete torrent: {}", e))?;

        Ok(())
    }

    /// Get the stream server port
    pub fn stream_port(&self) -> u16 {
        self.stream_server.port()
    }

    /// Store a VTT subtitle in the stream server and return its URL
    pub async fn store_subtitle(&self, id: String, vtt_content: String) -> String {
        self.stream_server.store_subtitle(id.clone(), vtt_content).await;
        format!("http://127.0.0.1:{}/subtitle/{}", self.stream_server.port(), id)
    }
}
