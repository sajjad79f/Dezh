use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DdeError {
    #[error("decision not found: {0}")]
    DecisionNotFound(Uuid),

    #[error("invalid transition for decision {0}")]
    InvalidTransition(Uuid),

    #[error("invalid description")]
    InvalidDescription,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type DdeResult<T> = Result<T, DdeError>;