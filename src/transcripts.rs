#[derive(Debug, Copy, Clone)]
pub enum Language {
    En,
}

impl Default for Language {
    fn default() -> Self {
        Self::En
    }
}

pub fn transcript_welcome(lang: Language) -> &'static str {
    match lang {
        Language::En => {
            "Welcome to work with me! But we need to configure something first.\n\
            Please use:\n\n/help to check more details."
        }
    }
}

pub fn database_error(lang: Language) -> &'static str {
    match lang {
        Language::En => "I got a bad memory...\nPlease let me stay alone for a while.",
    }
}

pub fn message_from_no_one(lang: Language) -> &'static str {
    match lang {
        Language::En => "I don't know who you are, it's better not to talk to strangers",
    }
}

pub fn configure_again(lang: Language) -> &'static str {
    match lang {
        Language::En => "I'm so sorry I forgot who you are, can we /start all over again? 🥺",
    }
}

pub fn input_empty_imdb_token(lang: Language) -> &'static str {
    match lang {
        Language::En => "IMDb API token should follow the /set_imdb_token command.\n e.g. /set_imdb_token abc123",
    }
}

pub fn input_empty_notion_token(lang: Language) -> &'static str {
    match lang {
        Language::En => "Notion token should follow the /set_notion_token command.\n e.g. /set_notion_token abc123",
    }
}

pub fn input_empty_notion_page_id(lang: Language) -> &'static str {
    match lang {
        Language::En => "Notion page ID or its link should follow the /create_notion_db command.\n e.g. /create_notion_db abc123",
    }
}

pub fn input_empty_keyword(lang: Language) -> &'static str {
    match lang {
        Language::En => "Please send me the title",
    }
}

pub fn imdb_token_set_as(lang: Language, token: &str) -> String {
    match lang {
        Language::En => format!("IMDb token has been set as: {}", token),
    }
}

pub fn notion_token_set_as(lang: Language, token: &str) -> String {
    match lang {
        Language::En => format!("Notion token has been set as: {}", token),
    }
}

pub fn notion_database_created(lang: Language) -> &'static str {
    match lang {
        Language::En => "Movie list database ID token has been created",
    }
}

pub fn need_notion_token_first(lang: Language) -> &'static str {
    match lang {
        Language::En => "Please /set_notion_token first.",
    }
}

pub fn no_search_result(lang: Language) -> &'static str {
    match lang {
        Language::En => "No result was found.",
    }
}

pub fn add_to_movie_list(lang: Language) -> &'static str {
    match lang {
        Language::En => "Add to Movie List",
    }
}

pub fn add_to_movie_list_successfully(lang: Language, title: &str) -> String {
    match lang {
        Language::En => format!(
            "<b>{}</b> has been added to your movie list successfully!",
            title
        ),
    }
}

pub fn not_set(lang: Language) -> &'static str {
    match lang {
        Language::En => "not set",
    }
}

pub fn user_hint_title(lang: Language) -> &'static str {
    match lang {
        Language::En => "Please config required settings.",
    }
}

pub fn user_hint_help_command(lang: Language) -> &'static str {
    match lang {
        Language::En => "/help - display this text.",
    }
}

pub fn user_hint_start_command(lang: Language) -> &'static str {
    match lang {
        Language::En => "/start - start to do some work with me.",
    }
}

pub fn user_hint_settings_command(lang: Language) -> &'static str {
    match lang {
        Language::En => "/settings - show your tokens information.",
    }
}

pub fn user_hint_set_imdb_token(lang: Language) -> &'static str {
    match lang {
        Language::En => {
            "/set_imdb_token `token` - set the IMDb API token for getting movie information. \
            If not set, the shared default API token will be used. This token can easily reach the \
            limit of 100 requests per day, please set your own API token."
        }
    }
}

pub fn user_hint_set_notion_token(lang: Language) -> &'static str {
    match lang {
        Language::En => "/set_notion_token `token` - set the Notion internal integration token.",
    }
}

pub fn user_hint_create_notion_database(lang: Language) -> &'static str {
    match lang {
        Language::En => {
            "/create_notion_db `page link or id` - create a Notion database as your movie list."
        }
    }
}

pub fn invalid_notion_page_url(lang: Language) -> &'static str {
    match lang {
        Language::En => "Invalid Notion Page Url",
    }
}

pub fn parse_notion_error_message_failed(lang: Language) -> &'static str {
    match lang {
        Language::En => "Can't understand what's wrong with Notion...",
    }
}

pub fn cannot_reach_server(lang: Language, server_name: &str) -> String {
    match lang {
        Language::En => format!(
            "There's something wrong when I was requesting data from {}",
            server_name
        ),
    }
}

pub fn parse_imdb_api_error_message_failed(lang: Language) -> &'static str {
    match lang {
        Language::En => "Can't understand what's wrong with IMDb-API...",
    }
}

pub fn parse_imdb_api_response_failed(lang: Language) -> &'static str {
    match lang {
        Language::En => "IMDb-API told me bullshit...",
    }
}

pub fn help_message(lang: Language, help_page: &str) -> String {
    match lang {
        Language::En => format!(
            "<b>Supported commands:</b>\n\n{}\n{}\n{}\n{}\n{}\n{}\n\n\
            Please visit <a href=\"{}\"><b>this page</b></a> to get more help.",
            user_hint_help_command(lang),
            user_hint_start_command(lang),
            user_hint_settings_command(lang),
            user_hint_set_imdb_token(lang),
            user_hint_set_notion_token(lang),
            user_hint_create_notion_database(lang),
            help_page
        ),
    }
}
