use crate::omdb::{MovieRating, OmdbClient};

#[tauri::command]
pub async fn get_movie_rating(
    omdb_client: tauri::State<'_, OmdbClient>,
    imdb_code: String,
) -> Result<MovieRating, String> {
    omdb_client.get_rating(&imdb_code).await
}
