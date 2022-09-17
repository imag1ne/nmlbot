use crate::api::*;
use crate::bot::{get_user_from_msg, BotWork, Database, HandlerResult, MovieInfoApi};
use crate::error::{feedback_error, propagate_error};
use crate::{transcripts, Language};

use anyhow::anyhow;
use reqwest::Client;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::Bot;

use crate::config::CONFIG;

// start command handler
pub async fn start(bot: AutoSend<Bot>, msg: Message, database: Database) -> HandlerResult {
    BotWork::new(&bot, msg.chat.id)
        .do_it(async {
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            // check if user's config is already in database
            let user_tokens = database.get_user_tokens(user_id, lang).await?;

            if user_tokens.is_none() {
                database.init_user_tokens(user_id).await?;
            }

            bot.send_message(msg.chat.id, transcripts::transcript_welcome(lang))
                .await
                .map_err(propagate_error)?;

            Ok(())
        })
        .await
}

pub async fn help(bot: AutoSend<Bot>, msg: Message) -> HandlerResult {
    let help_text = transcripts::help_message(Language::default(), &CONFIG.help_page);
    bot.send_message(msg.chat.id, help_text)
        .parse_mode(ParseMode::Html)
        .await?;
    Ok(())
}

pub async fn settings(bot: AutoSend<Bot>, msg: Message, database: Database) -> HandlerResult {
    BotWork::new(&bot, msg.chat.id)
        .do_it(async {
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            let user_tokens = database.user_tokens(user_id, lang).await?;

            bot.send_message(msg.chat.id, user_tokens.summary(lang))
                .await
                .map_err(propagate_error)?;

            Ok(())
        })
        .await
}

pub async fn set_imdb_token(
    bot: AutoSend<Bot>,
    msg: Message,
    token: String,
    database: Database,
) -> HandlerResult {
    BotWork::new(&bot, msg.chat.id)
        .do_it(async {
            let token = token.trim();
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            if token.is_empty() {
                return Err(feedback_error(anyhow!(
                    transcripts::input_empty_imdb_token(lang)
                )));
            }

            let success = database.store_imdb_token(user_id, token, lang).await?;

            let reply_text = success
                .then(|| transcripts::imdb_token_set_as(lang, token))
                .ok_or_else(|| {
                    feedback_error(anyhow!(transcripts::configure_again(lang).to_string()))
                })?;

            bot.send_message(msg.chat.id, reply_text)
                .await
                .map_err(propagate_error)?;

            Ok(())
        })
        .await
}

pub async fn set_notion_token(
    bot: AutoSend<Bot>,
    msg: Message,
    token: String,
    database: Database,
) -> HandlerResult {
    BotWork::new(&bot, msg.chat.id)
        .do_it(async {
            let token = token.trim();
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            if token.is_empty() {
                return Err(feedback_error(anyhow!(
                    transcripts::input_empty_notion_token(lang)
                )));
            }

            let success = database.store_notion_token(user_id, token, lang).await?;

            let reply_text = success
                .then(|| transcripts::notion_token_set_as(lang, token))
                .ok_or_else(|| {
                    feedback_error(anyhow!(transcripts::configure_again(lang).to_string()))
                })?;

            bot.send_message(msg.chat.id, reply_text)
                .await
                .map_err(propagate_error)?;

            Ok(())
        })
        .await
}

pub async fn handle_notion_page_link_or_id(
    bot: AutoSend<Bot>,
    msg: Message,
    page_id: String,
    database: Database,
    client: Client,
) -> HandlerResult {
    let create_notion_database_on_page = BotWork::new(&bot, msg.chat.id);
    create_notion_database_on_page
        .do_it(async {
            let page_id = page_id.trim();
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            if page_id.is_empty() {
                return Err(feedback_error(anyhow!(
                    transcripts::input_empty_notion_page_id(lang)
                )));
            }

            let notion_token = database.notion_integration_token(user_id, lang).await?;

            if notion_token.is_empty() {
                return Err(feedback_error(anyhow!(
                    transcripts::need_notion_token_first(lang)
                )));
            }

            let page_id = parse_notion_page_id_from_user_input(page_id, lang)?;
            let db_id = create_database(&client, &notion_token, &page_id, lang).await?;
            let success = database
                .store_notion_database_id(user_id, &db_id, lang)
                .await?;

            let reply_text = success
                .then(|| transcripts::notion_database_created(lang))
                .ok_or_else(|| feedback_error(anyhow!(transcripts::configure_again(lang))))?;

            bot.send_message(msg.chat.id, reply_text)
                .await
                .map_err(propagate_error)?;

            Ok(())
        })
        .await
}

pub async fn receive_keyword(
    bot: AutoSend<Bot>,
    msg: Message,
    database: Database,
    client: Client,
    movie_info_api: MovieInfoApi,
) -> HandlerResult {
    let search_on_imdb = BotWork::new(&bot, msg.chat.id);

    search_on_imdb
        .do_it(async {
            let user = get_user_from_msg(&msg)?;
            let user_id = user.id.0;

            let lang = Language::default();
            let user_tokens = database.user_tokens(user_id, lang).await?;

            if !user_tokens.notion_token_is_good() {
                return Err(feedback_error(anyhow!(user_tokens.user_hint(lang))));
            }

            let title = msg
                .text()
                .ok_or_else(|| feedback_error(anyhow!(transcripts::input_empty_keyword(lang))))?
                .trim();
            let search_results = movie_info_api
                .search(&client, &user_tokens.imdb_token, title, 5)
                .await?;

            if search_results.is_empty() {
                bot.send_message(
                    msg.chat.id,
                    format!("<b>{}</b>", transcripts::no_search_result(lang)),
                )
                .parse_mode(ParseMode::Html)
                .await
                .map_err(propagate_error)?;

                return Ok(());
            }

            for search_result in search_results.into_iter().rev() {
                let button = InlineKeyboardButton::callback(
                    transcripts::add_to_movie_list(lang),
                    search_result.id.clone(),
                );

                bot.send_message(
                    msg.chat.id,
                    format!(
                        "<a href=\"https://www.imdb.com/title/{}\"><b>{}</b></a>",
                        search_result.id, search_result.title
                    ),
                )
                .parse_mode(ParseMode::Html)
                .reply_markup(InlineKeyboardMarkup::new([[button]]))
                .await
                .map_err(propagate_error)?;
            }

            Ok(())
        })
        .await
}

pub async fn receive_item_selection(
    bot: AutoSend<Bot>,
    q: CallbackQuery,
    database: Database,
    client: Client,
    movie_info_api: MovieInfoApi,
) -> HandlerResult {
    if let Some(imdb_id) = &q.data {
        if let Some(msg) = &q.message {
            let add_to_movie_list = BotWork::new(&bot, msg.chat.id);
            add_to_movie_list
                .do_it(async {
                    let user = &q.from;
                    let user_id = user.id.0;

                    let lang = Language::default();
                    let user_tokens = database.user_tokens(user_id, lang).await?;

                    if !user_tokens.notion_token_is_good() {
                        return Err(feedback_error(anyhow!(user_tokens.user_hint(lang))));
                    }

                    let movie_info = movie_info_api
                        .request_movie_information(&client, &user_tokens.imdb_token, imdb_id, lang)
                        .await?;

                    insert_movie_info_to_notion_database(
                        &client,
                        &user_tokens.notion_token.integration_token,
                        &user_tokens.notion_token.database_id,
                        &movie_info,
                        lang,
                    )
                    .await?;

                    let message =
                        transcripts::add_to_movie_list_successfully(lang, &movie_info.title);
                    bot.send_message(msg.chat.id, message)
                        .parse_mode(ParseMode::Html)
                        .await
                        .map_err(propagate_error)?;

                    Ok(())
                })
                .await?
        }
    }

    Ok(())
}
