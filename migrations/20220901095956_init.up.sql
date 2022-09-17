CREATE TABLE IF NOT EXISTS user_tokens
(
    user_id NUMERIC PRIMARY KEY,
    imdb_token TEXT DEFAULT '' NOT NULL,
    notion_token TEXT DEFAULT '' NOT NULL,
    notion_database_id TEXT DEFAULT '' NOT NULL
)