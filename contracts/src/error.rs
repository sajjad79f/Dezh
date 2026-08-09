use thiserror::Error;

#[derive(Debug, Error)]
pub enum DezhError {

    #[error("Module already registered: {0}")]
    ModuleAlreadyRegistered(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}