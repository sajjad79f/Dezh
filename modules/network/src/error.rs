use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("{0}")]
    Internal(String),
    #[error("not found: {0}")]
    NotFound(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;