use lazy_static::lazy_static;

lazy_static! {
    pub static ref CONFIG: Config = Config::from_env();
}

pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub default_imdb_api_key: String,
    pub help_page: String,
}

impl Config {
    pub fn from_env() -> Self {
        let bot_token = std::env::var("TG_BOT_TOKEN")
            .expect("can't find `TG_BOT_TOKEN` in environment variables.");
        let database_url = std::env::var("DATABASE_URL")
            .expect("can't find `DATABASE_URL` in environment variables.");
        let host = std::env::var("HOST").expect("`HOST` env variable is not set");
        let port = std::env::var("PORT")
            .expect("can't find `PORT` in environment variables")
            .parse()
            .expect("`PORT` is not an integer");
        let default_imdb_api_key = std::env::var("DEFAULT_IMDB_API_KEY")
            .expect("can't find `DEFAULT_IMDB_API_KEY` in environment variables.");
        let help_page = std::env::var("HELP_PAGE").unwrap_or_else(|_| {
            "https://www.notion.so/octocat/ca61deb6472a4c73b9b43b0ecd549397".into()
        });

        Self {
            bot_token,
            database_url,
            host,
            port,
            default_imdb_api_key,
            help_page,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn test_parse_config() {
        std::env::set_var("TG_BOT_TOKEN", "a");
        std::env::set_var("DATABASE_URL", "b");
        std::env::set_var("HOST", "c");
        std::env::set_var("PORT", "80");
        std::env::set_var("DEFAULT_IMDB_API_KEY", "d");
        std::env::set_var("HELP_PAGE", "e");

        let config = Config::from_env();
        assert_eq!(config.bot_token, "a".to_string());
        assert_eq!(config.database_url, "b".to_string());
        assert_eq!(config.host, "c".to_string());
        assert_eq!(config.port, 80);
        assert_eq!(config.default_imdb_api_key, "d".to_string());
        assert_eq!(config.help_page, "e".to_string());
    }
}
