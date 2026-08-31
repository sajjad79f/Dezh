use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("invalid route: {0}")]
    Invalid(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub type RoutingResult<T> = Result<T, RoutingError>;