use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(String),
    #[error("File or directory not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Serial communication error: {0}")]
    SerialError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Database error: {0}")]
    DbError(String),
    #[error("File already exists: {0}")]
    AlreadyExists(String),
    #[error("Error: {0}")]
    General(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::PermissionDenied(e.to_string()),
            _ => AppError::IoError(e.to_string()),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DbError(e.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::General(s.to_string())
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::General(s.to_string())
    }
}
