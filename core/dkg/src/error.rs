use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DkgError {
    #[error("node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("edge not found: {0}")]
    EdgeNotFound(Uuid),

    #[error("invalid label")]
    InvalidLabel,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type DkgResult<T> = Result<T, DkgError>;