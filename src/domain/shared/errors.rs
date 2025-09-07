#![allow(dead_code)]

use thiserror::Error;

/// Base error type for all domain errors
#[derive(Error, Debug)]
pub enum DomainError {
    // Project Management Errors
    #[error("Project with code '{code}' not found")]
    ProjectNotFound { code: String },

    #[error("Project with code '{code}' already exists")]
    ProjectAlreadyExists { code: String },

    #[error("Project is in invalid state '{current}', expected '{expected}'")]
    ProjectInvalidState { current: String, expected: String },

    #[error("Project validation failed: {details}")]
    ProjectValidationFailed { details: String },

    // Resource Management Errors
    #[error("Resource with code '{code}' not found")]
    ResourceNotFound { code: String },

    #[error("Resource with code '{code}' already exists")]
    ResourceAlreadyExists { code: String },

    #[error("Resource is in invalid state '{current}', expected '{expected}'")]
    ResourceInvalidState { current: String, expected: String },

    #[error("Resource validation failed: {details}")]
    ResourceValidationFailed { details: String },

    // Task Management Errors
    #[error("Task with code '{code}' not found")]
    TaskNotFound { code: String },

    #[error("Task with code '{code}' already exists")]
    TaskAlreadyExists { code: String },

    #[error("Task is in invalid state '{current}', expected '{expected}'")]
    TaskInvalidState { current: String, expected: String },

    #[error("Task validation failed: {details}")]
    TaskValidationFailed { details: String },

    #[error("Task assignment failed: {reason}")]
    TaskAssignmentFailed { reason: String },

    // Configuration Errors
    #[error("Invalid configuration for field '{field}': {value}")]
    ConfigurationInvalid { field: String, value: String },

    #[error("Missing configuration for field '{field}'")]
    ConfigurationMissing { field: String },

    // Repository Errors
    #[error("Repository error during {operation}: {details}")]
    RepositoryError { operation: String, details: String },

    #[error("Persistence error during {operation}: {details}")]
    PersistenceError { operation: String, details: String },

    // Validation Errors
    #[error("Validation error for field '{field}': {message}")]
    ValidationError { field: String, message: String },

    // Generic Errors
    #[error("{message}")]
    Generic { message: String },

    #[error("I/O error during {operation}")]
    Io {
        operation: String,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error during {operation} on path '{path}'")]
    IoWithPath {
        operation: String,
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Serialization error for format '{format}': {details}")]
    Serialization {
        format: String,
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl DomainError {
    /// Create a project not found error
    pub fn project_not_found(code: impl Into<String>) -> Self {
        Self::ProjectNotFound { code: code.into() }
    }

    /// Create a resource not found error
    pub fn resource_not_found(code: impl Into<String>) -> Self {
        Self::ResourceNotFound { code: code.into() }
    }

    /// Create a task not found error
    pub fn task_not_found(code: impl Into<String>) -> Self {
        Self::TaskNotFound { code: code.into() }
    }

    /// Create a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a project invalid state error
    pub fn project_invalid_state(current: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::ProjectInvalidState {
            current: current.into(),
            expected: expected.into(),
        }
    }

    /// Create a project validation failed error
    pub fn project_validation_failed(details: impl Into<String>) -> Self {
        Self::ProjectValidationFailed {
            details: details.into(),
        }
    }

    /// Create a resource validation failed error
    pub fn resource_validation_failed(details: impl Into<String>) -> Self {
        Self::ResourceValidationFailed {
            details: details.into(),
        }
    }

    /// Create a task validation failed error
    pub fn task_validation_failed(details: impl Into<String>) -> Self {
        Self::TaskValidationFailed {
            details: details.into(),
        }
    }

    /// Create a repository error
    pub fn repository_error(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::RepositoryError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create a persistence error
    pub fn persistence_error(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::PersistenceError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create an I/O error
    pub fn io_error(operation: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            source,
        }
    }

    /// Create an I/O error with path
    pub fn io_error_with_path(operation: impl Into<String>, path: impl Into<String>, source: std::io::Error) -> Self {
        Self::IoWithPath {
            operation: operation.into(),
            path: path.into(),
            source,
        }
    }

    /// Create a serialization error
    pub fn serialization_error(format: impl Into<String>, details: impl Into<String>) -> Self {
        Self::Serialization {
            format: format.into(),
            details: details.into(),
            source: None,
        }
    }

    /// Create a serialization error with source
    pub fn serialization_error_with_source(
        format: impl Into<String>,
        details: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Serialization {
            format: format.into(),
            details: details.into(),
            source: Some(source),
        }
    }

    /// Check if this is a project not found error
    pub fn is_project_not_found(&self) -> bool {
        matches!(self, Self::ProjectNotFound { .. })
    }

    /// Check if this is a resource not found error
    pub fn is_resource_not_found(&self) -> bool {
        matches!(self, Self::ResourceNotFound { .. })
    }

    /// Check if this is a task not found error
    pub fn is_task_not_found(&self) -> bool {
        matches!(self, Self::TaskNotFound { .. })
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError { .. })
    }
}

// Automatic conversions for common error types
impl From<String> for DomainError {
    fn from(message: String) -> Self {
        Self::Generic { message }
    }
}

impl From<&str> for DomainError {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            operation: "file operation".to_string(),
            source: err,
        }
    }
}

impl From<serde_yaml::Error> for DomainError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Serialization {
            format: "YAML".to_string(),
            details: err.to_string(),
            source: Some(Box::new(err)),
        }
    }
}

// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_domain_error_creation() {
        let error = DomainError::project_not_found("PROJ-001");
        assert!(matches!(error, DomainError::ProjectNotFound { code } if code == "PROJ-001"));
    }

    #[test]
    fn test_domain_error_display_formatting() {
        let error = DomainError::project_not_found("PROJ-001");
        let display = format!("{}", error);
        assert!(display.contains("Project with code 'PROJ-001' not found"));
    }

    #[test]
    fn test_domain_error_from_string() {
        let error: DomainError = "Custom error message".to_string().into();
        assert!(matches!(error, DomainError::Generic { message } if message == "Custom error message"));
    }

    #[test]
    fn test_domain_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let domain_error: DomainError = io_error.into();
        assert!(matches!(domain_error, DomainError::Io { operation, .. } if operation == "file operation"));
    }

    #[test]
    fn test_domain_error_from_serde_yaml_error() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let domain_error: DomainError = yaml_error.into();
        assert!(matches!(domain_error, DomainError::Serialization { format, .. } if format == "YAML"));
    }

    #[test]
    fn test_domain_error_is_project_not_found() {
        let error = DomainError::project_not_found("PROJ-001");
        assert!(error.is_project_not_found());
        assert!(!error.is_resource_not_found());
        assert!(!error.is_task_not_found());
        assert!(!error.is_validation_error());
    }

    #[test]
    fn test_domain_error_io_with_path() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = DomainError::io_error_with_path("read", "/path/to/file", io_error);
        let display = format!("{}", error);
        assert!(display.contains("I/O error during read on path '/path/to/file'"));
    }

    #[test]
    fn test_domain_error_serialization() {
        let error = DomainError::serialization_error("JSON", "Invalid UTF-8");
        let display = format!("{}", error);
        assert!(display.contains("Serialization error for format 'JSON': Invalid UTF-8"));
    }
}
