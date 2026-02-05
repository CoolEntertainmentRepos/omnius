use reqwest::Client;
use std::time::Duration;

use super::types::*;

const YTS_BASE_URL: &str = "https://yts.torrentbay.st/api/v2";
const LOCAL_API_URL: &str = "http://localhost:8080/api/v2";

pub struct YtsClient {
    client: Client,
}

impl YtsClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn list_movies(
        &self,
        params: ListMoviesParams,
    ) -> Result<MovieListData, reqwest::Error> {
        let url = format!("{}/list_movies.json", YTS_BASE_URL);
        println!("[YtsClient] Fetching: {}", url);
        println!("[YtsClient] Params: {:?}", params);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        println!("[YtsClient] Response status: {}", response.status());

        let data = response.json::<YtsApiResponse<MovieListData>>().await?;
        println!("[YtsClient] Movies count: {}", data.data.movie_count);

        // Debug: log first movie's rating
        if let Some(first) = data.data.movies.first() {
            println!("[YtsClient] First movie: {} - Rating: {}", first.title, first.rating);
        }

        Ok(data.data)
    }

    pub async fn movie_details(
        &self,
        movie_id: u32,
        with_cast: bool,
        with_images: bool,
    ) -> Result<MovieDetails, reqwest::Error> {
        let url = format!("{}/movie_details.json", YTS_BASE_URL);
        println!("[YtsClient] Fetching movie details: {}", url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("movie_id", movie_id.to_string()),
                ("with_cast", with_cast.to_string()),
                ("with_images", with_images.to_string()),
            ])
            .send()
            .await?;

        println!("[YtsClient] Movie details response status: {}", response.status());

        let data = response.json::<YtsApiResponse<MovieDetailsData>>().await?;
        println!("[YtsClient] Got movie: {}", data.data.movie.title);

        Ok(data.data.movie)
    }

    // ==================== LOCAL API METHODS ====================
    // These fetch from the local torrent-server database

    pub async fn list_movies_local(
        &self,
        params: ListMoviesParams,
    ) -> Result<MovieListData, reqwest::Error> {
        let url = format!("{}/list_movies.json", LOCAL_API_URL);
        println!("[YtsClient] Fetching LOCAL: {}", url);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        println!("[YtsClient] LOCAL Response status: {}", response.status());

        let data = response.json::<YtsApiResponse<MovieListData>>().await?;
        println!("[YtsClient] LOCAL Movies count: {}", data.data.movie_count);

        Ok(data.data)
    }

    pub async fn movie_details_local(
        &self,
        movie_id: u32,
        with_cast: bool,
        with_images: bool,
    ) -> Result<MovieDetails, reqwest::Error> {
        let url = format!("{}/movie_details.json", LOCAL_API_URL);
        println!("[YtsClient] Fetching LOCAL movie details: {} for id: {}", url, movie_id);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("movie_id", movie_id.to_string()),
                ("with_cast", with_cast.to_string()),
                ("with_images", with_images.to_string()),
            ])
            .send()
            .await?;

        println!("[YtsClient] LOCAL Movie details response status: {}", response.status());

        let data = response.json::<YtsApiResponse<MovieDetailsData>>().await?;
        println!("[YtsClient] LOCAL Got movie: {}", data.data.movie.title);

        Ok(data.data.movie)
    }

    pub async fn movie_suggestions_local(&self, movie_id: u32) -> Result<Vec<Movie>, String> {
        let url = format!("{}/movie_suggestions.json", LOCAL_API_URL);
        println!("[YtsClient] Fetching LOCAL movie suggestions for movie_id: {}", movie_id);

        let response = self
            .client
            .get(&url)
            .query(&[("movie_id", movie_id.to_string())])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        let parsed: YtsApiResponse<MovieSuggestionsData> = serde_json::from_str(&text)
            .map_err(|e| format!("Parse error: {}", e))?;

        println!("[YtsClient] LOCAL Got {} suggestions", parsed.data.movies.len());
        Ok(parsed.data.movies)
    }

    // ==================== EXTERNAL YTS METHODS ====================
    // These fetch from external YTS for search/discovery

    pub async fn movie_suggestions(&self, movie_id: u32) -> Result<Vec<Movie>, String> {
        let url = format!("{}/movie_suggestions.json", YTS_BASE_URL);
        println!("[YtsClient] Fetching movie suggestions for movie_id: {}", movie_id);

        let response = self
            .client
            .get(&url)
            .query(&[("movie_id", movie_id.to_string())])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        println!("[YtsClient] Suggestions response status: {}", response.status());

        let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
        println!("[YtsClient] Suggestions raw response length: {} bytes", text.len());

        let parsed: YtsApiResponse<MovieSuggestionsData> = serde_json::from_str(&text)
            .map_err(|e| {
                println!("[YtsClient] Suggestions parse error: {}", e);
                println!("[YtsClient] First 1000 chars: {}", &text[..text.len().min(1000)]);
                format!("Parse error: {}", e)
            })?;

        println!("[YtsClient] Got {} suggestions", parsed.data.movies.len());
        Ok(parsed.data.movies)
    }
}

impl Default for YtsClient {
    fn default() -> Self {
        Self::new()
    }
}
