use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Resource-specific error types
#[derive(Debug)]
pub enum ResourceError {
    NotFound { code: String },
    AlreadyExists { code: String },
    InvalidState { current: String, expected: String },
    ValidationFailed { details: Vec<String> },
    ModificationNotAllowed { state: String },
    InvalidEmail { email: String, reason: String },
    InvalidName { name: String, reason: String },
    InvalidCode { code: String, reason: String },
    DeactivationFailed { reason: String },
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::NotFound { code } => {
                write!(f, "Resource with code '{}' not found", code)
            }
            ResourceError::AlreadyExists { code } => {
                write!(f, "Resource with code '{}' already exists", code)
            }
            ResourceError::InvalidState { current, expected } => {
                write!(f, "Resource is in invalid state '{}', expected '{}'", current, expected)
            }
            ResourceError::ValidationFailed { details } => {
                write!(f, "Resource validation failed: {}", details.join(", "))
            }
            ResourceError::ModificationNotAllowed { state } => {
                write!(f, "Cannot modify resource in state '{}'", state)
            }
            ResourceError::InvalidEmail { email, reason } => {
                write!(f, "Invalid email '{}': {}", email, reason)
            }
            ResourceError::InvalidName { name, reason } => {
                write!(f, "Invalid name '{}': {}", name, reason)
            }
            ResourceError::InvalidCode { code, reason } => {
                write!(f, "Resource code '{}' is invalid: {}", code, reason)
            }
            ResourceError::DeactivationFailed { reason } => {
                write!(f, "Resource deactivation failed: {}", reason)
            }
        }
    }
}

impl StdError for ResourceError {}

impl From<ResourceError> for DomainError {
    fn from(err: ResourceError) -> Self {
        match err {
            ResourceError::NotFound { code } => DomainError::new(DomainErrorKind::ResourceNotFound { code }),
            ResourceError::AlreadyExists { code } => DomainError::new(DomainErrorKind::ResourceAlreadyExists { code }),
            ResourceError::InvalidState { current, expected } => {
                DomainError::new(DomainErrorKind::ResourceInvalidState { current, expected })
            }
            ResourceError::ValidationFailed { details } => {
                DomainError::new(DomainErrorKind::ResourceValidationFailed { details })
            }
            ResourceError::ModificationNotAllowed { state } => {
                DomainError::new(DomainErrorKind::ResourceInvalidState {
                    current: state,
                    expected: "modifiable state".to_string(),
                })
            }
            ResourceError::InvalidEmail { email, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "email".to_string(),
                message: format!("Email '{}' is invalid: {}", email, reason),
            }),
            ResourceError::InvalidName { name, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: format!("Name '{}' is invalid: {}", name, reason),
            }),
            ResourceError::InvalidCode { code, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            }),
            ResourceError::DeactivationFailed { reason } => DomainError::new(DomainErrorKind::ResourceInvalidState {
                current: "active".to_string(),
                expected: "deactivated".to_string(),
            })
            .with_context(reason),
        }
    }
}

// Result type for resource operations
pub type ResourceResult<T> = Result<T, ResourceError>;
