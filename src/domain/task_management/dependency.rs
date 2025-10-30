use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid7::uuid7;

use crate::domain::shared::errors::{DomainError, DomainResult};

/// Types of task dependencies following PMI standards
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyType {
    /// Finish-to-Start: Task B cannot start until Task A finishes
    FinishToStart,
    /// Start-to-Start: Task B cannot start until Task A starts
    StartToStart,
    /// Finish-to-Finish: Task B cannot finish until Task A finishes
    FinishToFinish,
    /// Start-to-Finish: Task B cannot finish until Task A starts
    StartToFinish,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::FinishToStart => write!(f, "FS"),
            DependencyType::StartToStart => write!(f, "SS"),
            DependencyType::FinishToFinish => write!(f, "FF"),
            DependencyType::StartToFinish => write!(f, "SF"),
        }
    }
}

impl DependencyType {
    /// Get the dependency type from string representation
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> DomainResult<Self> {
        match s.to_uppercase().as_str() {
            "FS" | "FINISHTOSTART" => Ok(DependencyType::FinishToStart),
            "SS" | "STARTTOSTART" => Ok(DependencyType::StartToStart),
            "FF" | "FINISHTOFINISH" => Ok(DependencyType::FinishToFinish),
            "SF" | "STARTTOFINISH" => Ok(DependencyType::StartToFinish),
            _ => Err(DomainError::validation_error(
                "dependency_type",
                &format!("Invalid dependency type: {}", s),
            )),
        }
    }

    /// Get description of the dependency type
    pub fn description(&self) -> &'static str {
        match self {
            DependencyType::FinishToStart => "Task B cannot start until Task A finishes",
            DependencyType::StartToStart => "Task B cannot start until Task A starts",
            DependencyType::FinishToFinish => "Task B cannot finish until Task A finishes",
            DependencyType::StartToFinish => "Task B cannot finish until Task A starts",
        }
    }
}

/// Status of a task dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DependencyStatus {
    /// Dependency is active and being enforced
    Active,
    /// Dependency is inactive (temporarily disabled)
    Inactive,
    /// Dependency is blocked by another constraint
    Blocked,
}

impl fmt::Display for DependencyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyStatus::Active => write!(f, "Active"),
            DependencyStatus::Inactive => write!(f, "Inactive"),
            DependencyStatus::Blocked => write!(f, "Blocked"),
        }
    }
}

/// Represents a dependency relationship between two tasks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDependency {
    /// Unique identifier for the dependency
    pub id: String,
    /// Code of the predecessor task (Task A)
    pub predecessor: String,
    /// Code of the successor task (Task B)
    pub successor: String,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Lag time - additional delay after the dependency condition is met
    pub lag_time: Option<Duration>,
    /// Lead time - how much earlier the successor can start (negative lag)
    pub lead_time: Option<Duration>,
    /// Current status of the dependency
    pub status: DependencyStatus,
    /// Description of the dependency
    pub description: Option<String>,
    /// When the dependency was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the dependency was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Who created the dependency
    pub created_by: String,
}

impl TaskDependency {
    /// Create a new task dependency
    pub fn new(
        predecessor: String,
        successor: String,
        dependency_type: DependencyType,
        created_by: String,
    ) -> DomainResult<Self> {
        if predecessor.is_empty() {
            return Err(DomainError::validation_error(
                "predecessor",
                "Predecessor task code cannot be empty",
            ));
        }

        if successor.is_empty() {
            return Err(DomainError::validation_error(
                "successor",
                "Successor task code cannot be empty",
            ));
        }

        if predecessor == successor {
            return Err(DomainError::validation_error(
                "dependency",
                "A task cannot depend on itself",
            ));
        }

        let now = chrono::Utc::now();

        Ok(Self {
            id: uuid7().to_string(),
            predecessor,
            successor,
            dependency_type,
            lag_time: None,
            lead_time: None,
            status: DependencyStatus::Active,
            description: None,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }

    /// Set lag time for the dependency
    pub fn with_lag_time(mut self, lag_time: Duration) -> Self {
        self.lag_time = Some(lag_time);
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Set lead time for the dependency
    pub fn with_lead_time(mut self, lead_time: Duration) -> Self {
        self.lead_time = Some(lead_time);
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Set description for the dependency
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Change the status of the dependency
    pub fn change_status(&mut self, new_status: DependencyStatus) {
        self.status = new_status;
        self.updated_at = chrono::Utc::now();
    }

    /// Get the effective time offset (lag - lead)
    pub fn effective_offset(&self) -> Duration {
        let lag = self.lag_time.unwrap_or(Duration::zero());
        let lead = self.lead_time.unwrap_or(Duration::zero());
        lag - lead
    }

    /// Check if this dependency creates a direct cycle with another dependency
    pub fn would_create_cycle(&self, other: &TaskDependency) -> bool {
        // A cycle exists if: A->B and B->A
        self.predecessor == other.successor && self.successor == other.predecessor
    }

    /// Get a human-readable description of the dependency
    pub fn human_description(&self) -> String {
        let offset = self.effective_offset();
        let offset_str = if offset != Duration::zero() {
            if offset > Duration::zero() {
                format!(" with {} lag", self.format_duration(offset))
            } else {
                format!(" with {} lead", self.format_duration(-offset))
            }
        } else {
            String::new()
        };

        format!(
            "{} {} {} ({}){}",
            self.predecessor,
            self.dependency_type,
            self.successor,
            self.dependency_type.description(),
            offset_str
        )
    }

    /// Format duration in a human-readable way
    fn format_duration(&self, duration: Duration) -> String {
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
}

/// Error types for dependency operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyError {
    /// A cycle was detected in the dependency graph
    CycleDetected { path: Vec<String> },
    /// A task was not found
    TaskNotFound { code: String },
    /// Invalid dependency configuration
    InvalidConfiguration { message: String },
    /// Dependency already exists
    DependencyExists { predecessor: String, successor: String },
    /// Dependency not found
    DependencyNotFound { id: String },
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyError::CycleDetected { path } => {
                write!(f, "Cycle detected in dependency path: {}", path.join(" -> "))
            }
            DependencyError::TaskNotFound { code } => {
                write!(f, "Task not found: {}", code)
            }
            DependencyError::InvalidConfiguration { message } => {
                write!(f, "Invalid dependency configuration: {}", message)
            }
            DependencyError::DependencyExists { predecessor, successor } => {
                write!(f, "Dependency already exists: {} -> {}", predecessor, successor)
            }
            DependencyError::DependencyNotFound { id } => {
                write!(f, "Dependency not found: {}", id)
            }
        }
    }
}

impl std::error::Error for DependencyError {}

impl From<DependencyError> for DomainError {
    fn from(error: DependencyError) -> Self {
        DomainError::validation_error("dependency", &error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_dependency_type_from_str() {
        assert_eq!(DependencyType::from_str("FS").unwrap(), DependencyType::FinishToStart);
        assert_eq!(DependencyType::from_str("SS").unwrap(), DependencyType::StartToStart);
        assert_eq!(DependencyType::from_str("FF").unwrap(), DependencyType::FinishToFinish);
        assert_eq!(DependencyType::from_str("SF").unwrap(), DependencyType::StartToFinish);

        assert!(DependencyType::from_str("INVALID").is_err());
    }

    #[test]
    fn test_task_dependency_creation() {
        let dep = TaskDependency::new(
            "TASK-1".to_string(),
            "TASK-2".to_string(),
            DependencyType::FinishToStart,
            "user".to_string(),
        )
        .unwrap();

        assert_eq!(dep.predecessor, "TASK-1");
        assert_eq!(dep.successor, "TASK-2");
        assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
        assert_eq!(dep.status, DependencyStatus::Active);
    }

    #[test]
    fn test_task_dependency_self_reference() {
        let result = TaskDependency::new(
            "TASK-1".to_string(),
            "TASK-1".to_string(),
            DependencyType::FinishToStart,
            "user".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_with_lag_time() {
        let dep = TaskDependency::new(
            "TASK-1".to_string(),
            "TASK-2".to_string(),
            DependencyType::FinishToStart,
            "user".to_string(),
        )
        .unwrap()
        .with_lag_time(Duration::days(2));

        assert_eq!(dep.lag_time, Some(Duration::days(2)));
    }

    #[test]
    fn test_cycle_detection() {
        let dep1 = TaskDependency::new(
            "TASK-1".to_string(),
            "TASK-2".to_string(),
            DependencyType::FinishToStart,
            "user".to_string(),
        )
        .unwrap();

        let dep2 = TaskDependency::new(
            "TASK-2".to_string(),
            "TASK-1".to_string(),
            DependencyType::FinishToStart,
            "user".to_string(),
        )
        .unwrap();

        assert!(dep1.would_create_cycle(&dep2));
    }
}
