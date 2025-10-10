//! Domain-specific error types
//!
//! This module defines error types specific to the domain layer,
//! following DDD principles and avoiding dependencies on other layers.

use std::fmt;

/// Domain error type that represents business logic errors
/// without depending on application or infrastructure layers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    // Entity Not Found Errors
    EntityNotFound {
        entity_type: String,
        identifier: String,
    },

    // Entity Already Exists Errors
    EntityAlreadyExists {
        entity_type: String,
        identifier: String,
    },

    // Validation Errors
    ValidationError {
        field: String,
        message: String,
    },

    // Business Logic Errors
    BusinessRuleViolation {
        rule: String,
        details: String,
    },

    // State Errors
    InvalidState {
        current: String,
        expected: String,
        entity: String,
    },

    // Dependency Errors
    CircularDependency {
        entities: Vec<String>,
    },

    // Constraint Errors
    ConstraintViolation {
        constraint: String,
        details: String,
    },

    // I/O errors
    IoError {
        operation: String,
        details: String,
    },

    // I/O errors with path information
    IoErrorWithPath {
        operation: String,
        path: String,
        details: String,
    },

    // Serialization errors
    SerializationError {
        operation: String,
        details: String,
    },

    // Configuration errors
    ConfigurationError {
        details: String,
    },

    // Configuration invalid
    ConfigurationInvalid {
        field: String,
        value: String,
        reason: String,
    },

    // Operation not allowed
    OperationNotAllowed {
        operation: String,
        reason: String,
    },

    // Specific entity errors for backward compatibility
    ProjectNotFound {
        code: String,
    },
    ResourceNotFound {
        code: String,
    },
    TaskNotFound {
        code: String,
    },
    CompanyNotFound {
        code: String,
    },
    ProjectAlreadyExists {
        code: String,
    },
    ResourceAlreadyExists {
        code: String,
    },
    TaskAlreadyExists {
        code: String,
    },
    CompanyAlreadyExists {
        code: String,
    },
    ProjectInvalidState {
        current: String,
        expected: String,
    },
    ResourceInvalidState {
        current: String,
        expected: String,
    },
    TaskInvalidState {
        current: String,
        expected: String,
    },
    // Generic domain error
    DomainError {
        message: String,
    },
}

impl DomainError {
    /// Create a new entity not found error
    pub fn entity_not_found(entity_type: &str, identifier: &str) -> Self {
        Self::EntityNotFound {
            entity_type: entity_type.to_string(),
            identifier: identifier.to_string(),
        }
    }

    /// Create a new entity already exists error
    pub fn entity_already_exists(entity_type: &str, identifier: &str) -> Self {
        Self::EntityAlreadyExists {
            entity_type: entity_type.to_string(),
            identifier: identifier.to_string(),
        }
    }

    /// Create a new validation error
    pub fn validation_error(field: &str, message: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            message: message.to_string(),
        }
    }

    /// Create a new business rule violation error
    pub fn business_rule_violation(rule: &str, details: &str) -> Self {
        Self::BusinessRuleViolation {
            rule: rule.to_string(),
            details: details.to_string(),
        }
    }

    /// Create a new invalid state error
    pub fn invalid_state(current: &str, expected: &str, entity: &str) -> Self {
        Self::InvalidState {
            current: current.to_string(),
            expected: expected.to_string(),
            entity: entity.to_string(),
        }
    }

    /// Create a new circular dependency error
    pub fn circular_dependency(entities: Vec<String>) -> Self {
        Self::CircularDependency { entities }
    }

    /// Create a new constraint violation error
    pub fn constraint_violation(constraint: &str, details: &str) -> Self {
        Self::ConstraintViolation {
            constraint: constraint.to_string(),
            details: details.to_string(),
        }
    }

    /// Create a new I/O error
    pub fn io_error(operation: &str, details: &str) -> Self {
        Self::IoError {
            operation: operation.to_string(),
            details: details.to_string(),
        }
    }

    /// Create a new I/O error with path
    pub fn io_error_with_path(operation: &str, path: &str, details: &str) -> Self {
        Self::IoErrorWithPath {
            operation: operation.to_string(),
            path: path.to_string(),
            details: details.to_string(),
        }
    }

    /// Create a new serialization error
    pub fn serialization_error(operation: &str, details: &str) -> Self {
        Self::SerializationError {
            operation: operation.to_string(),
            details: details.to_string(),
        }
    }

    /// Create a new configuration error
    pub fn configuration_error(details: &str) -> Self {
        Self::ConfigurationError {
            details: details.to_string(),
        }
    }

    /// Create a new configuration invalid error
    pub fn configuration_invalid(field: &str, value: &str, reason: &str) -> Self {
        Self::ConfigurationInvalid {
            field: field.to_string(),
            value: value.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a new operation not allowed error
    pub fn operation_not_allowed(operation: &str, reason: &str) -> Self {
        Self::OperationNotAllowed {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a new project not found error
    pub fn project_not_found(code: &str) -> Self {
        Self::ProjectNotFound { code: code.to_string() }
    }

    /// Create a new resource not found error
    pub fn resource_not_found(code: &str) -> Self {
        Self::ResourceNotFound { code: code.to_string() }
    }

    /// Create a new task not found error
    pub fn task_not_found(code: &str) -> Self {
        Self::TaskNotFound { code: code.to_string() }
    }

    /// Create a new company not found error
    pub fn company_not_found(code: &str) -> Self {
        Self::CompanyNotFound { code: code.to_string() }
    }

    /// Create a new project already exists error
    pub fn project_already_exists(code: &str) -> Self {
        Self::ProjectAlreadyExists { code: code.to_string() }
    }

    /// Create a new resource already exists error
    pub fn resource_already_exists(code: &str) -> Self {
        Self::ResourceAlreadyExists { code: code.to_string() }
    }

    /// Create a new task already exists error
    pub fn task_already_exists(code: &str) -> Self {
        Self::TaskAlreadyExists { code: code.to_string() }
    }

    /// Create a new company already exists error
    pub fn company_already_exists(code: &str) -> Self {
        Self::CompanyAlreadyExists { code: code.to_string() }
    }

    /// Create a new project invalid state error
    pub fn project_invalid_state(current: &str, expected: &str) -> Self {
        Self::ProjectInvalidState {
            current: current.to_string(),
            expected: expected.to_string(),
        }
    }

    /// Create a new resource invalid state error
    pub fn resource_invalid_state(current: &str, expected: &str) -> Self {
        Self::ResourceInvalidState {
            current: current.to_string(),
            expected: expected.to_string(),
        }
    }

    /// Create a new task invalid state error
    pub fn task_invalid_state(current: &str, expected: &str) -> Self {
        Self::TaskInvalidState {
            current: current.to_string(),
            expected: expected.to_string(),
        }
    }

    /// Create a new generic domain error
    pub fn create_domain_error(message: &str) -> Self {
        Self::DomainError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::EntityNotFound {
                entity_type,
                identifier,
            } => {
                write!(f, "{} with identifier '{}' not found", entity_type, identifier)
            }
            DomainError::EntityAlreadyExists {
                entity_type,
                identifier,
            } => {
                write!(f, "{} with identifier '{}' already exists", entity_type, identifier)
            }
            DomainError::ValidationError { field, message } => {
                write!(f, "Validation error in field '{}': {}", field, message)
            }
            DomainError::BusinessRuleViolation { rule, details } => {
                write!(f, "Business rule violation '{}': {}", rule, details)
            }
            DomainError::InvalidState {
                current,
                expected,
                entity,
            } => {
                write!(
                    f,
                    "Invalid state for {}: current '{}', expected '{}'",
                    entity, current, expected
                )
            }
            DomainError::CircularDependency { entities } => {
                write!(f, "Circular dependency detected: {}", entities.join(" -> "))
            }
            DomainError::ConstraintViolation { constraint, details } => {
                write!(f, "Constraint violation '{}': {}", constraint, details)
            }
            DomainError::IoError { operation, details } => {
                write!(f, "I/O error during {}: {}", operation, details)
            }
            DomainError::IoErrorWithPath {
                operation,
                path,
                details,
            } => {
                write!(f, "I/O error during {} on path '{}': {}", operation, path, details)
            }
            DomainError::SerializationError { operation, details } => {
                write!(f, "Serialization error during {}: {}", operation, details)
            }
            DomainError::ConfigurationError { details } => {
                write!(f, "Configuration error: {}", details)
            }
            DomainError::ConfigurationInvalid { field, value, reason } => {
                write!(
                    f,
                    "Configuration invalid for field '{}' with value '{}': {}",
                    field, value, reason
                )
            }
            DomainError::OperationNotAllowed { operation, reason } => {
                write!(f, "Operation '{}' not allowed: {}", operation, reason)
            }
            DomainError::ProjectNotFound { code } => {
                write!(f, "Project with code '{}' not found", code)
            }
            DomainError::ResourceNotFound { code } => {
                write!(f, "Resource with code '{}' not found", code)
            }
            DomainError::TaskNotFound { code } => {
                write!(f, "Task with code '{}' not found", code)
            }
            DomainError::CompanyNotFound { code } => {
                write!(f, "Company with code '{}' not found", code)
            }
            DomainError::ProjectAlreadyExists { code } => {
                write!(f, "Project with code '{}' already exists", code)
            }
            DomainError::ResourceAlreadyExists { code } => {
                write!(f, "Resource with code '{}' already exists", code)
            }
            DomainError::TaskAlreadyExists { code } => {
                write!(f, "Task with code '{}' already exists", code)
            }
            DomainError::CompanyAlreadyExists { code } => {
                write!(f, "Company with code '{}' already exists", code)
            }
            DomainError::ProjectInvalidState { current, expected } => {
                write!(
                    f,
                    "Project invalid state: current '{}', expected '{}'",
                    current, expected
                )
            }
            DomainError::ResourceInvalidState { current, expected } => {
                write!(
                    f,
                    "Resource invalid state: current '{}', expected '{}'",
                    current, expected
                )
            }
            DomainError::TaskInvalidState { current, expected } => {
                write!(f, "Task invalid state: current '{}', expected '{}'", current, expected)
            }
            DomainError::DomainError { message } => {
                write!(f, "Domain error: {}", message)
            }
        }
    }
}

impl std::error::Error for DomainError {}

impl From<std::io::Error> for DomainError {
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError {
            operation: "I/O operation".to_string(),
            details: io_error.to_string(),
        }
    }
}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

// Conversion from AppError to DomainError
impl From<crate::application::errors::AppError> for DomainError {
    fn from(err: crate::application::errors::AppError) -> Self {
        match err {
            crate::application::errors::AppError::ProjectNotFound { code } => DomainError::ProjectNotFound { code },
            crate::application::errors::AppError::ResourceNotFound { code } => DomainError::ResourceNotFound { code },
            crate::application::errors::AppError::TaskNotFound { code } => DomainError::TaskNotFound { code },
            crate::application::errors::AppError::CompanyNotFound { code } => DomainError::CompanyNotFound { code },
            crate::application::errors::AppError::ValidationError { field, message } => {
                DomainError::ValidationError { field, message }
            }
            crate::application::errors::AppError::OperationNotAllowed { operation, reason } => {
                DomainError::OperationNotAllowed { operation, reason }
            }
            crate::application::errors::AppError::IoError { operation, details } => {
                DomainError::IoError { operation, details }
            }
            crate::application::errors::AppError::IoErrorWithPath {
                operation,
                path,
                details,
            } => DomainError::IoErrorWithPath {
                operation,
                path,
                details,
            },
            crate::application::errors::AppError::SerializationError { format, details } => {
                DomainError::SerializationError {
                    operation: format,
                    details,
                }
            }
            crate::application::errors::AppError::ConfigurationNotFound { path } => DomainError::ConfigurationError {
                details: format!("Configuration not found at path '{}'", path),
            },
            crate::application::errors::AppError::ConfigurationInvalid { field, value, reason } => {
                DomainError::ConfigurationInvalid { field, value, reason }
            }
            _ => DomainError::DomainError {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_not_found_error() {
        let error = DomainError::entity_not_found("Project", "PROJ-001");
        assert_eq!(error.to_string(), "Project with identifier 'PROJ-001' not found");
    }

    #[test]
    fn test_validation_error() {
        let error = DomainError::validation_error("name", "Name cannot be empty");
        assert_eq!(
            error.to_string(),
            "Validation error in field 'name': Name cannot be empty"
        );
    }

    #[test]
    fn test_business_rule_violation() {
        let error = DomainError::business_rule_violation(
            "no_circular_deps",
            "Task A depends on Task B which depends on Task A",
        );
        assert_eq!(
            error.to_string(),
            "Business rule violation 'no_circular_deps': Task A depends on Task B which depends on Task A"
        );
    }

    #[test]
    fn test_circular_dependency_error() {
        let error =
            DomainError::circular_dependency(vec!["Task A".to_string(), "Task B".to_string(), "Task A".to_string()]);
        assert_eq!(
            error.to_string(),
            "Circular dependency detected: Task A -> Task B -> Task A"
        );
    }
}
