use serde::Serialize;

/// A marker trait for all task states.
pub trait TaskState: Sized + std::fmt::Debug {
    /// Check if the task can be started
    fn can_start(&self) -> bool;
    
    /// Check if the task can be completed
    fn can_complete(&self) -> bool;
    
    /// Check if the task can be blocked
    fn can_block(&self) -> bool;
    
    /// Check if the task can be cancelled
    fn can_cancel(&self) -> bool;
    
    /// Get the display name for this state
    fn display_name(&self) -> &'static str;
}

/// State for a task that has been planned but not yet started.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Planned;
impl TaskState for Planned {
    fn can_start(&self) -> bool { true }
    fn can_complete(&self) -> bool { false }
    fn can_block(&self) -> bool { true }
    fn can_cancel(&self) -> bool { true }
    fn display_name(&self) -> &'static str { "Planned" }
}

/// State for a task that is currently in progress.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InProgress {
    pub progress: u8,
}
impl TaskState for InProgress {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { self.progress >= 100 }
    fn can_block(&self) -> bool { true }
    fn can_cancel(&self) -> bool { true }
    fn display_name(&self) -> &'static str { "In Progress" }
}

/// State for a task that is blocked.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Blocked {
    pub reason: String,
}
impl TaskState for Blocked {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { false }
    fn can_block(&self) -> bool { false }
    fn can_cancel(&self) -> bool { true }
    fn display_name(&self) -> &'static str { "Blocked" }
}

/// State for a task that has been completed.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Completed;
impl TaskState for Completed {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { false }
    fn can_block(&self) -> bool { false }
    fn can_cancel(&self) -> bool { false }
    fn display_name(&self) -> &'static str { "Completed" }
}

/// State for a task that has been cancelled.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Cancelled;
impl TaskState for Cancelled {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { false }
    fn can_block(&self) -> bool { false }
    fn can_cancel(&self) -> bool { false }
    fn display_name(&self) -> &'static str { "Cancelled" }
}

/// Trait for state transitions with validation
pub trait StateTransition {
    type NextState: TaskState;
    
    /// Attempt to transition to the next state
    fn transition_to(self) -> Result<Self::NextState, String>;
    
    /// Get the reason why transition is not allowed
    fn transition_blocked_reason(&self) -> Option<String>;
}

impl StateTransition for Planned {
    type NextState = InProgress;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(InProgress { progress: 0 })
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        None // Planned can always transition to InProgress
    }
}

impl StateTransition for InProgress {
    type NextState = Completed;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        if self.progress >= 100 {
            Ok(Completed)
        } else {
            Err(format!("Cannot complete task with {}% progress", self.progress))
        }
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        if self.progress < 100 {
            Some(format!("Task must be 100% complete (currently {}%)", self.progress))
        } else {
            None
        }
    }
}

impl StateTransition for Blocked {
    type NextState = InProgress;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(InProgress { progress: 0 }) // Reset progress when unblocking
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        None // Blocked can always transition back to InProgress
    }
}

impl StateTransition for Completed {
    type NextState = InProgress;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Err("Completed tasks cannot transition to other states".to_string())
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        Some("Completed tasks cannot transition to other states".to_string())
    }
}

impl StateTransition for Cancelled {
    type NextState = Planned;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Err("Cancelled tasks cannot transition to other states".to_string())
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        Some("Cancelled tasks cannot transition to other states".to_string())
    }
}
