#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use thiserror::Error;

/// Infrastructure-specific error types
#[derive(Error, Debug, PartialEq)]
pub enum InfrastructureError {
    #[error("File not found at path '{path}'")]
    FileNotFound { path: String },

    #[error("Error reading file at path '{path}': {details}")]
    FileReadError { path: String, details: String },

    #[error("Error writing file at path '{path}': {details}")]
    FileWriteError { path: String, details: String },

    #[error("Error parsing {format} file at path '{path}': {details}")]
    FileParseError {
        path: String,
        format: String,
        details: String,
    },

    #[error("Directory not found at path '{path}'")]
    DirectoryNotFound { path: String },

    #[error("Error creating directory at path '{path}': {details}")]
    DirectoryCreateError { path: String, details: String },

    #[error("Invalid path '{path}': {reason}")]
    PathInvalid { path: String, reason: String },

    #[error("Serialization error for format '{format}': {details}")]
    SerializationError { format: String, details: String },

    #[error("Deserialization error for format '{format}': {details}")]
    DeserializationError { format: String, details: String },

    #[error("Network error during {operation}: {details}")]
    NetworkError { operation: String, details: String },

    #[error("Database error during {operation}: {details}")]
    DatabaseError { operation: String, details: String },

    #[error("Cache error during {operation}: {details}")]
    CacheError { operation: String, details: String },
}

impl From<InfrastructureError> for DomainError {
    fn from(err: InfrastructureError) -> Self {
        match err {
            InfrastructureError::FileNotFound { path } => DomainError::IoWithPath {
                operation: "file read".to_string(),
                path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            },
            InfrastructureError::FileReadError { path, details } => DomainError::IoWithPath {
                operation: "file read".to_string(),
                path,
                source: std::io::Error::other(details),
            },
            InfrastructureError::FileWriteError { path, details } => DomainError::IoWithPath {
                operation: "file write".to_string(),
                path,
                source: std::io::Error::other(details),
            },
            InfrastructureError::FileParseError { path, format, details } => DomainError::Serialization {
                format,
                details: format!("Parse error at path '{}': {}", path, details),
                source: None,
            },
            InfrastructureError::DirectoryNotFound { path } => DomainError::IoWithPath {
                operation: "directory access".to_string(),
                path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "Directory not found"),
            },
            InfrastructureError::DirectoryCreateError { path, details } => DomainError::IoWithPath {
                operation: "directory creation".to_string(),
                path,
                source: std::io::Error::other(details),
            },
            InfrastructureError::PathInvalid { path, reason } => DomainError::ValidationError {
                field: "path".to_string(),
                message: format!("Path '{}' is invalid: {}", path, reason),
            },
            InfrastructureError::SerializationError { format, details } => DomainError::Serialization {
                format,
                details,
                source: None,
            },
            InfrastructureError::DeserializationError { format, details } => DomainError::Serialization {
                format,
                details,
                source: None,
            },
            InfrastructureError::NetworkError { operation, details } => {
                DomainError::RepositoryError { operation, details }
            }
            InfrastructureError::DatabaseError { operation, details } => {
                DomainError::RepositoryError { operation, details }
            }
            InfrastructureError::CacheError { operation, details } => {
                DomainError::RepositoryError { operation, details }
            }
        }
    }
}

// Result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infrastructure_error_display() {
        let error = InfrastructureError::FileNotFound {
            path: "/test/path".to_string(),
        };
        assert!(error.to_string().contains("File not found at path '/test/path'"));
    }

    #[test]
    fn test_infrastructure_error_conversion_to_domain_error() {
        let infra_error = InfrastructureError::FileNotFound {
            path: "/test/path".to_string(),
        };
        let domain_error: DomainError = infra_error.into();
        assert!(matches!(domain_error, DomainError::IoWithPath { path, .. } if path == "/test/path"));
    }

    #[test]
    fn test_infrastructure_error_serialization() {
        let error = InfrastructureError::SerializationError {
            format: "JSON".to_string(),
            details: "Invalid data".to_string(),
        };
        assert!(error.to_string().contains("Serialization error for format 'JSON'"));
    }

    #[test]
    fn test_infrastructure_result() {
        let result: InfrastructureResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
    }
}
