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
    MissingField(&'static str),
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
}
