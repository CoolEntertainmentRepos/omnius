use serde::{Deserialize, Serialize};

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

// OpenSubtitles.com API response types
#[derive(Debug, Deserialize)]
pub struct OsApiResponse {
    pub total_count: i32,
    pub total_pages: i32,
    pub per_page: i32,
    pub page: i32,
    pub data: Vec<OsSubtitleData>,
}

#[derive(Debug, Deserialize)]
pub struct OsSubtitleData {
    pub id: String,
    pub r#type: String,
    pub attributes: OsSubtitleAttributes,
}

#[derive(Debug, Deserialize)]
pub struct OsSubtitleAttributes {
    pub subtitle_id: String,
    pub language: String,
    pub download_count: i64,
    pub hearing_impaired: bool,
    pub fps: Option<f64>,
    pub release: Option<String>,
    pub uploader: Option<OsUploader>,
    pub files: Vec<OsFile>,
}

#[derive(Debug, Deserialize)]
pub struct OsUploader {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OsFile {
    pub file_id: i64,
    pub file_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OsDownloadResponse {
    pub link: String,
    pub file_name: String,
    pub requests: i32,
    pub remaining: i32,
    pub message: String,
    pub reset_time: String,
    pub reset_time_utc: String,
}

// Language mapping
pub fn get_language_name(code: &str) -> String {
    match code {
        "en" => "English".to_string(),
        "es" => "Spanish".to_string(),
        "fr" => "French".to_string(),
        "de" => "German".to_string(),
        "it" => "Italian".to_string(),
        "pt" => "Portuguese".to_string(),
        "ru" => "Russian".to_string(),
        "zh" => "Chinese".to_string(),
        "ja" => "Japanese".to_string(),
        "ko" => "Korean".to_string(),
        "ar" => "Arabic".to_string(),
        "nl" => "Dutch".to_string(),
        "pl" => "Polish".to_string(),
        "tr" => "Turkish".to_string(),
        "sv" => "Swedish".to_string(),
        "no" => "Norwegian".to_string(),
        "da" => "Danish".to_string(),
        "fi" => "Finnish".to_string(),
        "el" => "Greek".to_string(),
        "he" => "Hebrew".to_string(),
        "hi" => "Hindi".to_string(),
        "th" => "Thai".to_string(),
        "vi" => "Vietnamese".to_string(),
        "id" => "Indonesian".to_string(),
        "cs" => "Czech".to_string(),
        "hu" => "Hungarian".to_string(),
        "ro" => "Romanian".to_string(),
        "bg" => "Bulgarian".to_string(),
        "uk" => "Ukrainian".to_string(),
        "hr" => "Croatian".to_string(),
        "sr" => "Serbian".to_string(),
        "sk" => "Slovak".to_string(),
        "sl" => "Slovenian".to_string(),
        "sq" => "Albanian".to_string(),
        _ => code.to_uppercase(),
    }
}
