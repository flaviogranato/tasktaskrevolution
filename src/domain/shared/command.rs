use crate::domain::shared::errors::DomainError;
use std::collections::HashMap;

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
            self.retry_count < self.max_retries()
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
        let command = MockCommand::new("test", "Test command").with_can_execute(false);
        let result = command.execute();
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("Command cannot be executed"));
        }
    }

    #[test]
    fn test_mock_command_validation_success() {
        let command = MockCommand::new("test", "Test command");
        assert!(command.validate().is_ok());
    }

    #[test]
    fn test_mock_command_validation_failure() {
        let command = MockCommand::new("test", "Test command")
            .with_validation_result(Err(DomainError::new(
                crate::domain::shared::errors::DomainErrorKind::ValidationError {
                    field: "test".to_string(),
                    message: "Validation failed".to_string(),
                },
            )));
        let result = command.validate();
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("Validation failed"));
        }
    }

    // Tests for UndoableCommand trait
    #[test]
    fn test_undoable_command_can_undo() {
        let command = MockUndoableCommand::new("test", "Test command");
        assert!(command.can_undo());
    }

    #[test]
    fn test_undoable_command_undo_success() {
        let command = MockUndoableCommand::new("test", "Test command");
        let result = command.undo().unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Command undone successfully");
    }

    #[test]
    fn test_undoable_command_undo_failure() {
        let command = MockUndoableCommand::new("test", "Test command").with_can_undo(false);
        let result = command.undo();
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("Command cannot be undone"));
        }
    }

    // Tests for RetryableCommand trait
    #[test]
    fn test_retryable_command_can_retry() {
        let command = MockRetryableCommand::new("test", "Test command");
        assert!(command.can_retry());
        assert_eq!(command.retry_count(), 0);
        assert_eq!(command.max_retries(), 3);
    }

    #[test]
    fn test_retryable_command_retry_count() {
        let command = MockRetryableCommand::new("test", "Test command")
            .with_retry_count(2);
        assert_eq!(command.retry_count(), 2);
        assert!(command.can_retry());
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

    #[test]
    fn test_retryable_command_cannot_retry() {
        let mut command = MockRetryableCommand::new("test", "Test command")
            .with_retry_count(3);
        assert!(!command.can_retry());
        command.increment_retry_count();
        assert!(!command.can_retry());
    }

    // Tests for SchedulableCommand trait
    #[test]
    fn test_schedulable_command_should_execute_now_no_schedule() {
        let command = MockSchedulableCommand::new("test", "Test command");
        assert!(command.should_execute_now());
    }

    #[test]
    fn test_schedulable_command_should_execute_now_past_schedule() {
        let past_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let command = MockSchedulableCommand::new("test", "Test command")
            .with_scheduled_time(past_time);
        assert!(command.should_execute_now());
    }

    #[test]
    fn test_schedulable_command_should_execute_now_future_schedule() {
        let future_time = chrono::Utc::now() + chrono::Duration::hours(1);
        let command = MockSchedulableCommand::new("test", "Test command")
            .with_scheduled_time(future_time);
        assert!(!command.should_execute_now());
    }

    // Tests for ValidatableCommand trait
    #[test]
    fn test_validatable_command_trait() {
        let command = MockValidatableCommand::new("test", "Test command");
        assert!(<MockValidatableCommand as Command>::validate(&command).is_ok());
        let errors = command.validation_errors();
        assert!(errors.is_empty());
    }

    // Tests for AuthorizableCommand trait
    #[test]
    fn test_authorizable_command_trait() {
        let command = MockAuthorizableCommand::new("test", "Test command");
        assert!(command.is_authorized("admin"));
        assert!(!command.is_authorized("user"));
        let permissions = command.required_permissions();
        assert_eq!(permissions.len(), 2);
        assert!(permissions.contains(&"admin".to_string()));
        assert!(permissions.contains(&"write".to_string()));
    }

    // Tests for LoggableCommand trait
    #[test]
    fn test_loggable_command_trait() {
        let command = MockLoggableCommand::new("test", "Test command");
        let log_entry = command.log_entry();
        assert_eq!(log_entry.command_name, "test");
        assert_eq!(log_entry.user, "test_user");
        assert!(log_entry.result.is_none());
    }

    // Tests for CommandBus
    #[test]
    fn test_command_bus_new() {
        let bus = CommandBus::new();
        assert!(bus.handlers.is_empty());
    }

    #[test]
    fn test_command_bus_default() {
        let bus = CommandBus::default();
        assert!(bus.handlers.is_empty());
    }

    #[test]
    fn test_command_bus_execute_no_handler() {
        let bus = CommandBus::new();
        let command = MockCommand::new("test", "Test command");
        let result = bus.execute(&command);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("No handler found for command"));
        }
    }

    // Tests for CommandLogEntry
    #[test]
    fn test_command_log_entry_new() {
        let params = serde_yaml::to_value("test_params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", params.clone());
        assert_eq!(entry.user, "test_user");
        assert_eq!(entry.command_name, "test_command");
        assert_eq!(entry.parameters, params);
        assert!(entry.result.is_none());
    }

    #[test]
    fn test_command_log_entry_with_result() {
        let params = serde_yaml::to_value("test_params").unwrap();
        let result = CommandResult::success("Success");
        let entry = CommandLogEntry::new("test_user", "test_command", params)
            .with_result(result.clone());
        assert_eq!(entry.result, Some(result));
    }

    #[test]
    fn test_command_log_entry_debug_formatting() {
        let params = serde_yaml::to_value("test_params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", params);
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("test_user"));
        assert!(debug_str.contains("test_command"));
    }

    #[test]
    fn test_command_log_entry_clone() {
        let params = serde_yaml::to_value("test_params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", params);
        let cloned = entry.clone();
        assert_eq!(entry.user, cloned.user);
        assert_eq!(entry.command_name, cloned.command_name);
        assert_eq!(entry.parameters, cloned.parameters);
        assert_eq!(entry.result, cloned.result);
    }

    // Additional mock implementations for comprehensive testing
    #[derive(Debug)]
    struct MockSchedulableCommand {
        base: MockCommand,
        scheduled_time: Option<chrono::DateTime<chrono::Utc>>,
    }

    impl MockSchedulableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
                scheduled_time: None,
            }
        }

        fn with_scheduled_time(mut self, time: chrono::DateTime<chrono::Utc>) -> Self {
            self.scheduled_time = Some(time);
            self
        }
    }

    impl Command for MockSchedulableCommand {
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

    impl SchedulableCommand for MockSchedulableCommand {
        fn scheduled_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.scheduled_time
        }
    }

    #[derive(Debug)]
    struct MockValidatableCommand {
        base: MockCommand,
    }

    impl MockValidatableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
            }
        }
    }

    impl Command for MockValidatableCommand {
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

    impl ValidatableCommand for MockValidatableCommand {
        fn validate(&self) -> Result<(), DomainError> {
            self.base.validate()
        }

        fn validation_errors(&self) -> Vec<String> {
            match self.base.validate() {
                Ok(()) => Vec::new(),
                Err(err) => vec![err.to_string()],
            }
        }
    }

    #[derive(Debug)]
    struct MockAuthorizableCommand {
        base: MockCommand,
    }

    impl MockAuthorizableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
            }
        }
    }

    impl Command for MockAuthorizableCommand {
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

    impl AuthorizableCommand for MockAuthorizableCommand {
        fn is_authorized(&self, user: &str) -> bool {
            user == "admin"
        }

        fn required_permissions(&self) -> Vec<String> {
            vec!["admin".to_string(), "write".to_string()]
        }
    }

    #[derive(Debug)]
    struct MockLoggableCommand {
        base: MockCommand,
    }

    impl MockLoggableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockCommand::new(name, description),
            }
        }
    }

    impl Command for MockLoggableCommand {
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

    impl LoggableCommand for MockLoggableCommand {
        fn log_entry(&self) -> CommandLogEntry {
            CommandLogEntry::new("test_user", self.name(), serde_yaml::to_value("{}").unwrap())
        }
    }

    // Tests for RedoableCommand trait
    #[test]
    fn test_redoable_command_redo_success() {
        let command = MockRedoableCommand::new("test", "Test command");
        let result = command.redo().unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Command redone successfully");
    }

    #[test]
    fn test_redoable_command_redo_failure() {
        let command = MockRedoableCommand::new("test", "Test command").with_can_redo(false);
        let result = command.redo();
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("Command cannot be redone"));
        }
    }

    #[derive(Debug)]
    struct MockRedoableCommand {
        base: MockUndoableCommand,
        can_redo: bool,
    }

    impl MockRedoableCommand {
        fn new(name: &str, description: &str) -> Self {
            Self {
                base: MockUndoableCommand::new(name, description),
                can_redo: true,
            }
        }

        fn with_can_redo(mut self, can_redo: bool) -> Self {
            self.can_redo = can_redo;
            self
        }
    }

    impl Command for MockRedoableCommand {
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

    impl UndoableCommand for MockRedoableCommand {
        fn undo(&self) -> Result<CommandResult, DomainError> {
            self.base.undo()
        }

        fn can_undo(&self) -> bool {
            self.base.can_undo()
        }
    }

    impl RedoableCommand for MockRedoableCommand {
        fn redo(&self) -> Result<CommandResult, DomainError> {
            if self.can_redo {
                Ok(CommandResult::success("Command redone successfully"))
            } else {
                Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::Generic {
                        message: "Command cannot be redone".to_string(),
                    },
                ))
            }
        }
    }

    // Tests for CommandBus register_handler (even though it's not fully implemented)
    #[test]
    fn test_command_bus_register_handler() {
        let mut bus = CommandBus::new();
        // This test just ensures the method doesn't panic
        bus.register_handler::<MockCommand, MockCommandHandler>(MockCommandHandler);
    }

    #[derive(Debug)]
    struct MockCommandHandler;

    impl CommandHandler<MockCommand> for MockCommandHandler {
        fn handle(&self, _command: &MockCommand) -> Result<CommandResult, DomainError> {
            Ok(CommandResult::success("Handled by mock handler"))
        }
    }

    // Tests for edge cases and error conditions
    #[test]
    fn test_command_result_with_complex_data() {
        let complex_data = serde_yaml::to_value(vec![1, 2, 3]).unwrap();
        let result = CommandResult::success_with_data("Complex operation", complex_data.clone());
        assert!(result.success);
        assert_eq!(result.data, Some(complex_data));
    }

    #[test]
    fn test_command_log_entry_timestamp() {
        let params = serde_yaml::to_value("test_params").unwrap();
        let entry = CommandLogEntry::new("test_user", "test_command", params);
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(entry.timestamp);
        // Should be within 1 second
        assert!(diff.num_seconds() <= 1);
    }

    #[test]
    fn test_command_bus_handlers_initialization() {
        let bus = CommandBus::new();
        assert_eq!(bus.handlers.len(), 0);
    }

    // Additional tests for better coverage
    #[test]
    fn test_command_with_metadata() {
        #[derive(Debug)]
        struct MetadataCommand {
            name: String,
            description: String,
            metadata: HashMap<String, String>,
        }

        impl Command for MetadataCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                let metadata_yaml = serde_yaml::to_value(&self.metadata).unwrap();
                Ok(CommandResult::success_with_data(
                    "Command executed with metadata",
                    metadata_yaml,
                ))
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());
        metadata.insert("category".to_string(), "test".to_string());

        let command = MetadataCommand {
            name: "metadata_test".to_string(),
            description: "Test command with metadata".to_string(),
            metadata,
        };

        let result = command.execute().unwrap();
        assert!(result.success);
        assert!(result.data.is_some());
        
        if let Some(data) = result.data {
            let metadata_map: HashMap<String, String> = serde_yaml::from_value(data).unwrap();
            assert_eq!(metadata_map.get("priority"), Some(&"high".to_string()));
            assert_eq!(metadata_map.get("category"), Some(&"test".to_string()));
        }
    }

    #[test]
    fn test_command_with_validation_rules() {
        #[derive(Debug)]
        struct ValidatedCommand {
            name: String,
            description: String,
            value: i32,
            max_value: i32,
        }

        impl Command for ValidatedCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                if self.value > self.max_value {
                    Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::ValidationError {
                            field: "value".to_string(),
                            message: format!("Value {} exceeds maximum {}", self.value, self.max_value),
                        },
                    ))
                } else {
                    Ok(CommandResult::success("Value within limits"))
                }
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }

            fn validate(&self) -> Result<(), DomainError> {
                if self.value < 0 {
                    Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::ValidationError {
                            field: "value".to_string(),
                            message: "Value cannot be negative".to_string(),
                        },
                    ))
                } else {
                    Ok(())
                }
            }
        }

        // Test valid command
        let valid_command = ValidatedCommand {
            name: "valid_test".to_string(),
            description: "Valid command test".to_string(),
            value: 50,
            max_value: 100,
        };

        assert!(valid_command.validate().is_ok());
        let result = valid_command.execute().unwrap();
        assert!(result.success);

        // Test invalid command (value too high)
        let invalid_command = ValidatedCommand {
            name: "invalid_test".to_string(),
            description: "Invalid command test".to_string(),
            value: 150,
            max_value: 100,
        };

        let result = invalid_command.execute();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("exceeds maximum"));
        }

        // Test negative value validation
        let negative_command = ValidatedCommand {
            name: "negative_test".to_string(),
            description: "Negative command test".to_string(),
            value: -10,
            max_value: 100,
        };

        let result = negative_command.validate();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("cannot be negative"));
        }
    }

    #[test]
    fn test_command_with_conditional_execution() {
        #[derive(Debug)]
        struct ConditionalCommand {
            name: String,
            description: String,
            should_execute: bool,
            condition_met: bool,
        }

        impl Command for ConditionalCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                if !self.should_execute {
                    return Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::Generic {
                            message: "Command execution disabled".to_string(),
                        },
                    ));
                }

                if !self.condition_met {
                    return Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::Generic {
                            message: "Condition not met".to_string(),
                        },
                    ));
                }

                Ok(CommandResult::success("Conditional command executed"))
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }

            fn can_execute(&self) -> bool {
                self.should_execute && self.condition_met
            }
        }

        // Test successful execution
        let success_command = ConditionalCommand {
            name: "success_test".to_string(),
            description: "Success test".to_string(),
            should_execute: true,
            condition_met: true,
        };

        assert!(success_command.can_execute());
        let result = success_command.execute().unwrap();
        assert!(result.success);

        // Test execution disabled
        let disabled_command = ConditionalCommand {
            name: "disabled_test".to_string(),
            description: "Disabled test".to_string(),
            should_execute: false,
            condition_met: true,
        };

        assert!(!disabled_command.can_execute());
        let result = disabled_command.execute();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("execution disabled"));
        }

        // Test condition not met
        let condition_failed_command = ConditionalCommand {
            name: "condition_failed_test".to_string(),
            description: "Condition failed test".to_string(),
            should_execute: true,
            condition_met: false,
        };

        assert!(!condition_failed_command.can_execute());
        let result = condition_failed_command.execute();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Condition not met"));
        }
    }

    #[test]
    fn test_command_with_retry_logic() {
        #[derive(Debug)]
        struct RetryableCommand {
            name: String,
            description: String,
            max_attempts: u32,
            current_attempt: u32,
            success_on_attempt: u32,
        }

        impl Command for RetryableCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                if self.current_attempt >= self.max_attempts {
                    return Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::Generic {
                            message: "Max attempts exceeded".to_string(),
                        },
                    ));
                }

                if self.current_attempt == self.success_on_attempt {
                    Ok(CommandResult::success("Command succeeded on retry"))
                } else {
                    Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::Generic {
                            message: format!("Attempt {} failed", self.current_attempt),
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
        }

        // Test command that succeeds on first attempt
        let first_attempt_success = RetryableCommand {
            name: "first_success".to_string(),
            description: "First attempt success".to_string(),
            max_attempts: 3,
            current_attempt: 0,
            success_on_attempt: 0,
        };

        let result = first_attempt_success.execute().unwrap();
        assert!(result.success);

        // Test command that succeeds on retry
        let retry_success = RetryableCommand {
            name: "retry_success".to_string(),
            description: "Retry success".to_string(),
            max_attempts: 3,
            current_attempt: 2,
            success_on_attempt: 2,
        };

        let result = retry_success.execute().unwrap();
        assert!(result.success);

        // Test command that fails all attempts
        let all_failed = RetryableCommand {
            name: "all_failed".to_string(),
            description: "All attempts failed".to_string(),
            max_attempts: 3,
            current_attempt: 3,
            success_on_attempt: 5, // Never reached
        };

        let result = all_failed.execute();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Max attempts exceeded"));
        }
    }

    #[test]
    fn test_command_with_async_behavior() {
        #[derive(Debug)]
        struct AsyncCommand {
            name: String,
            description: String,
            delay_ms: u64,
        }

        impl Command for AsyncCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                // Simulate async behavior with a small delay
                std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
                Ok(CommandResult::success("Async command completed"))
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }
        }

        let async_command = AsyncCommand {
            name: "async_test".to_string(),
            description: "Async command test".to_string(),
            delay_ms: 10, // Small delay for testing
        };

        let result = async_command.execute().unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Async command completed");
    }

    #[test]
    fn test_command_with_resource_management() {
        #[derive(Debug)]
        struct ResourceCommand {
            name: String,
            description: String,
            resource_id: String,
            resource_available: bool,
        }

        impl Command for ResourceCommand {
            fn execute(&self) -> Result<CommandResult, DomainError> {
                if !self.resource_available {
                    return Err(DomainError::new(
                        crate::domain::shared::errors::DomainErrorKind::Generic {
                            message: format!("Resource {} not available", self.resource_id),
                        },
                    ));
                }

                Ok(CommandResult::success_with_data(
                    "Resource acquired successfully",
                    serde_yaml::to_value(&self.resource_id).unwrap(),
                ))
            }

            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> &str {
                &self.description
            }
        }

        // Test with available resource
        let available_resource_command = ResourceCommand {
            name: "available_resource".to_string(),
            description: "Available resource test".to_string(),
            resource_id: "res-001".to_string(),
            resource_available: true,
        };

        let result = available_resource_command.execute().unwrap();
        assert!(result.success);
        assert!(result.data.is_some());

        // Test with unavailable resource
        let unavailable_resource_command = ResourceCommand {
            name: "unavailable_resource".to_string(),
            description: "Unavailable resource test".to_string(),
            resource_id: "res-002".to_string(),
            resource_available: false,
        };

        let result = unavailable_resource_command.execute();
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("not available"));
        }
    }
}
