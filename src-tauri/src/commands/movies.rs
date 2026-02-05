use tauri::State;

use crate::yts::{ListMoviesParams, Movie, MovieDetails, MovieListData, YtsClient};

#[tauri::command]
pub async fn list_movies(
    client: State<'_, YtsClient>,
    params: ListMoviesParams,
) -> Result<MovieListData, String> {
    println!("[list_movies] Called with params: {:?}", params);

    match client.list_movies(params).await {
        Ok(data) => {
            println!("[list_movies] Success: {} movies fetched", data.movies.len());
            Ok(data)
        }
        Err(e) => {
            println!("[list_movies] Error: {}", e);
            Err(format!("Failed to fetch movies: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_movie_details(
    client: State<'_, YtsClient>,
    movie_id: u32,
    with_cast: Option<bool>,
    with_images: Option<bool>,
) -> Result<MovieDetails, String> {
    println!("[get_movie_details] Fetching movie_id: {}", movie_id);

    match client
        .movie_details(movie_id, with_cast.unwrap_or(true), with_images.unwrap_or(true))
        .await
    {
        Ok(details) => {
            println!("[get_movie_details] Success: {}", details.title);
            Ok(details)
        }
        Err(e) => {
            println!("[get_movie_details] Error: {}", e);
            Err(format!("Failed to fetch movie details: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_movie_suggestions(
    client: State<'_, YtsClient>,
    movie_id: u32,
) -> Result<Vec<Movie>, String> {
    println!("[get_movie_suggestions] Fetching suggestions for movie_id: {}", movie_id);
    match client.movie_suggestions(movie_id).await {
        Ok(movies) => {
            println!("[get_movie_suggestions] Success: {} suggestions", movies.len());
            Ok(movies)
        }
        Err(e) => {
            println!("[get_movie_suggestions] Error: {}", e);
            Err(format!("Failed to fetch movie suggestions: {}", e))
        }
    }
}

// ==================== LOCAL API COMMANDS ====================
// These fetch from the local torrent-server database

#[tauri::command]
pub async fn list_movies_local(
    client: State<'_, YtsClient>,
    params: ListMoviesParams,
) -> Result<MovieListData, String> {
    println!("[list_movies_local] Called with params: {:?}", params);

    match client.list_movies_local(params).await {
        Ok(data) => {
            println!("[list_movies_local] Success: {} movies fetched", data.movies.len());
            Ok(data)
        }
        Err(e) => {
            println!("[list_movies_local] Error: {}", e);
            Err(format!("Failed to fetch local movies: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_movie_details_local(
    client: State<'_, YtsClient>,
    movie_id: u32,
    with_cast: Option<bool>,
    with_images: Option<bool>,
) -> Result<MovieDetails, String> {
    println!("[get_movie_details_local] Fetching movie_id: {}", movie_id);

    match client
        .movie_details_local(movie_id, with_cast.unwrap_or(true), with_images.unwrap_or(true))
        .await
    {
        Ok(details) => {
            println!("[get_movie_details_local] Success: {}", details.title);
            Ok(details)
        }
        Err(e) => {
            println!("[get_movie_details_local] Error: {}", e);
            Err(format!("Failed to fetch local movie details: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_movie_suggestions_local(
    client: State<'_, YtsClient>,
    movie_id: u32,
) -> Result<Vec<Movie>, String> {
    println!("[get_movie_suggestions_local] Fetching suggestions for movie_id: {}", movie_id);
    match client.movie_suggestions_local(movie_id).await {
        Ok(movies) => {
            println!("[get_movie_suggestions_local] Success: {} suggestions", movies.len());
            Ok(movies)
        }
        Err(e) => {
            println!("[get_movie_suggestions_local] Error: {}", e);
            Err(format!("Failed to fetch local movie suggestions: {}", e))
        }
    }
}
