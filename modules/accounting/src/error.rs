use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountingError {
    #[error("{0}")]
    Message(String),

    #[error("identity not found")]
    IdentityNotFound,

    #[error("identity disabled")]
    IdentityDisabled,

    #[error("database unavailable")]
    DbUnavailable,
}

pub type AccountingResult<T> = Result<T, AccountingError>;