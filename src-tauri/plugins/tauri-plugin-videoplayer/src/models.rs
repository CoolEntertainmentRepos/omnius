use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub value: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleTrack {
    pub url: String,
    pub language: String,
    pub label: String,
    pub mime_type: String, // "text/vtt", "application/x-subrip", "text/x-ssa"
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayVideoRequest {
    pub path: String,
    pub subtitle_url: Option<String>,
    pub subtitles: Option<Vec<SubtitleTrack>>,
    pub start_position: Option<i64>, // resume position in ms
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayVideoResponse {
    pub last_position: Option<i64>, // last playback position in ms
    pub duration: Option<i64>,      // total duration in ms
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayLiveVideoRequest {
    pub url: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayLiveVideoResponse {
    pub finished: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceFocusRequest {
    pub main_activity_class_name: String,
}
