use std::collections::HashMap;
use crate::domain::project_management::{ProjectTemplate, repository::ProjectRepository};
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::errors::DomainError;
use crate::application::create::project::CreateProjectUseCase;
use crate::application::create::resource::CreateResourceUseCase;
use crate::application::create::task::CreateTaskUseCase;
use chrono::NaiveDate;

pub struct CreateFromTemplateUseCase<PR: ProjectRepository, RR: ResourceRepository> {
    create_project_use_case: CreateProjectUseCase<PR>,
    create_resource_use_case: CreateResourceUseCase<RR>,
    create_task_use_case: CreateTaskUseCase<PR>,
}

impl<PR: ProjectRepository, RR: ResourceRepository> CreateFromTemplateUseCase<PR, RR> {
    pub fn new(
        create_project_use_case: CreateProjectUseCase<PR>,
        create_resource_use_case: CreateResourceUseCase<RR>,
        create_task_use_case: CreateTaskUseCase<PR>,
    ) -> Self {
        Self {
            create_project_use_case,
            create_resource_use_case,
            create_task_use_case,
        }
    }

    pub fn execute(
        &self,
        template: &ProjectTemplate,
        variables: &HashMap<String, String>,
        company_code: String,
    ) -> Result<CreatedProject, DomainError> {
        // Render template with variables
        let rendered = template.render(variables)
            .map_err(|e| DomainError::new(crate::domain::shared::errors::DomainErrorKind::Generic {
                message: format!("Template rendering failed: {}", e),
            }))?;

        // Create project
        let project_description = if rendered.project.description.is_empty() {
            None
        } else {
            Some(rendered.project.description.as_str())
        };

        let project = self.create_project_use_case.execute(
            &rendered.project.name,
            project_description,
            company_code.clone(),
        )?;

        let mut created_resources = Vec::new();
        let mut created_tasks = Vec::new();

        // Create resources
        for resource in &rendered.resources {
            self.create_resource_use_case.execute(
                &resource.name,
                &resource.r#type,
                company_code.clone(),
                None, // Global to company
            )?;

            created_resources.push(CreatedResource {
                name: resource.name.clone(),
                r#type: resource.r#type.clone(),
                skills: resource.skills.clone(),
                capacity: resource.capacity,
            });
        }

        // Create tasks
        for task in &rendered.tasks {
            // Parse dates
            let start_date = NaiveDate::parse_from_str(&rendered.project.start_date, "%Y-%m-%d")
                .map_err(|e| DomainError::new(crate::domain::shared::errors::DomainErrorKind::Generic {
                    message: format!("Invalid start date format: {}", e),
                }))?;

            let due_date = NaiveDate::parse_from_str(&rendered.project.end_date, "%Y-%m-%d")
                .map_err(|e| DomainError::new(crate::domain::shared::errors::DomainErrorKind::Generic {
                    message: format!("Invalid end date format: {}", e),
                }))?;

            let _task_description = if task.description.is_empty() {
                None
            } else {
                Some(task.description.as_str())
            };

            self.create_task_use_case.execute(
                crate::application::create::task::CreateTaskArgs {
                    company_code: company_code.clone(),
                    project_code: project.code().to_string(),
                    name: task.name.clone(),
                    start_date,
                    due_date,
                    assigned_resources: Vec::new(), // TODO: Assign resources based on template
                },
            )?;

            created_tasks.push(CreatedTask {
                name: task.name.clone(),
                description: task.description.clone(),
                priority: task.priority.clone(),
                category: task.category.clone(),
                estimated_hours: task.estimated_hours,
                dependencies: task.dependencies.clone(),
            });
        }

        Ok(CreatedProject {
            name: rendered.project.name,
            description: rendered.project.description,
            start_date: rendered.project.start_date,
            end_date: rendered.project.end_date,
            timezone: rendered.project.timezone,
            resources: created_resources,
            tasks: created_tasks,
            phases: rendered.phases,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CreatedProject {
    pub name: String,
    pub description: String,
    pub start_date: String,
    pub end_date: String,
    pub timezone: String,
    pub resources: Vec<CreatedResource>,
    pub tasks: Vec<CreatedTask>,
    pub phases: Vec<crate::domain::project_management::template::TemplatePhase>,
}

#[derive(Debug, Clone)]
pub struct CreatedResource {
    pub name: String,
    pub r#type: String,
    pub skills: Vec<String>,
    pub capacity: u8,
}

#[derive(Debug, Clone)]
pub struct CreatedTask {
    pub name: String,
    pub description: String,
    pub priority: String,
    pub category: String,
    pub estimated_hours: u32,
    pub dependencies: Vec<String>,
}

impl CreatedProject {
    pub fn display_summary(&self) -> String {
        format!(
            "Project '{}' created successfully with {} resources, {} tasks, and {} phases",
            self.name,
            self.resources.len(),
            self.tasks.len(),
            self.phases.len()
        )
    }

    pub fn display_resources(&self) -> String {
        if self.resources.is_empty() {
            "No resources created".to_string()
        } else {
            self.resources
                .iter()
                .map(|r| format!("- {} ({})", r.name, r.r#type))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    pub fn display_tasks(&self) -> String {
        if self.tasks.is_empty() {
            "No tasks created".to_string()
        } else {
            self.tasks
                .iter()
                .map(|t| format!("- {} ({}h, {})", t.name, t.estimated_hours, t.priority))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_created_project_display() {
        let project = CreatedProject {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            start_date: "2024-01-01".to_string(),
            end_date: "2024-12-31".to_string(),
            timezone: "UTC".to_string(),
            resources: vec![
                CreatedResource {
                    name: "Alice".to_string(),
                    r#type: "Developer".to_string(),
                    skills: vec!["Rust".to_string()],
                    capacity: 8,
                },
            ],
            tasks: vec![
                CreatedTask {
                    name: "Setup".to_string(),
                    description: "Project setup".to_string(),
                    priority: "high".to_string(),
                    category: "setup".to_string(),
                    estimated_hours: 8,
                    dependencies: vec![],
                },
            ],
            phases: vec![],
        };

        assert!(project.display_summary().contains("Test Project"));
        assert!(project.display_summary().contains("1 resources"));
        assert!(project.display_summary().contains("1 tasks"));
    }
}
