use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OmdbResponse {
    pub title: Option<String>,
    pub year: Option<String>,
    pub rated: Option<String>,
    pub released: Option<String>,
    pub runtime: Option<String>,
    pub genre: Option<String>,
    pub director: Option<String>,
    pub writer: Option<String>,
    pub actors: Option<String>,
    pub plot: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub awards: Option<String>,
    pub poster: Option<String>,
    pub ratings: Option<Vec<OmdbRating>>,
    pub metascore: Option<String>,
    #[serde(rename = "imdbRating")]
    pub imdb_rating: Option<String>,
    #[serde(rename = "imdbVotes")]
    pub imdb_votes: Option<String>,
    #[serde(rename = "imdbID")]
    pub imdb_id: Option<String>,
    #[serde(rename = "Type")]
    pub media_type: Option<String>,
    #[serde(rename = "DVD")]
    pub dvd: Option<String>,
    pub box_office: Option<String>,
    pub production: Option<String>,
    pub website: Option<String>,
    pub response: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OmdbRating {
    pub source: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MovieRating {
    pub imdb_rating: Option<f64>,
    pub imdb_votes: Option<String>,
    pub metascore: Option<u32>,
    pub rotten_tomatoes: Option<u32>,
    pub director: Option<String>,
    pub actors: Option<String>,
    pub plot: Option<String>,
    pub language: Option<String>,
    pub country: Option<String>,
    pub awards: Option<String>,
    pub box_office: Option<String>,
    pub rated: Option<String>,
}

impl From<OmdbResponse> for MovieRating {
    fn from(resp: OmdbResponse) -> Self {
        let imdb_rating = resp.imdb_rating
            .as_ref()
            .and_then(|s| s.parse::<f64>().ok());

        let metascore = resp.metascore
            .as_ref()
            .filter(|s| *s != "N/A")
            .and_then(|s| s.parse::<u32>().ok());

        let rotten_tomatoes = resp.ratings
            .as_ref()
            .and_then(|ratings| {
                ratings.iter()
                    .find(|r| r.source == "Rotten Tomatoes")
                    .and_then(|r| r.value.trim_end_matches('%').parse::<u32>().ok())
            });

        MovieRating {
            imdb_rating,
            imdb_votes: resp.imdb_votes.filter(|s| s != "N/A"),
            metascore,
            rotten_tomatoes,
            director: resp.director.filter(|s| s != "N/A"),
            actors: resp.actors.filter(|s| s != "N/A"),
            plot: resp.plot.filter(|s| s != "N/A"),
            language: resp.language.filter(|s| s != "N/A"),
            country: resp.country.filter(|s| s != "N/A"),
            awards: resp.awards.filter(|s| s != "N/A"),
            box_office: resp.box_office.filter(|s| s != "N/A"),
            rated: resp.rated.filter(|s| s != "N/A"),
        }
    }
}
