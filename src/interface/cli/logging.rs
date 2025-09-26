use log::{debug, error, info, warn};

/// Logging utilities for the CLI
pub struct Logger;

impl Logger {
    /// Log debug information if verbose mode is enabled
    pub fn debug(message: &str) {
        debug!("{}", message);
    }

    /// Log debug information with format if verbose mode is enabled
    pub fn debug_fmt<F>(f: F)
    where
        F: FnOnce() -> String,
    {
        debug!("{}", f());
    }

    /// Log info information (always shown unless quiet mode)
    pub fn info(message: &str) {
        info!("{}", message);
    }

    /// Log warning information
    pub fn warn(message: &str) {
        warn!("{}", message);
    }

    /// Log error information
    pub fn error(message: &str) {
        error!("{}", message);
    }

    /// Log debug information about command execution context
    pub fn debug_command_context(command: &str, params: &[(&str, &str)]) {
        if Self::is_verbose() {
            debug!("DEBUG: {} called with parameters:", command);
            for (key, value) in params {
                debug!("  {}: {:?}", key, value);
            }
        }
    }

    /// Log debug information about file operations
    pub fn debug_file_operation(operation: &str, path: &str) {
        if Self::is_verbose() {
            debug!("DEBUG: {} - {}", operation, path);
        }
    }

    /// Log debug information about data loading
    pub fn debug_data_loaded(entity_type: &str, identifier: &str, name: &str) {
        if Self::is_verbose() {
            debug!("DEBUG: Loaded {}: {} - {}", entity_type, identifier, name);
        }
    }

    /// Log debug information about context detection
    pub fn debug_context(context: &str) {
        if Self::is_verbose() {
            debug!("DEBUG: Current context: {}", context);
        }
    }

    /// Log debug information about project operations
    pub fn debug_project_operation(operation: &str, project_code: &str, details: &str) {
        if Self::is_verbose() {
            debug!("DEBUG: Project {} - {}: {}", operation, project_code, details);
        }
    }

    /// Log debug information about task operations
    pub fn debug_task_operation(operation: &str, task_code: &str, details: &str) {
        if Self::is_verbose() {
            debug!("DEBUG: Task {} - {}: {}", operation, task_code, details);
        }
    }

    /// Check if verbose mode is enabled
    fn is_verbose() -> bool {
        std::env::var("TTR_VERBOSE").unwrap_or_default() == "1"
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet() -> bool {
        std::env::var("TTR_QUIET").unwrap_or_default() == "1"
    }
}

/// Convenience macros for common debug patterns
#[macro_export]
macro_rules! debug_command {
    ($command:expr, $($param:ident: $value:expr),*) => {
        if $crate::interface::cli::logging::Logger::is_verbose() {
            let params = vec![
                $(stringify!($param), format!("{:?}", $value).as_str()),*
            ];
            $crate::interface::cli::logging::Logger::debug_command_context($command, &params);
        }
    };
}

#[macro_export]
macro_rules! debug_file {
    ($operation:expr, $path:expr) => {
        $crate::interface::cli::logging::Logger::debug_file_operation($operation, $path);
    };
}

#[macro_export]
macro_rules! debug_loaded {
    ($entity_type:expr, $identifier:expr, $name:expr) => {
        $crate::interface::cli::logging::Logger::debug_data_loaded($entity_type, $identifier, $name);
    };
}

#[macro_export]
macro_rules! debug_context {
    ($context:expr) => {
        $crate::interface::cli::logging::Logger::debug_context($context);
    };
}

#[macro_export]
macro_rules! debug_project {
    ($operation:expr, $project_code:expr, $details:expr) => {
        $crate::interface::cli::logging::Logger::debug_project_operation($operation, $project_code, $details);
    };
}

#[macro_export]
macro_rules! debug_task {
    ($operation:expr, $task_code:expr, $details:expr) => {
        $crate::interface::cli::logging::Logger::debug_task_operation($operation, $task_code, $details);
    };
}
