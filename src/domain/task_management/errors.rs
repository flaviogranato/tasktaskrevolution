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
                write!(f, "Resource with code '{}' not found for task assignment", resource_code)
            }
            TaskError::ProjectNotFound { project_code } => {
                write!(f, "Project with code '{}' not found for task", project_code)
            }
            TaskError::CircularDependency { task_codes } => {
                write!(f, "Circular dependency detected between tasks: {}", task_codes.join(" -> "))
            }
        }
    }
}

impl StdError for TaskError {}

impl From<TaskError> for DomainError {
    fn from(err: TaskError) -> Self {
        match err {
            TaskError::NotFound { code } => {
                DomainError::new(DomainErrorKind::TaskNotFound { code })
            }
            TaskError::AlreadyExists { code } => {
                DomainError::new(DomainErrorKind::TaskAlreadyExists { code })
            }
            TaskError::InvalidState { current, expected } => {
                DomainError::new(DomainErrorKind::TaskInvalidState { current, expected })
            }
            TaskError::ValidationFailed { details } => {
                DomainError::new(DomainErrorKind::TaskValidationFailed { details })
            }
            TaskError::ModificationNotAllowed { state } => {
                DomainError::new(DomainErrorKind::TaskInvalidState {
                    current: state,
                    expected: "modifiable state".to_string(),
                })
            }
            TaskError::AssignmentFailed { reason } => {
                DomainError::new(DomainErrorKind::TaskAssignmentFailed { reason })
            }
            TaskError::InvalidCode { code, reason } => {
                DomainError::new(DomainErrorKind::ValidationError {
                    field: "code".to_string(),
                    message: format!("Code '{}' is invalid: {}", code, reason),
                })
            }
            TaskError::InvalidName { name, reason } => {
                DomainError::new(DomainErrorKind::ValidationError {
                    field: "name".to_string(),
                    message: format!("Name '{}' is invalid: {}", name, reason),
                })
            }
            TaskError::InvalidDates { reason } => {
                DomainError::new(DomainErrorKind::ValidationError {
                    field: "dates".to_string(),
                    message: reason,
                })
            }
            TaskError::ResourceNotFound { resource_code } => {
                DomainError::new(DomainErrorKind::ResourceNotFound { code: resource_code })
            }
            TaskError::ProjectNotFound { project_code } => {
                DomainError::new(DomainErrorKind::ProjectNotFound { code: project_code })
            }
            TaskError::CircularDependency { task_codes } => {
                DomainError::new(DomainErrorKind::ValidationError {
                    field: "dependencies".to_string(),
                    message: format!("Circular dependency: {}", task_codes.join(" -> ")),
                })
            }
        }
    }
}

// Result type for task operations
pub type TaskResult<T> = Result<T, TaskError>;
