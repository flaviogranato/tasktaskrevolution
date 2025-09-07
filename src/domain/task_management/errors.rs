#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use thiserror::Error;

/// Task-specific error types
#[derive(Error, Debug, PartialEq)]
pub enum TaskError {
    #[error("Task with code '{code}' not found")]
    NotFound { code: String },

    #[error("Task with code '{code}' already exists")]
    AlreadyExists { code: String },

    #[error("Task is in invalid state '{current}', expected '{expected}'")]
    InvalidState { current: String, expected: String },

    #[error("Task validation failed: {details}")]
    ValidationFailed { details: String },

    #[error("Cannot modify task in state '{state}'")]
    ModificationNotAllowed { state: String },

    #[error("Task assignment failed: {reason}")]
    AssignmentFailed { reason: String },

    #[error("Task code '{code}' is invalid: {reason}")]
    InvalidCode { code: String, reason: String },

    #[error("Task name '{name}' is invalid: {reason}")]
    InvalidName { name: String, reason: String },

    #[error("Task dates are invalid: {reason}")]
    InvalidDates { reason: String },

    #[error("Resource with code '{resource_code}' not found for task assignment")]
    ResourceNotFound { resource_code: String },

    #[error("Project with code '{project_code}' not found for task")]
    ProjectNotFound { project_code: String },

    #[error("Circular dependency detected between tasks: {task_codes}")]
    CircularDependency { task_codes: String },
}

impl From<TaskError> for DomainError {
    fn from(err: TaskError) -> Self {
        match err {
            TaskError::NotFound { code } => DomainError::TaskNotFound { code },
            TaskError::AlreadyExists { code } => DomainError::TaskAlreadyExists { code },
            TaskError::InvalidState { current, expected } => DomainError::TaskInvalidState { current, expected },
            TaskError::ValidationFailed { details } => DomainError::TaskValidationFailed { details },
            TaskError::ModificationNotAllowed { state } => DomainError::TaskInvalidState {
                current: state,
                expected: "modifiable state".to_string(),
            },
            TaskError::AssignmentFailed { reason } => DomainError::TaskAssignmentFailed { reason },
            TaskError::InvalidCode { code, reason } => DomainError::ValidationError {
                field: "code".to_string(),
                message: format!("Code '{}' is invalid: {}", code, reason),
            },
            TaskError::InvalidName { name, reason } => DomainError::ValidationError {
                field: "name".to_string(),
                message: format!("Name '{}' is invalid: {}", name, reason),
            },
            TaskError::InvalidDates { reason } => DomainError::ValidationError {
                field: "dates".to_string(),
                message: reason,
            },
            TaskError::ResourceNotFound { resource_code } => DomainError::ResourceNotFound { code: resource_code },
            TaskError::ProjectNotFound { project_code } => DomainError::ProjectNotFound { code: project_code },
            TaskError::CircularDependency { task_codes } => DomainError::ValidationError {
                field: "dependencies".to_string(),
                message: format!("Circular dependency: {}", task_codes),
            },
        }
    }
}

// Result type for task operations
pub type TaskResult<T> = Result<T, TaskError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_error_display() {
        let error = TaskError::NotFound {
            code: "TASK-001".to_string(),
        };
        assert!(error.to_string().contains("Task with code 'TASK-001' not found"));
    }

    #[test]
    fn test_task_error_conversion_to_domain_error() {
        let task_error = TaskError::NotFound {
            code: "TASK-001".to_string(),
        };
        let domain_error: DomainError = task_error.into();
        assert!(matches!(domain_error, DomainError::TaskNotFound { code } if code == "TASK-001"));
    }

    #[test]
    fn test_task_error_circular_dependency() {
        let error = TaskError::CircularDependency {
            task_codes: "TASK-001 -> TASK-002".to_string(),
        };
        assert!(error.to_string().contains("Circular dependency detected between tasks"));
    }

    #[test]
    fn test_task_result() {
        let result: TaskResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
    }
}
