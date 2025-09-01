use serde::Serialize;

/// A marker trait for all project states.
pub trait ProjectState: Sized + std::fmt::Debug {
    /// Check if the project can be started
    fn can_start(&self) -> bool;
    
    /// Check if the project can be completed
    fn can_complete(&self) -> bool;
    
    /// Check if the project can be cancelled
    fn can_cancel(&self) -> bool;
    
    /// Get the display name for this state
    fn display_name(&self) -> &'static str;
}

/// State for a project that has been planned but not yet started.
#[derive(Debug, Clone, Serialize)]
pub struct Planned;
impl ProjectState for Planned {
    fn can_start(&self) -> bool { true }
    fn can_complete(&self) -> bool { false }
    fn can_cancel(&self) -> bool { true }
    fn display_name(&self) -> &'static str { "Planned" }
}

/// State for a project that is currently in progress.
#[derive(Debug, Clone, Serialize)]
pub struct InProgress;
impl ProjectState for InProgress {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { true }
    fn can_cancel(&self) -> bool { true }
    fn display_name(&self) -> &'static str { "In Progress" }
}

/// State for a project that has been completed.
#[derive(Debug, Clone, Serialize)]
pub struct Completed;
impl ProjectState for Completed {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { false }
    fn can_cancel(&self) -> bool { false }
    fn display_name(&self) -> &'static str { "Completed" }
}

/// State for a project that has been cancelled.
#[derive(Debug, Clone, Serialize)]
pub struct Cancelled;
impl ProjectState for Cancelled {
    fn can_start(&self) -> bool { false }
    fn can_complete(&self) -> bool { false }
    fn can_cancel(&self) -> bool { false }
    fn display_name(&self) -> &'static str { "Cancelled" }
}

/// Trait for state transitions with validation
pub trait StateTransition {
    type NextState: ProjectState;
    
    /// Attempt to transition to the next state
    fn transition_to(self) -> Result<Self::NextState, String>;
    
    /// Get the reason why transition is not allowed
    fn transition_blocked_reason(&self) -> Option<String>;
}

impl StateTransition for Planned {
    type NextState = InProgress;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(InProgress)
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        None // Planned can always transition to InProgress
    }
}

impl StateTransition for InProgress {
    type NextState = Completed;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(Completed)
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        None // InProgress can always transition to Completed
    }
}

impl StateTransition for Completed {
    type NextState = InProgress;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Err("Completed projects cannot transition to other states".to_string())
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        Some("Completed projects cannot transition to other states".to_string())
    }
}

impl StateTransition for Cancelled {
    type NextState = Planned;
    
    fn transition_to(self) -> Result<Self::NextState, String> {
        Err("Cancelled projects cannot transition to other states".to_string())
    }
    
    fn transition_blocked_reason(&self) -> Option<String> {
        Some("Cancelled projects cannot transition to other states".to_string())
    }
}
