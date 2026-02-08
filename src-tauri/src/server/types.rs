use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_url: String,
    pub file_name: String,
    pub total_size: u64,
    pub file_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamStatus {
    pub downloaded: u64,
    pub total_size: u64,
    pub download_speed: u64,
    pub peers: u32,
    pub progress: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentFile {
    pub index: usize,
    pub name: String,
    pub length: u64,
    pub is_video: bool,
    pub is_subtitle: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtitle {
    pub id: String,
    pub language: String,
    pub language_name: String,
    pub download_url: String,
    pub release_name: Option<String>,
    pub uploader: Option<String>,
    pub download_count: i64,
    pub hearing_impaired: bool,
    pub fps: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtitleSearchResult {
    pub subtitles: Vec<Subtitle>,
    pub total_count: i32,
}
