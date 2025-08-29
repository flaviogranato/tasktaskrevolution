use super::state::{Blocked, Cancelled, Completed, InProgress, Planned, TaskState};
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use uuid7::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Task<S: TaskState> {
    pub id: Uuid,
    pub project_code: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(flatten)]
    pub state: S,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub actual_end_date: Option<NaiveDate>,
    pub dependencies: Vec<String>,
    pub assigned_resources: Vec<String>,
}

// Transitions for a Planned task
impl Task<Planned> {
    /// Starts a planned task, moving it to the InProgress state.
    #[allow(dead_code)]
    pub fn start(self) -> Task<InProgress> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: InProgress { progress: 0 },
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }

    /// Cancels a planned task.
    #[allow(dead_code)]
    pub fn cancel(self) -> Task<Cancelled> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Cancelled,
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }
}

// Actions and transitions for an InProgress task
impl Task<InProgress> {
    /// Updates the progress of the task.
    #[allow(dead_code)]
    pub fn update_progress(mut self, progress: u8) -> Self {
        self.state.progress = progress;
        self
    }

    /// Blocks the task for a given reason.
    #[allow(dead_code)]
    pub fn block(self, reason: String) -> Task<Blocked> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Blocked { reason },
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }

    /// Completes the task.
    #[allow(dead_code)]
    pub fn complete(self) -> Task<Completed> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Completed,
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: Some(Utc::now().date_naive()),
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }

    /// Cancels the task.
    #[allow(dead_code)]
    pub fn cancel(self) -> Task<Cancelled> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Cancelled,
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }
}

// Transitions for a Blocked task
impl Task<Blocked> {
    /// Unblocks the task, returning it to the InProgress state.
    #[allow(dead_code)]
    pub fn unblock(self) -> Task<InProgress> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: InProgress { progress: 0 }, // Assuming progress resets
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }

    /// Cancels the blocked task.
    #[allow(dead_code)]
    pub fn cancel(self) -> Task<Cancelled> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Cancelled,
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: None,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }
}

#[allow(dead_code)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[allow(dead_code)]
impl DateRange {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TaskError {
    InvalidDateRange,
    ResourceOnVacation(String),
    MissingField(String),  // Otimizado: removido &'static str desnecessário
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::InvalidDateRange => write!(f, "Data inicial é posterior à data final."),
            TaskError::ResourceOnVacation(res) => {
                write!(f, "Recurso {res} está de férias neste período.")
            }
            TaskError::MissingField(field) => {
                write!(f, "Campo obrigatório não informado: {field}")
            }
        }
    }
}

impl std::error::Error for TaskError {}

// Common methods for all Task states
impl<S: TaskState> Task<S> {
    // --- Zero-copy accessors ---

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn project_code(&self) -> &str {
        &self.project_code
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn start_date(&self) -> &NaiveDate {
        &self.start_date
    }

    pub fn due_date(&self) -> &NaiveDate {
        &self.due_date
    }

    pub fn actual_end_date(&self) -> Option<&NaiveDate> {
        self.actual_end_date.as_ref()
    }

    pub fn dependencies(&self) -> &[String] {
        self.dependencies.as_slice()
    }

    pub fn assigned_resources(&self) -> &[String] {
        self.assigned_resources.as_slice()
    }

    // Nota: Task não tem campos estimated_hours e actual_hours
    // Esses campos foram removidos na refatoração anterior
    // Os métodos foram removidos para manter consistência

    // --- Iterators ---

    pub fn dependencies_iter(&self) -> impl Iterator<Item = &String> {
        self.dependencies.iter()
    }

    pub fn assigned_resources_iter(&self) -> impl Iterator<Item = &String> {
        self.assigned_resources.iter()
    }

    // Validation methods
    pub fn is_code_valid(&self) -> bool {
        !self.code.trim().is_empty()
    }

    pub fn is_name_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn is_date_range_valid(&self) -> bool {
        self.start_date <= self.due_date
    }

    pub fn validate(&self) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        if !self.is_code_valid() {
            errors.push("Task code cannot be empty".to_string());
        }

        if !self.is_name_valid() {
            errors.push("Task name cannot be empty".to_string());
        }

        if !self.is_date_range_valid() {
            errors.push("Task due date must be after start date".to_string());
        }

        Ok(errors)
    }
}

// Transition trait for state changes
pub trait Transition {
    type NextState: TaskState;
    fn transition(self) -> Task<Self::NextState>;
}

impl Transition for Task<Planned> {
    type NextState = InProgress;
    
    fn transition(self) -> Task<InProgress> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: InProgress { progress: 0 },
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: self.actual_end_date,
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }
}

impl Transition for Task<InProgress> {
    type NextState = Completed;
    
    fn transition(self) -> Task<Completed> {
        Task {
            id: self.id,
            project_code: self.project_code,
            code: self.code,
            name: self.name,
            description: self.description,
            state: Completed,
            start_date: self.start_date,
            due_date: self.due_date,
            actual_end_date: Some(chrono::Utc::now().date_naive()),
            dependencies: self.dependencies,
            assigned_resources: self.assigned_resources,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid7::uuid7;

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn create_planned_task() -> Task<Planned> {
        Task {
            id: uuid7(),
            project_code: "proj-x".to_string(),
            code: "T1".to_string(),
            name: "My test task".to_string(),
            description: None,
            state: Planned,
            start_date: d(2024, 7, 1),
            due_date: d(2024, 7, 31),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
        }
    }

    #[test]
    fn test_date_range_overlaps() {
        let base_range = DateRange {
            start: d(2024, 1, 10),
            end: d(2024, 1, 20),
        };

        let inside_range = DateRange {
            start: d(2024, 1, 12),
            end: d(2024, 1, 18),
        };
        assert!(base_range.overlaps(&inside_range));

        let after_range = DateRange {
            start: d(2024, 1, 21),
            end: d(2024, 1, 25),
        };
        assert!(!base_range.overlaps(&after_range));
    }

    #[test]
    fn test_task_error_display_formatting() {
        let invalid_date_err = TaskError::InvalidDateRange;
        assert_eq!(format!("{invalid_date_err}"), "Data inicial é posterior à data final.");

        let vacation_err = TaskError::ResourceOnVacation("RES-123".to_string());
        assert_eq!(
            format!("{vacation_err}"),
            "Recurso RES-123 está de férias neste período."
        );
    }

    #[test]
    fn test_planned_to_in_progress() {
        let task = create_planned_task();
        let task_id = task.id;
        let in_progress_task = task.start();
        assert_eq!(in_progress_task.state.progress, 0);
        // Verify other fields are carried over
        assert_eq!(in_progress_task.id, task_id);
    }

    #[test]
    fn test_in_progress_to_completed() {
        let task = create_planned_task().start();
        let completed_task = task.complete();
        assert!(completed_task.actual_end_date.is_some());
    }

    #[test]
    fn test_in_progress_update_progress() {
        let task = create_planned_task().start();
        assert_eq!(task.state.progress, 0);
        let updated_task = task.update_progress(50);
        assert_eq!(updated_task.state.progress, 50);
    }

    #[test]
    fn test_in_progress_to_blocked() {
        let task = create_planned_task().start();
        let reason = "Waiting for review".to_string();
        let blocked_task = task.block(reason.clone());
        assert_eq!(blocked_task.state.reason, reason);
    }

    #[test]
    fn test_blocked_to_unblocked() {
        let task = create_planned_task().start().block("Needs clarification".to_string());
        let in_progress_task = task.unblock();
        assert_eq!(in_progress_task.state.progress, 0);
    }

    #[test]
    fn test_cancel_from_planned() {
        let task = create_planned_task();
        let task_id = task.id;
        let cancelled_task = task.cancel();
        // This is a compile-time check, but we can assert on the type if we had a way to get a string from it.
        // For now, just creating it is enough to test the transition exists.
        assert_eq!(cancelled_task.id, task_id);
    }

    #[test]
    fn test_task_creation_with_valid_data() {
        let task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Implement Login Feature".to_string(),
            description: Some("Create user authentication system".to_string()),
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert_eq!(task.code(), "TASK-001");
        assert_eq!(task.name(), "Implement Login Feature");
        assert_eq!(task.description(), Some("Create user authentication system"));
        assert_eq!(task.start_date(), &chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(task.due_date(), &chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert!(task.is_code_valid());
        assert!(task.is_name_valid());
        assert!(task.is_date_range_valid());
    }

    #[test]
    fn test_task_code_validation() {
        // Valid code
        let valid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(valid_task.is_code_valid());

        // Invalid code (empty)
        let invalid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(!invalid_task.is_code_valid());
    }

    #[test]
    fn test_task_name_validation() {
        // Valid name
        let valid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Implement Feature".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(valid_task.is_name_valid());

        // Invalid name (empty)
        let invalid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(!invalid_task.is_name_valid());
    }

    #[test]
    fn test_task_date_validation() {
        // Valid date range
        let valid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(valid_task.is_date_range_valid());

        // Invalid date range (end before start)
        let invalid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        assert!(!invalid_task.is_date_range_valid());
    }



    #[test]
    fn test_task_comprehensive_validation() {
        let valid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Implement Feature".to_string(),
            description: Some("A comprehensive test task".to_string()),
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        let validation_result = valid_task.validate();
        assert!(validation_result.is_ok());
        assert_eq!(validation_result.unwrap().len(), 0); // No validation errors

        let invalid_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "".to_string(),
            name: "".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        let validation_result = invalid_task.validate();
        assert!(validation_result.is_ok());
        let errors = validation_result.unwrap();
        assert!(errors.len() > 0); // Should have validation errors
        assert!(errors.iter().any(|e| e.contains("code")));
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("date")));
    }

    #[test]
    fn test_task_state_transitions() {
        let planned_task = Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        // Transition from Planned to InProgress
        let in_progress_task: Task<InProgress> = planned_task.transition();
        assert!(matches!(in_progress_task.state, InProgress));

        // Transition from InProgress to Completed
        let in_progress_task = Task::<InProgress> {
            id: uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: InProgress { progress: 50 },
            start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: Vec::new(),
            assigned_resources: Vec::new(),
        };

        let completed_task: Task<Completed> = in_progress_task.transition();
        assert!(matches!(completed_task.state, Completed));
    }
}
