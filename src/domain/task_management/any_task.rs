use super::{
    state::{Blocked, Cancelled, Completed, InProgress, Planned},
    task::Task,
};
use chrono::NaiveDate;
use serde::Serialize;
use uuid7::Uuid;

/// An enum to represent a Task in any of its possible states.
/// This is useful for storing tasks in a repository or a collection
/// where the exact state is not known at compile time.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")] // This will serialize the enum with a "status" field identifying the variant.
pub enum AnyTask {
    Planned(Task<Planned>),
    InProgress(Task<InProgress>),
    Blocked(Task<Blocked>),
    Completed(Task<Completed>),
    Cancelled(Task<Cancelled>),
}

// Implement helper methods on AnyTask to access common Task fields.
// This avoids repetitive match statements in other parts of the code.
impl AnyTask {
    pub fn code(&self) -> &str {
        match self {
            AnyTask::Planned(task) => &task.code,
            AnyTask::InProgress(task) => &task.code,
            AnyTask::Blocked(task) => &task.code,
            AnyTask::Completed(task) => &task.code,
            AnyTask::Cancelled(task) => &task.code,
        }
    }

    pub fn id(&self) -> &Uuid {
        match self {
            AnyTask::Planned(task) => &task.id,
            AnyTask::InProgress(task) => &task.id,
            AnyTask::Blocked(task) => &task.id,
            AnyTask::Completed(task) => &task.id,
            AnyTask::Cancelled(task) => &task.id,
        }
    }

    pub fn assigned_resources(&self) -> &[String] {
        match self {
            AnyTask::Planned(task) => &task.assigned_resources,
            AnyTask::InProgress(task) => &task.assigned_resources,
            AnyTask::Blocked(task) => &task.assigned_resources,
            AnyTask::Completed(task) => &task.assigned_resources,
            AnyTask::Cancelled(task) => &task.assigned_resources,
        }
    }

    pub fn start_date(&self) -> NaiveDate {
        match self {
            AnyTask::Planned(task) => task.start_date,
            AnyTask::InProgress(task) => task.start_date,
            AnyTask::Blocked(task) => task.start_date,
            AnyTask::Completed(task) => task.start_date,
            AnyTask::Cancelled(task) => task.start_date,
        }
    }

    pub fn due_date(&self) -> NaiveDate {
        match self {
            AnyTask::Planned(task) => task.due_date,
            AnyTask::InProgress(task) => task.due_date,
            AnyTask::Blocked(task) => task.due_date,
            AnyTask::Completed(task) => task.due_date,
            AnyTask::Cancelled(task) => task.due_date,
        }
    }
}

// Provide From implementations to easily convert a specific Task<State> into an AnyTask.
impl From<Task<Planned>> for AnyTask {
    fn from(task: Task<Planned>) -> Self {
        AnyTask::Planned(task)
    }
}

impl From<Task<InProgress>> for AnyTask {
    fn from(task: Task<InProgress>) -> Self {
        AnyTask::InProgress(task)
    }
}

impl From<Task<Blocked>> for AnyTask {
    fn from(task: Task<Blocked>) -> Self {
        AnyTask::Blocked(task)
    }
}

impl From<Task<Completed>> for AnyTask {
    fn from(task: Task<Completed>) -> Self {
        AnyTask::Completed(task)
    }
}

impl From<Task<Cancelled>> for AnyTask {
    fn from(task: Task<Cancelled>) -> Self {
        AnyTask::Cancelled(task)
    }
}
