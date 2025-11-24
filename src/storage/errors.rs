use std::fmt;
use thiserror::Error;

/// Domain-specific errors for storage operations
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("File already exists: {0}")]
    FileExists(String),

    #[error("Content type not supported: {0}")]
    UnsupportedContentType(String),
}

pub type StorageResult<T> = Result<T, StorageError>;