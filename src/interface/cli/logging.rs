use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

/// Logging utilities for the CLI using tracing
pub struct Logger;

impl Logger {
    /// Initialize tracing with appropriate configuration
    pub fn init(verbose: bool, quiet: bool, json_format: bool) -> Result<(), Box<dyn std::error::Error>> {
        // Determine log level based on flags
        let level = if quiet {
            Level::ERROR
        } else if verbose {
            Level::DEBUG
        } else {
            Level::INFO
        };

        // Create environment filter
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(format!("{}", level)));

        // Create formatter
        let fmt_layer = if json_format {
            fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .boxed()
        } else {
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .boxed()
        };

        // Initialize tracing
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init()?;

        Ok(())
    }

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
        let span = tracing::span!(Level::DEBUG, "command_context", command = command);
        let _enter = span.enter();
        
        debug!("Command called with parameters:");
        for (key, value) in params {
            debug!(parameter = key, value = %value, "Parameter");
        }
    }

    /// Log debug information about file operations
    pub fn debug_file_operation(operation: &str, path: &str) {
        let span = tracing::span!(Level::DEBUG, "file_operation", operation = operation, path = path);
        let _enter = span.enter();
        debug!("File operation completed");
    }

    /// Log debug information about data loading
    pub fn debug_data_loaded(entity_type: &str, identifier: &str, name: &str) {
        let span = tracing::span!(
            Level::DEBUG, 
            "data_loaded", 
            entity_type = entity_type, 
            identifier = identifier, 
            name = name
        );
        let _enter = span.enter();
        debug!("Data loaded successfully");
    }

    /// Log debug information about context detection
    pub fn debug_context(context: &str) {
        let span = tracing::span!(Level::DEBUG, "context_detection", context = context);
        let _enter = span.enter();
        debug!("Context detected");
    }

    /// Log debug information about project operations
    pub fn debug_project_operation(operation: &str, project_code: &str, details: &str) {
        let span = tracing::span!(
            Level::DEBUG, 
            "project_operation", 
            operation = operation, 
            project_code = project_code, 
            details = details
        );
        let _enter = span.enter();
        debug!("Project operation completed");
    }

    /// Log debug information about task operations
    pub fn debug_task_operation(operation: &str, task_code: &str, details: &str) {
        let span = tracing::span!(
            Level::DEBUG, 
            "task_operation", 
            operation = operation, 
            task_code = task_code, 
            details = details
        );
        let _enter = span.enter();
        debug!("Task operation completed");
    }

    /// Check if verbose mode is enabled
    #[allow(dead_code)]
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
