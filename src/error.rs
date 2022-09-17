use anyhow::anyhow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error(transparent)]
    FeedBack(anyhow::Error),
    #[error(transparent)]
    FeedBackPropagate(anyhow::Error),
    #[error(transparent)]
    Propagate(anyhow::Error),
}

pub fn feedback_error(error: impl Into<anyhow::Error>) -> BotError {
    BotError::FeedBack(anyhow!(error))
}

pub fn propagate_error(error: impl Into<anyhow::Error>) -> BotError {
    BotError::Propagate(anyhow!(error))
}

pub fn feedback_propagate_error(error: impl Into<anyhow::Error>) -> BotError {
    BotError::FeedBackPropagate(anyhow!(error))
}
