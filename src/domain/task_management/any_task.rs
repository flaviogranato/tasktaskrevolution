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
#[derive(Debug, Clone, Serialize, PartialEq)]
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

    pub fn name(&self) -> &str {
        match self {
            AnyTask::Planned(task) => &task.name,
            AnyTask::InProgress(task) => &task.name,
            AnyTask::Blocked(task) => &task.name,
            AnyTask::Completed(task) => &task.name,
            AnyTask::Cancelled(task) => &task.name,
        }
    }

    pub fn status(&self) -> &str {
        match self {
            AnyTask::Planned(_) => "Planned",
            AnyTask::InProgress(_) => "InProgress",
            AnyTask::Blocked(_) => "Blocked",
            AnyTask::Completed(_) => "Completed",
            AnyTask::Cancelled(_) => "Cancelled",
        }
    }

    pub fn project_code(&self) -> &str {
        match self {
            AnyTask::Planned(task) => &task.project_code,
            AnyTask::InProgress(task) => &task.project_code,
            AnyTask::Blocked(task) => &task.project_code,
            AnyTask::Completed(task) => &task.project_code,
            AnyTask::Cancelled(task) => &task.project_code,
        }
    }

    pub fn assigned_resources(&self) -> &[String] {  // Zero-copy: retorna slice em vez de referência a Vec
        match self {
            AnyTask::Planned(task) => task.assigned_resources.as_slice(),
            AnyTask::InProgress(task) => task.assigned_resources.as_slice(),
            AnyTask::Blocked(task) => task.assigned_resources.as_slice(),
            AnyTask::Completed(task) => task.assigned_resources.as_slice(),
            AnyTask::Cancelled(task) => task.assigned_resources.as_slice(),
        }
    }

    // Adiciona método para iterador zero-copy
    pub fn assigned_resources_iter(&self) -> impl Iterator<Item = &String> {
        match self {
            AnyTask::Planned(task) => task.assigned_resources.iter(),
            AnyTask::InProgress(task) => task.assigned_resources.iter(),
            AnyTask::Blocked(task) => task.assigned_resources.iter(),
            AnyTask::Completed(task) => task.assigned_resources.iter(),
            AnyTask::Cancelled(task) => task.assigned_resources.iter(),
        }
    }

    pub fn description(&self) -> Option<&str> {  // Zero-copy: retorna &str em vez de &String
        match self {
            AnyTask::Planned(t) => t.description.as_deref(),
            AnyTask::InProgress(t) => t.description.as_deref(),
            AnyTask::Blocked(t) => t.description.as_deref(),
            AnyTask::Completed(t) => t.description.as_deref(),
            AnyTask::Cancelled(t) => t.description.as_deref(),
        }
    }

    pub fn start_date(&self) -> &NaiveDate {
        match self {
            AnyTask::Planned(t) => &t.start_date,
            AnyTask::InProgress(t) => &t.start_date,
            AnyTask::Blocked(t) => &t.start_date,
            AnyTask::Completed(t) => &t.start_date,
            AnyTask::Cancelled(t) => &t.start_date,
        }
    }

    pub fn due_date(&self) -> &NaiveDate {
        match self {
            AnyTask::Planned(t) => &t.due_date,
            AnyTask::InProgress(t) => &t.due_date,
            AnyTask::Blocked(t) => &t.due_date,
            AnyTask::Completed(t) => &t.due_date,
            AnyTask::Cancelled(t) => &t.due_date,
        }
    }

    // --- Zero-copy accessors ---

    // Nota: Task não tem campos estimated_hours e actual_hours
    // Esses campos foram removidos na refatoração anterior
    // Os métodos foram removidos para manter consistência

    pub fn dependencies_iter(&self) -> impl Iterator<Item = &String> {
        match self {
            AnyTask::Planned(t) => t.dependencies.iter(),
            AnyTask::InProgress(t) => t.dependencies.iter(),
            AnyTask::Blocked(t) => t.dependencies.iter(),
            AnyTask::Completed(t) => t.dependencies.iter(),
            AnyTask::Cancelled(t) => t.dependencies.iter(),
        }
    }

    pub fn dependencies(&self) -> &[String] {
        match self {
            AnyTask::Planned(t) => t.dependencies.as_slice(),
            AnyTask::InProgress(t) => t.dependencies.as_slice(),
            AnyTask::Blocked(t) => t.dependencies.as_slice(),
            AnyTask::Completed(t) => t.dependencies.as_slice(),
            AnyTask::Cancelled(t) => t.dependencies.as_slice(),
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
