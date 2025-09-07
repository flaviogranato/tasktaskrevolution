use super::super::commands::UpdateCommand;
use crate::{
    application::{
        project::update_project::{UpdateProjectArgs, UpdateProjectUseCase},
        resource::update_resource::{UpdateResourceArgs, UpdateResourceUseCase},
        task::update_task::{UpdateTaskArgs, UpdateTaskUseCase},
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};
use chrono::NaiveDate;

pub fn handle_update_command(command: UpdateCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        UpdateCommand::Project {
            code,
            company,
            name,
            description,
            start_date,
            end_date,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let update_use_case = UpdateProjectUseCase::new(project_repository);

            let start = start_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let end = end_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            let args = UpdateProjectArgs { name, description };

            match update_use_case.execute(&code, args) {
                Ok(_) => {
                    println!("✅ Project updated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to update project: {}", e);
                    Err(e.into())
                }
            }
        }
        UpdateCommand::Task {
            code,
            project,
            company,
            name,
            description,
            start_date,
            due_date,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let update_use_case = UpdateTaskUseCase::new(project_repository);

            let start = start_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let due = due_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid due date format: {}", e))?;

            let args = UpdateTaskArgs {
                name,
                description,
                start_date: start,
                due_date: due,
            };

            match update_use_case.execute(&code, &project, args) {
                Ok(_) => {
                    println!("✅ Task updated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to update task: {}", e);
                    Err(e.into())
                }
            }
        }
        UpdateCommand::Resource {
            code,
            name,
            email,
            description,
        } => {
            let resource_repository = FileResourceRepository::new(".");
            let update_use_case = UpdateResourceUseCase::new(resource_repository);

            let args = UpdateResourceArgs {
                name,
                email,
                resource_type: description,
            };

            match update_use_case.execute(&code, args) {
                Ok(_) => {
                    println!("✅ Resource updated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to update resource: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
