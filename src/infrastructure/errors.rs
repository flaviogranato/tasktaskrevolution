use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Infrastructure-specific error types
#[derive(Debug)]
pub enum InfrastructureError {
    FileNotFound { path: String },
    FileReadError { path: String, details: String },
    FileWriteError { path: String, details: String },
    FileParseError { path: String, format: String, details: String },
    DirectoryNotFound { path: String },
    DirectoryCreateError { path: String, details: String },
    PathInvalid { path: String, reason: String },
    SerializationError { format: String, details: String },
    DeserializationError { format: String, details: String },
    NetworkError { operation: String, details: String },
    DatabaseError { operation: String, details: String },
    CacheError { operation: String, details: String },
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfrastructureError::FileNotFound { path } => {
                write!(f, "File not found at path '{}'", path)
            }
            InfrastructureError::FileReadError { path, details } => {
                write!(f, "Error reading file at path '{}': {}", path, details)
            }
            InfrastructureError::FileWriteError { path, details } => {
                write!(f, "Error writing file at path '{}': {}", path, details)
            }
            InfrastructureError::FileParseError { path, format, details } => {
                write!(f, "Error parsing {} file at path '{}': {}", format, path, details)
            }
            InfrastructureError::DirectoryNotFound { path } => {
                write!(f, "Directory not found at path '{}'", path)
            }
            InfrastructureError::DirectoryCreateError { path, details } => {
                write!(f, "Error creating directory at path '{}': {}", path, details)
            }
            InfrastructureError::PathInvalid { path, reason } => {
                write!(f, "Invalid path '{}': {}", path, reason)
            }
            InfrastructureError::SerializationError { format, details } => {
                write!(f, "Serialization error for format '{}': {}", format, details)
            }
            InfrastructureError::DeserializationError { format, details } => {
                write!(f, "Deserialization error for format '{}': {}", format, details)
            }
            InfrastructureError::NetworkError { operation, details } => {
                write!(f, "Network error during {}: {}", operation, details)
            }
            InfrastructureError::DatabaseError { operation, details } => {
                write!(f, "Database error during {}: {}", operation, details)
            }
            InfrastructureError::CacheError { operation, details } => {
                write!(f, "Cache error during {}: {}", operation, details)
            }
        }
    }
}

impl StdError for InfrastructureError {}

impl From<InfrastructureError> for DomainError {
    fn from(err: InfrastructureError) -> Self {
        match err {
            InfrastructureError::FileNotFound { path } => {
                DomainError::new(DomainErrorKind::Io {
                    operation: "file read".to_string(),
                    path: Some(path),
                })
            }
            InfrastructureError::FileReadError { path, details } => {
                DomainError::new(DomainErrorKind::Io {
                    operation: "file read".to_string(),
                    path: Some(path),
                }).with_context(details)
            }
            InfrastructureError::FileWriteError { path, details } => {
                DomainError::new(DomainErrorKind::Io {
                    operation: "file write".to_string(),
                    path: Some(path),
                }).with_context(details)
            }
            InfrastructureError::FileParseError { path, format, details } => {
                DomainError::new(DomainErrorKind::Serialization {
                    format,
                    details: format!("Parse error at path '{}': {}", path, details),
                })
            }
            InfrastructureError::DirectoryNotFound { path } => {
                DomainError::new(DomainErrorKind::Io {
                    operation: "directory access".to_string(),
                    path: Some(path),
                })
            }
            InfrastructureError::DirectoryCreateError { path, details } => {
                DomainError::new(DomainErrorKind::Io {
                    operation: "directory creation".to_string(),
                    path: Some(path),
                }).with_context(details)
            }
            InfrastructureError::PathInvalid { path, reason } => {
                DomainError::new(DomainErrorKind::ValidationError {
                    field: "path".to_string(),
                    message: format!("Path '{}' is invalid: {}", path, reason),
                })
            }
            InfrastructureError::SerializationError { format, details } => {
                DomainError::new(DomainErrorKind::Serialization { format, details })
            }
            InfrastructureError::DeserializationError { format, details } => {
                DomainError::new(DomainErrorKind::Serialization { format, details })
            }
            InfrastructureError::NetworkError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
            InfrastructureError::DatabaseError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
            InfrastructureError::CacheError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
        }
    }
}

// Result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;
