use reqwest::Client;
use super::types::*;

pub struct OmniusClient {
    client: Client,
    base_url: String,
}

impl OmniusClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn set_base_url(&mut self, url: &str) {
        self.base_url = url.trim_end_matches('/').to_string();
    }

    /// POST /api/v2/stream/start
    pub async fn start_stream(&self, hash: &str, file_index: Option<usize>) -> Result<StreamInfo, String> {
        let url = format!("{}/api/v2/stream/start", self.base_url);
        let body = serde_json::json!({
            "hash": hash,
            "file_index": file_index,
        });

        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to start stream: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Server error {}: {}", status, text));
        }

        resp.json::<StreamInfo>()
            .await
            .map_err(|e| format!("Failed to parse stream info: {}", e))
    }

    /// GET /api/v2/stream/status?hash={hash}
    pub async fn get_stream_status(&self, hash: &str) -> Result<StreamStatus, String> {
        let url = format!("{}/api/v2/stream/status?hash={}", self.base_url, hash);

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get stream status: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Server error: {}", text));
        }

        resp.json::<StreamStatus>()
            .await
            .map_err(|e| format!("Failed to parse stream status: {}", e))
    }

    /// POST /api/v2/stream/stop
    pub async fn stop_stream(&self, hash: &str) -> Result<(), String> {
        let url = format!("{}/api/v2/stream/stop", self.base_url);
        let body = serde_json::json!({ "hash": hash });

        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to stop stream: {}", e))?;

        Ok(())
    }

    /// GET /api/v2/torrent_files?hash={hash}
    pub async fn list_torrent_files(&self, hash: &str) -> Result<Vec<TorrentFile>, String> {
        let url = format!("{}/api/v2/torrent_files?hash={}", self.base_url, hash);

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list files: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Server error: {}", text));
        }

        resp.json::<Vec<TorrentFile>>()
            .await
            .map_err(|e| format!("Failed to parse file list: {}", e))
    }

    /// GET /api/v2/subtitles/search?imdb_id={id}&languages={langs}
    pub async fn search_subtitles(&self, imdb_id: &str, languages: Option<&str>) -> Result<SubtitleSearchResult, String> {
        let mut url = format!("{}/api/v2/subtitles/search?imdb_id={}", self.base_url, imdb_id);
        if let Some(langs) = languages {
            url.push_str(&format!("&languages={}", langs));
        }

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to search subtitles: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Server error: {}", text));
        }

        resp.json::<SubtitleSearchResult>()
            .await
            .map_err(|e| format!("Failed to parse subtitle results: {}", e))
    }

    /// GET /api/v2/subtitles/search_by_filename?filename={name}&languages={langs}
    pub async fn search_subtitles_by_filename(&self, filename: &str, languages: Option<&str>) -> Result<SubtitleSearchResult, String> {
        let encoded = urlencoding::encode(filename);
        let mut url = format!("{}/api/v2/subtitles/search_by_filename?filename={}", self.base_url, encoded);
        if let Some(langs) = languages {
            url.push_str(&format!("&languages={}", langs));
        }

        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to search subtitles by filename: {}", e))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Server error: {}", text));
        }

        resp.json::<SubtitleSearchResult>()
            .await
            .map_err(|e| format!("Failed to parse subtitle results: {}", e))
    }

    /// Construct full stream URL for a given hash and file index
    pub fn stream_url(&self, hash: &str, file_index: usize) -> String {
        format!("{}/stream/{}/{}", self.base_url, hash, file_index)
    }

    /// Construct subtitle download URL
    pub fn subtitle_download_url(&self, url: &str) -> String {
        let encoded = urlencoding::encode(url);
        format!("{}/api/v2/subtitles/download?url={}", self.base_url, encoded)
    }
}
