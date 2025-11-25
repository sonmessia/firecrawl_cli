use std::fmt;
use thiserror::Error;

/// Domain-specific errors for storage operations
#[derive(Error, Debug, Clone)]
pub enum StorageError {
    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("File already exists: {0}")]
    FileExists(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Content type not supported: {0}")]
    UnsupportedContentType(String),
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::FileSystem(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::Serialization(err.to_string())
    }
}

pub type StorageResult<T> = Result<T, StorageError>;