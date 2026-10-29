//! Error types for the RPG Maker MV/MZ decryption library.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for operations in this crate.
pub type Result<T> = std::result::Result<T, DecryptionError>;

/// Errors that can occur during decryption, encryption, key detection, or batch processing.
#[derive(Debug, Error)]
pub enum DecryptionError {
    #[error("I/O error at '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    GeneralIo(#[from] std::io::Error),

    #[error("Failed to decode hexadecimal key: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("File is too short: expected at least {min_expected} bytes, found {found} bytes")]
    FileTooShort { min_expected: usize, found: usize },

    #[error("Invalid RPG Maker fake header: expected '{expected}', found '{found}'")]
    InvalidFakeHeader { expected: String, found: String },

    #[error("Invalid header length: {0} bytes (must be at least 1)")]
    InvalidHeaderLength(usize),

    #[error("Decryption key was not provided and could not be detected")]
    MissingKey,

    #[error("Encryption key field '{searched_field}' not found in System.json")]
    KeyNotFoundInJson { searched_field: String },

    #[error("No encrypted PNG image found to derive encryption key")]
    NoEncryptedImageFound,

    #[error("Specified path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("Unsupported file extension for encrypted asset: '{0}'")]
    UnsupportedExtension(String),

    #[error(
        "Project verification failed: '{0}' does not appear to be a valid RPG Maker directory"
    )]
    InvalidProjectDirectory(PathBuf),

    #[error("{0}")]
    Other(String),
}

impl DecryptionError {
    /// Helper to attach a path to an `io::Error`.
    pub fn io_with_path(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
