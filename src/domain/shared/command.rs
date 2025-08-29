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
}
