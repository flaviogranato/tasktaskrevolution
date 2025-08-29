use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Task-specific error types
#[derive(Debug)]
pub enum TaskError {
    NotFound { code: String },
    AlreadyExists { code: String },
    InvalidState { current: String, expected: String },
    ValidationFailed { details: Vec<String> },
    ModificationNotAllowed { state: String },
    AssignmentFailed { reason: String },
    InvalidCode { code: String, reason: String },
    InvalidName { name: String, reason: String },
    InvalidDates { reason: String },
    ResourceNotFound { resource_code: String },
    ProjectNotFound { project_code: String },
    CircularDependency { task_codes: Vec<String> },
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::NotFound { code } => {
                write!(f, "Task with code '{}' not found", code)
            }
            TaskError::AlreadyExists { code } => {
                write!(f, "Task with code '{}' already exists", code)
            }
            TaskError::InvalidState { current, expected } => {
                write!(f, "Task is in invalid state '{}', expected '{}'", current, expected)
            }
            TaskError::ValidationFailed { details } => {
                write!(f, "Task validation failed: {}", details.join(", "))
            }
            TaskError::ModificationNotAllowed { state } => {
                write!(f, "Cannot modify task in state '{}'", state)
            }
            TaskError::AssignmentFailed { reason } => {
                write!(f, "Task assignment failed: {}", reason)
            }
            TaskError::InvalidCode { code, reason } => {
                write!(f, "Task code '{}' is invalid: {}", code, reason)
            }
            TaskError::InvalidName { name, reason } => {
                write!(f, "Task name '{}' is invalid: {}", name, reason)
            }
            TaskError::InvalidDates { reason } => {
                write!(f, "Task dates are invalid: {}", reason)
            }
            TaskError::ResourceNotFound { resource_code } => {
                write!(
                    f,
                    "Resource with code '{}' not found for task assignment",
                    resource_code
                )
            }
            TaskError::ProjectNotFound { project_code } => {
                write!(f, "Project with code '{}' not found for task", project_code)
            }
            TaskError::CircularDependency { task_codes } => {
                write!(
                    f,
                    "Circular dependency detected between tasks: {}",
                    task_codes.join(" -> ")
                )
            }
        }
    }
}

impl StdError for TaskError {}

impl From<TaskError> for DomainError {
    fn from(err: TaskError) -> Self {
        match err {
            TaskError::NotFound { code } => DomainError::new(DomainErrorKind::TaskNotFound { code }),
            TaskError::AlreadyExists { code } => DomainError::new(DomainErrorKind::TaskAlreadyExists { code }),
            TaskError::InvalidState { current, expected } => {
                DomainError::new(DomainErrorKind::TaskInvalidState { current, expected })
            }
            TaskError::ValidationFailed { details } => {
                DomainError::new(DomainErrorKind::TaskValidationFailed { details })
            }
            TaskError::ModificationNotAllowed { state } => DomainError::new(DomainErrorKind::TaskInvalidState {
                current: state,
                expected: "modifiable state".to_string(),
            }),
            TaskError::AssignmentFailed { reason } => {
                DomainError::new(DomainErrorKind::TaskAssignmentFailed { reason })
            }
            TaskError::InvalidCode { code, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            }),
            TaskError::InvalidName { name, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: format!("Name '{}' is invalid: {}", name, reason),
            }),
            TaskError::InvalidDates { reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "dates".to_string(),
                message: reason,
            }),
            TaskError::ResourceNotFound { resource_code } => {
                DomainError::new(DomainErrorKind::ResourceNotFound { code: resource_code })
            }
            TaskError::ProjectNotFound { project_code } => {
                DomainError::new(DomainErrorKind::ProjectNotFound { code: project_code })
            }
            TaskError::CircularDependency { task_codes } => DomainError::new(DomainErrorKind::ValidationError {
                field: "dependencies".to_string(),
                message: format!("Circular dependency: {}", task_codes.join(" -> ")),
            }),
        }
    }
}

// Result type for task operations
pub type TaskResult<T> = Result<T, TaskError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_error_not_found_display() {
        let error = TaskError::NotFound { code: "TASK-001".to_string() };
        let expected = "Task with code 'TASK-001' not found";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_already_exists_display() {
        let error = TaskError::AlreadyExists { code: "TASK-002".to_string() };
        let expected = "Task with code 'TASK-002' already exists";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_invalid_state_display() {
        let error = TaskError::InvalidState { 
            current: "Completed".to_string(), 
            expected: "In Progress".to_string() 
        };
        let expected = "Task is in invalid state 'Completed', expected 'In Progress'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_validation_failed_display() {
        let error = TaskError::ValidationFailed { 
            details: vec!["Name is required".to_string(), "Code is invalid".to_string()] 
        };
        let expected = "Task validation failed: Name is required, Code is invalid";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_modification_not_allowed_display() {
        let error = TaskError::ModificationNotAllowed { state: "Completed".to_string() };
        let expected = "Cannot modify task in state 'Completed'";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_assignment_failed_display() {
        let error = TaskError::AssignmentFailed { 
            reason: "Resource not available".to_string() 
        };
        let expected = "Task assignment failed: Resource not available";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_invalid_code_display() {
        let error = TaskError::InvalidCode { 
            code: "INVALID".to_string(), 
            reason: "Contains invalid characters".to_string() 
        };
        let expected = "Task code 'INVALID' is invalid: Contains invalid characters";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_invalid_name_display() {
        let error = TaskError::InvalidName { 
            name: "123".to_string(), 
            reason: "Contains numbers".to_string() 
        };
        let expected = "Task name '123' is invalid: Contains numbers";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_invalid_dates_display() {
        let error = TaskError::InvalidDates { 
            reason: "End date before start date".to_string() 
        };
        let expected = "Task dates are invalid: End date before start date";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_resource_not_found_display() {
        let error = TaskError::ResourceNotFound { 
            resource_code: "RES-001".to_string() 
        };
        let expected = "Resource with code 'RES-001' not found for task assignment";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_project_not_found_display() {
        let error = TaskError::ProjectNotFound { 
            project_code: "PROJ-001".to_string() 
        };
        let expected = "Project with code 'PROJ-001' not found for task";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_circular_dependency_display() {
        let error = TaskError::CircularDependency { 
            task_codes: vec!["TASK-001".to_string(), "TASK-002".to_string(), "TASK-001".to_string()] 
        };
        let expected = "Circular dependency detected between tasks: TASK-001 -> TASK-002 -> TASK-001";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn test_task_error_debug_formatting() {
        let error = TaskError::NotFound { code: "TASK-001".to_string() };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("TASK-001"));
    }

    #[test]
    fn test_from_task_error_to_domain_error_not_found() {
        let task_error = TaskError::NotFound { code: "TASK-001".to_string() };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskNotFound { code } => {
                assert_eq!(code, "TASK-001");
            }
            _ => panic!("Expected TaskNotFound error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_already_exists() {
        let task_error = TaskError::AlreadyExists { code: "TASK-002".to_string() };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskAlreadyExists { code } => {
                assert_eq!(code, "TASK-002");
            }
            _ => panic!("Expected TaskAlreadyExists error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_invalid_state() {
        let task_error = TaskError::InvalidState { 
            current: "Completed".to_string(), 
            expected: "In Progress".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskInvalidState { current, expected } => {
                assert_eq!(current, "Completed");
                assert_eq!(expected, "In Progress");
            }
            _ => panic!("Expected TaskInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_validation_failed() {
        let task_error = TaskError::ValidationFailed { 
            details: vec!["Name is required".to_string(), "Code is invalid".to_string()] 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskValidationFailed { details } => {
                assert_eq!(details.len(), 2);
                assert!(details.contains(&"Name is required".to_string()));
                assert!(details.contains(&"Code is invalid".to_string()));
            }
            _ => panic!("Expected TaskValidationFailed error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_modification_not_allowed() {
        let task_error = TaskError::ModificationNotAllowed { state: "Completed".to_string() };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskInvalidState { current, expected } => {
                assert_eq!(current, "Completed");
                assert_eq!(expected, "modifiable state");
            }
            _ => panic!("Expected TaskInvalidState error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_assignment_failed() {
        let task_error = TaskError::AssignmentFailed { 
            reason: "Resource not available".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::TaskAssignmentFailed { reason } => {
                assert_eq!(reason, "Resource not available");
            }
            _ => panic!("Expected TaskAssignmentFailed error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_invalid_code() {
        let task_error = TaskError::InvalidCode { 
            code: "INVALID".to_string(), 
            reason: "Contains invalid characters".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "code");
                assert_eq!(message, "Code 'INVALID' is invalid: Contains invalid characters");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_invalid_name() {
        let task_error = TaskError::InvalidName { 
            name: "123".to_string(), 
            reason: "Contains numbers".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "name");
                assert_eq!(message, "Name '123' is invalid: Contains numbers");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_invalid_dates() {
        let task_error = TaskError::InvalidDates { 
            reason: "End date before start date".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "dates");
                assert_eq!(message, "End date before start date");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_resource_not_found() {
        let task_error = TaskError::ResourceNotFound { 
            resource_code: "RES-001".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ResourceNotFound { code } => {
                assert_eq!(code, "RES-001");
            }
            _ => panic!("Expected ResourceNotFound error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_project_not_found() {
        let task_error = TaskError::ProjectNotFound { 
            project_code: "PROJ-001".to_string() 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ProjectNotFound { code } => {
                assert_eq!(code, "PROJ-001");
            }
            _ => panic!("Expected ProjectNotFound error kind"),
        }
    }

    #[test]
    fn test_from_task_error_to_domain_error_circular_dependency() {
        let task_error = TaskError::CircularDependency { 
            task_codes: vec!["TASK-001".to_string(), "TASK-002".to_string(), "TASK-001".to_string()] 
        };
        let domain_error: DomainError = task_error.into();
        
        match domain_error.kind() {
            DomainErrorKind::ValidationError { field, message } => {
                assert_eq!(field, "dependencies");
                assert_eq!(message, "Circular dependency: TASK-001 -> TASK-002 -> TASK-001");
            }
            _ => panic!("Expected ValidationError error kind"),
        }
    }

    #[test]
    fn test_task_result_success() {
        let result: TaskResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
    }

    #[test]
    fn test_task_result_failure() {
        let result: TaskResult<String> = Err(TaskError::NotFound { code: "TASK-001".to_string() });
        assert!(result.is_err());
        
        match result {
            Err(TaskError::NotFound { code }) => {
                assert_eq!(code, "TASK-001");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_all_error_variants_covered() {
        // Test that all TaskError variants can be created and converted
        let errors = vec![
            TaskError::NotFound { code: "TEST".to_string() },
            TaskError::AlreadyExists { code: "TEST".to_string() },
            TaskError::InvalidState { 
                current: "TEST".to_string(), 
                expected: "TEST".to_string() 
            },
            TaskError::ValidationFailed { 
                details: vec!["TEST".to_string()] 
            },
            TaskError::ModificationNotAllowed { state: "TEST".to_string() },
            TaskError::AssignmentFailed { reason: "TEST".to_string() },
            TaskError::InvalidCode { 
                code: "TEST".to_string(), 
                reason: "TEST".to_string() 
            },
            TaskError::InvalidName { 
                name: "TEST".to_string(), 
                reason: "TEST".to_string() 
            },
            TaskError::InvalidDates { reason: "TEST".to_string() },
            TaskError::ResourceNotFound { resource_code: "TEST".to_string() },
            TaskError::ProjectNotFound { project_code: "TEST".to_string() },
            TaskError::CircularDependency { 
                task_codes: vec!["TEST".to_string()] 
            },
        ];

        for error in errors {
            let domain_error: DomainError = error.into();
            assert!(domain_error.to_string().len() > 0);
        }
    }
}
