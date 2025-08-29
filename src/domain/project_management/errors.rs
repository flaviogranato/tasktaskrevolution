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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_error_not_found_display() {
        let error = ProjectError::NotFound { code: "PROJ-001".to_string() };
        let expected = "Project with code 'PROJ-001' not found";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_already_exists_display() {
        let error = ProjectError::AlreadyExists { code: "PROJ-002".to_string() };
        let expected = "Project with code 'PROJ-002' already exists";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_invalid_state_display() {
        let error = ProjectError::InvalidState { 
            current: "Completed".to_string(), 
            expected: "In Progress".to_string() 
        };
        let expected = "Project is in invalid state 'Completed', expected 'In Progress'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_validation_failed_display() {
        let error = ProjectError::ValidationFailed { 
            details: vec!["Name is required".to_string(), "Code is invalid".to_string()] 
        };
        let expected = "Project validation failed: Name is required, Code is invalid";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_modification_not_allowed_display() {
        let error = ProjectError::ModificationNotAllowed { state: "Completed".to_string() };
        let expected = "Cannot modify project in state 'Completed'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_invalid_dates_display() {
        let error = ProjectError::InvalidDates { reason: "End date before start date".to_string() };
        let expected = "Project dates are invalid: End date before start date";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_invalid_code_display() {
        let error = ProjectError::InvalidCode { 
            code: "INVALID".to_string(), 
            reason: "Contains invalid characters".to_string() 
        };
        let expected = "Project code 'INVALID' is invalid: Contains invalid characters";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_project_error_debug_formatting() {
        let error = ProjectError::NotFound { code: "PROJ-001".to_string() };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("PROJ-001"));
    }

    #[test]
    fn test_from_project_error_to_domain_error_not_found() {
        let project_error = ProjectError::NotFound { code: "PROJ-001".to_string() };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectNotFound { code } => {
                assert_eq!(code, "PROJ-001");
            }
            _ => panic!("Expected ProjectNotFound error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_already_exists() {
        let project_error = ProjectError::AlreadyExists { code: "PROJ-002".to_string() };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectAlreadyExists { code } => {
                assert_eq!(code, "PROJ-002");
            }
            _ => panic!("Expected ProjectAlreadyExists error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_invalid_state() {
        let project_error = ProjectError::InvalidState { 
            current: "Completed".to_string(), 
            expected: "In Progress".to_string() 
        };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectInvalidState { current, expected } => {
                assert_eq!(current, "Completed");
                assert_eq!(expected, "In Progress");
            }
            _ => panic!("Expected ProjectInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_validation_failed() {
        let project_error = ProjectError::ValidationFailed { 
            details: vec!["Name is required".to_string(), "Code is invalid".to_string()] 
        };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectValidationFailed { details } => {
                assert_eq!(details.len(), 2);
                assert!(details.contains(&"Name is required".to_string()));
                assert!(details.contains(&"Code is invalid".to_string()));
            }
            _ => panic!("Expected ProjectValidationFailed error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_modification_not_allowed() {
        let project_error = ProjectError::ModificationNotAllowed { state: "Completed".to_string() };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectInvalidState { current, expected } => {
                assert_eq!(current, "Completed");
                assert_eq!(expected, "modifiable state");
            }
            _ => panic!("Expected ProjectInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_invalid_dates() {
        let project_error = ProjectError::InvalidDates { 
            reason: "End date before start date".to_string() 
        };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "dates");
                assert_eq!(message, "End date before start date");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_project_error_to_domain_error_invalid_code() {
        let project_error = ProjectError::InvalidCode { 
            code: "INVALID".to_string(), 
            reason: "Contains invalid characters".to_string() 
        };
        let domain_error: DomainError = project_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "code");
                assert_eq!(message, "Code 'INVALID' is invalid: Contains invalid characters");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_project_result_success() {
        let result: ProjectResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
    }

    #[test]
    fn test_project_result_failure() {
        let result: ProjectResult<String> = Err(ProjectError::NotFound { code: "PROJ-001".to_string() });
        assert!(result.is_err());
        
        match result {
            Err(ProjectError::NotFound { code }) => {
                assert_eq!(code, "PROJ-001");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_all_error_variants_covered() {
        // Test that all ProjectError variants can be created and converted
        let errors = vec![
            ProjectError::NotFound { code: "TEST".to_string() },
            ProjectError::AlreadyExists { code: "TEST".to_string() },
            ProjectError::InvalidState { 
                current: "TEST".to_string(), 
                expected: "TEST".to_string() 
            },
            ProjectError::ValidationFailed { 
                details: vec!["TEST".to_string()] 
            },
            ProjectError::ModificationNotAllowed { state: "TEST".to_string() },
            ProjectError::InvalidDates { reason: "TEST".to_string() },
            ProjectError::InvalidCode { 
                code: "TEST".to_string(), 
                reason: "TEST".to_string() 
            },
        ];

        for error in errors {
            let domain_error: DomainError = error.into();
            assert!(domain_error.to_string().len() > 0);
        }
    }
}
