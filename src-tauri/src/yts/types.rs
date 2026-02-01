use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMoviesParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_rating: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_rt_ratings: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtsApiResponse<T> {
    pub status: String,
    pub status_message: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieListData {
    pub movie_count: u32,
    pub limit: u32,
    pub page_number: u32,
    #[serde(default)]
    pub movies: Vec<Movie>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: u32,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub imdb_code: String,
    pub title: String,
    #[serde(default)]
    pub title_english: Option<String>,
    #[serde(default)]
    pub title_long: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub year: u32,
    #[serde(default)]
    pub rating: f32,
    #[serde(default)]
    pub runtime: u32,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description_full: String,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub yt_trailer_code: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub background_image: String,
    #[serde(default)]
    pub background_image_original: String,
    #[serde(default)]
    pub small_cover_image: String,
    #[serde(default)]
    pub medium_cover_image: String,
    #[serde(default)]
    pub large_cover_image: String,
    #[serde(default)]
    pub torrents: Vec<Torrent>,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub date_uploaded: String,
    #[serde(default)]
    pub date_uploaded_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub url: String,
    pub hash: String,
    pub quality: String,
    #[serde(rename = "type", default)]
    pub torrent_type: String,
    #[serde(default)]
    pub is_repack: String,
    #[serde(default)]
    pub video_codec: String,
    #[serde(default)]
    pub bit_depth: String,
    #[serde(default)]
    pub audio_channels: String,
    #[serde(default)]
    pub seeds: u32,
    #[serde(default)]
    pub peers: u32,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub date_uploaded: String,
    #[serde(default)]
    pub date_uploaded_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetailsData {
    pub movie: MovieDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieDetails {
    pub id: u32,
    pub url: String,
    pub imdb_code: String,
    pub title: String,
    #[serde(default)]
    pub title_english: Option<String>,
    pub title_long: String,
    pub slug: String,
    pub year: u32,
    #[serde(default)]
    pub rating: f32,
    #[serde(default)]
    pub runtime: u32,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description_full: String,
    #[serde(default)]
    pub synopsis: String,
    #[serde(default)]
    pub yt_trailer_code: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub background_image: String,
    #[serde(default)]
    pub background_image_original: String,
    #[serde(default)]
    pub small_cover_image: String,
    #[serde(default)]
    pub medium_cover_image: String,
    #[serde(default)]
    pub large_cover_image: String,
    #[serde(default)]
    pub torrents: Vec<Torrent>,
    #[serde(default)]
    pub cast: Vec<Cast>,
    #[serde(default)]
    pub like_count: Option<u32>,
    #[serde(default)]
    pub download_count: Option<u32>,
    #[serde(default)]
    pub date_uploaded: String,
    #[serde(default)]
    pub date_uploaded_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cast {
    pub name: String,
    pub character_name: String,
    #[serde(default)]
    pub url_small_image: Option<String>,
    pub imdb_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovieSuggestionsData {
    pub movie_count: u32,
    #[serde(default)]
    pub movies: Vec<Movie>,
}
