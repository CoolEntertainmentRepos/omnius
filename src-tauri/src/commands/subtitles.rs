use tauri::State;
use crate::subtitles::{SubtitleClient, SubtitleSearchResult};

#[tauri::command]
pub async fn search_subtitles(
    client: State<'_, SubtitleClient>,
    imdb_id: String,
    api_key: Option<String>,
    languages: Option<String>,
) -> Result<SubtitleSearchResult, String> {
    println!("[search_subtitles] Searching for IMDB: {}, languages: {:?}", imdb_id, languages);
    client.search_by_imdb(&imdb_id, api_key.as_deref(), languages.as_deref()).await
}

#[derive(serde::Serialize)]
pub struct SubtitleDownloadResult {
    pub vtt_data_url: String,
}

#[tauri::command]
pub async fn download_subtitle(
    client: State<'_, SubtitleClient>,
    download_url: String,
) -> Result<SubtitleDownloadResult, String> {
    println!("[download_subtitle] Downloading from: {}", download_url);
    let vtt_data_url = client.download_subtitle(&download_url).await?;
    Ok(SubtitleDownloadResult { vtt_data_url })
}

#[tauri::command]
pub async fn search_subtitles_by_filename(
    client: State<'_, SubtitleClient>,
    filename: String,
    languages: Option<String>,
) -> Result<SubtitleSearchResult, String> {
    println!("[search_subtitles_by_filename] Searching for filename: {}, languages: {:?}", filename, languages);
    client.search_by_filename(&filename, languages.as_deref()).await
}

#[tauri::command]
pub async fn get_subtitle_languages() -> Result<Vec<SubtitleLanguage>, String> {
    Ok(vec![
        // Major languages
        SubtitleLanguage { code: "en".to_string(), name: "English".to_string() },
        SubtitleLanguage { code: "es".to_string(), name: "Spanish".to_string() },
        SubtitleLanguage { code: "fr".to_string(), name: "French".to_string() },
        SubtitleLanguage { code: "de".to_string(), name: "German".to_string() },
        SubtitleLanguage { code: "it".to_string(), name: "Italian".to_string() },
        SubtitleLanguage { code: "pt".to_string(), name: "Portuguese".to_string() },
        SubtitleLanguage { code: "ru".to_string(), name: "Russian".to_string() },
        SubtitleLanguage { code: "zh".to_string(), name: "Chinese".to_string() },
        SubtitleLanguage { code: "ja".to_string(), name: "Japanese".to_string() },
        SubtitleLanguage { code: "ko".to_string(), name: "Korean".to_string() },
        SubtitleLanguage { code: "ar".to_string(), name: "Arabic".to_string() },
        SubtitleLanguage { code: "nl".to_string(), name: "Dutch".to_string() },
        SubtitleLanguage { code: "pl".to_string(), name: "Polish".to_string() },
        SubtitleLanguage { code: "tr".to_string(), name: "Turkish".to_string() },
        // Nordic
        SubtitleLanguage { code: "sv".to_string(), name: "Swedish".to_string() },
        SubtitleLanguage { code: "no".to_string(), name: "Norwegian".to_string() },
        SubtitleLanguage { code: "da".to_string(), name: "Danish".to_string() },
        SubtitleLanguage { code: "fi".to_string(), name: "Finnish".to_string() },
        SubtitleLanguage { code: "is".to_string(), name: "Icelandic".to_string() },
        // Eastern European
        SubtitleLanguage { code: "cs".to_string(), name: "Czech".to_string() },
        SubtitleLanguage { code: "sk".to_string(), name: "Slovak".to_string() },
        SubtitleLanguage { code: "hu".to_string(), name: "Hungarian".to_string() },
        SubtitleLanguage { code: "ro".to_string(), name: "Romanian".to_string() },
        SubtitleLanguage { code: "bg".to_string(), name: "Bulgarian".to_string() },
        SubtitleLanguage { code: "uk".to_string(), name: "Ukrainian".to_string() },
        SubtitleLanguage { code: "hr".to_string(), name: "Croatian".to_string() },
        SubtitleLanguage { code: "sr".to_string(), name: "Serbian".to_string() },
        SubtitleLanguage { code: "sl".to_string(), name: "Slovenian".to_string() },
        SubtitleLanguage { code: "bs".to_string(), name: "Bosnian".to_string() },
        SubtitleLanguage { code: "mk".to_string(), name: "Macedonian".to_string() },
        SubtitleLanguage { code: "sq".to_string(), name: "Albanian".to_string() },
        SubtitleLanguage { code: "et".to_string(), name: "Estonian".to_string() },
        SubtitleLanguage { code: "lv".to_string(), name: "Latvian".to_string() },
        SubtitleLanguage { code: "lt".to_string(), name: "Lithuanian".to_string() },
        // Western European
        SubtitleLanguage { code: "el".to_string(), name: "Greek".to_string() },
        SubtitleLanguage { code: "he".to_string(), name: "Hebrew".to_string() },
        SubtitleLanguage { code: "ca".to_string(), name: "Catalan".to_string() },
        SubtitleLanguage { code: "eu".to_string(), name: "Basque".to_string() },
        SubtitleLanguage { code: "gl".to_string(), name: "Galician".to_string() },
        // Asian
        SubtitleLanguage { code: "hi".to_string(), name: "Hindi".to_string() },
        SubtitleLanguage { code: "bn".to_string(), name: "Bengali".to_string() },
        SubtitleLanguage { code: "ta".to_string(), name: "Tamil".to_string() },
        SubtitleLanguage { code: "te".to_string(), name: "Telugu".to_string() },
        SubtitleLanguage { code: "ml".to_string(), name: "Malayalam".to_string() },
        SubtitleLanguage { code: "th".to_string(), name: "Thai".to_string() },
        SubtitleLanguage { code: "vi".to_string(), name: "Vietnamese".to_string() },
        SubtitleLanguage { code: "id".to_string(), name: "Indonesian".to_string() },
        SubtitleLanguage { code: "ms".to_string(), name: "Malay".to_string() },
        SubtitleLanguage { code: "tl".to_string(), name: "Filipino".to_string() },
        SubtitleLanguage { code: "my".to_string(), name: "Burmese".to_string() },
        SubtitleLanguage { code: "km".to_string(), name: "Khmer".to_string() },
        // Middle East
        SubtitleLanguage { code: "fa".to_string(), name: "Persian".to_string() },
        SubtitleLanguage { code: "ur".to_string(), name: "Urdu".to_string() },
        SubtitleLanguage { code: "ku".to_string(), name: "Kurdish".to_string() },
        // African
        SubtitleLanguage { code: "sw".to_string(), name: "Swahili".to_string() },
        SubtitleLanguage { code: "af".to_string(), name: "Afrikaans".to_string() },
        // Portuguese variants
        SubtitleLanguage { code: "pt-br".to_string(), name: "Portuguese (Brazil)".to_string() },
        // Chinese variants
        SubtitleLanguage { code: "zh-cn".to_string(), name: "Chinese (Simplified)".to_string() },
        SubtitleLanguage { code: "zh-tw".to_string(), name: "Chinese (Traditional)".to_string() },
        // Spanish variants
        SubtitleLanguage { code: "es-la".to_string(), name: "Spanish (Latin America)".to_string() },
    ])
}

#[derive(serde::Serialize)]
pub struct SubtitleLanguage {
    pub code: String,
    pub name: String,
}
