use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DieError {
    #[error("finding not found: {0}")]
    FindingNotFound(Uuid),

    #[error("invalid message")]
    InvalidMessage,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type DieResult<T> = Result<T, DieError>;