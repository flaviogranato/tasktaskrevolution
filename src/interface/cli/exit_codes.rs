//! Standardized exit codes for CLI operations
//!
//! This module defines standard exit codes following Unix conventions
//! and provides utilities for consistent error handling across the CLI.

use std::process;

/// Standard exit codes following Unix conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,
    /// General error (1)
    GeneralError = 1,
    /// Misuse of shell builtins (2)
    Misuse = 2,
    /// Cannot execute (126)
    CannotExecute = 126,
    /// Command not found (127)
    CommandNotFound = 127,
    /// Invalid exit argument (128)
    InvalidExit = 128,
    /// User-defined error codes (129-255)
    UserDefined(u8),
}

impl ExitCode {
    /// Create a user-defined exit code (129-255)
    pub fn user_defined(code: u8) -> Self {
        Self::UserDefined(code)
    }

    /// Get the numeric value of the exit code
    pub fn value(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::GeneralError => 1,
            Self::Misuse => 2,
            Self::CannotExecute => 126,
            Self::CommandNotFound => 127,
            Self::InvalidExit => 128,
            Self::UserDefined(code) => 128 + code as i32,
        }
    }

    /// Exit the process with this code
    pub fn exit(self) -> ! {
        process::exit(self.value());
    }
}

/// CLI-specific error categories with standardized exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliError {
    /// Validation error (1)
    Validation = 1,
    /// File system error (2)
    FileSystem = 2,
    /// Network error (3)
    Network = 3,
    /// Configuration error (4)
    Configuration = 4,
    /// Permission error (5)
    Permission = 5,
    /// Resource not found (6)
    NotFound = 6,
    /// Resource conflict (7)
    Conflict = 7,
    /// Invalid argument (8)
    InvalidArgument = 8,
    /// Operation not supported (9)
    NotSupported = 9,
    /// Internal error (10)
    Internal = 10,
}

impl CliError {
    /// Get the exit code for this error
    pub fn exit_code(self) -> ExitCode {
        match self {
            Self::Validation => ExitCode::GeneralError,
            Self::FileSystem => ExitCode::Misuse,
            Self::Network => ExitCode::user_defined(3),
            Self::Configuration => ExitCode::user_defined(4),
            Self::Permission => ExitCode::user_defined(5),
            Self::NotFound => ExitCode::user_defined(6),
            Self::Conflict => ExitCode::user_defined(7),
            Self::InvalidArgument => ExitCode::user_defined(8),
            Self::NotSupported => ExitCode::user_defined(9),
            Self::Internal => ExitCode::user_defined(10),
        }
    }

    /// Exit the process with this error code
    pub fn exit(self) -> ! {
        self.exit_code().exit();
    }
}

/// Result type for CLI operations with standardized error handling
pub type CliResult<T> = Result<T, CliError>;

/// Trait for operations that can provide standardized exit codes
pub trait ExitCodeProvider {
    /// Get the exit code for this operation result
    fn exit_code(&self) -> ExitCode;
}

/// Utility functions for common error scenarios
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle validation errors
    pub fn validation_error(message: &str) -> ! {
        eprintln!("Validation error: {}", message);
        CliError::Validation.exit();
    }

    /// Handle file system errors
    pub fn file_system_error(operation: &str, path: &str, error: &str) -> ! {
        eprintln!("File system error during {} on '{}': {}", operation, path, error);
        CliError::FileSystem.exit();
    }

    /// Handle configuration errors
    pub fn configuration_error(message: &str) -> ! {
        eprintln!("Configuration error: {}", message);
        CliError::Configuration.exit();
    }

    /// Handle permission errors
    pub fn permission_error(resource: &str) -> ! {
        eprintln!("Permission denied: {}", resource);
        CliError::Permission.exit();
    }

    /// Handle resource not found errors
    pub fn not_found_error(resource_type: &str, identifier: &str) -> ! {
        eprintln!("{} not found: {}", resource_type, identifier);
        CliError::NotFound.exit();
    }

    /// Handle resource conflict errors
    pub fn conflict_error(resource: &str, conflict: &str) -> ! {
        eprintln!("Resource conflict for {}: {}", resource, conflict);
        CliError::Conflict.exit();
    }

    /// Handle invalid argument errors
    pub fn invalid_argument_error(argument: &str, reason: &str) -> ! {
        eprintln!("Invalid argument '{}': {}", argument, reason);
        CliError::InvalidArgument.exit();
    }

    /// Handle operation not supported errors
    pub fn not_supported_error(operation: &str) -> ! {
        eprintln!("Operation not supported: {}", operation);
        CliError::NotSupported.exit();
    }

    /// Handle internal errors
    pub fn internal_error(component: &str, error: &str) -> ! {
        eprintln!("Internal error in {}: {}", component, error);
        CliError::Internal.exit();
    }
}

/// Macro for handling errors with standardized exit codes
#[macro_export]
macro_rules! handle_cli_error {
    ($result:expr, $error_type:ident) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                $crate::interface::cli::exit_codes::ErrorHandler::$error_type(&format!("{}", e));
            }
        }
    };
    ($result:expr, $error_type:ident, $($arg:expr),*) => {
        match $result {
            Ok(value) => value,
            Err(e) => {
                $crate::interface::cli::exit_codes::ErrorHandler::$error_type(&format!("{}", e), $($arg),*);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.value(), 0);
        assert_eq!(ExitCode::GeneralError.value(), 1);
        assert_eq!(ExitCode::Misuse.value(), 2);
        assert_eq!(ExitCode::UserDefined(1).value(), 129);
        assert_eq!(ExitCode::UserDefined(10).value(), 138);
    }

    #[test]
    fn test_cli_error_exit_codes() {
        assert_eq!(CliError::Validation.exit_code().value(), 1);
        assert_eq!(CliError::FileSystem.exit_code().value(), 2);
        assert_eq!(CliError::Network.exit_code().value(), 131); // 3 + 128
        assert_eq!(CliError::Configuration.exit_code().value(), 132); // 4 + 128
        assert_eq!(CliError::Permission.exit_code().value(), 133); // 5 + 128
        assert_eq!(CliError::NotFound.exit_code().value(), 134); // 6 + 128
        assert_eq!(CliError::Conflict.exit_code().value(), 135); // 7 + 128
        assert_eq!(CliError::InvalidArgument.exit_code().value(), 136); // 8 + 128
        assert_eq!(CliError::NotSupported.exit_code().value(), 137); // 9 + 128
        assert_eq!(CliError::Internal.exit_code().value(), 138); // 10 + 128
    }

    #[test]
    fn test_user_defined_exit_codes() {
        let code = ExitCode::user_defined(5);
        assert_eq!(code.value(), 133); // 5 + 128
    }
}
