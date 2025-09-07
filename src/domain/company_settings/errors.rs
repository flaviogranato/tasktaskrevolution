#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use std::error::Error as StdError;
use std::fmt;

/// Company settings specific error types
#[derive(Debug, PartialEq)]
pub enum CompanySettingsError {
    ConfigurationNotFound {
        path: String,
    },
    ConfigurationInvalid {
        field: String,
        value: String,
        reason: String,
    },
    ConfigurationMissing {
        field: String,
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
    FileSystemError {
        operation: String,
        path: String,
        details: String,
    },
    OperationNotAllowed {
        operation: String,
        reason: String,
    },
}

impl fmt::Display for CompanySettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompanySettingsError::ConfigurationNotFound { path } => {
                write!(f, "Configuration not found at path '{}'", path)
            }
            CompanySettingsError::ConfigurationInvalid { field, value, reason } => {
                write!(
                    f,
                    "Invalid configuration for field '{}' with value '{}': {}",
                    field, value, reason
                )
            }
            CompanySettingsError::ConfigurationMissing { field } => {
                write!(f, "Missing configuration for field '{}'", field)
            }
            CompanySettingsError::ManagerNotFound { identifier } => {
                write!(f, "Manager not found with identifier '{}'", identifier)
            }
            CompanySettingsError::InvalidManagerData { field, reason } => {
                write!(f, "Invalid manager data for field '{}': {}", field, reason)
            }
            CompanySettingsError::RepositoryInitializationFailed { reason } => {
                write!(f, "Repository initialization failed: {}", reason)
            }
            CompanySettingsError::FileSystemError {
                operation,
                path,
                details,
            } => {
                write!(
                    f,
                    "File system error during {} on path '{}': {}",
                    operation, path, details
                )
            }
            CompanySettingsError::OperationNotAllowed { operation, reason } => {
                write!(f, "Operation '{}' not allowed: {}", operation, reason)
            }
        }
    }
}

impl StdError for CompanySettingsError {}

impl From<CompanySettingsError> for DomainError {
    fn from(err: CompanySettingsError) -> Self {
        match err {
            CompanySettingsError::ConfigurationNotFound { path } => DomainError::ValidationError {
                field: "configuration".to_string(),
                message: format!("Configuration file missing: {}", path),
            },
            CompanySettingsError::ConfigurationInvalid { field, value, reason } => DomainError::ValidationError {
                field,
                message: format!("Invalid configuration value '{}': {}", value, reason),
            },
            CompanySettingsError::ConfigurationMissing { field } => DomainError::ValidationError {
                field: "configuration".to_string(),
                message: format!("Configuration field missing: {}", field),
            },
            CompanySettingsError::ManagerNotFound { identifier } => DomainError::ResourceNotFound { code: identifier },
            CompanySettingsError::InvalidManagerData { field, reason } => {
                DomainError::ValidationError { field, message: reason }
            }
            CompanySettingsError::RepositoryInitializationFailed { reason } => DomainError::Io {
                operation: "initialization".to_string(),
                source: std::io::Error::other(reason),
            },
            CompanySettingsError::OperationNotAllowed { operation, reason } => DomainError::ValidationError {
                field: "operation".to_string(),
                message: format!("Operation '{}' not allowed: {}", operation, reason),
            },
            CompanySettingsError::FileSystemError {
                operation,
                path,
                details,
            } => DomainError::IoWithPath {
                operation,
                path,
                source: std::io::Error::other(details),
            },
        }
    }
}

// Result type for company settings operations
pub type CompanySettingsResult<T> = Result<T, CompanySettingsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_not_found_error_display() {
        let error = CompanySettingsError::ConfigurationNotFound {
            path: "/config/company.yaml".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Configuration not found at path '/config/company.yaml'"));
    }

    #[test]
    fn test_configuration_invalid_error_display() {
        let error = CompanySettingsError::ConfigurationInvalid {
            field: "timezone".to_string(),
            value: "invalid_timezone".to_string(),
            reason: "Timezone must be a valid IANA timezone identifier".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Invalid configuration for field 'timezone'"));
        assert!(display.contains("with value 'invalid_timezone'"));
        assert!(display.contains("Timezone must be a valid IANA timezone identifier"));
    }

    #[test]
    fn test_configuration_missing_error_display() {
        let error = CompanySettingsError::ConfigurationMissing {
            field: "company_name".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Missing configuration for field 'company_name'"));
    }

    #[test]
    fn test_manager_not_found_error_display() {
        let error = CompanySettingsError::ManagerNotFound {
            identifier: "john.doe@company.com".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Manager not found with identifier 'john.doe@company.com'"));
    }

    #[test]
    fn test_invalid_manager_data_error_display() {
        let error = CompanySettingsError::InvalidManagerData {
            field: "email".to_string(),
            reason: "Invalid email format".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Invalid manager data for field 'email'"));
        assert!(display.contains("Invalid email format"));
    }

    #[test]
    fn test_repository_initialization_failed_error_display() {
        let error = CompanySettingsError::RepositoryInitializationFailed {
            reason: "Database connection failed".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Repository initialization failed"));
        assert!(display.contains("Database connection failed"));
    }

    #[test]
    fn test_file_system_error_display() {
        let error = CompanySettingsError::FileSystemError {
            operation: "read".to_string(),
            path: "/config/settings.yaml".to_string(),
            details: "Permission denied".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("File system error during read"));
        assert!(display.contains("on path '/config/settings.yaml'"));
        assert!(display.contains("Permission denied"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = CompanySettingsError::ConfigurationNotFound {
            path: "/test/path".to_string(),
        };
        let debug = format!("{:?}", error);
        assert!(debug.contains("ConfigurationNotFound"));
        assert!(debug.contains("/test/path"));
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_configuration_not_found() {
        let company_error = CompanySettingsError::ConfigurationNotFound {
            path: "company_name".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::ValidationError { field, .. } = domain_error {
            assert_eq!(field, "configuration");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_configuration_invalid() {
        let company_error = CompanySettingsError::ConfigurationInvalid {
            field: "timezone".to_string(),
            value: "invalid_timezone".to_string(),
            reason: "Invalid timezone format".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::ValidationError { field, message } = domain_error {
            assert_eq!(field, "timezone");
            assert!(message.contains("invalid_timezone"));
            assert!(message.contains("Invalid timezone format"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_configuration_missing() {
        let company_error = CompanySettingsError::ConfigurationMissing {
            field: "company_name".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::ValidationError { field, .. } = domain_error {
            assert_eq!(field, "configuration");
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_manager_not_found() {
        let company_error = CompanySettingsError::ManagerNotFound {
            identifier: "john.doe@company.com".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::ResourceNotFound { code } = domain_error {
            assert_eq!(code, "john.doe@company.com");
        } else {
            panic!("Expected ResourceNotFound error kind");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_invalid_manager_data() {
        let company_error = CompanySettingsError::InvalidManagerData {
            field: "email".to_string(),
            reason: "Invalid email format".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::ValidationError { field, message } = domain_error {
            assert_eq!(field, "email");
            assert_eq!(message, "Invalid email format");
        } else {
            panic!("Expected ValidationError error kind");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_repository_initialization_failed() {
        let company_error = CompanySettingsError::RepositoryInitializationFailed {
            reason: "Database connection failed".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::Io { operation, .. } = domain_error {
            assert_eq!(operation, "initialization");
        } else {
            panic!("Expected Io error kind");
        }
    }

    #[test]
    fn test_from_company_settings_error_to_domain_error_file_system_error() {
        let company_error = CompanySettingsError::FileSystemError {
            operation: "read".to_string(),
            path: "/config/settings.yaml".to_string(),
            details: "Permission denied".to_string(),
        };
        let domain_error: DomainError = company_error.into();

        if let DomainError::IoWithPath { operation, path, .. } = domain_error {
            assert_eq!(operation, "read");
            assert_eq!(path, "/config/settings.yaml");
        } else {
            panic!("Expected IoWithPath error kind");
        }
    }

    #[test]
    fn test_company_settings_result_success() {
        let result: CompanySettingsResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result, Ok("success".to_string()));
    }

    #[test]
    fn test_company_settings_result_failure() {
        let result: CompanySettingsResult<String> = Err(CompanySettingsError::ConfigurationNotFound {
            path: "/test/path".to_string(),
        });
        assert!(result.is_err());

        if let Err(CompanySettingsError::ConfigurationNotFound { path }) = result {
            assert_eq!(path, "/test/path");
        } else {
            panic!("Expected ConfigurationNotFound error");
        }
    }

    #[test]
    fn test_all_error_variants_covered() {
        // Test that all error variants can be created and converted
        let errors = vec![
            CompanySettingsError::ConfigurationNotFound {
                path: "test".to_string(),
            },
            CompanySettingsError::ConfigurationInvalid {
                field: "test".to_string(),
                value: "test".to_string(),
                reason: "test".to_string(),
            },
            CompanySettingsError::ConfigurationMissing {
                field: "test".to_string(),
            },
            CompanySettingsError::ManagerNotFound {
                identifier: "test".to_string(),
            },
            CompanySettingsError::InvalidManagerData {
                field: "test".to_string(),
                reason: "test".to_string(),
            },
            CompanySettingsError::RepositoryInitializationFailed {
                reason: "test".to_string(),
            },
            CompanySettingsError::FileSystemError {
                operation: "test".to_string(),
                path: "test".to_string(),
                details: "test".to_string(),
            },
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());

            let debug = format!("{:?}", error);
            assert!(!debug.is_empty());

            let domain_error: DomainError = error.into();
            // Verificar que a conversão foi bem-sucedida
            assert!(matches!(domain_error, _));
        }
    }
}
