use super::super::task_management::any_task::AnyTask;
use super::project::{Project, ProjectStatus};
use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An enum to represent a Project in any of its possible states.
/// This is now a wrapper around the unified Project entity for backward compatibility.
#[derive(Debug, Clone, Serialize)]
pub enum AnyProject {
    Project(Project),
}

impl AnyProject {
    pub fn name(&self) -> &str {
        match self {
            AnyProject::Project(p) => &p.name,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AnyProject::Project(p) => &p.code,
        }
    }

    pub fn description(&self) -> Option<&String> {
        match self {
            AnyProject::Project(p) => p.description.as_ref(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        match self {
            AnyProject::Project(p) => p.name = name,
        }
    }

    pub fn set_description(&mut self, description: Option<String>) {
        match self {
            AnyProject::Project(p) => p.description = description,
        }
    }

    pub fn cancel_task(&mut self, task_code: &str) -> Result<AnyTask, String> {
        match self {
            AnyProject::Project(p) => {
                if let Some(task) = p.tasks.get(task_code) {
                    // Create a new cancelled task by cloning and cancelling
                    let cancelled_task = task.clone().cancel();
                    // Replace the old task with the cancelled one
                    p.tasks.insert(task_code.to_string(), cancelled_task.clone());
                    Ok(cancelled_task)
                } else {
                    Err(format!("Task '{}' not found in project", task_code))
                }
            }
        }
    }

    pub fn add_dependency_to_task(&mut self, task_code: &str, dependency_code: &str) -> Result<AnyTask, String> {
        match self {
            AnyProject::Project(p) => {
                if let Some(task) = p.tasks.get_mut(task_code) {
                    // Add the dependency to the task
                    let updated_task = task.add_dependency(dependency_code.to_string());
                    // Replace the old task with the updated one
                    p.tasks.insert(task_code.to_string(), updated_task.clone());
                    Ok(updated_task)
                } else {
                    Err(format!("Task '{}' not found in project", task_code))
                }
            }
        }
    }

    pub fn remove_dependency_from_task(&mut self, task_code: &str, dependency_code: &str) -> Result<AnyTask, String> {
        match self {
            AnyProject::Project(p) => {
                if let Some(task) = p.tasks.get_mut(task_code) {
                    // Remove the dependency from the task
                    let updated_task = task.remove_dependency(dependency_code);
                    // Replace the old task with the updated one
                    p.tasks.insert(task_code.to_string(), updated_task.clone());
                    Ok(updated_task)
                } else {
                    Err(format!("Task '{}' not found in project", task_code))
                }
            }
        }
    }

    pub fn update_task(
        &mut self,
        task_code: &str,
        name: Option<String>,
        description: Option<String>,
        start_date: Option<NaiveDate>,
        due_date: Option<NaiveDate>,
    ) -> Result<AnyTask, String> {
        match self {
            AnyProject::Project(p) => {
                if let Some(task) = p.tasks.get_mut(task_code) {
                    // Update the task with new values
                    let updated_task = task.update_fields(name, description, start_date, due_date);
                    // Replace the old task with the updated one
                    p.tasks.insert(task_code.to_string(), updated_task.clone());
                    Ok(updated_task)
                } else {
                    Err(format!("Task '{}' not found in project", task_code))
                }
            }
        }
    }

    pub fn timezone(&self) -> Option<&String> {
        match self {
            AnyProject::Project(p) => p.settings.timezone.as_ref(),
        }
    }

    pub fn vacation_rules(&self) -> Option<&super::project::VacationRules> {
        match self {
            AnyProject::Project(p) => p.settings.vacation_rules.as_ref(),
        }
    }

    pub fn tasks(&self) -> &HashMap<String, AnyTask> {
        match self {
            AnyProject::Project(p) => &p.tasks,
        }
    }

    pub fn tasks_iter(&self) -> impl Iterator<Item = (&String, &AnyTask)> {
        match self {
            AnyProject::Project(p) => p.tasks.iter(),
        }
    }

    pub fn task_codes(&self) -> impl Iterator<Item = &String> {
        match self {
            AnyProject::Project(p) => p.tasks.keys(),
        }
    }

    pub fn add_task(&mut self, task: AnyTask) {
        match self {
            AnyProject::Project(p) => {
                // Insert the task directly using its code
                p.tasks.insert(task.code().to_string(), task);
            }
        }
    }

    pub fn assign_resource_to_task(&mut self, task_code: &str, resource_codes: &[&str]) -> Result<(), String> {
        match self {
            AnyProject::Project(p) => {
                // First, get the task and collect the new assigned resources
                let mut new_assigned_resources = if let Some(task) = p.tasks.get(task_code) {
                    let mut resources = task.assigned_resources().to_vec();
                    for &resource_code in resource_codes {
                        if !resources.contains(&resource_code.to_string()) {
                            resources.push(resource_code.to_string());
                        }
                    }
                    resources
                } else {
                    return Err(format!("Task '{}' not found in project", task_code));
                };

                // Now update the task with new resources
                if let Some(task) = p.tasks.get(task_code) {
                    let new_task = task.with_assigned_resources(new_assigned_resources);
                    p.tasks.insert(task_code.to_string(), new_task);
                }
                
                Ok(())
            }
        }
    }



    pub fn reschedule_dependents_of(&mut self, updated_task_code: &str) -> Result<(), String> {
        match self {
            AnyProject::Project(p) => {
                // TODO: Implementar reagendamento de dependentes
                // Por enquanto, apenas retornamos sucesso
                Ok(())
            }
        }
    }

    pub fn cancel(self) -> Result<AnyProject, String> {
        match self {
            AnyProject::Project(mut p) => {
                if let Err(e) = p.change_status(ProjectStatus::Cancelled) {
                    return Err(format!("Failed to cancel project: {:?}", e));
                }
                Ok(AnyProject::Project(p))
            }
        }
    }

    pub fn start(self) -> Result<AnyProject, String> {
        match self {
            AnyProject::Project(mut p) => {
                if let Err(e) = p.change_status(ProjectStatus::InProgress) {
                    return Err(format!("Failed to start project: {:?}", e));
                }
                Ok(AnyProject::Project(p))
            }
        }
    }

    pub fn complete(self) -> Result<AnyProject, String> {
        match self {
            AnyProject::Project(mut p) => {
                if let Err(e) = p.change_status(ProjectStatus::Completed) {
                    return Err(format!("Failed to complete project: {:?}", e));
                }
                Ok(AnyProject::Project(p))
            }
        }
    }

    pub fn put_on_hold(self) -> Result<AnyProject, String> {
        match self {
            AnyProject::Project(mut p) => {
                if let Err(e) = p.change_status(ProjectStatus::OnHold) {
                    return Err(format!("Failed to put project on hold: {:?}", e));
                }
                Ok(AnyProject::Project(p))
            }
        }
    }

    pub fn resume(self) -> Result<AnyProject, String> {
        match self {
            AnyProject::Project(mut p) => {
                if let Err(e) = p.change_status(ProjectStatus::InProgress) {
                    return Err(format!("Failed to resume project: {:?}", e));
                }
                Ok(AnyProject::Project(p))
            }
        }
    }

    pub fn status(&self) -> ProjectStatus {
        match self {
            AnyProject::Project(p) => p.status,
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            AnyProject::Project(p) => p.status.is_active(),
        }
    }

    pub fn is_completed(&self) -> bool {
        match self {
            AnyProject::Project(p) => matches!(p.status, ProjectStatus::Completed),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        match self {
            AnyProject::Project(p) => matches!(p.status, ProjectStatus::Cancelled),
        }
    }

    pub fn is_planned(&self) -> bool {
        match self {
            AnyProject::Project(p) => matches!(p.status, ProjectStatus::Planned),
        }
    }

    pub fn is_in_progress(&self) -> bool {
        match self {
            AnyProject::Project(p) => matches!(p.status, ProjectStatus::InProgress),
        }
    }

    pub fn is_on_hold(&self) -> bool {
        match self {
            AnyProject::Project(p) => matches!(p.status, ProjectStatus::OnHold),
        }
    }

    pub fn has_tasks(&self) -> bool {
        match self {
            AnyProject::Project(p) => p.has_tasks(),
        }
    }

    pub fn has_resources(&self) -> bool {
        match self {
            AnyProject::Project(p) => p.has_resources(),
        }
    }

    pub fn is_on_schedule(&self) -> bool {
        match self {
            AnyProject::Project(p) => p.is_on_schedule(),
        }
    }

    pub fn completion_percentage(&self) -> f64 {
        match self {
            AnyProject::Project(p) => p.completion_percentage(),
        }
    }

    pub fn company_code(&self) -> &str {
        match self {
            AnyProject::Project(p) => &p.company_code,
        }
    }

    pub fn created_by(&self) -> &str {
        match self {
            AnyProject::Project(p) => &p.created_by,
        }
    }

    pub fn priority(&self) -> super::project::ProjectPriority {
        match self {
            AnyProject::Project(p) => p.priority,
        }
    }

    pub fn start_date(&self) -> Option<chrono::NaiveDate> {
        match self {
            AnyProject::Project(p) => p.start_date,
        }
    }

    pub fn end_date(&self) -> Option<chrono::NaiveDate> {
        match self {
            AnyProject::Project(p) => p.end_date,
        }
    }

    pub fn actual_start_date(&self) -> Option<chrono::NaiveDate> {
        match self {
            AnyProject::Project(p) => p.actual_start_date,
        }
    }

    pub fn actual_end_date(&self) -> Option<chrono::NaiveDate> {
        match self {
            AnyProject::Project(p) => p.actual_end_date,
        }
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            AnyProject::Project(p) => p.created_at,
        }
    }

    pub fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            AnyProject::Project(p) => p.updated_at,
        }
    }

    pub fn manager_id(&self) -> Option<&String> {
        match self {
            AnyProject::Project(p) => p.manager_id.as_ref(),
        }
    }

    pub fn resources(&self) -> &HashMap<String, super::project::ResourceAssignment> {
        match self {
            AnyProject::Project(p) => &p.resources,
        }
    }

    pub fn settings(&self) -> &super::project::ProjectSettings {
        match self {
            AnyProject::Project(p) => &p.settings,
        }
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        match self {
            AnyProject::Project(p) => &p.metadata,
        }
    }
}

impl From<Project> for AnyProject {
    fn from(project: Project) -> Self {
        AnyProject::Project(project)
    }
}

impl From<AnyProject> for Project {
    fn from(any_project: AnyProject) -> Self {
        match any_project {
            AnyProject::Project(project) => project,
        }
    }
}

impl AsRef<Project> for AnyProject {
    fn as_ref(&self) -> &Project {
        match self {
            AnyProject::Project(project) => project,
        }
    }
}

impl AsMut<Project> for AnyProject {
    fn as_mut(&mut self) -> &mut Project {
        match self {
            AnyProject::Project(project) => project,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::project::{ProjectPriority, ProjectStatus};

    #[test]
    fn test_any_project_creation() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        let any_project = AnyProject::from(project);
        
        assert_eq!(any_project.name(), "Test Project");
        assert_eq!(any_project.code(), "PROJ-001");
        assert_eq!(any_project.company_code(), "COMP-001");
        assert_eq!(any_project.status(), ProjectStatus::Planned);
        assert_eq!(any_project.priority(), ProjectPriority::Medium);
    }

    #[test]
    fn test_any_project_status_transitions() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        let any_project = AnyProject::from(project);
        
        // Start project
        let started_project = any_project.start().unwrap();
        assert!(started_project.is_in_progress());
        
        // Complete project
        let completed_project = started_project.complete().unwrap();
        assert!(completed_project.is_completed());
    }

    #[test]
    fn test_any_project_cancel() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        let any_project = AnyProject::from(project);
        let cancelled_project = any_project.cancel().unwrap();
        
        assert!(cancelled_project.is_cancelled());
    }

    #[test]
    fn test_any_project_conversion() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        let any_project = AnyProject::from(project);
        let converted_project: Project = any_project.into();
        
        assert_eq!(converted_project.code(), "PROJ-001");
        assert_eq!(converted_project.name(), "Test Project");
    }
}
