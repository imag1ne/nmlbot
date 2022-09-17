mod imdb;
mod notion;

pub use imdb::ImdbApi;
pub use notion::{
    create_database, insert_movie_info_to_notion_database, parse_notion_page_id_from_user_input,
};

use crate::error::BotError;
use crate::Language;
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait RequestMovieInfo {
    async fn search(
        &self,
        client: &Client,
        api_key: &str,
        keyword: &str,
        limits: usize,
    ) -> Result<Vec<SearchResult>, BotError>;

    async fn request_movie_information(
        &self,
        client: &Client,
        api_key: &str,
        id: &str,
        lang: Language,
    ) -> Result<MovieInfo, BotError>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug)]
pub struct MovieInfo {
    pub title: String,
    pub movie_type: String,
    pub year: Option<u32>,
    pub image: String,
    pub release_date: Option<NaiveDate>,
    pub runtime: Option<u32>,
    pub plot: String,
    pub director_list: Vec<String>,
    pub star_list: Vec<String>,
    pub genre_list: Vec<String>,
    pub country_list: Vec<String>,
    pub language_list: Vec<String>,
    pub content_rating: String,
    pub imdb_rating: Option<f64>,
    pub imdb_link: String,
}
