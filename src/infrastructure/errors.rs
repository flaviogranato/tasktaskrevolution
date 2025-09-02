#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use std::error::Error as StdError;
use std::fmt;

/// Infrastructure-specific error types
#[derive(Debug, PartialEq)]
pub enum InfrastructureError {
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
    FileParseError {
        path: String,
        format: String,
        details: String,
    },
    DirectoryNotFound {
        path: String,
    },
    DirectoryCreateError {
        path: String,
        details: String,
    },
    PathInvalid {
        path: String,
        reason: String,
    },
    SerializationError {
        format: String,
        details: String,
    },
    DeserializationError {
        format: String,
        details: String,
    },
    NetworkError {
        operation: String,
        details: String,
    },
    DatabaseError {
        operation: String,
        details: String,
    },
    CacheError {
        operation: String,
        details: String,
    },
}

impl fmt::Display for InfrastructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfrastructureError::FileNotFound { path } => {
                write!(f, "File not found at path '{}'", path)
            }
            InfrastructureError::FileReadError { path, details } => {
                write!(f, "Error reading file at path '{}': {}", path, details)
            }
            InfrastructureError::FileWriteError { path, details } => {
                write!(f, "Error writing file at path '{}': {}", path, details)
            }
            InfrastructureError::FileParseError { path, format, details } => {
                write!(f, "Error parsing {} file at path '{}': {}", format, path, details)
            }
            InfrastructureError::DirectoryNotFound { path } => {
                write!(f, "Directory not found at path '{}'", path)
            }
            InfrastructureError::DirectoryCreateError { path, details } => {
                write!(f, "Error creating directory at path '{}': {}", path, details)
            }
            InfrastructureError::PathInvalid { path, reason } => {
                write!(f, "Invalid path '{}': {}", path, reason)
            }
            InfrastructureError::SerializationError { format, details } => {
                write!(f, "Serialization error for format '{}': {}", format, details)
            }
            InfrastructureError::DeserializationError { format, details } => {
                write!(f, "Deserialization error for format '{}': {}", format, details)
            }
            InfrastructureError::NetworkError { operation, details } => {
                write!(f, "Network error during {}: {}", operation, details)
            }
            InfrastructureError::DatabaseError { operation, details } => {
                write!(f, "Database error during {}: {}", operation, details)
            }
            InfrastructureError::CacheError { operation, details } => {
                write!(f, "Cache error during {}: {}", operation, details)
            }
        }
    }
}

impl StdError for InfrastructureError {}

impl From<InfrastructureError> for DomainError {
    fn from(err: InfrastructureError) -> Self {
        match err {
            InfrastructureError::FileNotFound { path } => DomainError::new(DomainErrorKind::Io {
                operation: "file read".to_string(),
                path: Some(path),
            }),
            InfrastructureError::FileReadError { path, details } => DomainError::new(DomainErrorKind::Io {
                operation: "file read".to_string(),
                path: Some(path),
            })
            .with_context(details),
            InfrastructureError::FileWriteError { path, details } => DomainError::new(DomainErrorKind::Io {
                operation: "file write".to_string(),
                path: Some(path),
            })
            .with_context(details),
            InfrastructureError::FileParseError { path, format, details } => {
                DomainError::new(DomainErrorKind::Serialization {
                    format,
                    details: format!("Parse error at path '{}': {}", path, details),
                })
            }
            InfrastructureError::DirectoryNotFound { path } => DomainError::new(DomainErrorKind::Io {
                operation: "directory access".to_string(),
                path: Some(path),
            }),
            InfrastructureError::DirectoryCreateError { path, details } => DomainError::new(DomainErrorKind::Io {
                operation: "directory creation".to_string(),
                path: Some(path),
            })
            .with_context(details),
            InfrastructureError::PathInvalid { path, reason } => DomainError::new(DomainErrorKind::ValidationError {
                field: "path".to_string(),
                message: format!("Path '{}' is invalid: {}", path, reason),
            }),
            InfrastructureError::SerializationError { format, details } => {
                DomainError::new(DomainErrorKind::Serialization { format, details })
            }
            InfrastructureError::DeserializationError { format, details } => {
                DomainError::new(DomainErrorKind::Serialization { format, details })
            }
            InfrastructureError::NetworkError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
            InfrastructureError::DatabaseError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
            InfrastructureError::CacheError { operation, details } => {
                DomainError::new(DomainErrorKind::RepositoryError { operation, details })
            }
        }
    }
}

// Result type for infrastructure operations
pub type InfrastructureResult<T> = Result<T, InfrastructureError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error_display() {
        let error = InfrastructureError::FileNotFound {
            path: "/path/to/file.txt".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("File not found at path '/path/to/file.txt'"));
    }

    #[test]
    fn test_file_read_error_display() {
        let error = InfrastructureError::FileReadError {
            path: "/path/to/file.txt".to_string(),
            details: "Permission denied".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Error reading file at path '/path/to/file.txt'"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_file_write_error_display() {
        let error = InfrastructureError::FileWriteError {
            path: "/path/to/file.txt".to_string(),
            details: "Disk full".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Error writing file at path '/path/to/file.txt'"));
        assert!(display.contains("Disk full"));
    }

    #[test]
    fn test_file_parse_error_display() {
        let error = InfrastructureError::FileParseError {
            path: "/path/to/file.yaml".to_string(),
            format: "YAML".to_string(),
            details: "Invalid syntax".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Error parsing YAML file at path '/path/to/file.yaml'"));
        assert!(display.contains("Invalid syntax"));
    }

    #[test]
    fn test_directory_not_found_error_display() {
        let error = InfrastructureError::DirectoryNotFound {
            path: "/path/to/dir".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Directory not found at path '/path/to/dir'"));
    }

    #[test]
    fn test_directory_create_error_display() {
        let error = InfrastructureError::DirectoryCreateError {
            path: "/path/to/dir".to_string(),
            details: "Permission denied".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Error creating directory at path '/path/to/dir'"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_path_invalid_error_display() {
        let error = InfrastructureError::PathInvalid {
            path: "/invalid/path".to_string(),
            reason: "Contains invalid characters".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Invalid path '/invalid/path'"));
        assert!(display.contains("Contains invalid characters"));
    }

    #[test]
    fn test_serialization_error_display() {
        let error = InfrastructureError::SerializationError {
            format: "JSON".to_string(),
            details: "Invalid data structure".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Serialization error for format 'JSON'"));
        assert!(display.contains("Invalid data structure"));
    }

    #[test]
    fn test_deserialization_error_display() {
        let error = InfrastructureError::DeserializationError {
            format: "XML".to_string(),
            details: "Malformed XML".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Deserialization error for format 'XML'"));
        assert!(display.contains("Malformed XML"));
    }

    #[test]
    fn test_network_error_display() {
        let error = InfrastructureError::NetworkError {
            operation: "HTTP request".to_string(),
            details: "Connection timeout".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Network error during HTTP request"));
        assert!(display.contains("Connection timeout"));
    }

    #[test]
    fn test_database_error_display() {
        let error = InfrastructureError::DatabaseError {
            operation: "SELECT query".to_string(),
            details: "Connection lost".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Database error during SELECT query"));
        assert!(display.contains("Connection lost"));
    }

    #[test]
    fn test_cache_error_display() {
        let error = InfrastructureError::CacheError {
            operation: "GET operation".to_string(),
            details: "Cache miss".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Cache error during GET operation"));
        assert!(display.contains("Cache miss"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = InfrastructureError::FileNotFound {
            path: "/test/path".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("FileNotFound"));
        assert!(debug.contains("/test/path"));
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_file_not_found() {
        let infra_error = InfrastructureError::FileNotFound {
            path: "/test/path".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Io { operation, path } = domain_error.kind() {
            assert_eq!(operation, "file read");
            assert_eq!(path, &Some("/test/path".to_string()));
        } else {
            panic!("Expected Io error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_file_read_error() {
        let infra_error = InfrastructureError::FileReadError {
            path: "/test/path".to_string(),
            details: "Permission denied".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Io { operation, path } = domain_error.kind() {
            assert_eq!(operation, "file read");
            assert_eq!(path, &Some("/test/path".to_string()));
        } else {
            panic!("Expected Io error kind");
        }

        assert_eq!(domain_error.context(), Some(&"Permission denied".to_string()));
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_file_write_error() {
        let infra_error = InfrastructureError::FileWriteError {
            path: "/test/path".to_string(),
            details: "Disk full".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Io { operation, path } = domain_error.kind() {
            assert_eq!(operation, "file write");
            assert_eq!(path, &Some("/test/path".to_string()));
        } else {
            panic!("Expected Io error kind");
        }

        assert_eq!(domain_error.context(), Some(&"Disk full".to_string()));
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_file_parse_error() {
        let infra_error = InfrastructureError::FileParseError {
            path: "/test/path.yaml".to_string(),
            format: "YAML".to_string(),
            details: "Invalid syntax".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Serialization { format, details } = domain_error.kind() {
            assert_eq!(format, "YAML");
            assert!(details.contains("Parse error at path '/test/path.yaml'"));
            assert!(details.contains("Invalid syntax"));
        } else {
            panic!("Expected Serialization error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_directory_not_found() {
        let infra_error = InfrastructureError::DirectoryNotFound {
            path: "/test/dir".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Io { operation, path } = domain_error.kind() {
            assert_eq!(operation, "directory access");
            assert_eq!(path, &Some("/test/dir".to_string()));
        } else {
            panic!("Expected Io error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_directory_create_error() {
        let infra_error = InfrastructureError::DirectoryCreateError {
            path: "/test/dir".to_string(),
            details: "Permission denied".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Io { operation, path } = domain_error.kind() {
            assert_eq!(operation, "directory creation");
            assert_eq!(path, &Some("/test/dir".to_string()));
        } else {
            panic!("Expected Io error kind");
        }

        assert_eq!(domain_error.context(), Some(&"Permission denied".to_string()));
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_path_invalid() {
        let infra_error = InfrastructureError::PathInvalid {
            path: "/invalid/path".to_string(),
            reason: "Contains invalid characters".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::ValidationError { field, message } = domain_error.kind() {
            assert_eq!(field, "path");
            assert!(message.contains("Path '/invalid/path' is invalid"));
            assert!(message.contains("Contains invalid characters"));
        } else {
            panic!("Expected ValidationError error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_serialization_error() {
        let infra_error = InfrastructureError::SerializationError {
            format: "JSON".to_string(),
            details: "Invalid data structure".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Serialization { format, details } = domain_error.kind() {
            assert_eq!(format, "JSON");
            assert_eq!(details, "Invalid data structure");
        } else {
            panic!("Expected Serialization error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_deserialization_error() {
        let infra_error = InfrastructureError::DeserializationError {
            format: "XML".to_string(),
            details: "Malformed XML".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::Serialization { format, details } = domain_error.kind() {
            assert_eq!(format, "XML");
            assert_eq!(details, "Malformed XML");
        } else {
            panic!("Expected Serialization error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_network_error() {
        let infra_error = InfrastructureError::NetworkError {
            operation: "HTTP request".to_string(),
            details: "Connection timeout".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::RepositoryError { operation, details } = domain_error.kind() {
            assert_eq!(operation, "HTTP request");
            assert_eq!(details, "Connection timeout");
        } else {
            panic!("Expected RepositoryError error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_database_error() {
        let infra_error = InfrastructureError::DatabaseError {
            operation: "SELECT query".to_string(),
            details: "Connection lost".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::RepositoryError { operation, details } = domain_error.kind() {
            assert_eq!(operation, "SELECT query");
            assert_eq!(details, "Connection lost");
        } else {
            panic!("Expected RepositoryError error kind");
        }
    }

    #[test]
    fn test_from_infrastructure_error_to_domain_error_cache_error() {
        let infra_error = InfrastructureError::CacheError {
            operation: "GET operation".to_string(),
            details: "Cache miss".to_string(),
        };
        let domain_error: DomainError = infra_error.into();

        if let DomainErrorKind::RepositoryError { operation, details } = domain_error.kind() {
            assert_eq!(operation, "GET operation");
            assert_eq!(details, "Cache miss");
        } else {
            panic!("Expected RepositoryError error kind");
        }
    }

    #[test]
    fn test_infrastructure_result_success() {
        let result: InfrastructureResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result, Ok("success".to_string()));
    }

    #[test]
    fn test_infrastructure_result_failure() {
        let result: InfrastructureResult<String> = Err(InfrastructureError::FileNotFound {
            path: "/test/path".to_string(),
        });
        assert!(result.is_err());

        if let Err(InfrastructureError::FileNotFound { path }) = result {
            assert_eq!(path, "/test/path");
        } else {
            panic!("Expected FileNotFound error");
        }
    }
}
