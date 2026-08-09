use thiserror::Error;

#[derive(Debug, Error)]

pub enum DefError {

    #[error("no handler registered for topic '{0}'")]

    NoHandler(String),
}