use crate::error::{feedback_error, feedback_propagate_error, BotError};
use crate::{transcripts, Language};

use anyhow::anyhow;
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Decimal;
use sqlx::PgPool;

#[async_trait]
pub trait BotDatabase {
    async fn init_user_tokens(&self, user_id: u64) -> Result<(), BotError>;

    async fn user_tokens(&self, user_id: u64, fb_lang: Language) -> Result<UserTokens, BotError>;

    async fn get_user_tokens(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<UserTokens>, BotError>;

    async fn imdb_token(&self, user_id: u64, fb_lang: Language) -> Result<String, BotError>;

    async fn get_imdb_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<String>, BotError>;

    async fn notion_integration_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<String, BotError>;

    async fn get_notion_integration_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<String>, BotError>;

    async fn store_imdb_token(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError>;

    async fn store_notion_token(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError>;

    async fn store_notion_database_id(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError>;

    async fn remove_user_tokens(&self, user_id: u64, fb_lang: Language) -> Result<bool, BotError>;
}

#[derive(Debug, Clone)]
pub struct PgBotDatabase {
    pg_pool: PgPool,
}

impl PgBotDatabase {
    pub async fn connect(db_url: &str) -> anyhow::Result<Self> {
        let pg_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        Ok(Self { pg_pool })
    }

    async fn insert_default_user_tokens(&self, user_id: Decimal) -> anyhow::Result<()> {
        sqlx::query(
            r#"
INSERT INTO user_tokens ( user_id, imdb_token, notion_token, notion_database_id )
VALUES ( $1, $2, $3, $4 )
        "#,
        )
        .bind(user_id)
        .bind("")
        .bind("")
        .bind("")
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }

    async fn select_user_tokens(&self, user_id: Decimal) -> anyhow::Result<Option<UserTokens>> {
        let user_token = sqlx::query_as(
            r#"
SELECT *
FROM user_tokens
WHERE user_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pg_pool)
        .await?;

        Ok(user_token)
    }

    async fn select_imdb_token(&self, user_id: Decimal) -> anyhow::Result<Option<String>> {
        let imdb_token = sqlx::query_as::<_, (String,)>(
            r#"
SELECT imdb_token
FROM user_tokens
WHERE user_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pg_pool)
        .await?
        .map(|t| t.0);

        Ok(imdb_token)
    }

    async fn select_notion_integration_token(
        &self,
        user_id: Decimal,
    ) -> anyhow::Result<Option<String>> {
        let imdb_token = sqlx::query_as::<_, (String,)>(
            r#"
SELECT notion_token
FROM user_tokens
WHERE user_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pg_pool)
        .await?
        .map(|t| t.0);

        Ok(imdb_token)
    }

    async fn update_imdb_token(&self, user_id: Decimal, token: &str) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
UPDATE user_tokens
SET imdb_token = $1
WHERE user_id = $2
        "#,
        )
        .bind(token)
        .bind(user_id)
        .execute(&self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn update_notion_token(&self, user_id: Decimal, token: &str) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
UPDATE user_tokens
SET notion_token = $1
WHERE user_id = $2
        "#,
        )
        .bind(token)
        .bind(user_id)
        .execute(&self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn update_notion_database_id(
        &self,
        user_id: Decimal,
        token: &str,
    ) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
UPDATE user_tokens
SET notion_database_id = $1
WHERE user_id = $2
        "#,
        )
        .bind(token)
        .bind(user_id)
        .execute(&self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn delete_user_tokens(&self, user_id: Decimal) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query(
            r#"
DELETE FROM user_tokens
WHERE user_id = $1
        "#,
        )
        .bind(user_id)
        .execute(&self.pg_pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    #[allow(dead_code)]
    pub async fn reset_user_tokens(&self, user_id: u64, fb_lang: Language) -> Result<(), BotError> {
        self.remove_user_tokens(user_id, fb_lang).await?;
        self.init_user_tokens(user_id).await
    }
}

#[async_trait]
impl BotDatabase for PgBotDatabase {
    async fn init_user_tokens(&self, user_id: u64) -> Result<(), BotError> {
        self.insert_default_user_tokens(user_id.into())
            .await
            .map_err(|e| {
                feedback_propagate_error(
                    e.context(transcripts::database_error(Language::default())),
                )
            })
    }

    async fn user_tokens(&self, user_id: u64, fb_lang: Language) -> Result<UserTokens, BotError> {
        self.get_user_tokens(user_id, fb_lang)
            .await?
            .ok_or_else(|| feedback_error(anyhow!(transcripts::configure_again(fb_lang))))
    }

    async fn get_user_tokens(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<UserTokens>, BotError> {
        self.select_user_tokens(user_id.into())
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn imdb_token(&self, user_id: u64, fb_lang: Language) -> Result<String, BotError> {
        self.get_imdb_token(user_id, fb_lang)
            .await?
            .ok_or_else(|| feedback_error(anyhow!(transcripts::configure_again(fb_lang))))
    }

    async fn get_imdb_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<String>, BotError> {
        self.select_imdb_token(user_id.into())
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn notion_integration_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<String, BotError> {
        self.get_notion_integration_token(user_id, fb_lang)
            .await?
            .ok_or_else(|| feedback_error(anyhow!(transcripts::configure_again(fb_lang))))
    }

    async fn get_notion_integration_token(
        &self,
        user_id: u64,
        fb_lang: Language,
    ) -> Result<Option<String>, BotError> {
        self.select_notion_integration_token(user_id.into())
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn store_imdb_token(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError> {
        self.update_imdb_token(user_id.into(), token)
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn store_notion_token(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError> {
        self.update_notion_token(user_id.into(), token)
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn store_notion_database_id(
        &self,
        user_id: u64,
        token: &str,
        fb_lang: Language,
    ) -> Result<bool, BotError> {
        self.update_notion_database_id(user_id.into(), token)
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }

    async fn remove_user_tokens(&self, user_id: u64, fb_lang: Language) -> Result<bool, BotError> {
        self.delete_user_tokens(user_id.into())
            .await
            .map_err(|e| feedback_propagate_error(e.context(transcripts::database_error(fb_lang))))
    }
}

#[derive(sqlx::FromRow)]
pub struct UserTokens {
    pub user_id: Decimal,
    pub imdb_token: String,
    #[sqlx(flatten)]
    pub notion_token: NotionToken,
}

impl UserTokens {
    pub fn notion_token_is_good(&self) -> bool {
        !(self.notion_token.integration_token.is_empty()
            || self.notion_token.database_id.is_empty())
    }

    pub fn user_hint(&self, lang: Language) -> String {
        let mut hint_msg = format!("{}\n", transcripts::user_hint_title(lang));

        if self.notion_token.integration_token.is_empty() {
            hint_msg += "\n";
            hint_msg += transcripts::user_hint_set_notion_token(lang);
        }

        if self.notion_token.database_id.is_empty() {
            hint_msg += "\n";
            hint_msg += transcripts::user_hint_create_notion_database(lang);
        }

        hint_msg
    }

    pub fn summary(&self, lang: Language) -> String {
        format!(
            "Tokens:\nIMDb token: {}\nNotion token:{}\nNotion database ID: {}",
            token_to_string(&self.imdb_token, lang),
            token_to_string(&self.notion_token.integration_token, lang),
            token_to_string(&self.notion_token.database_id, lang)
        )
    }
}

fn token_to_string(s: &str, lang: Language) -> &str {
    if s.is_empty() {
        transcripts::not_set(lang)
    } else {
        s
    }
}

#[derive(sqlx::FromRow)]
pub struct NotionToken {
    #[sqlx(rename = "notion_token")]
    pub integration_token: String,
    #[sqlx(rename = "notion_database_id")]
    pub database_id: String,
}
