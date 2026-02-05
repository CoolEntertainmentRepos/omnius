use reqwest::Client;
use std::time::Duration;

use super::types::*;

// Free OMDB API - limited requests but works for our use case
// Users can get their own key at https://www.omdbapi.com/apikey.aspx
const OMDB_API_KEY: &str = "trilogy"; // Public demo key
const OMDB_BASE_URL: &str = "https://www.omdbapi.com/";

pub struct OmdbClient {
    client: Client,
}

impl OmdbClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn get_rating(&self, imdb_id: &str) -> Result<MovieRating, String> {
        // Ensure imdb_id starts with "tt"
        let imdb_id = if imdb_id.starts_with("tt") {
            imdb_id.to_string()
        } else {
            format!("tt{}", imdb_id)
        };

        println!("[OmdbClient] Fetching rating for: {}", imdb_id);

        let response = self
            .client
            .get(OMDB_BASE_URL)
            .query(&[
                ("i", imdb_id.as_str()),
                ("apikey", OMDB_API_KEY),
            ])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let data: OmdbResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Check for API error
        if data.response.as_deref() == Some("False") {
            return Err(data.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        println!(
            "[OmdbClient] Got rating - IMDB: {:?}, Metascore: {:?}",
            data.imdb_rating, data.metascore
        );

        Ok(MovieRating::from(data))
    }
}

impl Default for OmdbClient {
    fn default() -> Self {
        Self::new()
    }
}
