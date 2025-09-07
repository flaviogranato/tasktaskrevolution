#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use thiserror::Error;

/// Resource-specific error types
#[derive(Error, Debug, PartialEq)]
pub enum ResourceError {
    #[error("Resource with code '{code}' not found")]
    NotFound { code: String },

    #[error("Resource with code '{code}' already exists")]
    AlreadyExists { code: String },

    #[error("Resource is in invalid state '{current}', expected '{expected}'")]
    InvalidState { current: String, expected: String },

    #[error("Resource validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Cannot modify resource in state '{state}'")]
    ModificationNotAllowed { state: String },

    #[error("Invalid email '{email}': {reason}")]
    InvalidEmail { email: String, reason: String },

    #[error("Invalid name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    #[error("Resource code '{code}' is invalid: {reason}")]
    InvalidCode { code: String, reason: String },

    #[error("Resource deactivation failed: {reason}")]
    DeactivationFailed { reason: String },
}

impl From<ResourceError> for DomainError {
    fn from(err: ResourceError) -> Self {
        match err {
            ResourceError::NotFound { code } => DomainError::ResourceNotFound { code },
            ResourceError::AlreadyExists { code } => DomainError::ResourceAlreadyExists { code },
            ResourceError::InvalidState { current, expected } => {
                DomainError::ResourceInvalidState { current, expected }
            }
            ResourceError::ValidationFailed { details } => DomainError::ResourceValidationFailed { details },
            ResourceError::ModificationNotAllowed { state } => DomainError::ResourceInvalidState {
                current: state,
                expected: "modifiable state".to_string(),
            },
            ResourceError::InvalidEmail { email, reason } => DomainError::ValidationError {
                field: "email".to_string(),
                message: format!("Email '{}' is invalid: {}", email, reason),
            },
            ResourceError::InvalidName { name, reason } => DomainError::ValidationError {
                field: "name".to_string(),
                message: format!("Name '{}' is invalid: {}", name, reason),
            },
            ResourceError::InvalidCode { code, reason } => DomainError::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            },
            ResourceError::DeactivationFailed { reason } => DomainError::ResourceInvalidState {
                current: "active".to_string(),
                expected: "deactivated".to_string(),
            },
        }
    }
}

// Result type for resource operations
pub type ResourceResult<T> = Result<T, ResourceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_error_display() {
        let error = ResourceError::NotFound {
            code: "RES-001".to_string(),
        };
        assert!(error.to_string().contains("Resource with code 'RES-001' not found"));
    }

    #[test]
    fn test_resource_error_conversion_to_domain_error() {
        let resource_error = ResourceError::NotFound {
            code: "RES-001".to_string(),
        };
        let domain_error: DomainError = resource_error.into();
        assert!(matches!(domain_error, DomainError::ResourceNotFound { code } if code == "RES-001"));
    }

    #[test]
    fn test_resource_error_invalid_email() {
        let error = ResourceError::InvalidEmail {
            email: "invalid-email".to_string(),
            reason: "Missing @ symbol".to_string(),
        };
        assert!(error.to_string().contains("Invalid email 'invalid-email'"));
    }

    #[test]
    fn test_resource_result() {
        let result: ResourceResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
    }
}
