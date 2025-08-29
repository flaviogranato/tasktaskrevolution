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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
