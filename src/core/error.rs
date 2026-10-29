use crate::crypto::error::DecryptionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpgError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error in {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid game directory: {0}")]
    InvalidGameDir(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Casing error: {0}")]
    Casing(String),

    #[error("Setup error: {0}")]
    Setup(String),

    #[error("Optimizer error: {0}")]
    Optimizer(String),

    #[error("General error: {0}")]
    General(String),
}

impl From<DecryptionError> for RpgError {
    fn from(err: DecryptionError) -> Self {
        RpgError::Crypto(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RpgError>;
