#![allow(dead_code)]

use super::resource::ProjectAssignment;
use serde::{Deserialize, Serialize};

/// A marker trait for all resource states.
pub trait ResourceState: Sized + std::fmt::Debug {
    /// Check if the resource can be assigned to a project
    fn can_assign(&self) -> bool;

    /// Check if the resource can be deactivated
    fn can_deactivate(&self) -> bool;

    /// Check if the resource can be reactivated
    fn can_reactivate(&self) -> bool;

    /// Get the display name for this state
    fn display_name(&self) -> &'static str;

    /// Get the number of project assignments
    fn assignment_count(&self) -> usize;
}

/// State for a resource that is available for project assignments.
#[derive(Debug, Clone, Serialize)]
pub struct Available;
impl ResourceState for Available {
    fn can_assign(&self) -> bool {
        true
    }
    fn can_deactivate(&self) -> bool {
        true
    }
    fn can_reactivate(&self) -> bool {
        false
    }
    fn display_name(&self) -> &'static str {
        "Available"
    }
    fn assignment_count(&self) -> usize {
        0
    }
}

/// State for a resource that is currently assigned to one or more projects.
#[derive(Debug, Clone, Serialize)]
pub struct Assigned {
    pub project_assignments: Vec<ProjectAssignment>,
}
impl ResourceState for Assigned {
    fn can_assign(&self) -> bool {
        true
    }
    fn can_deactivate(&self) -> bool {
        true
    }
    fn can_reactivate(&self) -> bool {
        false
    }
    fn display_name(&self) -> &'static str {
        "Assigned"
    }
    fn assignment_count(&self) -> usize {
        self.project_assignments.len()
    }
}

/// State for a resource that is no longer active in the system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inactive;
impl ResourceState for Inactive {
    fn can_assign(&self) -> bool {
        false
    }
    fn can_deactivate(&self) -> bool {
        false
    }
    fn can_reactivate(&self) -> bool {
        true
    }
    fn display_name(&self) -> &'static str {
        "Inactive"
    }
    fn assignment_count(&self) -> usize {
        0
    }
}

/// Trait for state transitions with validation
pub trait StateTransition {
    type NextState: ResourceState;

    /// Attempt to transition to the next state
    fn transition_to(self) -> Result<Self::NextState, String>;

    /// Get the reason why transition is not allowed
    fn transition_blocked_reason(&self) -> Option<String>;
}

impl StateTransition for Available {
    type NextState = Assigned;

    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(Assigned {
            project_assignments: Vec::new(),
        })
    }

    fn transition_blocked_reason(&self) -> Option<String> {
        None // Available can always transition to Assigned
    }
}

impl StateTransition for Assigned {
    type NextState = Inactive;

    fn transition_to(self) -> Result<Self::NextState, String> {
        if self.project_assignments.is_empty() {
            Ok(Inactive)
        } else {
            Err("Cannot deactivate resource with active project assignments".to_string())
        }
    }

    fn transition_blocked_reason(&self) -> Option<String> {
        if !self.project_assignments.is_empty() {
            Some("Resource has active project assignments".to_string())
        } else {
            None
        }
    }
}

impl StateTransition for Inactive {
    type NextState = Available;

    fn transition_to(self) -> Result<Self::NextState, String> {
        Ok(Available)
    }

    fn transition_blocked_reason(&self) -> Option<String> {
        None // Inactive can always transition back to Available
    }
}

/// Extension methods for resource states
pub trait ResourceStateExt: ResourceState {
    /// Check if the resource is overloaded (too many assignments)
    fn is_overloaded(&self, max_assignments: usize) -> bool {
        self.assignment_count() > max_assignments
    }

    /// Check if the resource is underutilized
    fn is_underutilized(&self, min_assignments: usize) -> bool {
        self.assignment_count() < min_assignments
    }

    /// Get the utilization percentage
    fn utilization_percentage(&self, max_assignments: usize) -> f64 {
        if max_assignments == 0 {
            0.0
        } else {
            (self.assignment_count() as f64 / max_assignments as f64) * 100.0
        }
    }
}

impl<T: ResourceState> ResourceStateExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn create_test_project_assignment() -> ProjectAssignment {
        ProjectAssignment {
            project_id: "PROJ-001".to_string(),
            start_date: Local::now(),
            end_date: Local::now(),
            allocation_percentage: 100,
        }
    }

    #[test]
    fn test_available_state() {
        let state = Available;
        assert!(state.can_assign());
        assert!(state.can_deactivate());
        assert!(!state.can_reactivate());
        assert_eq!(state.display_name(), "Available");
        assert_eq!(state.assignment_count(), 0);
    }

    #[test]
    fn test_assigned_state() {
        let assignments = vec![create_test_project_assignment(), create_test_project_assignment()];
        let state = Assigned {
            project_assignments: assignments,
        };
        assert!(state.can_assign());
        assert!(state.can_deactivate());
        assert!(!state.can_reactivate());
        assert_eq!(state.display_name(), "Assigned");
        assert_eq!(state.assignment_count(), 2);
    }

    #[test]
    fn test_inactive_state() {
        let state = Inactive;
        assert!(!state.can_assign());
        assert!(!state.can_deactivate());
        assert!(state.can_reactivate());
        assert_eq!(state.display_name(), "Inactive");
        assert_eq!(state.assignment_count(), 0);
    }

    #[test]
    fn test_available_to_assigned_transition() {
        let state = Available;
        let result = state.transition_to();
        assert!(result.is_ok());
        let assigned = result.unwrap();
        assert_eq!(assigned.project_assignments.len(), 0);
        assert!(assigned.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_assigned_to_inactive_transition_success() {
        let state = Assigned {
            project_assignments: Vec::new(),
        };
        let result = state.transition_to();
        assert!(result.is_ok());
        let inactive = result.unwrap();
        assert!(matches!(inactive, Inactive));
        // Check transition_blocked_reason before calling transition_to
        let state2 = Assigned {
            project_assignments: Vec::new(),
        };
        assert!(state2.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_assigned_to_inactive_transition_failure() {
        let state = Assigned {
            project_assignments: vec![create_test_project_assignment()],
        };
        let result = state.transition_to();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Cannot deactivate resource with active project assignments")
        );
        // Check transition_blocked_reason before calling transition_to
        let state2 = Assigned {
            project_assignments: vec![create_test_project_assignment()],
        };
        assert!(state2.transition_blocked_reason().is_some());
        assert!(
            state2
                .transition_blocked_reason()
                .unwrap()
                .contains("Resource has active project assignments")
        );
    }

    #[test]
    fn test_inactive_to_available_transition() {
        let state = Inactive;
        let result = state.transition_to();
        assert!(result.is_ok());
        let available = result.unwrap();
        assert!(matches!(available, Available));
        // Check transition_blocked_reason before calling transition_to
        let state2 = Inactive;
        assert!(state2.transition_blocked_reason().is_none());
    }

    #[test]
    fn test_resource_state_ext_is_overloaded() {
        let state = Assigned {
            project_assignments: vec![
                create_test_project_assignment(),
                create_test_project_assignment(),
                create_test_project_assignment(),
            ],
        };
        assert!(state.is_overloaded(2));
        assert!(!state.is_overloaded(5));
    }

    #[test]
    fn test_resource_state_ext_is_underutilized() {
        let state = Assigned {
            project_assignments: vec![create_test_project_assignment()],
        };
        assert!(state.is_underutilized(3));
        assert!(!state.is_underutilized(1));
    }

    #[test]
    fn test_resource_state_ext_utilization_percentage() {
        let state = Assigned {
            project_assignments: vec![create_test_project_assignment(), create_test_project_assignment()],
        };
        assert_eq!(state.utilization_percentage(4), 50.0);
        assert_eq!(state.utilization_percentage(2), 100.0);
        assert_eq!(state.utilization_percentage(0), 0.0);
    }

    #[test]
    fn test_available_state_ext_methods() {
        let state = Available;
        assert!(!state.is_overloaded(1));
        assert!(state.is_underutilized(1));
        assert_eq!(state.utilization_percentage(1), 0.0);
    }

    #[test]
    fn test_inactive_state_ext_methods() {
        let state = Inactive;
        assert!(!state.is_overloaded(1));
        assert!(state.is_underutilized(1));
        assert_eq!(state.utilization_percentage(1), 0.0);
    }
}
