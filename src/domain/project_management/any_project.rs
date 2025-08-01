use super::{
    super::task_management::any_task::AnyTask,
    project::Project,
    state::{Cancelled, Completed, InProgress, Planned},
    vacation_rules::VacationRules,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// An enum to represent a Project in any of its possible states.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum AnyProject {
    Planned(Project<Planned>),
    InProgress(Project<InProgress>),
    Completed(Project<Completed>),
    Cancelled(Project<Cancelled>),
}

impl AnyProject {
    pub fn name(&self) -> &str {
        match self {
            AnyProject::Planned(p) => &p.name,
            AnyProject::InProgress(p) => &p.name,
            AnyProject::Completed(p) => &p.name,
            AnyProject::Cancelled(p) => &p.name,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AnyProject::Planned(p) => &p.code,
            AnyProject::InProgress(p) => &p.code,
            AnyProject::Completed(p) => &p.code,
            AnyProject::Cancelled(p) => &p.code,
        }
    }

    pub fn timezone(&self) -> Option<&String> {
        match self {
            AnyProject::Planned(p) => p.timezone.as_ref(),
            AnyProject::InProgress(p) => p.timezone.as_ref(),
            AnyProject::Completed(p) => p.timezone.as_ref(),
            AnyProject::Cancelled(p) => p.timezone.as_ref(),
        }
    }

    pub fn vacation_rules(&self) -> Option<&VacationRules> {
        match self {
            AnyProject::Planned(p) => p.vacation_rules.as_ref(),
            AnyProject::InProgress(p) => p.vacation_rules.as_ref(),
            AnyProject::Completed(p) => p.vacation_rules.as_ref(),
            AnyProject::Cancelled(p) => p.vacation_rules.as_ref(),
        }
    }

    pub fn tasks(&self) -> &HashMap<String, AnyTask> {
        match self {
            AnyProject::Planned(p) => &p.tasks,
            AnyProject::InProgress(p) => &p.tasks,
            AnyProject::Completed(p) => &p.tasks,
            AnyProject::Cancelled(p) => &p.tasks,
        }
    }

    pub fn add_task(&mut self, task: AnyTask) {
        let tasks = match self {
            AnyProject::Planned(p) => &mut p.tasks,
            AnyProject::InProgress(p) => &mut p.tasks,
            AnyProject::Completed(p) => &mut p.tasks,
            AnyProject::Cancelled(p) => &mut p.tasks,
        };
        tasks.insert(task.code().to_string(), task);
    }

    pub fn assign_resource_to_task(&mut self, task_code: &str, resource_codes: &[&str]) -> Result<(), String> {
        let tasks_map = match self {
            AnyProject::Planned(p) => &mut p.tasks,
            AnyProject::InProgress(p) => &mut p.tasks,
            AnyProject::Completed(_) => return Err("Cannot modify tasks in a completed project.".to_string()),
            AnyProject::Cancelled(_) => return Err("Cannot modify tasks in a cancelled project.".to_string()),
        };

        let task = tasks_map
            .get_mut(task_code)
            .ok_or_else(|| format!("Task '{task_code}' not found in project."))?;

        // Logic to update assignees, handling duplicates
        let mut current_assignees: HashSet<String> = match task {
            AnyTask::Planned(t) => t.assigned_resources.iter().cloned().collect(),
            AnyTask::InProgress(t) => t.assigned_resources.iter().cloned().collect(),
            AnyTask::Blocked(t) => t.assigned_resources.iter().cloned().collect(),
            AnyTask::Completed(_) => return Err("Cannot assign resources to a completed task.".to_string()),
            AnyTask::Cancelled(_) => return Err("Cannot assign resources to a cancelled task.".to_string()),
        };

        for code in resource_codes {
            current_assignees.insert(code.to_string());
        }

        let new_assignees: Vec<String> = current_assignees.into_iter().collect();

        // Re-assign the updated list
        match task {
            AnyTask::Planned(t) => t.assigned_resources = new_assignees,
            AnyTask::InProgress(t) => t.assigned_resources = new_assignees,
            AnyTask::Blocked(t) => t.assigned_resources = new_assignees,
            _ => {} // Other states already returned an error
        }

        Ok(())
    }
}

impl From<Project<Planned>> for AnyProject {
    fn from(project: Project<Planned>) -> Self {
        AnyProject::Planned(project)
    }
}

impl From<Project<InProgress>> for AnyProject {
    fn from(project: Project<InProgress>) -> Self {
        AnyProject::InProgress(project)
    }
}

impl From<Project<Completed>> for AnyProject {
    fn from(project: Project<Completed>) -> Self {
        AnyProject::Completed(project)
    }
}

impl From<Project<Cancelled>> for AnyProject {
    fn from(project: Project<Cancelled>) -> Self {
        AnyProject::Cancelled(project)
    }
}
