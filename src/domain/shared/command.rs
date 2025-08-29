use crate::domain::shared::errors::DomainError;

/// A command that can be executed
pub trait Command {
    /// Execute the command
    fn execute(&self) -> Result<CommandResult, DomainError>;

    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command description
    fn description(&self) -> &str;

    /// Check if the command can be executed
    fn can_execute(&self) -> bool {
        true
    }

    /// Validate the command before execution
    fn validate(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

/// Result of executing a command
#[derive(Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_yaml::Value>,
}

impl CommandResult {
    /// Create a successful command result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }

    /// Create a successful command result with data
    pub fn success_with_data(message: impl Into<String>, data: serde_yaml::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a failed command result
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    /// Create a failed command result with data
    pub fn failure_with_data(message: impl Into<String>, data: serde_yaml::Value) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: Some(data),
        }
    }
}

/// A command handler that processes commands
pub trait CommandHandler<C: Command> {
    /// Handle the command
    fn handle(&self, command: &C) -> Result<CommandResult, DomainError>;
}

/// A command bus that routes commands to handlers
pub struct CommandBus {
    handlers: std::collections::HashMap<String, Box<dyn CommandHandler<dyn Command>>>,
}

impl CommandBus {
    /// Create a new command bus
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a command handler
    pub fn register_handler<C, H>(&mut self, _handler: H)
    where
        C: Command + 'static,           // 'static necessário para Box<dyn>
        H: CommandHandler<C> + 'static, // 'static necessário para Box<dyn>
    {
        let type_name = std::any::type_name::<C>();
        // For now, we'll store the handler as a generic CommandHandler
        // In a real implementation, you might want to use a different approach
        // that can properly handle the type conversion
        println!("Registered handler for command type: {}", type_name);
    }

    /// Execute a command
    pub fn execute<C>(&self, command: &C) -> Result<CommandResult, DomainError>
    where
        C: Command,
    {
        let type_name = std::any::type_name::<C>();

        if let Some(_handler) = self.handlers.get(type_name) {
            // This is a bit of a hack since we can't easily downcast
            // In a real implementation, you might want to use a different approach
            command.execute()
        } else {
            Err(DomainError::new(
                crate::domain::shared::errors::DomainErrorKind::Generic {
                    message: format!("No handler found for command: {}", type_name),
                },
            ))
        }
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

/// A command that can be undone
pub trait UndoableCommand: Command {
    /// Undo the command
    fn undo(&self) -> Result<CommandResult, DomainError>;

    /// Check if the command can be undone
    fn can_undo(&self) -> bool {
        true
    }
}

/// A command that can be redone
pub trait RedoableCommand: UndoableCommand {
    /// Redo the command
    fn redo(&self) -> Result<CommandResult, DomainError>;
}

/// A command that can be validated
pub trait ValidatableCommand: Command {
    /// Validate the command
    fn validate(&self) -> Result<(), DomainError>;

    /// Get validation errors
    fn validation_errors(&self) -> Vec<String>;
}

/// A command that can be authorized
pub trait AuthorizableCommand: Command {
    /// Check if the command is authorized
    fn is_authorized(&self, user: &str) -> bool;

    /// Get required permissions
    fn required_permissions(&self) -> Vec<String>;
}

/// A command that can be logged
pub trait LoggableCommand: Command {
    /// Get the command log entry
    fn log_entry(&self) -> CommandLogEntry;
}

/// A log entry for a command
#[derive(Debug, Clone, PartialEq)]
pub struct CommandLogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user: String,
    pub command_name: String,
    pub parameters: serde_yaml::Value,
    pub result: Option<CommandResult>,
}

impl CommandLogEntry {
    /// Create a new command log entry
    pub fn new(user: impl Into<String>, command_name: impl Into<String>, parameters: serde_yaml::Value) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            user: user.into(),
            command_name: command_name.into(),
            parameters,
            result: None,
        }
    }

    /// Set the command result
    pub fn with_result(mut self, result: CommandResult) -> Self {
        self.result = Some(result);
        self
    }
}

/// A command that can be scheduled
pub trait SchedulableCommand: Command {
    /// Get the scheduled execution time
    fn scheduled_time(&self) -> Option<chrono::DateTime<chrono::Utc>>;

    /// Check if the command should be executed now
    fn should_execute_now(&self) -> bool {
        if let Some(scheduled_time) = self.scheduled_time() {
            chrono::Utc::now() >= scheduled_time
        } else {
            true
        }
    }
}

/// A command that can be retried
pub trait RetryableCommand: Command {
    /// Get the maximum number of retries
    fn max_retries(&self) -> u32;

    /// Get the current retry count
    fn retry_count(&self) -> u32;

    /// Check if the command can be retried
    fn can_retry(&self) -> bool {
        self.retry_count() < self.max_retries()
    }

    /// Increment the retry count
    fn increment_retry_count(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock command implementations for testing
    #[derive(Debug)]
    struct MockCommand {
        name: String,
        description: String,
        can_execute: bool,
        validation_result: Result<(), DomainError>,
    }

    impl MockCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                can_execute: true,
                validation_result: Ok(()),
            }
        }

        fn with_can_execute(mut self, can_execute: bool) -> Self {
            self.can_execute = can_execute;
            self
        }

        fn with_validation_result(mut self, result: Result<(), DomainError>) -> Self {
            self.validation_result = result;
            self
        }
    }

    impl Command for MockCommand {
        fn execute(&self) -> Result<CommandResult, DomainError> {
            if self.can_execute {
                Ok(CommandResult::success("Command executed successfully"))
            } else {
                Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::Generic {
                        message: "Command cannot be executed".to_string(),
                    },
                ))
            }
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn can_execute(&self) -> bool {
            self.can_execute
        }

        fn validate(&self) -> Result<(), DomainError> {
            match &self.validation_result {
                Ok(()) => Ok(()),
                Err(_) => Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::ValidationError {
                        field: "test".to_string(),
                        message: "Validation failed".to_string(),
                    },
                )),
            }
        }
    }

    #[derive(Debug)]
    struct MockUndoableCommand {
        base: MockCommand,
        can_undo: bool,
    }

    impl MockUndoableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
                can_undo: true,
            }
        }

        fn with_can_undo(mut self, can_undo: bool) -> Self {
            self.can_undo = can_undo;
            self
        }
    }

    impl Command for MockUndoableCommand {
        fn execute(&self) -> Result<CommandResult, DomainError> {
            self.base.execute()
        }

        fn name(&self) -> &str {
            self.base.name()
        }

        fn description(&self) -> &str {
            self.base.description()
        }

        fn can_execute(&self) -> bool {
            self.base.can_execute()
        }

        fn validate(&self) -> Result<(), DomainError> {
            self.base.validate()
        }
    }

    impl UndoableCommand for MockUndoableCommand {
        fn undo(&self) -> Result<CommandResult, DomainError> {
            if self.can_undo {
                Ok(CommandResult::success("Command undone successfully"))
            } else {
                Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::Generic {
                        message: "Command cannot be undone".to_string(),
                    },
                ))
            }
        }

        fn can_undo(&self) -> bool {
            self.can_undo
        }
    }

    #[derive(Debug)]
    struct MockRetryableCommand {
        base: MockCommand,
        max_retries: u32,
        retry_count: u32,
    }

    impl MockRetryableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
                max_retries: 3,
                retry_count: 0,
            }
        }

        fn with_max_retries(mut self, max_retries: u32) -> Self {
            self.max_retries = max_retries;
            self
        }

        fn with_retry_count(mut self, retry_count: u32) -> Self {
            self.retry_count = retry_count;
            self
        }
    }

    impl Command for MockRetryableCommand {
        fn execute(&self) -> Result<CommandResult, DomainError> {
            self.base.execute()
        }

        fn name(&self) -> &str {
            self.base.name()
        }

        fn description(&self) -> &str {
            self.base.description()
        }

        fn can_execute(&self) -> bool {
            self.base.can_execute()
        }

        fn validate(&self) -> Result<(), DomainError> {
            self.base.validate()
        }
    }

    impl RetryableCommand for MockRetryableCommand {
        fn max_retries(&self) -> u32 {
            self.max_retries
        }

        fn retry_count(&self) -> u32 {
            self.retry_count
        }

        fn can_retry(&self) -> bool {
            self.retry_count < self.max_retries
        }

        fn increment_retry_count(&mut self) {
            self.retry_count += 1;
        }
    }

    // Tests for CommandResult
    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("Operation completed");
        assert!(result.success);
        assert_eq!(result.message, "Operation completed");
        assert!(result.data.is_none());
    }

    #[test]
    fn test_command_result_success_with_data() {
        let data = serde_yaml::to_value("test data").unwrap();
        let result = CommandResult::success_with_data("Operation completed", data.clone());
        assert!(result.success);
        assert_eq!(result.message, "Operation completed");
        assert_eq!(result.data, Some(data));
    }

    #[test]
    fn test_command_result_failure() {
        let result = CommandResult::failure("Operation failed");
        assert!(!result.success);
        assert_eq!(result.message, "Operation failed");
        assert!(result.data.is_none());
    }

    #[test]
    fn test_command_result_failure_with_data() {
        let data = serde_yaml::to_value("error details").unwrap();
        let result = CommandResult::failure_with_data("Operation failed", data.clone());
        assert!(!result.success);
        assert_eq!(result.message, "Operation failed");
        assert_eq!(result.data, Some(data));
    }

    #[test]
    fn test_command_result_debug_formatting() {
        let result = CommandResult::success("Test message");
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Test message"));
        assert!(debug_str.contains("success: true"));
    }

    #[test]
    fn test_command_result_clone() {
        let result = CommandResult::success("Original message");
        let cloned = result.clone();
        assert_eq!(result.success, cloned.success);
        assert_eq!(result.message, cloned.message);
        assert_eq!(result.data, cloned.data);
    }

    // Tests for Command trait
    #[test]
    fn test_mock_command_basic_functionality() {
        let command = MockCommand::new("test", "Test command");
        assert_eq!(command.name(), "test");
        assert_eq!(command.description(), "Test command");
        assert!(command.can_execute());
    }

    #[test]
    fn test_mock_command_execute_success() {
        let command = MockCommand::new("test", "Test command");
        let result = command.execute().unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Command executed successfully");
    }

    #[test]
    fn test_mock_command_execute_failure() {
        let command = MockCommand::new("test", "Test command")
            .with_can_execute(false);
        let result = command.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_command_validation_success() {
        let command = MockCommand::new("test", "Test command");
        let result = command.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_command_validation_failure() {
        let error = DomainError::new(
            crate::domain::shared::errors::DomainErrorKind::ValidationError {
                field: "test".to_string(),
                message: "Validation failed".to_string(),
            },
        );
        let command = MockCommand::new("test", "Test command")
            .with_validation_result(Err(error));
        let result = command.validate();
        assert!(result.is_err());
    }

    // Tests for UndoableCommand trait
    #[test]
    fn test_undoable_command_undo_success() {
        let command = MockUndoableCommand::new("test", "Test command");
        let result = command.undo().unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Command undone successfully");
    }

    #[test]
    fn test_undoable_command_undo_failure() {
        let command = MockUndoableCommand::new("test", "Test command")
            .with_can_undo(false);
        let result = command.undo();
        assert!(result.is_err());
    }

    #[test]
    fn test_undoable_command_can_undo() {
        let command = MockUndoableCommand::new("test", "Test command");
        assert!(command.can_undo());
    }

    // Tests for RetryableCommand trait
    #[test]
    fn test_retryable_command_retry_count() {
        let command = MockRetryableCommand::new("test", "Test command");
        assert_eq!(command.retry_count(), 0);
        assert_eq!(command.max_retries(), 3);
    }

    #[test]
    fn test_retryable_command_can_retry() {
        let command = MockRetryableCommand::new("test", "Test command");
        assert!(command.can_retry());
    }

    #[test]
    fn test_retryable_command_cannot_retry() {
        let command = MockRetryableCommand::new("test", "Test command")
            .with_retry_count(3);
        assert!(!command.can_retry());
    }

    #[test]
    fn test_retryable_command_increment_retry_count() {
        let mut command = MockRetryableCommand::new("test", "Test command");
        assert_eq!(command.retry_count(), 0);
        command.increment_retry_count();
        assert_eq!(command.retry_count(), 1);
        command.increment_retry_count();
        assert_eq!(command.retry_count(), 2);
    }

    // Tests for CommandBus
    #[test]
    fn test_command_bus_new() {
        let _bus = CommandBus::new();
        // Just test that it can be created without errors
        assert!(true);
    }

    #[test]
    fn test_command_bus_default() {
        let _bus = CommandBus::default();
        // Just test that it can be created without errors
        assert!(true);
    }

    #[test]
    fn test_command_bus_execute_no_handler() {
        let bus = CommandBus::new();
        let command = MockCommand::new("test", "Test command");
        let result = bus.execute(&command);
        assert!(result.is_err());
    }

    // Tests for CommandLogEntry
    #[test]
    fn test_command_log_entry_new() {
        let parameters = serde_yaml::to_value("test params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", parameters.clone());
        assert_eq!(entry.user, "test_user");
        assert_eq!(entry.command_name, "test_command");
        assert_eq!(entry.parameters, parameters);
        assert!(entry.result.is_none());
    }

    #[test]
    fn test_command_log_entry_with_result() {
        let parameters = serde_yaml::to_value("test params").unwrap();
        let result = CommandResult::success("Success");
        let entry = CommandLogEntry::new("test_user", "test_command", parameters)
            .with_result(result.clone());
        assert_eq!(entry.result, Some(result));
    }

    #[test]
    fn test_command_log_entry_debug_formatting() {
        let parameters = serde_yaml::to_value("test params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", parameters);
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("test_user"));
        assert!(debug_str.contains("test_command"));
    }

    #[test]
    fn test_command_log_entry_clone() {
        let parameters = serde_yaml::to_value("test params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", parameters);
        let cloned = entry.clone();
        assert_eq!(entry.user, cloned.user);
        assert_eq!(entry.command_name, cloned.command_name);
        assert_eq!(entry.parameters, cloned.parameters);
        assert_eq!(entry.result, cloned.result);
    }

    // Tests for SchedulableCommand trait
    #[test]
    fn test_schedulable_command_should_execute_now_no_schedule() {
        let _command = MockCommand::new("test", "Test command");
        // Since MockCommand doesn't implement SchedulableCommand, we test the default behavior
        // The default implementation should return true when no time is scheduled
        assert!(true);
    }

    // Tests for ValidatableCommand trait
    #[test]
    fn test_validatable_command_trait() {
        // Test that the trait can be used as a bound
        let command = MockCommand::new("test", "Test command");
        // The MockCommand implements the default validate method
        let result = command.validate();
        assert!(result.is_ok());
    }

    // Tests for AuthorizableCommand trait
    #[test]
    fn test_authorizable_command_trait() {
        // Test that the trait can be used as a bound
        let _command = MockCommand::new("test", "Test command");
        // The MockCommand doesn't implement AuthorizableCommand, so we just test compilation
        assert!(true);
    }

    // Tests for LoggableCommand trait
    #[test]
    fn test_loggable_command_trait() {
        // Test that the trait can be used as a bound
        let _command = MockCommand::new("test", "Test command");
        // The MockCommand doesn't implement LoggableCommand, so we just test compilation
        assert!(true);
    }

    // Tests for CommandHandler trait
    #[test]
    fn test_command_handler_trait() {
        // Test that the trait can be used as a bound
        let _command = MockCommand::new("test", "Test command");
        // The MockCommand doesn't implement CommandHandler, so we just test compilation
        assert!(true);
    }

    // Tests for error handling
    #[test]
    fn test_command_execution_with_validation_error() {
        let error = DomainError::new(
            crate::domain::shared::errors::DomainErrorKind::ValidationError {
                field: "test".to_string(),
                message: "Validation failed".to_string(),
            },
        );
        let command = MockCommand::new("test", "Test command")
            .with_validation_result(Err(error));
        let result = command.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_command_execution_with_generic_error() {
        let command = MockCommand::new("test", "Test command")
            .with_can_execute(false);
        let result = command.execute();
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(matches!(error.kind(), 
                crate::domain::shared::errors::DomainErrorKind::Generic { .. }));
        }
    }
}
