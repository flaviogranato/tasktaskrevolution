#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use thiserror::Error;

/// Project-specific error types
#[derive(Error, Debug, PartialEq)]
pub enum ProjectError {
    #[error("Project with code '{code}' not found")]
    NotFound { code: String },

    #[error("Project with code '{code}' already exists")]
    AlreadyExists { code: String },

    #[error("Project is in invalid state '{current}', expected '{expected}'")]
    InvalidState { current: String, expected: String },

    #[error("Project validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Cannot modify project in state '{state}'")]
    ModificationNotAllowed { state: String },

    #[error("Project dates are invalid: {reason}")]
    InvalidDates { reason: String },

    #[error("Project code '{code}' is invalid: {reason}")]
    InvalidCode { code: String, reason: String },
}

impl From<ProjectError> for DomainError {
    fn from(err: ProjectError) -> Self {
        match err {
            ProjectError::NotFound { code } => DomainError::ProjectNotFound { code },
            ProjectError::AlreadyExists { code } => DomainError::ProjectAlreadyExists { code },
            ProjectError::InvalidState { current, expected } => DomainError::ProjectInvalidState { current, expected },
            ProjectError::ValidationFailed { details } => DomainError::ProjectValidationFailed { details },
            ProjectError::ModificationNotAllowed { state } => DomainError::ProjectInvalidState {
                current: state,
                expected: "modifiable state".to_string(),
            },
            ProjectError::InvalidDates { reason } => DomainError::ValidationError {
                field: "dates".to_string(),
                message: reason,
            },
            ProjectError::InvalidCode { code, reason } => DomainError::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            },
        }
    }
}

// Result type for project operations
pub type ProjectResult<T> = Result<T, ProjectError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_error_display() {
        let error = ProjectError::NotFound {
            code: "PROJ-001".to_string(),
        };
        assert!(error.to_string().contains("Project with code 'PROJ-001' not found"));
    }

    #[test]
    fn test_project_error_conversion_to_domain_error() {
        let project_error = ProjectError::NotFound {
            code: "PROJ-001".to_string(),
        };
        let domain_error: DomainError = project_error.into();
        assert!(matches!(domain_error, DomainError::ProjectNotFound { code } if code == "PROJ-001"));
    }

    #[test]
    fn test_project_error_validation_failed() {
        let error = ProjectError::ValidationFailed {
            details: "Name is required".to_string(),
        };
        assert!(
            error
                .to_string()
                .contains("Project validation failed: Name is required")
        );
    }

    #[test]
    fn test_project_result() {
        let result: ProjectResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
    }
}
