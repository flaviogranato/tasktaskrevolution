use std::error::Error as StdError;
use std::fmt;

/// Unified application error type
/// Replaces all domain-specific error types with a single, idiomatic error enum
#[derive(Debug, PartialEq)]
pub enum AppError {
    // Entity Not Found Errors
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
    ConfigurationNotFound {
        path: String,
    },

    // Entity Already Exists Errors
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

    // Invalid State Errors
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

    // Validation Errors
    ValidationError {
        field: String,
        message: String,
    },
    ProjectValidationFailed {
        details: String,
    },
    ResourceValidationFailed {
        details: String,
    },
    TaskValidationFailed {
        details: String,
    },
    ConfigurationInvalid {
        field: String,
        value: String,
        reason: String,
    },
    ConfigurationMissing {
        field: String,
    },

    // Business Logic Errors
    TaskAssignmentFailed {
        reason: String,
    },
    CircularDependency {
        task_codes: String,
    },
    ModificationNotAllowed {
        entity: String,
        state: String,
        reason: String,
    },
    OperationNotAllowed {
        operation: String,
        reason: String,
    },

    // I/O Errors
    IoError {
        operation: String,
        details: String,
    },
    IoErrorWithPath {
        operation: String,
        path: String,
        details: String,
    },
    FileNotFound {
        path: String,
    },
    FileReadError {
        path: String,
        details: String,
    },
    FileWriteError {
        path: String,
        details: String,
    },
    DirectoryNotFound {
        path: String,
    },
    DirectoryCreateError {
        path: String,
        details: String,
    },

    // Serialization Errors
    SerializationError {
        format: String,
        details: String,
    },
    DeserializationError {
        format: String,
        details: String,
    },
    FileParseError {
        path: String,
        format: String,
        details: String,
    },

    // Repository Errors
    RepositoryError {
        operation: String,
        details: String,
    },
    PersistenceError {
        operation: String,
        details: String,
    },
    DatabaseError {
        operation: String,
        details: String,
    },
    NetworkError {
        operation: String,
        details: String,
    },
    CacheError {
        operation: String,
        details: String,
    },

    // Path and Configuration Errors
    PathInvalid {
        path: String,
        reason: String,
    },
    ManagerNotFound {
        identifier: String,
    },
    InvalidManagerData {
        field: String,
        reason: String,
    },
    RepositoryInitializationFailed {
        reason: String,
    },

    // Generic Errors
    Generic {
        message: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Entity Not Found Errors
            AppError::ProjectNotFound { code } => {
                write!(f, "Project with code '{}' not found", code)
            }
            AppError::ResourceNotFound { code } => {
                write!(f, "Resource with code '{}' not found", code)
            }
            AppError::TaskNotFound { code } => {
                write!(f, "Task with code '{}' not found", code)
            }
            AppError::CompanyNotFound { code } => {
                write!(f, "Company with code '{}' not found", code)
            }
            AppError::ConfigurationNotFound { path } => {
                write!(f, "Configuration not found at path '{}'", path)
            }

            // Entity Already Exists Errors
            AppError::ProjectAlreadyExists { code } => {
                write!(f, "Project with code '{}' already exists", code)
            }
            AppError::ResourceAlreadyExists { code } => {
                write!(f, "Resource with code '{}' already exists", code)
            }
            AppError::TaskAlreadyExists { code } => {
                write!(f, "Task with code '{}' already exists", code)
            }
            AppError::CompanyAlreadyExists { code } => {
                write!(f, "Company with code '{}' already exists", code)
            }

            // Invalid State Errors
            AppError::ProjectInvalidState { current, expected } => {
                write!(f, "Project is in invalid state '{}', expected '{}'", current, expected)
            }
            AppError::ResourceInvalidState { current, expected } => {
                write!(f, "Resource is in invalid state '{}', expected '{}'", current, expected)
            }
            AppError::TaskInvalidState { current, expected } => {
                write!(f, "Task is in invalid state '{}', expected '{}'", current, expected)
            }

            // Validation Errors
            AppError::ValidationError { field, message } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            AppError::ProjectValidationFailed { details } => {
                write!(f, "Project validation failed: {}", details)
            }
            AppError::ResourceValidationFailed { details } => {
                write!(f, "Resource validation failed: {}", details)
            }
            AppError::TaskValidationFailed { details } => {
                write!(f, "Task validation failed: {}", details)
            }
            AppError::ConfigurationInvalid { field, value, reason } => {
                write!(
                    f,
                    "Invalid configuration for field '{}' with value '{}': {}",
                    field, value, reason
                )
            }
            AppError::ConfigurationMissing { field } => {
                write!(f, "Missing configuration for field '{}'", field)
            }

            // Business Logic Errors
            AppError::TaskAssignmentFailed { reason } => {
                write!(f, "Task assignment failed: {}", reason)
            }
            AppError::CircularDependency { task_codes } => {
                write!(f, "Circular dependency detected between tasks: {}", task_codes)
            }
            AppError::ModificationNotAllowed { entity, state, reason } => {
                write!(f, "Cannot modify {} in state '{}': {}", entity, state, reason)
            }
            AppError::OperationNotAllowed { operation, reason } => {
                write!(f, "Operation '{}' not allowed: {}", operation, reason)
            }

            // I/O Errors
            AppError::IoError { operation, details } => {
                write!(f, "I/O error during {}: {}", operation, details)
            }
            AppError::IoErrorWithPath {
                operation,
                path,
                details,
            } => {
                write!(f, "I/O error during {} on path '{}': {}", operation, path, details)
            }
            AppError::FileNotFound { path } => {
                write!(f, "File not found at path '{}'", path)
            }
            AppError::FileReadError { path, details } => {
                write!(f, "Error reading file at path '{}': {}", path, details)
            }
            AppError::FileWriteError { path, details } => {
                write!(f, "Error writing file at path '{}': {}", path, details)
            }
            AppError::DirectoryNotFound { path } => {
                write!(f, "Directory not found at path '{}'", path)
            }
            AppError::DirectoryCreateError { path, details } => {
                write!(f, "Error creating directory at path '{}': {}", path, details)
            }

            // Serialization Errors
            AppError::SerializationError { format, details } => {
                write!(f, "Serialization error for format '{}': {}", format, details)
            }
            AppError::DeserializationError { format, details } => {
                write!(f, "Deserialization error for format '{}': {}", format, details)
            }
            AppError::FileParseError { path, format, details } => {
                write!(f, "Error parsing {} file at path '{}': {}", format, path, details)
            }

            // Repository Errors
            AppError::RepositoryError { operation, details } => {
                write!(f, "Repository error during {}: {}", operation, details)
            }
            AppError::PersistenceError { operation, details } => {
                write!(f, "Persistence error during {}: {}", operation, details)
            }
            AppError::DatabaseError { operation, details } => {
                write!(f, "Database error during {}: {}", operation, details)
            }
            AppError::NetworkError { operation, details } => {
                write!(f, "Network error during {}: {}", operation, details)
            }
            AppError::CacheError { operation, details } => {
                write!(f, "Cache error during {}: {}", operation, details)
            }

            // Path and Configuration Errors
            AppError::PathInvalid { path, reason } => {
                write!(f, "Invalid path '{}': {}", path, reason)
            }
            AppError::ManagerNotFound { identifier } => {
                write!(f, "Manager not found with identifier '{}'", identifier)
            }
            AppError::InvalidManagerData { field, reason } => {
                write!(f, "Invalid manager data for field '{}': {}", field, reason)
            }
            AppError::RepositoryInitializationFailed { reason } => {
                write!(f, "Repository initialization failed: {}", reason)
            }

            // Generic Errors
            AppError::Generic { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl StdError for AppError {}

// Convenience constructors
impl AppError {
    /// Create a project not found error
    pub fn project_not_found(code: impl Into<String>) -> Self {
        Self::ProjectNotFound { code: code.into() }
    }

    /// Create a resource not found error
    pub fn resource_not_found(code: impl Into<String>) -> Self {
        Self::ResourceNotFound { code: code.into() }
    }

    /// Create a task not found error
    pub fn task_not_found(code: impl Into<String>) -> Self {
        Self::TaskNotFound { code: code.into() }
    }

    /// Create a company not found error
    pub fn company_not_found(code: impl Into<String>) -> Self {
        Self::CompanyNotFound { code: code.into() }
    }

    /// Create a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a project validation failed error
    pub fn project_validation_failed(details: impl Into<String>) -> Self {
        Self::ProjectValidationFailed {
            details: details.into(),
        }
    }

    /// Create a resource validation failed error
    pub fn resource_validation_failed(details: impl Into<String>) -> Self {
        Self::ResourceValidationFailed {
            details: details.into(),
        }
    }

    /// Create a task validation failed error
    pub fn task_validation_failed(details: impl Into<String>) -> Self {
        Self::TaskValidationFailed {
            details: details.into(),
        }
    }

    /// Create a repository error
    pub fn repository_error(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::RepositoryError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create a persistence error
    pub fn persistence_error(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::PersistenceError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create an I/O error
    pub fn io_error(operation: impl Into<String>, details: impl Into<String>) -> Self {
        Self::IoError {
            operation: operation.into(),
            details: details.into(),
        }
    }

    /// Create an I/O error with path
    pub fn io_error_with_path(
        operation: impl Into<String>,
        path: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::IoErrorWithPath {
            operation: operation.into(),
            path: path.into(),
            details: details.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error(format: impl Into<String>, details: impl Into<String>) -> Self {
        Self::SerializationError {
            format: format.into(),
            details: details.into(),
        }
    }

    /// Create a deserialization error
    pub fn deserialization_error(format: impl Into<String>, details: impl Into<String>) -> Self {
        Self::DeserializationError {
            format: format.into(),
            details: details.into(),
        }
    }

    /// Check if this is a project not found error
    pub fn is_project_not_found(&self) -> bool {
        matches!(self, Self::ProjectNotFound { .. })
    }

    /// Check if this is a resource not found error
    pub fn is_resource_not_found(&self) -> bool {
        matches!(self, Self::ResourceNotFound { .. })
    }

    /// Check if this is a task not found error
    pub fn is_task_not_found(&self) -> bool {
        matches!(self, Self::TaskNotFound { .. })
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::ValidationError { .. })
    }
}

// Automatic conversions for common error types
impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::Generic { message }
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            operation: "file operation".to_string(),
            details: err.to_string(),
        }
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::SerializationError {
            format: "YAML".to_string(),
            details: err.to_string(),
        }
    }
}

// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_app_error_creation() {
        let error = AppError::project_not_found("PROJ-001");
        assert!(matches!(error, AppError::ProjectNotFound { code } if code == "PROJ-001"));
    }

    #[test]
    fn test_app_error_display_formatting() {
        let error = AppError::project_not_found("PROJ-001");
        let display = format!("{}", error);
        assert!(display.contains("Project with code 'PROJ-001' not found"));
    }

    #[test]
    fn test_app_error_from_string() {
        let error: AppError = "Custom error message".to_string().into();
        assert!(matches!(error, AppError::Generic { message } if message == "Custom error message"));
    }

    #[test]
    fn test_app_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let app_error: AppError = io_error.into();
        assert!(matches!(app_error, AppError::IoError { operation, .. } if operation == "file operation"));
    }

    #[test]
    fn test_app_error_from_serde_yaml_error() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let app_error: AppError = yaml_error.into();
        assert!(matches!(app_error, AppError::SerializationError { format, .. } if format == "YAML"));
    }

    #[test]
    fn test_app_error_is_project_not_found() {
        let error = AppError::project_not_found("PROJ-001");
        assert!(error.is_project_not_found());
        assert!(!error.is_resource_not_found());
        assert!(!error.is_task_not_found());
        assert!(!error.is_validation_error());
    }

    #[test]
    fn test_app_error_io_with_path() {
        let error = AppError::io_error_with_path("read", "/path/to/file", "Permission denied");
        let display = format!("{}", error);
        assert!(display.contains("I/O error during read on path '/path/to/file'"));
    }

    #[test]
    fn test_app_error_serialization() {
        let error = AppError::serialization_error("JSON", "Invalid UTF-8");
        let display = format!("{}", error);
        assert!(display.contains("Serialization error for format 'JSON': Invalid UTF-8"));
    }

    #[test]
    fn test_app_result() {
        let result: AppResult<String> = Ok("Success".to_string());
        assert!(result.is_ok());
    }

    // Test all error variants creation
    #[test]
    fn test_all_error_variants_creation() {
        // Entity Not Found Errors
        let project_not_found = AppError::project_not_found("PROJ-001");
        assert!(matches!(project_not_found, AppError::ProjectNotFound { code } if code == "PROJ-001"));

        let resource_not_found = AppError::resource_not_found("RES-001");
        assert!(matches!(resource_not_found, AppError::ResourceNotFound { code } if code == "RES-001"));

        let task_not_found = AppError::task_not_found("TASK-001");
        assert!(matches!(task_not_found, AppError::TaskNotFound { code } if code == "TASK-001"));

        let company_not_found = AppError::company_not_found("COMP-001");
        assert!(matches!(company_not_found, AppError::CompanyNotFound { code } if code == "COMP-001"));

        // Validation Errors
        let validation_error = AppError::validation_error("name", "Name cannot be empty");
        assert!(matches!(validation_error, AppError::ValidationError { field, message } 
            if field == "name" && message == "Name cannot be empty"));

        let project_validation = AppError::project_validation_failed("Invalid project data");
        assert!(
            matches!(project_validation, AppError::ProjectValidationFailed { details } 
            if details == "Invalid project data")
        );

        let resource_validation = AppError::resource_validation_failed("Invalid resource data");
        assert!(
            matches!(resource_validation, AppError::ResourceValidationFailed { details }
            if details == "Invalid resource data")
        );

        let task_validation = AppError::task_validation_failed("Invalid task data");
        assert!(matches!(task_validation, AppError::TaskValidationFailed { details }
            if details == "Invalid task data"));

        // Repository and Persistence Errors
        let repository_error = AppError::repository_error("find", "Database connection failed");
        assert!(
            matches!(repository_error, AppError::RepositoryError { operation, details }
            if operation == "find" && details == "Database connection failed")
        );

        let persistence_error = AppError::persistence_error("save", "File write failed");
        assert!(
            matches!(persistence_error, AppError::PersistenceError { operation, details }
            if operation == "save" && details == "File write failed")
        );

        // I/O Errors
        let io_error = AppError::io_error("read", "File not found");
        assert!(matches!(io_error, AppError::IoError { operation, details }
            if operation == "read" && details == "File not found"));

        let io_error_with_path = AppError::io_error_with_path("write", "/path/to/file", "Permission denied");
        assert!(
            matches!(io_error_with_path, AppError::IoErrorWithPath { operation, path, details }
            if operation == "write" && path == "/path/to/file" && details == "Permission denied")
        );

        // Serialization Errors
        let serialization_error = AppError::serialization_error("JSON", "Invalid UTF-8");
        assert!(
            matches!(serialization_error, AppError::SerializationError { format, details }
            if format == "JSON" && details == "Invalid UTF-8")
        );

        let deserialization_error = AppError::deserialization_error("YAML", "Invalid format");
        assert!(
            matches!(deserialization_error, AppError::DeserializationError { format, details }
            if format == "YAML" && details == "Invalid format")
        );
    }

    #[test]
    fn test_all_error_display_formatting() {
        // Test display formatting for all error variants
        let errors = vec![
            (
                AppError::project_not_found("PROJ-001"),
                "Project with code 'PROJ-001' not found",
            ),
            (
                AppError::resource_not_found("RES-001"),
                "Resource with code 'RES-001' not found",
            ),
            (
                AppError::task_not_found("TASK-001"),
                "Task with code 'TASK-001' not found",
            ),
            (
                AppError::company_not_found("COMP-001"),
                "Company with code 'COMP-001' not found",
            ),
            (
                AppError::validation_error("name", "Cannot be empty"),
                "Validation error for field 'name': Cannot be empty",
            ),
            (
                AppError::project_validation_failed("Invalid data"),
                "Project validation failed: Invalid data",
            ),
            (
                AppError::resource_validation_failed("Invalid data"),
                "Resource validation failed: Invalid data",
            ),
            (
                AppError::task_validation_failed("Invalid data"),
                "Task validation failed: Invalid data",
            ),
            (
                AppError::repository_error("find", "Database connection failed"),
                "Repository error during find: Database connection failed",
            ),
            (
                AppError::persistence_error("save", "File write failed"),
                "Persistence error during save: File write failed",
            ),
            (
                AppError::io_error("read", "File not found"),
                "I/O error during read: File not found",
            ),
            (
                AppError::io_error_with_path("write", "/path", "Permission denied"),
                "I/O error during write on path '/path': Permission denied",
            ),
            (
                AppError::serialization_error("JSON", "Invalid UTF-8"),
                "Serialization error for format 'JSON': Invalid UTF-8",
            ),
            (
                AppError::deserialization_error("YAML", "Invalid format"),
                "Deserialization error for format 'YAML': Invalid format",
            ),
        ];

        for (error, expected) in errors {
            let display = format!("{}", error);
            assert!(
                display.contains(expected),
                "Expected '{}' to contain '{}', got '{}'",
                display,
                expected,
                display
            );
        }
    }

    #[test]
    fn test_error_type_checking_methods() {
        let project_error = AppError::project_not_found("PROJ-001");
        assert!(project_error.is_project_not_found());
        assert!(!project_error.is_resource_not_found());
        assert!(!project_error.is_task_not_found());
        assert!(!project_error.is_validation_error());

        let resource_error = AppError::resource_not_found("RES-001");
        assert!(!resource_error.is_project_not_found());
        assert!(resource_error.is_resource_not_found());
        assert!(!resource_error.is_task_not_found());
        assert!(!resource_error.is_validation_error());

        let task_error = AppError::task_not_found("TASK-001");
        assert!(!task_error.is_project_not_found());
        assert!(!task_error.is_resource_not_found());
        assert!(task_error.is_task_not_found());
        assert!(!task_error.is_validation_error());

        let company_error = AppError::company_not_found("COMP-001");
        assert!(!company_error.is_project_not_found());
        assert!(!company_error.is_resource_not_found());
        assert!(!company_error.is_task_not_found());
        assert!(!company_error.is_validation_error());

        let validation_error = AppError::validation_error("field", "message");
        assert!(!validation_error.is_project_not_found());
        assert!(!validation_error.is_resource_not_found());
        assert!(!validation_error.is_task_not_found());
        assert!(validation_error.is_validation_error());
    }

    #[test]
    fn test_error_from_traits() {
        // Test From<String>
        let string_error: AppError = "Custom error".to_string().into();
        assert!(matches!(string_error, AppError::Generic { message } if message == "Custom error"));

        // Test From<&str>
        let str_error: AppError = "Custom error".into();
        assert!(matches!(str_error, AppError::Generic { message } if message == "Custom error"));

        // Test From<io::Error>
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_error: AppError = io_error.into();
        assert!(matches!(app_error, AppError::IoError { operation, .. } if operation == "file operation"));

        // Test From<serde_yaml::Error>
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let app_error: AppError = yaml_error.into();
        assert!(matches!(app_error, AppError::SerializationError { format, .. } if format == "YAML"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = AppError::project_not_found("PROJ-001");
        let debug = format!("{:?}", error);
        assert!(debug.contains("ProjectNotFound"));
        assert!(debug.contains("PROJ-001"));
    }

    #[test]
    fn test_error_partial_eq() {
        let error1 = AppError::project_not_found("PROJ-001");
        let error2 = AppError::project_not_found("PROJ-001");
        let error3 = AppError::project_not_found("PROJ-002");
        let error4 = AppError::resource_not_found("PROJ-001");

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
    }

    #[test]
    fn test_app_result_error_handling() {
        let success: AppResult<String> = Ok("Success".to_string());
        let failure: AppResult<String> = Err(AppError::project_not_found("PROJ-001"));

        assert!(success.is_ok());
        assert!(failure.is_err());

        if let Ok(value) = success {
            assert_eq!(value, "Success");
        }

        if let Err(error) = failure {
            assert!(error.is_project_not_found());
        }
    }
}
