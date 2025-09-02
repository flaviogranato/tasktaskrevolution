#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Resource-specific error types
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_error_not_found_display() {
        let error = ResourceError::NotFound {
            code: "RES-001".to_string(),
        };
        let expected = "Resource with code 'RES-001' not found";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_already_exists_display() {
        let error = ResourceError::AlreadyExists {
            code: "RES-002".to_string(),
        };
        let expected = "Resource with code 'RES-002' already exists";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_invalid_state_display() {
        let error = ResourceError::InvalidState {
            current: "Inactive".to_string(),
            expected: "Active".to_string(),
        };
        let expected = "Resource is in invalid state 'Inactive', expected 'Active'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_validation_failed_display() {
        let error = ResourceError::ValidationFailed {
            details: vec!["Name is required".to_string(), "Email is invalid".to_string()],
        };
        let expected = "Resource validation failed: Name is required, Email is invalid";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_modification_not_allowed_display() {
        let error = ResourceError::ModificationNotAllowed {
            state: "Inactive".to_string(),
        };
        let expected = "Cannot modify resource in state 'Inactive'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_invalid_email_display() {
        let error = ResourceError::InvalidEmail {
            email: "invalid-email".to_string(),
            reason: "Missing @ symbol".to_string(),
        };
        let expected = "Invalid email 'invalid-email': Missing @ symbol";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_invalid_name_display() {
        let error = ResourceError::InvalidName {
            name: "123".to_string(),
            reason: "Contains numbers".to_string(),
        };
        let expected = "Invalid name '123': Contains numbers";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_invalid_code_display() {
        let error = ResourceError::InvalidCode {
            code: "INVALID".to_string(),
            reason: "Contains invalid characters".to_string(),
        };
        let expected = "Resource code 'INVALID' is invalid: Contains invalid characters";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_deactivation_failed_display() {
        let error = ResourceError::DeactivationFailed {
            reason: "Resource has active assignments".to_string(),
        };
        let expected = "Resource deactivation failed: Resource has active assignments";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_resource_error_debug_formatting() {
        let error = ResourceError::NotFound {
            code: "RES-001".to_string(),
        };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("RES-001"));
    }

    #[test]
    fn test_from_resource_error_to_domain_error_not_found() {
        let resource_error = ResourceError::NotFound {
            code: "RES-001".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceNotFound { code } => {
                assert_eq!(code, "RES-001");
            }
            _ => panic!("Expected ResourceNotFound error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_already_exists() {
        let resource_error = ResourceError::AlreadyExists {
            code: "RES-002".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceAlreadyExists { code } => {
                assert_eq!(code, "RES-002");
            }
            _ => panic!("Expected ResourceAlreadyExists error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_invalid_state() {
        let resource_error = ResourceError::InvalidState {
            current: "Inactive".to_string(),
            expected: "Active".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceInvalidState { current, expected } => {
                assert_eq!(current, "Inactive");
                assert_eq!(expected, "Active");
            }
            _ => panic!("Expected ResourceInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_validation_failed() {
        let resource_error = ResourceError::ValidationFailed {
            details: vec!["Name is required".to_string(), "Email is invalid".to_string()],
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceValidationFailed { details } => {
                assert_eq!(details.len(), 2);
                assert!(details.contains(&"Name is required".to_string()));
                assert!(details.contains(&"Email is invalid".to_string()));
            }
            _ => panic!("Expected ResourceValidationFailed error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_modification_not_allowed() {
        let resource_error = ResourceError::ModificationNotAllowed {
            state: "Inactive".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceInvalidState { current, expected } => {
                assert_eq!(current, "Inactive");
                assert_eq!(expected, "modifiable state");
            }
            _ => panic!("Expected ResourceInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_invalid_email() {
        let resource_error = ResourceError::InvalidEmail {
            email: "invalid-email".to_string(),
            reason: "Missing @ symbol".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "email");
                assert_eq!(message, "Email 'invalid-email' is invalid: Missing @ symbol");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_invalid_name() {
        let resource_error = ResourceError::InvalidName {
            name: "123".to_string(),
            reason: "Contains numbers".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "name");
                assert_eq!(message, "Name '123' is invalid: Contains numbers");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_invalid_code() {
        let resource_error = ResourceError::InvalidCode {
            code: "INVALID".to_string(),
            reason: "Contains invalid characters".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "code");
                assert_eq!(message, "Code 'INVALID' is invalid: Contains invalid characters");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_resource_error_to_domain_error_deactivation_failed() {
        let resource_error = ResourceError::DeactivationFailed {
            reason: "Resource has active assignments".to_string(),
        };
        let domain_error: DomainError = resource_error.into();

        match domain_error.kind() {
            DomainErrorKind::ResourceInvalidState { current, expected } => {
                assert_eq!(current, "active");
                assert_eq!(expected, "deactivated");
            }
            _ => panic!("Expected ResourceInvalidState error kind"),
        }
    }

    #[test]
    fn test_resource_result_success() {
        let result: ResourceResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
        assert_eq!(result, Ok("Success".to_string()));
    }

    #[test]
    fn test_resource_result_failure() {
        let result: ResourceResult<String> = Err(ResourceError::NotFound {
            code: "RES-001".to_string(),
        });
        assert!(result.is_err());

        match result {
            Err(ResourceError::NotFound { code }) => {
                assert_eq!(code, "RES-001");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_all_error_variants_covered() {
        // Test that all ResourceError variants can be created and converted
        let errors = vec![
            ResourceError::NotFound {
                code: "TEST".to_string(),
            },
            ResourceError::AlreadyExists {
                code: "TEST".to_string(),
            },
            ResourceError::InvalidState {
                current: "TEST".to_string(),
                expected: "TEST".to_string(),
            },
            ResourceError::ValidationFailed {
                details: vec!["TEST".to_string()],
            },
            ResourceError::ModificationNotAllowed {
                state: "TEST".to_string(),
            },
            ResourceError::InvalidEmail {
                email: "TEST".to_string(),
                reason: "TEST".to_string(),
            },
            ResourceError::InvalidName {
                name: "TEST".to_string(),
                reason: "TEST".to_string(),
            },
            ResourceError::InvalidCode {
                code: "TEST".to_string(),
                reason: "TEST".to_string(),
            },
            ResourceError::DeactivationFailed {
                reason: "TEST".to_string(),
            },
        ];

        for error in errors {
            let domain_error: DomainError = error.into();
            assert!(!domain_error.to_string().is_empty());
        }
    }
}
