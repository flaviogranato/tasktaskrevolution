use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Company settings specific error types
#[derive(Debug)]
pub enum CompanySettingsError {
    ConfigurationNotFound { path: String },
    ConfigurationInvalid { field: String, value: String, reason: String },
    ConfigurationMissing { field: String },
    ManagerNotFound { identifier: String },
    InvalidManagerData { field: String, reason: String },
    RepositoryInitializationFailed { reason: String },
    FileSystemError { operation: String, path: String, details: String },
}

impl fmt::Display for CompanySettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompanySettingsError::ConfigurationNotFound { path } => {
                write!(f, "Configuration not found at path '{}'", path)
            }
            CompanySettingsError::ConfigurationInvalid { field, value, reason } => {
                write!(f, "Invalid configuration for field '{}' with value '{}': {}", field, value, reason)
            }
            CompanySettingsError::ConfigurationMissing { field } => {
                write!(f, "Missing configuration for field '{}'", field)
            }
            CompanySettingsError::ManagerNotFound { identifier } => {
                write!(f, "Manager not found with identifier '{}'", identifier)
            }
            CompanySettingsError::InvalidManagerData { field, reason } => {
                write!(f, "Invalid manager data for field '{}': {}", field, reason)
            }
            CompanySettingsError::RepositoryInitializationFailed { reason } => {
                write!(f, "Repository initialization failed: {}", reason)
            }
            CompanySettingsError::FileSystemError { operation, path, details } => {
                write!(f, "File system error during {} on path '{}': {}", operation, path, details)
            }
        }
    }
}

impl StdError for CompanySettingsError {}

impl From<CompanySettingsError> for DomainError {
    fn from(err: CompanySettingsError) -> Self {
        match err {
            CompanySettingsError::ConfigurationNotFound { path } => {
                DomainError::new(DomainErrorKind::ConfigurationMissing { field: path })
            }
            CompanySettingsError::ConfigurationInvalid { field, value, reason } => {
                DomainError::new(DomainErrorKind::ConfigurationInvalid { field, value })
                    .with_context(reason)
            }
            CompanySettingsError::ConfigurationMissing { field } => {
                DomainError::new(DomainErrorKind::ConfigurationMissing { field })
            }
            CompanySettingsError::ManagerNotFound { identifier } => {
                DomainError::new(DomainErrorKind::ResourceNotFound { code: identifier })
            }
            CompanySettingsError::InvalidManagerData { field, reason } => {
                DomainError::new(DomainErrorKind::ValidationError { field, message: reason })
            }
            CompanySettingsError::RepositoryInitializationFailed { reason } => {
                DomainError::new(DomainErrorKind::RepositoryError {
                    operation: "initialization".to_string(),
                    details: reason,
                })
            }
            CompanySettingsError::FileSystemError { operation, path, details } => {
                DomainError::new(DomainErrorKind::Io {
                    operation,
                    path: Some(path),
                }).with_context(details)
            }
        }
    }
}

// Result type for company settings operations
pub type CompanySettingsResult<T> = Result<T, CompanySettingsError>;
