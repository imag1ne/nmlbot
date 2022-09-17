use crate::api::{MovieInfo, RequestMovieInfo, SearchResult};
use crate::error::{feedback_error, feedback_propagate_error, BotError};
use crate::{transcripts, Language};

use anyhow::anyhow;
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ImdbApi {
    url: &'static str,
    default_key: String,
}

impl ImdbApi {
    pub fn new(default_key: String) -> Self {
        Self {
            default_key,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    fn usage_api(&self, api_key: &str) -> String {
        format!("{}/Usage/{}", self.url, self.api_key_or_default(api_key))
    }

    fn search_api(&self, api_key: &str, title: &str) -> String {
        format!(
            "{}/API/SearchTitle/{}/{}",
            self.url,
            self.api_key_or_default(api_key),
            title
        )
    }

    fn api_key_or_default<'a>(&'a self, api_key: &'a str) -> &'a str {
        if api_key.is_empty() {
            &self.default_key
        } else {
            api_key
        }
    }

    pub fn title_info_api(&self, api_key: &str, id: &str, lang: Language) -> String {
        let lang = match lang {
            Language::En => "en",
        };

        format!(
            "{}/{}/API/Title/{}/{}",
            self.url,
            lang,
            self.api_key_or_default(api_key),
            id
        )
    }

    #[allow(dead_code)]
    pub async fn request_api_usage(
        &self,
        client: &Client,
        api_key: &str,
    ) -> anyhow::Result<UsageData> {
        let data =
            request_data_from_imdb_api(client, &self.usage_api(api_key), Language::En).await?;

        let count = data["count"].as_u64().unwrap_or_default();
        let maximum = data["maximum"].as_u64().unwrap_or_default();

        Ok(UsageData::new(count, maximum))
    }
}

impl Default for ImdbApi {
    fn default() -> Self {
        Self {
            url: "https://imdb-api.com",
            default_key: "".to_string(),
        }
    }
}

#[async_trait]
impl RequestMovieInfo for ImdbApi {
    async fn search(
        &self,
        client: &Client,
        api_key: &str,
        keyword: &str,
        limits: usize,
    ) -> Result<Vec<SearchResult>, BotError> {
        let url = self.search_api(api_key, keyword);

        let data = request_data_from_imdb_api(client, &url, Language::En).await?;
        let search_results = match data["results"].as_array() {
            Some(results) => results
                .iter()
                .take(limits)
                .cloned()
                .filter_map(|result| {
                    let mut search_result: SearchResult = serde_json::from_value(result).ok()?;
                    search_result.description = search_result
                        .description
                        .split_inclusive(')')
                        .take(1)
                        .collect();
                    Some(search_result)
                })
                .collect(),
            None => Vec::new(),
        };

        Ok(search_results)
    }

    async fn request_movie_information(
        &self,
        client: &Client,
        api_key: &str,
        id: &str,
        lang: Language,
    ) -> Result<MovieInfo, BotError> {
        let url = self.title_info_api(api_key, id, lang);

        let data = request_data_from_imdb_api(client, &url, lang).await?;
        let movie_info: ImdbApiMovieInfo = serde_json::from_value(data).map_err(|e| {
            feedback_propagate_error(
                anyhow!(e).context(transcripts::parse_imdb_api_response_failed(lang)),
            )
        })?;

        Ok(movie_info.into())
    }
}

async fn request_data_from_imdb_api(
    client: &Client,
    url: &str,
    fb_lang: Language,
) -> Result<serde_json::Value, BotError> {
    let response = client.get(url).send().await.map_err(|e| {
        feedback_propagate_error(
            anyhow!(e).context(transcripts::cannot_reach_server(fb_lang, "IMDb-API")),
        )
    })?;

    let data = response.json::<serde_json::Value>().await.map_err(|e| {
        feedback_propagate_error(
            anyhow!(e).context(transcripts::parse_imdb_api_error_message_failed(fb_lang)),
        )
    })?;

    if let Some(err) = data["errorMessage"].as_str() {
        if !err.is_empty() {
            return Err(feedback_error(anyhow!(err.to_string())));
        }
    }

    Ok(data)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImdbApiMovieInfo {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub full_title: Option<String>,
    #[serde(rename = "type")]
    pub movie_type: Option<String>,
    pub year: Option<String>,
    pub image: Option<String>,
    pub release_date: Option<String>,
    pub runtime_mins: Option<String>,
    pub plot: Option<String>,
    pub plot_local: Option<String>,
    pub director_list: Vec<IdentityObj>,
    pub star_list: Vec<IdentityObj>,
    pub genre_list: Vec<KeyValueObj>,
    pub country_list: Vec<KeyValueObj>,
    pub language_list: Vec<KeyValueObj>,
    pub content_rating: Option<String>,
    #[serde(rename = "imDbRating")]
    pub imdb_rating: Option<String>,
    pub keyword_list: Vec<String>,
}

impl From<ImdbApiMovieInfo> for MovieInfo {
    fn from(info: ImdbApiMovieInfo) -> Self {
        let title = info.title;
        let movie_type = info.movie_type.unwrap_or_default();
        let year = info.year.and_then(|y| y.parse().ok());
        let image = info.image.unwrap_or_default();
        let release_date = info
            .release_date
            .and_then(|date| NaiveDate::parse_from_str(&date, "%F").ok());
        let runtime = info.runtime_mins.and_then(|rt| rt.parse().ok());
        let plot = info.plot.unwrap_or_default();
        let director_list = info
            .director_list
            .iter()
            .map(|d| d.name.to_string())
            .collect();
        let star_list = info.star_list.iter().map(|s| s.name.to_string()).collect();
        let genre_list = info
            .genre_list
            .iter()
            .map(|g| g.value.to_string())
            .collect();
        let country_list = info
            .country_list
            .iter()
            .map(|c| c.value.to_string())
            .collect();
        let language_list = info
            .language_list
            .iter()
            .map(|l| l.value.to_string())
            .collect();
        let content_rating = info.content_rating.unwrap_or_default();
        let imdb_rating = info.imdb_rating.and_then(|r| r.parse().ok());
        let imdb_link = format!("https://www.imdb.com/title/{}", info.id);

        Self {
            title,
            movie_type,
            year,
            image,
            release_date,
            runtime,
            plot,
            director_list,
            star_list,
            genre_list,
            country_list,
            language_list,
            content_rating,
            imdb_rating,
            imdb_link,
        }
    }
}

pub struct UsageData {
    count: u64,
    maximum: u64,
}

impl UsageData {
    #[allow(dead_code)]
    pub fn new(count: u64, maximum: u64) -> Self {
        Self { count, maximum }
    }
}

impl Display for UsageData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.maximum != 0 {
            let percent = self.count as f64 / self.maximum as f64 * 100.0;
            write!(
                f,
                "Usage: {:.2}% ({} / {})",
                percent, self.count, self.maximum
            )
        } else {
            write!(f, "Usage: {} / {}", self.count, self.maximum)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KeyValueObj {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdentityObj {
    pub id: String,
    pub name: String,
}
