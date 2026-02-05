use reqwest::Client;
use std::io::{Read, Cursor};
use super::types::*;
use super::convert::{srt_to_vtt, ass_to_vtt};

// OpenSubtitles REST API - free, no key required
const OPENSUBTITLES_REST_URL: &str = "https://rest.opensubtitles.org/search";
// SubDL API - requires API key for better results
const SUBDL_API_URL: &str = "https://api.subdl.com/api/v1/subtitles";
// Hardcoded API key for TV builds (easier than typing with remote)
const SUBDL_API_KEY: &str = "4bHkwDgMS95KS34bCXOo7y1LgkAKkK6P";

pub struct SubtitleClient {
    client: Client,
}

impl SubtitleClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Search subtitles by IMDB ID
    /// Uses hardcoded SubDL API key, falls back to OpenSubtitles if needed
    /// Optional language parameter filters results (e.g., "en" or "en,sq")
    pub async fn search_by_imdb(&self, imdb_id: &str, _api_key: Option<&str>, languages: Option<&str>) -> Result<SubtitleSearchResult, String> {
        // Clean IMDB ID - remove 'tt' prefix if present
        let imdb = imdb_id.trim_start_matches("tt");

        println!("[SubtitleClient] Searching subtitles for IMDB: {}, languages: {:?}", imdb, languages);

        // Try SubDL first with hardcoded key
        println!("[SubtitleClient] Using SubDL API");
        match self.search_subdl(imdb, SUBDL_API_KEY, languages).await {
            Ok(result) if !result.subtitles.is_empty() => return Ok(result),
            Ok(_) => println!("[SubtitleClient] SubDL returned no results, trying OpenSubtitles"),
            Err(e) => println!("[SubtitleClient] SubDL error: {}, trying OpenSubtitles", e),
        }

        // Fallback to OpenSubtitles REST API (free, no key)
        self.search_opensubtitles_rest(imdb).await
    }

    async fn search_subdl(&self, imdb_id: &str, api_key: &str, languages: Option<&str>) -> Result<SubtitleSearchResult, String> {
        let mut url = format!("{}?api_key={}&imdb_id=tt{}", SUBDL_API_URL, api_key, imdb_id);

        // Add language filter if specified
        if let Some(langs) = languages {
            url.push_str(&format!("&languages={}", langs));
        }

        let response = self.client
            .get(&url)
            .header("User-Agent", "Streamer v1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch subtitles: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("SubDL API error: {}", response.status()));
        }

        let data: SubDlResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse SubDL response: {}", e))?;

        if !data.status {
            return Ok(SubtitleSearchResult {
                subtitles: vec![],
                total_count: 0,
            });
        }

        let subtitles: Vec<Subtitle> = data.subtitles
            .into_iter()
            .map(|sub| Subtitle {
                id: sub.url.clone(),
                language: sub.language.to_lowercase(),  // "EN" -> "en"
                language_name: sub.lang.clone(),        // "English"
                download_url: format!("https://dl.subdl.com{}", sub.url),
                release_name: Some(sub.release_name),
                uploader: sub.author.clone(),
                download_count: 0,
                hearing_impaired: sub.hi,
                fps: None,
            })
            .collect();

        let total_count = subtitles.len() as i32;
        println!("[SubtitleClient] SubDL found {} subtitles", total_count);

        Ok(SubtitleSearchResult {
            subtitles,
            total_count,
        })
    }

    async fn search_opensubtitles_rest(&self, imdb_id: &str) -> Result<SubtitleSearchResult, String> {
        let url = format!("{}/imdbid-{}", OPENSUBTITLES_REST_URL, imdb_id);

        println!("[SubtitleClient] Fetching: {}", url);

        let response = self.client
            .get(&url)
            .header("User-Agent", "Streamer v1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch subtitles: {}", e))?;

        if !response.status().is_success() {
            println!("[SubtitleClient] OpenSubtitles API error: {}", response.status());
            return Ok(SubtitleSearchResult {
                subtitles: vec![],
                total_count: 0,
            });
        }

        let data: Vec<OpenSubtitlesResult> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse subtitle response: {}", e))?;

        let subtitles: Vec<Subtitle> = data
            .into_iter()
            .map(|sub| Subtitle {
                id: sub.IDSubtitleFile.clone(),
                language: sub.SubLanguageID.clone(),
                language_name: sub.LanguageName.clone(),
                download_url: sub.SubDownloadLink,
                release_name: Some(sub.MovieReleaseName),
                uploader: Some(sub.UserNickName),
                download_count: sub.SubDownloadsCnt.parse().unwrap_or(0),
                hearing_impaired: sub.SubHearingImpaired == "1",
                fps: sub.MovieFPS.parse().ok(),
            })
            .collect();

        let total_count = subtitles.len() as i32;

        println!("[SubtitleClient] Found {} subtitles", total_count);

        Ok(SubtitleSearchResult {
            subtitles,
            total_count,
        })
    }

    /// Download a subtitle file from URL and convert to raw VTT string
    pub async fn download_subtitle_raw(&self, url: &str) -> Result<String, String> {
        println!("[SubtitleClient] Downloading subtitle from: {}", url);

        let response = self.client
            .get(url)
            .header("User-Agent", "Streamer v1.0")
            .send()
            .await
            .map_err(|e| format!("Failed to download subtitle: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to download subtitle: HTTP {}", response.status()));
        }

        let bytes = response.bytes().await
            .map_err(|e| format!("Failed to read subtitle bytes: {}", e))?;

        println!("[SubtitleClient] Downloaded {} bytes", bytes.len());

        // OpenSubtitles returns gzipped files
        let content = if bytes.starts_with(&[0x1f, 0x8b]) {
            self.decompress_gzip(&bytes)?
        } else if bytes.starts_with(&[0x50, 0x4B]) {
            self.extract_subtitle_from_zip(&bytes)?
        } else {
            String::from_utf8(bytes.to_vec())
                .unwrap_or_else(|_| {
                    bytes.iter().map(|&b| b as char).collect::<String>()
                })
        };

        let vtt_content = self.convert_to_vtt(&content)?;
        println!("[SubtitleClient] Converted to VTT ({} chars)", vtt_content.len());
        Ok(vtt_content)
    }

    /// Download a subtitle file from URL and convert to VTT
    /// Returns a base64 data URL that can be used directly in <track> element
    pub async fn download_subtitle(&self, url: &str) -> Result<String, String> {
        let vtt_content = self.download_subtitle_raw(url).await?;

        // Encode as base64 data URL
        let base64_content = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            vtt_content.as_bytes()
        );

        let data_url = format!("data:text/vtt;base64,{}", base64_content);

        println!("[SubtitleClient] Converted to VTT data URL ({} chars)", data_url.len());

        Ok(data_url)
    }

    /// Decompress gzip data
    fn decompress_gzip(&self, data: &[u8]) -> Result<String, String> {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut bytes = Vec::new();
        decoder.read_to_end(&mut bytes)
            .map_err(|e| format!("Failed to decompress gzip: {}", e))?;

        // Try UTF-8 first, fallback to Latin-1
        Ok(String::from_utf8(bytes.clone())
            .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect::<String>()))
    }

    /// Extract subtitle file from a ZIP archive
    fn extract_subtitle_from_zip(&self, zip_bytes: &[u8]) -> Result<String, String> {
        let cursor = Cursor::new(zip_bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to open ZIP archive: {}", e))?;

        println!("[SubtitleClient] ZIP contains {} files", archive.len());

        // Look for subtitle files (prefer SRT, then ASS/SSA, then SUB)
        let subtitle_extensions = ["srt", "ass", "ssa", "sub", "vtt"];

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

            let name = file.name().to_lowercase();

            for ext in &subtitle_extensions {
                if name.ends_with(ext) {
                    println!("[SubtitleClient] Found subtitle file: {}", file.name());

                    let mut content = String::new();
                    file.read_to_string(&mut content)
                        .map_err(|e| format!("Failed to read subtitle content: {}", e))?;

                    return Ok(content);
                }
            }
        }

        Err("No subtitle file found in ZIP archive".to_string())
    }

    /// Convert subtitle content to VTT format
    fn convert_to_vtt(&self, content: &str) -> Result<String, String> {
        // Check if already VTT
        if content.trim().starts_with("WEBVTT") {
            return Ok(content.to_string());
        }

        // Check for ASS/SSA format
        if content.contains("[Script Info]") || content.contains("[V4+ Styles]") {
            return Ok(ass_to_vtt(content));
        }

        // Assume SRT format (most common)
        Ok(srt_to_vtt(content))
    }

}

// OpenSubtitles REST API response type
#[derive(Debug, serde::Deserialize)]
#[allow(non_snake_case)]
struct OpenSubtitlesResult {
    #[serde(default)]
    IDSubtitleFile: String,
    #[serde(default)]
    SubLanguageID: String,
    #[serde(default)]
    LanguageName: String,
    #[serde(default)]
    SubDownloadLink: String,
    #[serde(default)]
    MovieReleaseName: String,
    #[serde(default)]
    UserNickName: String,
    #[serde(default)]
    SubDownloadsCnt: String,
    #[serde(default)]
    SubHearingImpaired: String,
    #[serde(default)]
    MovieFPS: String,
}

// SubDL API response types
#[derive(Debug, serde::Deserialize)]
struct SubDlResponse {
    #[serde(default)]
    status: bool,
    #[serde(default)]
    subtitles: Vec<SubDlSubtitle>,
}

#[derive(Debug, serde::Deserialize)]
struct SubDlSubtitle {
    #[serde(default)]
    release_name: String,
    #[serde(default)]
    lang: String,        // Full language name like "English"
    #[serde(default)]
    language: String,    // Language code like "EN"
    #[serde(default)]
    url: String,
    #[serde(default)]
    hi: bool,
    #[serde(default)]
    author: Option<String>,
}

impl Default for SubtitleClient {
    fn default() -> Self {
        Self::new()
    }
}
