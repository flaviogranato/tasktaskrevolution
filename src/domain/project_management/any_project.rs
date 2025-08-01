use super::{
    super::task_management::any_task::AnyTask,
    project::Project,
    state::{Cancelled, Completed, InProgress, Planned},
    vacation_rules::VacationRules,
};
use chrono::NaiveDate;
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

    pub fn description(&self) -> Option<&String> {
        match self {
            AnyProject::Planned(p) => p.description.as_ref(),
            AnyProject::InProgress(p) => p.description.as_ref(),
            AnyProject::Completed(p) => p.description.as_ref(),
            AnyProject::Cancelled(p) => p.description.as_ref(),
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

    pub fn update_task(
        &mut self,
        task_code: &str,
        name: Option<String>,
        description: Option<String>,
        start_date: Option<NaiveDate>,
        due_date: Option<NaiveDate>,
    ) -> Result<(), String> {
        let tasks_map = match self {
            AnyProject::Planned(p) => &mut p.tasks,
            AnyProject::InProgress(p) => &mut p.tasks,
            AnyProject::Completed(_) => return Err("Cannot modify tasks in a completed project.".to_string()),
            AnyProject::Cancelled(_) => return Err("Cannot modify tasks in a cancelled project.".to_string()),
        };

        let task = tasks_map
            .get_mut(task_code)
            .ok_or_else(|| format!("Task '{task_code}' not found in project."))?;

        macro_rules! update_field {
            ($task_struct:expr, $field:ident, $value:expr) => {
                if let Some(val) = $value {
                    $task_struct.$field = val;
                }
            };
        }

        macro_rules! update_optional_field {
            ($task_struct:expr, $field:ident, $value:expr) => {
                if let Some(val) = $value {
                    $task_struct.$field = Some(val);
                }
            };
        }

        match task {
            AnyTask::Planned(t) => {
                update_field!(t, name, name);
                update_optional_field!(t, description, description);
                update_field!(t, start_date, start_date);
                update_field!(t, due_date, due_date);
            }
            AnyTask::InProgress(t) => {
                update_field!(t, name, name);
                update_optional_field!(t, description, description);
                update_field!(t, start_date, start_date);
                update_field!(t, due_date, due_date);
            }
            AnyTask::Blocked(t) => {
                update_field!(t, name, name);
                update_optional_field!(t, description, description);
                update_field!(t, start_date, start_date);
                update_field!(t, due_date, due_date);
            }
            AnyTask::Completed(_) => return Err("Cannot modify a completed task.".to_string()),
            AnyTask::Cancelled(_) => return Err("Cannot modify a cancelled task.".to_string()),
        }

        Ok(())
    }

    pub fn cancel_task(&mut self, task_code: &str) -> Result<(), String> {
        let tasks_map = match self {
            AnyProject::Planned(p) => &mut p.tasks,
            AnyProject::InProgress(p) => &mut p.tasks,
            AnyProject::Completed(_) => return Err("Cannot cancel tasks in a completed project.".to_string()),
            AnyProject::Cancelled(_) => return Err("Cannot cancel tasks in a cancelled project.".to_string()),
        };

        // Take the task out of the map to be able to consume it in the state transition
        let task = tasks_map
            .remove(task_code)
            .ok_or_else(|| format!("Task '{task_code}' not found in project."))?;

        let cancelled_task: AnyTask = match task {
            AnyTask::Planned(t) => t.cancel().into(),
            AnyTask::InProgress(t) => t.cancel().into(),
            AnyTask::Blocked(t) => t.cancel().into(),
            AnyTask::Completed(t) => {
                // Cannot cancel a completed task, so we put it back and return an error.
                let original_task = t.into();
                tasks_map.insert(task_code.to_string(), original_task);
                return Err("Cannot cancel a completed task.".to_string());
            }
            AnyTask::Cancelled(t) => {
                // Task is already cancelled, no action needed. Just put it back.
                let original_task = t.into();
                tasks_map.insert(task_code.to_string(), original_task);
                return Ok(());
            }
        };

        tasks_map.insert(task_code.to_string(), cancelled_task);

        Ok(())
    }
    pub fn set_name(&mut self, name: String) {
        match self {
            AnyProject::Planned(p) => p.name = name,
            AnyProject::InProgress(p) => p.name = name,
            AnyProject::Completed(p) => p.name = name,
            AnyProject::Cancelled(p) => p.name = name,
        }
    }

    pub fn set_description(&mut self, description: Option<String>) {
        match self {
            AnyProject::Planned(p) => p.description = description,
            AnyProject::InProgress(p) => p.description = description,
            AnyProject::Completed(p) => p.description = description,
            AnyProject::Cancelled(p) => p.description = description,
        }
    }

    pub fn status(&self) -> &'static str {
        match self {
            AnyProject::Planned(_) => "Planned",
            AnyProject::InProgress(_) => "InProgress",
            AnyProject::Completed(_) => "Completed",
            AnyProject::Cancelled(_) => "Cancelled",
        }
    }

    pub fn cancel(self) -> Result<AnyProject, String> {
        let cancelled_project = match self {
            AnyProject::Planned(p) => p.cancel().into(),
            AnyProject::InProgress(p) => p.cancel().into(),
            AnyProject::Completed(_) => return Err("Cannot cancel a completed project.".to_string()),
            AnyProject::Cancelled(_) => return Err("Project is already cancelled.".to_string()),
        };
        Ok(cancelled_project)
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
