use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("configuration error: {0}")]
    Config(String),
}

pub type StorageResult<T> = Result<T, StorageError>;