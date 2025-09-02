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

    pub fn assigned_resources(&self) -> &[String] {
        // Zero-copy: retorna slice em vez de referência a Vec
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

    pub fn with_assigned_resources(&self, new_assigned_resources: Vec<String>) -> Self {
        match self {
            AnyTask::Planned(task) => {
                let mut new_task = task.clone();
                new_task.assigned_resources = new_assigned_resources;
                AnyTask::Planned(new_task)
            }
            AnyTask::InProgress(task) => {
                let mut new_task = task.clone();
                new_task.assigned_resources = new_assigned_resources;
                AnyTask::InProgress(new_task)
            }
            AnyTask::Blocked(task) => {
                let mut new_task = task.clone();
                new_task.assigned_resources = new_assigned_resources;
                AnyTask::Blocked(new_task)
            }
            AnyTask::Completed(task) => {
                let mut new_task = task.clone();
                new_task.assigned_resources = new_assigned_resources;
                AnyTask::Completed(new_task)
            }
            AnyTask::Cancelled(task) => {
                let mut new_task = task.clone();
                new_task.assigned_resources = new_assigned_resources;
                AnyTask::Cancelled(new_task)
            }
        }
    }

    pub fn cancel(self) -> Self {
        match self {
            AnyTask::Planned(task) => AnyTask::Cancelled(task.cancel()),
            AnyTask::InProgress(task) => AnyTask::Cancelled(task.cancel()),
            AnyTask::Blocked(task) => AnyTask::Cancelled(task.cancel()),
            AnyTask::Completed(_) => self, // Completed tasks cannot be cancelled
            AnyTask::Cancelled(_) => self, // Already cancelled
        }
    }

    pub fn description(&self) -> Option<&str> {
        // Zero-copy: retorna &str em vez de &String
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

    pub fn add_dependency(&self, dependency_code: String) -> Self {
        match self {
            AnyTask::Planned(task) => {
                let mut new_task = task.clone();
                if !new_task.dependencies.contains(&dependency_code) {
                    new_task.dependencies.push(dependency_code);
                }
                AnyTask::Planned(new_task)
            }
            AnyTask::InProgress(task) => {
                let mut new_task = task.clone();
                if !new_task.dependencies.contains(&dependency_code) {
                    new_task.dependencies.push(dependency_code);
                }
                AnyTask::InProgress(new_task)
            }
            AnyTask::Blocked(task) => {
                let mut new_task = task.clone();
                if !new_task.dependencies.contains(&dependency_code) {
                    new_task.dependencies.push(dependency_code);
                }
                AnyTask::Blocked(new_task)
            }
            AnyTask::Completed(task) => {
                let mut new_task = task.clone();
                if !new_task.dependencies.contains(&dependency_code) {
                    new_task.dependencies.push(dependency_code);
                }
                AnyTask::Completed(new_task)
            }
            AnyTask::Cancelled(task) => {
                let mut new_task = task.clone();
                if !new_task.dependencies.contains(&dependency_code) {
                    new_task.dependencies.push(dependency_code);
                }
                AnyTask::Cancelled(new_task)
            }
        }
    }

    pub fn remove_dependency(&self, dependency_code: &str) -> Self {
        match self {
            AnyTask::Planned(task) => {
                let mut new_task = task.clone();
                new_task.dependencies.retain(|dep| dep != dependency_code);
                AnyTask::Planned(new_task)
            }
            AnyTask::InProgress(task) => {
                let mut new_task = task.clone();
                new_task.dependencies.retain(|dep| dep != dependency_code);
                AnyTask::InProgress(new_task)
            }
            AnyTask::Blocked(task) => {
                let mut new_task = task.clone();
                new_task.dependencies.retain(|dep| dep != dependency_code);
                AnyTask::Blocked(new_task)
            }
            AnyTask::Completed(task) => {
                let mut new_task = task.clone();
                new_task.dependencies.retain(|dep| dep != dependency_code);
                AnyTask::Completed(new_task)
            }
            AnyTask::Cancelled(task) => {
                let mut new_task = task.clone();
                new_task.dependencies.retain(|dep| dep != dependency_code);
                AnyTask::Cancelled(new_task)
            }
        }
    }

    pub fn update_fields(
        &self,
        name: Option<String>,
        description: Option<String>,
        start_date: Option<NaiveDate>,
        due_date: Option<NaiveDate>,
    ) -> Self {
        match self {
            AnyTask::Planned(task) => {
                let mut new_task = task.clone();
                if let Some(name) = name {
                    new_task.name = name;
                }
                if let Some(description) = description {
                    new_task.description = Some(description);
                }
                if let Some(start_date) = start_date {
                    new_task.start_date = start_date;
                }
                if let Some(due_date) = due_date {
                    new_task.due_date = due_date;
                }
                AnyTask::Planned(new_task)
            }
            AnyTask::InProgress(task) => {
                let mut new_task = task.clone();
                if let Some(name) = name {
                    new_task.name = name;
                }
                if let Some(description) = description {
                    new_task.description = Some(description);
                }
                if let Some(start_date) = start_date {
                    new_task.start_date = start_date;
                }
                if let Some(due_date) = due_date {
                    new_task.due_date = due_date;
                }
                AnyTask::InProgress(new_task)
            }
            AnyTask::Blocked(task) => {
                let mut new_task = task.clone();
                if let Some(name) = name {
                    new_task.name = name;
                }
                if let Some(description) = description {
                    new_task.description = Some(description);
                }
                if let Some(start_date) = start_date {
                    new_task.start_date = start_date;
                }
                if let Some(due_date) = due_date {
                    new_task.due_date = due_date;
                }
                AnyTask::Blocked(new_task)
            }
            AnyTask::Completed(task) => {
                let mut new_task = task.clone();
                if let Some(name) = name {
                    new_task.name = name;
                }
                if let Some(description) = description {
                    new_task.description = Some(description);
                }
                if let Some(start_date) = start_date {
                    new_task.start_date = start_date;
                }
                if let Some(due_date) = due_date {
                    new_task.due_date = due_date;
                }
                AnyTask::Completed(new_task)
            }
            AnyTask::Cancelled(task) => {
                let mut new_task = task.clone();
                if let Some(name) = name {
                    new_task.name = name;
                }
                if let Some(description) = description {
                    new_task.description = Some(description);
                }
                if let Some(start_date) = start_date {
                    new_task.start_date = start_date;
                }
                if let Some(due_date) = due_date {
                    new_task.due_date = due_date;
                }
                AnyTask::Cancelled(new_task)
            }
        }
    }

    pub fn complete(self) -> AnyTask {
        match self {
            AnyTask::Planned(task) => {
                // First start the task, then complete it
                let in_progress_task = task.start();
                let completed_task = in_progress_task.complete();
                AnyTask::Completed(completed_task)
            }
            AnyTask::InProgress(task) => {
                let completed_task = task.complete();
                AnyTask::Completed(completed_task)
            }
            AnyTask::Blocked(task) => {
                // Unblock first, then complete
                let in_progress_task = task.unblock();
                let completed_task = in_progress_task.complete();
                AnyTask::Completed(completed_task)
            }
            AnyTask::Completed(_) => self, // Already completed
            AnyTask::Cancelled(_) => self, // Cannot complete a cancelled task
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
