use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DaiError {
    #[error("asset not found: {0}")]
    AssetNotFound(Uuid),

    #[error("asset already exists: {0}")]
    AssetAlreadyExists(String),

    #[error("invalid asset name")]
    InvalidName,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type DaiResult<T> = Result<T, DaiError>;