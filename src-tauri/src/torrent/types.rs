use serde::{Deserialize, Serialize};

/// Information about an active torrent stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    /// The info hash of the torrent (hex encoded)
    pub info_hash: String,
    /// The URL to stream the video from (e.g., http://127.0.0.1:PORT/stream/HASH/FILE_IDX)
    pub stream_url: String,
    /// The name of the video file being streamed
    pub file_name: String,
    /// Total size of the file in bytes
    pub total_size: u64,
    /// Index of the file within the torrent
    pub file_index: usize,
}

/// Statistics about an active torrent download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    /// Bytes downloaded so far
    pub downloaded_bytes: u64,
    /// Total bytes in the torrent
    pub total_bytes: u64,
    /// Current download speed in bytes/second
    pub download_speed: u64,
    /// Current upload speed in bytes/second
    pub upload_speed: u64,
    /// Number of connected peers
    pub peers_connected: u32,
    /// Progress as a percentage (0.0 - 100.0)
    pub progress_percent: f64,
    /// Whether the download is complete
    pub is_finished: bool,
}

/// Information about a file within a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    /// Index of the file in the torrent
    pub index: usize,
    /// Relative path/name of the file
    pub name: String,
    /// Size of the file in bytes
    pub size: u64,
    /// Whether this file is a video file
    pub is_video: bool,
    /// Whether this file is a subtitle file
    pub is_subtitle: bool,
}

/// Common video file extensions
pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp", "ts",
];

/// Check if a filename has a video extension
pub fn is_video_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    VIDEO_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{}", ext)))
}

/// Common subtitle file extensions
pub const SUBTITLE_EXTENSIONS: &[&str] = &["srt", "vtt", "ass", "ssa", "sub"];

/// Check if a filename has a subtitle extension
pub fn is_subtitle_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    SUBTITLE_EXTENSIONS.iter().any(|ext| lower.ends_with(&format!(".{}", ext)))
}

/// Error types for torrent operations
#[derive(Debug, thiserror::Error)]
pub enum TorrentError {
    #[error("Session not initialized")]
    SessionNotInitialized,

    #[error("Torrent not found: {0}")]
    TorrentNotFound(String),

    #[error("No video file found in torrent")]
    NoVideoFile,

    #[error("File not found in torrent: index {0}")]
    FileNotFound(usize),

    #[error("Stream server not running")]
    ServerNotRunning,

    #[error("librqbit error: {0}")]
    LibrqbitError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid magnet link: {0}")]
    InvalidMagnet(String),
}
