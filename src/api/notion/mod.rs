mod objects;

use crate::api::notion::objects::*;
use crate::api::MovieInfo;
use crate::error::{feedback_error, feedback_propagate_error, propagate_error, BotError};
use crate::{transcripts, Language};

use anyhow::anyhow;
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseObj {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotionErrorObj {
    pub object: String,
    pub status: u16,
    pub code: String,
    pub message: String,
}

pub async fn create_database(
    client: &Client,
    token: &str,
    page_id: &str,
    fb_lang: Language,
) -> Result<String, BotError> {
    let url = "https://api.notion.com/v1/databases";

    let body = json!({
        "parent": {
            "type": "page_id",
            "page_id": page_id
        },
        "icon": {
            "type": "emoji",
            "emoji": "🎬"
        },
        "title": [
            {
                "type": "text",
                "text": {
                    "content": "Movie List",
                    "link": null
                }
            }
        ],
        "properties": {
            "Title": {
                "title": {}
            },
            "Type": {
                "select": {
                    "options": []
                }
            },
            "Year": {
                "number": {}
            },
            "Release Date": {
                "date": {}
            },
            "Runtime": {
                "number": {}
            },
            "Plot": {
                "rich_text": {}
            },
            "Director": {
                "type": "multi_select",
                "multi_select": {
                    "options": []
                }
            },
            "Star": {
                "type": "multi_select",
                "multi_select": {
                    "options": []
                }
            },
            "Genre": {
                "type": "multi_select",
                "multi_select": {
                    "options": []
                }
            },
            "Country": {
                "type": "multi_select",
                "multi_select": {
                    "options": []
                }
            },
            "Language": {
                "type": "multi_select",
                "multi_select": {
                    "options": []
                }
            },
            "Content Rating": {
                "rich_text": {}
            },
            "IMDb Rating": {
                "number": {}
            },
            "IMDb Link": {
                "url": {}
            }
        }
    });

    let response = request_data_from_notion(client, url, token, &body, fb_lang).await?;

    if !response.status().is_success() {
        let error_message: NotionErrorObj = response.json().await.map_err(|e| {
            feedback_error(
                anyhow!(e).context(transcripts::parse_notion_error_message_failed(fb_lang)),
            )
        })?;

        return Err(feedback_error(anyhow!(error_message.message)));
    }

    let database_obj: DatabaseObj = response.json().await.map_err(propagate_error)?;

    Ok(database_obj.id)
}

pub fn parse_notion_page_id_from_user_input(
    input: &str,
    fb_lang: Language,
) -> Result<String, BotError> {
    get_notion_page_id_from_user_input(input)
        .ok_or_else(|| feedback_error(anyhow!(transcripts::invalid_notion_page_url(fb_lang))))
}

fn get_notion_page_id_from_user_input(input: &str) -> Option<String> {
    let token = match Url::parse(input) {
        Ok(url) => {
            let domain = url.domain()?;
            if !domain.ends_with("notion.so") {
                return None;
            }

            url.path_segments().and_then(|sgm| {
                sgm.last()
                    .and_then(|s| s.split('-').last())
                    .map(ToOwned::to_owned)
            })?
        }
        Err(_) => input.to_string(),
    };

    Some(token)
}

pub async fn insert_movie_info_to_notion_database(
    client: &Client,
    token: &str,
    db_id: &str,
    movie_info: &MovieInfo,
    fb_lang: Language,
) -> Result<(), BotError> {
    let url = "https://api.notion.com/v1/pages";
    let body = notion_create_page_body(db_id, movie_info);
    let response = request_data_from_notion(client, url, token, &body, fb_lang).await?;

    if !response.status().is_success() {
        handle_notion_error_response(response, fb_lang).await?;
    }

    Ok(())
}

async fn request_data_from_notion(
    client: &Client,
    url: &str,
    token: &str,
    body: &Value,
    fb_lang: Language,
) -> Result<Response, BotError> {
    client
        .post(url)
        .header("Notion-Version", "2022-06-28")
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .map_err(|e| {
            feedback_propagate_error(
                anyhow!(e).context(transcripts::cannot_reach_server(fb_lang, "Notion")),
            )
        })
}

async fn handle_notion_error_response(
    response: Response,
    fb_lang: Language,
) -> Result<(), BotError> {
    let error_message: NotionErrorObj = response.json().await.map_err(|e| {
        feedback_propagate_error(
            anyhow!(e).context(transcripts::parse_notion_error_message_failed(fb_lang)),
        )
    })?;

    Err(feedback_error(anyhow!(error_message.message)))
}

fn notion_create_page_body(db_id: &str, movie_info: &MovieInfo) -> Value {
    let mut body = new_database_object();

    if !movie_info.image.is_empty() {
        body["cover"] = file_object(&movie_info.image);
    }

    let properties = &mut body["properties"];

    properties["Title"] = title_database_property_object(&movie_info.title);

    if !movie_info.movie_type.is_empty() {
        properties["Type"] = select_database_property_object(&movie_info.movie_type);
    }

    if let Some(year) = movie_info.year {
        properties["Year"] = u32_number_database_property_object(year);
    }

    if let Some(date) = &movie_info.release_date {
        properties["Release Date"] = date_database_property_object(date);
    }

    if let Some(runtime) = movie_info.runtime {
        properties["Runtime"] = u32_number_database_property_object(runtime);
    }

    if !movie_info.plot.is_empty() {
        properties["Plot"] = text_database_property_object(&movie_info.plot);
    }

    if !movie_info.director_list.is_empty() {
        properties["Director"] = multi_select_database_property_object(&movie_info.director_list);
    }

    if !movie_info.star_list.is_empty() {
        properties["Star"] = multi_select_database_property_object(&movie_info.star_list);
    }

    if !movie_info.genre_list.is_empty() {
        properties["Genre"] = multi_select_database_property_object(&movie_info.genre_list);
    }

    if !movie_info.country_list.is_empty() {
        properties["Country"] = multi_select_database_property_object(&movie_info.country_list);
    }

    if !movie_info.language_list.is_empty() {
        properties["Language"] = multi_select_database_property_object(&movie_info.language_list);
    }

    if !movie_info.content_rating.is_empty() {
        properties["Content Rating"] = text_database_property_object(&movie_info.content_rating);
    }

    if let Some(rating) = movie_info.imdb_rating {
        properties["IMDb Rating"] = f64_number_database_property_object(rating);
    }

    properties["IMDb Link"] = url_database_property_object(&movie_info.imdb_link);

    body["parent"] = parent_object(db_id);

    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_notion_token() {
        let input = "https://www.notion.so/xxxx/25195fba545b4a5636d20ae776bf3189?v=d408c958e7c74846a298243fd4334f27";
        assert_eq!(
            get_notion_page_id_from_user_input(input).unwrap(),
            "25195fba545b4a5636d20ae776bf3189"
        );

        let input = "25195fba545b4a5636d20ae776bf3189";
        assert_eq!(
            get_notion_page_id_from_user_input(input).unwrap(),
            "25195fba545b4a5636d20ae776bf3189"
        );

        let input = "https://www.example.com/xxxx/25195fba545b4a5636d20ae776bf3189?v=d408c958e7c74846a298243fd4334f27";
        assert!(get_notion_page_id_from_user_input(input).is_none());

        let input = "https://www.example.com/?v=d408c958e7c74846a298243fd4334f27";
        assert!(get_notion_page_id_from_user_input(input).is_none());
    }
}
