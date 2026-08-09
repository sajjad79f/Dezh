use thiserror::Error;

#[derive(Debug, Error)]
pub enum DefError {
    #[error("topic not found: {0}")]
    TopicNotFound(String),

    #[error("no subscriber registered")]
    NoSubscriber,

    #[error("event validation failed: {0}")]
    Validation(String),

    #[error("middleware failed: {0}")]
    Middleware(String),

    #[error("dispatch failed: {0}")]
    Dispatch(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type DefResult<T> = Result<T, DefError>;