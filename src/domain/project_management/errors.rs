use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Project-specific error types
#[derive(Debug)]
pub enum ProjectError {
    NotFound { code: String },
    AlreadyExists { code: String },
    InvalidState { current: String, expected: String },
    ValidationFailed { details: Vec<String> },
    ModificationNotAllowed { state: String },
    InvalidDates { reason: String },
    InvalidCode { code: String, reason: String },
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::NotFound { code } => {
                write!(f, "Project with code '{}' not found", code)
            }
            ProjectError::AlreadyExists { code } => {
                write!(f, "Project with code '{}' already exists", code)
            }
            ProjectError::InvalidState { current, expected } => {
                write!(f, "Project is in invalid state '{}', expected '{}'", current, expected)
            }
            ProjectError::ValidationFailed { details } => {
                write!(f, "Project validation failed: {}", details.join(", "))
            }
            ProjectError::ModificationNotAllowed { state } => {
                write!(f, "Cannot modify project in state '{}'", state)
            }
            ProjectError::InvalidDates { reason } => {
                write!(f, "Project dates are invalid: {}", reason)
            }
            ProjectError::InvalidCode { code, reason } => {
                write!(f, "Project code '{}' is invalid: {}", code, reason)
            }
        }
    }
}

impl StdError for ProjectError {}

impl From<ProjectError> for DomainError {
    fn from(err: ProjectError) -> Self {
        match err {
            ProjectError::NotFound { code } => DomainError::new(DomainErrorKind::ProjectNotFound { code }),
            ProjectError::AlreadyExists { code } => DomainError::new(DomainErrorKind::ProjectAlreadyExists { code }),
            ProjectError::InvalidState { current, expected } => {
                DomainError::new(DomainErrorKind::ProjectInvalidState { current, expected })
            }
            ProjectError::ValidationFailed { details } => {
                DomainError::new(DomainErrorKind::ProjectValidationFailed { details })
            }
            ProjectError::ModificationNotAllowed { state } => DomainError::new(DomainErrorKind::ProjectInvalidState {
                current: state,
                expected: "modifiable state".to_string(),
            }),
            ProjectError::InvalidDates { reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "dates".to_string(),
                message: reason,
            }),
            ProjectError::InvalidCode { code, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            }),
        }
    }
}

// Result type for project operations
pub type ProjectResult<T> = Result<T, ProjectError>;
