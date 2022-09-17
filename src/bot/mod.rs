mod handler;

use crate::api::{ImdbApi, RequestMovieInfo};
use crate::bot::handler::*;
use crate::config::CONFIG;
use crate::db::{BotDatabase, PgBotDatabase};
use crate::error::{feedback_error, BotError};
use crate::{transcripts, Language};

use anyhow::anyhow;
use reqwest::{Client, Url};
use std::future::Future;
use std::sync::Arc;
use teloxide::dispatching::update_listeners::webhooks;
use teloxide::types::User;
use teloxide::{dispatching::UpdateHandler, prelude::*, utils::command::BotCommands};

type HandlerResult = Result<(), anyhow::Error>;
type Database = Arc<dyn BotDatabase + Send + Sync>;
type MovieInfoApi = Arc<dyn RequestMovieInfo + Send + Sync>;

#[derive(BotCommands, Clone)]
#[command(rename = "snake_case", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start to do some work with me.")]
    Start,
    #[command(description = "see your tokens information.")]
    Settings,
    #[command(description = "set the IMDb api token for getting movie information.")]
    SetImdbToken(String),
    #[command(description = "set the Notion internal integration token.")]
    SetNotionToken(String),
    #[command(
        description = "create a database for your movie list. Give me your page ID or its link"
    )]
    CreateNotionDb(String),
}

pub async fn start_bot() {
    let bot_token = &CONFIG.bot_token;
    let bot = Bot::new(bot_token).auto_send();
    let addr = ([0, 0, 0, 0], CONFIG.port).into();
    let url = Url::parse(&format!("https://{}/webhooks/{bot_token}", &CONFIG.host)).unwrap();

    let pool = Arc::new(
        PgBotDatabase::connect(&CONFIG.database_url)
            .await
            .expect("failed to connect to database"),
    ) as Database;
    let client = Client::new();
    let movie_info_api =
        Arc::new(ImdbApi::new(CONFIG.default_imdb_api_key.to_string())) as MovieInfoApi;

    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![pool, client, movie_info_api])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, LoggingErrorHandler::new())
        .await;
}

fn schema() -> UpdateHandler<anyhow::Error> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(help))
        .branch(case![Command::Start].endpoint(start))
        .branch(case![Command::Settings].endpoint(settings))
        .branch(case![Command::SetImdbToken(imdb_token)].endpoint(set_imdb_token))
        .branch(case![Command::SetNotionToken(notion_token)].endpoint(set_notion_token))
        .branch(case![Command::CreateNotionDb(page_link)].endpoint(handle_notion_page_link_or_id));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(dptree::endpoint(receive_keyword));

    let callback_query_handler =
        Update::filter_callback_query().branch(dptree::endpoint(receive_item_selection));

    dptree::entry()
        .branch(message_handler)
        .branch(callback_query_handler)
}

pub struct BotWork<'a> {
    bot: &'a AutoSend<Bot>,
    chat_id: ChatId,
}

impl<'a> BotWork<'a> {
    pub fn new(bot: &'a AutoSend<Bot>, chat_id: ChatId) -> Self {
        Self { bot, chat_id }
    }

    pub async fn do_it<T>(&self, job: T) -> HandlerResult
    where
        T: Future<Output = Result<(), BotError>>,
    {
        let result = job.await;
        handle_work_result(result, self.bot, self.chat_id).await
    }
}

async fn handle_work_result(
    result: Result<(), BotError>,
    bot: &AutoSend<Bot>,
    chat_id: ChatId,
) -> HandlerResult {
    if let Err(err) = result {
        match err {
            BotError::FeedBack(e) => {
                bot.send_message(chat_id, e.to_string()).await?;
                Ok(())
            }
            BotError::FeedBackPropagate(e) => {
                bot.send_message(chat_id, e.to_string()).await?;

                if let Some(source) = e.source() {
                    Err(anyhow!(source.to_string()))
                } else {
                    Err(e)
                }
            }
            BotError::Propagate(e) => Err(e),
        }
    } else {
        Ok(())
    }
}

pub fn get_user_from_msg(msg: &Message) -> Result<&User, BotError> {
    msg.from().ok_or_else(|| {
        feedback_error(anyhow!(transcripts::message_from_no_one(
            Language::default()
        )))
    })
}
