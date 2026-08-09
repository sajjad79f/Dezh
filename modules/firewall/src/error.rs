use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum FirewallError {
    #[error("rule not found: {0}")]
    RuleNotFound(Uuid),

    #[error("invalid rule: {0}")]
    InvalidRule(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type FirewallResult<T> = Result<T, FirewallError>;