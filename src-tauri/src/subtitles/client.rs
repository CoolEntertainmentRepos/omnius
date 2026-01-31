use reqwest::Client;
use super::types::*;

const OPENSUBTITLES_API_URL: &str = "https://api.opensubtitles.com/api/v1";
// Free API key - limited to 5 requests/second, 100/day
// Users should get their own key from https://www.opensubtitles.com/consumers
const API_KEY: &str = ""; // Will use subdl.com as fallback since it doesn't require API key

// Alternative: SubDL.com API - no key required for basic searches
const SUBDL_API_URL: &str = "https://api.subdl.com/api/v1/subtitles";

pub struct SubtitleClient {
    client: Client,
    api_key: Option<String>,
}

impl SubtitleClient {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENSUBTITLES_API_KEY").ok();

        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Search subtitles by IMDB ID
    pub async fn search_by_imdb(&self, imdb_id: &str) -> Result<SubtitleSearchResult, String> {
        // Clean IMDB ID - remove 'tt' prefix if present
        let imdb = imdb_id.trim_start_matches("tt");

        println!("[SubtitleClient] Searching subtitles for IMDB: {}", imdb_id);

        // Use SubDL API (no key required)
        self.search_subdl(imdb).await
    }

    async fn search_subdl(&self, imdb_id: &str) -> Result<SubtitleSearchResult, String> {
        let url = format!("{}?imdb_id={}", SUBDL_API_URL, imdb_id);

        let response = self.client
            .get(&url)
            .header("User-Agent", "Streamer/1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch subtitles: {}", e))?;

        if !response.status().is_success() {
            println!("[SubtitleClient] SubDL API error: {}", response.status());
            return Ok(SubtitleSearchResult {
                subtitles: vec![],
                total_count: 0,
            });
        }

        let data: SubDlResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse subtitle response: {}", e))?;

        if !data.status {
            println!("[SubtitleClient] SubDL returned no results");
            return Ok(SubtitleSearchResult {
                subtitles: vec![],
                total_count: 0,
            });
        }

        let subtitles: Vec<Subtitle> = data.subtitles
            .into_iter()
            .flat_map(|s| {
                s.subtitles.into_iter().map(move |sub| Subtitle {
                    id: sub.url.clone(),
                    language: sub.lang.clone(),
                    language_name: get_language_name(&sub.lang),
                    download_url: sub.url,
                    release_name: Some(sub.release_name),
                    uploader: sub.author.map(|a| a.name),
                    download_count: 0,
                    hearing_impaired: sub.hi,
                    fps: None,
                })
            })
            .collect();

        let total_count = subtitles.len() as i32;

        println!("[SubtitleClient] Found {} subtitles", total_count);

        Ok(SubtitleSearchResult {
            subtitles,
            total_count,
        })
    }

    /// Search using OpenSubtitles API (requires API key)
    #[allow(dead_code)]
    async fn search_opensubtitles(&self, imdb_id: &str) -> Result<SubtitleSearchResult, String> {
        let api_key = self.api_key.as_ref()
            .ok_or("OpenSubtitles API key not set")?;

        let url = format!("{}/subtitles?imdb_id={}", OPENSUBTITLES_API_URL, imdb_id);

        let response = self.client
            .get(&url)
            .header("Api-Key", api_key.as_str())
            .header("User-Agent", "Streamer v1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch subtitles: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("OpenSubtitles API error: {}", response.status()));
        }

        let data: OsApiResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let subtitles: Vec<Subtitle> = data.data
            .into_iter()
            .map(|s| {
                let attrs = s.attributes;
                Subtitle {
                    id: attrs.subtitle_id,
                    language: attrs.language.clone(),
                    language_name: get_language_name(&attrs.language),
                    download_url: format!("{}/download", OPENSUBTITLES_API_URL),
                    release_name: attrs.release,
                    uploader: attrs.uploader.and_then(|u| u.name),
                    download_count: attrs.download_count,
                    hearing_impaired: attrs.hearing_impaired,
                    fps: attrs.fps,
                }
            })
            .collect();

        Ok(SubtitleSearchResult {
            subtitles,
            total_count: data.total_count,
        })
    }
}

// SubDL API response types
#[derive(Debug, serde::Deserialize)]
struct SubDlResponse {
    status: bool,
    #[serde(default)]
    subtitles: Vec<SubDlSubtitle>,
}

#[derive(Debug, serde::Deserialize)]
struct SubDlSubtitle {
    #[serde(default)]
    subtitles: Vec<SubDlSubtitleFile>,
}

#[derive(Debug, serde::Deserialize)]
struct SubDlSubtitleFile {
    #[serde(default)]
    release_name: String,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    hi: bool,
    author: Option<SubDlAuthor>,
}

#[derive(Debug, serde::Deserialize)]
struct SubDlAuthor {
    #[serde(default)]
    name: String,
}

impl Default for SubtitleClient {
    fn default() -> Self {
        Self::new()
    }
}
