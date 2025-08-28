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
    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the kind of this error
    pub fn kind(&self) -> &DomainErrorKind {
        &self.kind
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
