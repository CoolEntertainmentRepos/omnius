use tauri::State;
use crate::subtitles::{SubtitleClient, Subtitle, SubtitleSearchResult};

#[tauri::command]
pub async fn search_subtitles(
    client: State<'_, SubtitleClient>,
    imdb_id: String,
) -> Result<SubtitleSearchResult, String> {
    println!("[search_subtitles] Searching for IMDB: {}", imdb_id);
    client.search_by_imdb(&imdb_id).await
}

#[tauri::command]
pub async fn get_subtitle_languages() -> Result<Vec<SubtitleLanguage>, String> {
    Ok(vec![
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
    ])
}

#[derive(serde::Serialize)]
pub struct SubtitleLanguage {
    pub code: String,
    pub name: String,
}
