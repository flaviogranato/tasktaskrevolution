use std::fmt;
use std::error::Error as StdError;

/// Base error type for all domain errors
#[derive(Debug)]
pub struct DomainError {
    kind: DomainErrorKind,
    source: Option<Box<dyn StdError + Send + Sync>>,
    context: Option<String>,
}

#[derive(Debug)]
pub enum DomainErrorKind {
    // Project Management Errors
    ProjectNotFound { code: String },
    ProjectAlreadyExists { code: String },
    ProjectInvalidState { current: String, expected: String },
    ProjectValidationFailed { details: Vec<String> },
    
    // Resource Management Errors
    ResourceNotFound { code: String },
    ResourceAlreadyExists { code: String },
    ResourceInvalidState { current: String, expected: String },
    ResourceValidationFailed { details: Vec<String> },
    
    // Task Management Errors
    TaskNotFound { code: String },
    TaskAlreadyExists { code: String },
    TaskInvalidState { current: String, expected: String },
    TaskValidationFailed { details: Vec<String> },
    TaskAssignmentFailed { reason: String },
    
    // Configuration Errors
    ConfigurationInvalid { field: String, value: String },
    ConfigurationMissing { field: String },
    
    // Repository Errors
    RepositoryError { operation: String, details: String },
    PersistenceError { operation: String, details: String },
    
    // Validation Errors
    ValidationError { field: String, message: String },
    
    // Generic Errors
    Generic { message: String },
    Io { operation: String, path: Option<String> },
    Serialization { format: String, details: String },
}

impl DomainError {
    /// Create a new domain error with a specific kind
    pub fn new(kind: DomainErrorKind) -> Self {
        Self {
            kind,
            source: None,
            context: None,
        }
    }

    /// Add context information to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a source error to this error
    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {  // 'static necessário para Box<dyn>
        self.source = Some(Box::new(source));
        self
    }

    /// Get the kind of this error
    pub fn kind(&self) -> &DomainErrorKind {
        &self.kind
    }

    /// Get the context of this error
    pub fn context(&self) -> Option<&String> {
        self.context.as_ref()
    }

    /// Check if this is a specific type of error
    pub fn is_project_not_found(&self) -> bool {
        matches!(self.kind, DomainErrorKind::ProjectNotFound { .. })
    }

    pub fn is_resource_not_found(&self) -> bool {
        matches!(self.kind, DomainErrorKind::ResourceNotFound { .. })
    }

    pub fn is_task_not_found(&self) -> bool {
        matches!(self.kind, DomainErrorKind::TaskNotFound { .. })
    }

    pub fn is_validation_error(&self) -> bool {
        matches!(self.kind, DomainErrorKind::ValidationError { .. })
    }

    /// Create a project not found error
    pub fn project_not_found(code: impl Into<String>) -> Self {
        Self::new(DomainErrorKind::ProjectNotFound { code: code.into() })
    }

    /// Create a resource not found error
    pub fn resource_not_found(code: impl Into<String>) -> Self {
        Self::new(DomainErrorKind::ResourceNotFound { code: code.into() })
    }

    /// Create a task not found error
    pub fn task_not_found(code: impl Into<String>) -> Self {
        Self::new(DomainErrorKind::TaskNotFound { code: code.into() })
    }

    /// Create a validation error
    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(DomainErrorKind::ValidationError {
            field: field.into(),
            message: message.into(),
        })
    }

    /// Create a project invalid state error
    pub fn project_invalid_state(current: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::new(DomainErrorKind::ProjectInvalidState {
            current: current.into(),
            expected: expected.into(),
        })
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            DomainErrorKind::ProjectNotFound { code } => {
                write!(f, "Project with code '{}' not found", code)?
            }
            DomainErrorKind::ProjectAlreadyExists { code } => {
                write!(f, "Project with code '{}' already exists", code)?
            }
            DomainErrorKind::ProjectInvalidState { current, expected } => {
                write!(f, "Project is in invalid state '{}', expected '{}'", current, expected)?
            }
            DomainErrorKind::ProjectValidationFailed { details } => {
                write!(f, "Project validation failed: {}", details.join(", "))?
            }
            DomainErrorKind::ResourceNotFound { code } => {
                write!(f, "Resource with code '{}' not found", code)?
            }
            DomainErrorKind::ResourceAlreadyExists { code } => {
                write!(f, "Resource with code '{}' already exists", code)?
            }
            DomainErrorKind::ResourceInvalidState { current, expected } => {
                write!(f, "Resource is in invalid state '{}', expected '{}'", current, expected)?
            }
            DomainErrorKind::ResourceValidationFailed { details } => {
                write!(f, "Resource validation failed: {}", details.join(", "))?
            }
            DomainErrorKind::TaskNotFound { code } => {
                write!(f, "Task with code '{}' not found", code)?
            }
            DomainErrorKind::TaskAlreadyExists { code } => {
                write!(f, "Task with code '{}' already exists", code)?
            }
            DomainErrorKind::TaskInvalidState { current, expected } => {
                write!(f, "Task is in invalid state '{}', expected '{}'", current, expected)?
            }
            DomainErrorKind::TaskValidationFailed { details } => {
                write!(f, "Task validation failed: {}", details.join(", "))?
            }
            DomainErrorKind::TaskAssignmentFailed { reason } => {
                write!(f, "Task assignment failed: {}", reason)?
            }
            DomainErrorKind::ConfigurationInvalid { field, value } => {
                write!(f, "Invalid configuration for field '{}': {}", field, value)?
            }
            DomainErrorKind::ConfigurationMissing { field } => {
                write!(f, "Missing configuration for field '{}'", field)?
            }
            DomainErrorKind::RepositoryError { operation, details } => {
                write!(f, "Repository error during {}: {}", operation, details)?
            }
            DomainErrorKind::PersistenceError { operation, details } => {
                write!(f, "Persistence error during {}: {}", operation, details)?
            }
            DomainErrorKind::ValidationError { field, message } => {
                write!(f, "Validation error for field '{}': {}", field, message)?
            }
            DomainErrorKind::Generic { message } => {
                write!(f, "{}", message)?
            }
            DomainErrorKind::Io { operation, path } => {
                match path {
                    Some(path) => write!(f, "I/O error during {} on path '{}'", operation, path)?,
                    None => write!(f, "I/O error during {}", operation)?,
                }
            }
            DomainErrorKind::Serialization { format, details } => {
                write!(f, "Serialization error for format '{}': {}", format, details)?
            }
        }

        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }

        Ok(())
    }
}

impl StdError for DomainError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn StdError + 'static))
    }
}

// Convenience constructors for common error patterns
impl From<String> for DomainError {
    fn from(message: String) -> Self {
        Self::new(DomainErrorKind::Generic { message })
    }
}

impl From<&str> for DomainError {
    fn from(message: &str) -> Self {
        Self::new(DomainErrorKind::Generic { message: message.to_string() })
    }
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        Self::new(DomainErrorKind::Io {
            operation: "file operation".to_string(),
            path: None,
        }).with_source(err)
    }
}

impl From<serde_yaml::Error> for DomainError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::new(DomainErrorKind::Serialization {
            format: "YAML".to_string(),
            details: err.to_string(),
        }).with_source(err)
    }
}

// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

// Extension trait for adding context to Results
pub trait ResultExt<T, E> {
    fn with_context<C>(self, context: C) -> Result<T, DomainError>
    where
        C: Into<String>,
        E: Into<DomainError>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Into<DomainError>,
{
    fn with_context<C>(self, context: C) -> Result<T, DomainError>
    where
        C: Into<String>,
    {
        self.map_err(|e| e.into().with_context(context))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_domain_error_creation() {
        let error = DomainError::new(DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        });
        
        assert!(matches!(error.kind(), DomainErrorKind::ProjectNotFound { code } if code == "PROJ-001"));
        assert!(error.source().is_none());
        assert!(error.context().is_none());
    }

    #[test]
    fn test_domain_error_with_context() {
        let error = DomainError::new(DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        })
        .with_context("Failed to load project from repository");
        
        assert!(matches!(error.kind(), DomainErrorKind::ProjectNotFound { code } if code == "PROJ-001"));
        assert_eq!(error.context(), Some(&"Failed to load project from repository".to_string()));
    }

    #[test]
    fn test_domain_error_with_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let domain_error = DomainError::new(DomainErrorKind::Io {
            operation: "read".to_string(),
            path: Some("/path/to/file".to_string()),
        })
        .with_source(io_error);
        
        assert!(matches!(domain_error.kind(), DomainErrorKind::Io { operation, path } 
            if operation == "read" && path.as_deref() == Some("/path/to/file")));
        assert!(domain_error.source().is_some());
    }

    #[test]
    fn test_domain_error_display_formatting() {
        let error = DomainError::new(DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        })
        .with_context("Repository operation failed");
        
        let display = format!("{}", error);
        assert!(display.contains("Project with code 'PROJ-001' not found"));
        assert!(display.contains("(Context: Repository operation failed)"));
    }

    #[test]
    fn test_domain_error_display_without_context() {
        let error = DomainError::new(DomainErrorKind::ResourceNotFound {
            code: "RES-001".to_string(),
        });
        
        let display = format!("{}", error);
        assert!(display.contains("Resource with code 'RES-001' not found"));
        assert!(!display.contains("(Context:"));
    }

    #[test]
    fn test_domain_error_display_io_with_path() {
        let error = DomainError::new(DomainErrorKind::Io {
            operation: "write".to_string(),
            path: Some("/tmp/file.yaml".to_string()),
        });
        
        let display = format!("{}", error);
        assert!(display.contains("I/O error during write on path '/tmp/file.yaml'"));
    }

    #[test]
    fn test_domain_error_display_io_without_path() {
        let error = DomainError::new(DomainErrorKind::Io {
            operation: "read".to_string(),
            path: None,
        });
        
        let display = format!("{}", error);
        assert!(display.contains("I/O error during read"));
        assert!(!display.contains("on path"));
    }

    #[test]
    fn test_domain_error_display_validation_error() {
        let error = DomainError::new(DomainErrorKind::ValidationError {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        });
        
        let display = format!("{}", error);
        assert!(display.contains("Validation error for field 'email': Invalid email format"));
    }

    #[test]
    fn test_domain_error_display_serialization_error() {
        let error = DomainError::new(DomainErrorKind::Serialization {
            format: "JSON".to_string(),
            details: "Invalid UTF-8 sequence".to_string(),
        });
        
        let display = format!("{}", error);
        assert!(display.contains("Serialization error for format 'JSON': Invalid UTF-8 sequence"));
    }

    #[test]
    fn test_domain_error_is_project_not_found() {
        let error = DomainError::project_not_found("PROJ-001");
        assert!(error.is_project_not_found());
        assert!(!error.is_resource_not_found());
        assert!(!error.is_task_not_found());
        assert!(!error.is_validation_error());
    }

    #[test]
    fn test_domain_error_is_resource_not_found() {
        let error = DomainError::resource_not_found("RES-001");
        assert!(!error.is_project_not_found());
        assert!(error.is_resource_not_found());
        assert!(!error.is_task_not_found());
        assert!(!error.is_validation_error());
    }

    #[test]
    fn test_domain_error_is_task_not_found() {
        let error = DomainError::task_not_found("TASK-001");
        assert!(!error.is_project_not_found());
        assert!(!error.is_resource_not_found());
        assert!(error.is_task_not_found());
        assert!(!error.is_validation_error());
    }

    #[test]
    fn test_domain_error_is_validation_error() {
        let error = DomainError::validation_error("email", "Invalid format");
        assert!(!error.is_project_not_found());
        assert!(!error.is_resource_not_found());
        assert!(!error.is_task_not_found());
        assert!(error.is_validation_error());
    }

    #[test]
    fn test_domain_error_from_string() {
        let error: DomainError = "Custom error message".to_string().into();
        
        assert!(matches!(error.kind(), DomainErrorKind::Generic { message } 
            if message == "Custom error message"));
    }

    #[test]
    fn test_domain_error_from_str() {
        let error: DomainError = "Static error message".into();
        
        assert!(matches!(error.kind(), DomainErrorKind::Generic { message } 
            if message == "Static error message"));
    }

    #[test]
    fn test_domain_error_from_io_error() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let domain_error: DomainError = io_error.into();
        
        assert!(matches!(domain_error.kind(), DomainErrorKind::Io { operation, path } 
            if operation == "file operation" && path.is_none()));
    }

    #[test]
    fn test_domain_error_from_serde_yaml_error() {
        // Simular um erro de YAML inválido
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let domain_error: DomainError = yaml_error.into();
        
        if let DomainErrorKind::Serialization { format, details } = domain_error.kind() {
            assert_eq!(format, "YAML");
            assert!(!details.is_empty());
        } else {
            panic!("Expected Serialization error");
        }
    }

    #[test]
    fn test_result_ext_with_context() {
        let result: Result<&str, &str> = Err("Database connection failed");
        let domain_result: DomainResult<&str> = result.with_context("Failed to load user data");
        
        assert!(domain_result.is_err());
        let error = domain_result.unwrap_err();
        assert!(matches!(error.kind(), DomainErrorKind::Generic { message } 
            if message == "Database connection failed"));
        assert_eq!(error.context(), Some(&"Failed to load user data".to_string()));
    }

    #[test]
    fn test_result_ext_with_context_success() {
        let result: Result<&str, &str> = Ok("Success");
        let domain_result: DomainResult<&str> = result.with_context("Operation completed");
        
        assert!(domain_result.is_ok());
        assert_eq!(domain_result.unwrap(), "Success");
    }

    #[test]
    fn test_domain_error_error_trait_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let domain_error = DomainError::new(DomainErrorKind::Io {
            operation: "read".to_string(),
            path: None,
        })
        .with_source(io_error);
        
        let source = domain_error.source();
        assert!(source.is_some());
        
        let source_error = source.unwrap();
        assert!(source_error.to_string().contains("File not found"));
    }

    #[test]
    fn test_domain_error_error_trait_no_source() {
        let domain_error = DomainError::new(DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        });
        
        let source = domain_error.source();
        assert!(source.is_none());
    }

    #[test]
    fn test_domain_error_debug_formatting() {
        let error = DomainError::new(DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        });
        
        let debug = format!("{:?}", error);
        assert!(debug.contains("ProjectNotFound"));
        assert!(debug.contains("PROJ-001"));
    }

    #[test]
    fn test_domain_error_kind_debug_formatting() {
        let kind = DomainErrorKind::ProjectNotFound {
            code: "PROJ-001".to_string(),
        };
        
        let debug = format!("{:?}", kind);
        assert!(debug.contains("ProjectNotFound"));
        assert!(debug.contains("PROJ-001"));
    }

    #[test]
    fn test_domain_error_complex_scenario() {
        // Simular um cenário complexo de erro
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let domain_error = DomainError::new(DomainErrorKind::RepositoryError {
            operation: "save".to_string(),
            details: "Failed to persist project data".to_string(),
        })
        .with_context("Project creation workflow failed")
        .with_source(io_error);
        
        // Verificar o tipo de erro
        assert!(matches!(domain_error.kind(), DomainErrorKind::RepositoryError { operation, details } 
            if operation == "save" && details == "Failed to persist project data"));
        
        // Verificar o contexto
        assert_eq!(domain_error.context(), Some(&"Project creation workflow failed".to_string()));
        
        // Verificar a fonte
        assert!(domain_error.source().is_some());
        
        // Verificar a formatação
        let display = format!("{}", domain_error);
        assert!(display.contains("Repository error during save: Failed to persist project data"));
        assert!(display.contains("(Context: Project creation workflow failed)"));
    }
}
