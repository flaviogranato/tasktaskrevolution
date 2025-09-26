#![allow(dead_code)]

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
    fn can_start(&self) -> bool {
        true
    }
    fn can_complete(&self) -> bool {
        false
    }
    fn can_block(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        true
    }
    fn display_name(&self) -> &'static str {
        "Planned"
    }
}

/// State for a task that is currently in progress.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InProgress {
    pub progress: u8,
}
impl TaskState for InProgress {
    fn can_start(&self) -> bool {
        false
    }
    fn can_complete(&self) -> bool {
        self.progress >= 100
    }
    fn can_block(&self) -> bool {
        true
    }
    fn can_cancel(&self) -> bool {
        true
    }
    fn display_name(&self) -> &'static str {
        "In Progress"
    }
}

/// State for a task that is blocked.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Blocked {
    pub reason: String,
}
impl TaskState for Blocked {
    fn can_start(&self) -> bool {
        false
    }
    fn can_complete(&self) -> bool {
        false
    }
    fn can_block(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        true
    }
    fn display_name(&self) -> &'static str {
        "Blocked"
    }
}

/// State for a task that has been completed.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Completed;
impl TaskState for Completed {
    fn can_start(&self) -> bool {
        false
    }
    fn can_complete(&self) -> bool {
        false
    }
    fn can_block(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn display_name(&self) -> &'static str {
        "Completed"
    }
}

/// State for a task that has been cancelled.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Cancelled;
impl TaskState for Cancelled {
    fn can_start(&self) -> bool {
        false
    }
    fn can_complete(&self) -> bool {
        false
    }
    fn can_block(&self) -> bool {
        false
    }
    fn can_cancel(&self) -> bool {
        false
    }
    fn display_name(&self) -> &'static str {
        "Cancelled"
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planned_state() {
        let state = Planned;
        assert!(state.can_start());
        assert!(!state.can_complete());
        assert!(state.can_block());
        assert!(state.can_cancel());
        assert_eq!(state.display_name(), "Planned");
    }

    #[test]
    fn test_in_progress_state() {
        let state = InProgress { progress: 50 };
        assert!(!state.can_start());
        assert!(!state.can_complete());
        assert!(state.can_block());
        assert!(state.can_cancel());
        assert_eq!(state.display_name(), "In Progress");

        let completed_state = InProgress { progress: 100 };
        assert!(completed_state.can_complete());
    }

    #[test]
    fn test_blocked_state() {
        let state = Blocked {
            reason: "Waiting for review".to_string(),
        };
        assert!(!state.can_start());
        assert!(!state.can_complete());
        assert!(!state.can_block());
        assert!(state.can_cancel());
        assert_eq!(state.display_name(), "Blocked");
    }

    #[test]
    fn test_completed_state() {
        let state = Completed;
        assert!(!state.can_start());
        assert!(!state.can_complete());
        assert!(!state.can_block());
        assert!(!state.can_cancel());
        assert_eq!(state.display_name(), "Completed");
    }

    #[test]
    fn test_cancelled_state() {
        let state = Cancelled;
        assert!(!state.can_start());
        assert!(!state.can_complete());
        assert!(!state.can_block());
        assert!(!state.can_cancel());
        assert_eq!(state.display_name(), "Cancelled");
    }

    #[test]
    fn test_planned_to_in_progress_transition() {
        let state = Planned;
        let result = state.transition_to();
        assert!(result.is_ok());
        let in_progress = result.unwrap();
        assert_eq!(in_progress.progress, 0);

        // Test transition_blocked_reason separately
        let state2 = Planned;
        assert!(state2.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_in_progress_to_completed_transition_success() {
        let state = InProgress { progress: 100 };
        let result = state.transition_to();
        assert!(result.is_ok());
        let completed = result.unwrap();
        assert!(matches!(completed, Completed));

        // Test transition_blocked_reason separately
        let state2 = InProgress { progress: 100 };
        assert!(state2.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_in_progress_to_completed_transition_failure() {
        let state = InProgress { progress: 50 };
        let result = state.transition_to();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot complete task with 50% progress"));

        // Test transition_blocked_reason separately
        let state2 = InProgress { progress: 50 };
        assert!(state2.transition_blocked_reason().is_some());
        assert!(
            state2
                .transition_blocked_reason()
                .unwrap()
                .contains("Task must be 100% complete")
        );
    }

    #[test]
    fn test_blocked_to_in_progress_transition() {
        let state = Blocked {
            reason: "Waiting for review".to_string(),
        };
        let result = state.transition_to();
        assert!(result.is_ok());
        let in_progress = result.unwrap();
        assert_eq!(in_progress.progress, 0);

        // Test transition_blocked_reason separately
        let state2 = Blocked {
            reason: "Waiting for review".to_string(),
        };
        assert!(state2.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_completed_to_in_progress_transition() {
        let state = Completed;
        let result = state.transition_to();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Completed tasks cannot transition to other states")
        );

        // Test transition_blocked_reason separately
        let state2 = Completed;
        assert!(state2.transition_blocked_reason().is_some());
        assert!(
            state2
                .transition_blocked_reason()
                .unwrap()
                .contains("Completed tasks cannot transition to other states")
        );
    }

    #[test]
    fn test_cancelled_to_planned_transition() {
        let state = Cancelled;
        let result = state.transition_to();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cancelled tasks cannot transition to other states")
        );

        // Test transition_blocked_reason separately
        let state2 = Cancelled;
        assert!(state2.transition_blocked_reason().is_some());
        assert!(
            state2
                .transition_blocked_reason()
                .unwrap()
                .contains("Cancelled tasks cannot transition to other states")
        );
    }

    #[test]
    fn test_state_equality() {
        assert_eq!(Planned, Planned);
        assert_eq!(InProgress { progress: 50 }, InProgress { progress: 50 });
        assert_ne!(InProgress { progress: 50 }, InProgress { progress: 75 });
        assert_eq!(Completed, Completed);
        assert_eq!(Cancelled, Cancelled);
    }

    #[test]
    fn test_state_clone() {
        let state = InProgress { progress: 75 };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_state_serialization() {
        let state = InProgress { progress: 50 };
        let serialized = serde_yaml::to_string(&state).unwrap();
        assert!(serialized.contains("progress: 50"));
    }
}
