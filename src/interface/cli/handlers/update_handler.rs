use super::super::commands::UpdateCommand;
use crate::{
    application::{
        execution_context::ExecutionContext,
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
    // Detect the current execution context
    let context = ExecutionContext::detect_current()
        .map_err(|e| format!("Failed to detect execution context: {}", e))?;

    println!("[INFO] Current context: {}", context.display_name());

    match command {
        UpdateCommand::Project {
            code,
            company,
            name,
            description,
            start_date,
            end_date,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("update", "project") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine company code based on context
            let company_code = match (&context, company) {
                (ExecutionContext::Root, Some(company)) => company,
                (ExecutionContext::Root, None) => {
                    return Err("Company parameter required in root context".into());
                }
                (ExecutionContext::Company(code), None) => code.clone(),
                (ExecutionContext::Company(_), Some(_)) => {
                    return Err("Company parameter not needed in company context".into());
                }
                (ExecutionContext::Project(company, _), None) => company.clone(),
                (ExecutionContext::Project(_, _), Some(_)) => {
                    return Err("Company parameter not needed in project context".into());
                }
            };

                let base_path = match context {
                    ExecutionContext::Root => ".".to_string(),
                    ExecutionContext::Company(_) => "../".to_string(),
                    ExecutionContext::Project(_, _) => ".".to_string(),
                };
            let project_repository = FileProjectRepository::with_base_path(base_path.into());
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
            // Validate command in context
            if let Err(e) = context.validate_command("update", "task") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine project and company codes based on context
            let (project_code, company_code) = match (&context, project, company) {
                (ExecutionContext::Root, Some(project), Some(company)) => (project, company),
                (ExecutionContext::Root, None, None) => {
                    return Err("Project and company parameters required in root context".into());
                }
                (ExecutionContext::Root, None, Some(_)) => {
                    return Err("Project parameter required in root context".into());
                }
                (ExecutionContext::Root, Some(_), None) => {
                    return Err("Company parameter required in root context".into());
                }
                (ExecutionContext::Company(company), Some(project), None) => (project, company.clone()),
                (ExecutionContext::Company(_), None, _) => {
                    return Err("Project parameter required in company context".into());
                }
                (ExecutionContext::Company(_), Some(_), Some(_)) => {
                    return Err("Company parameter not needed in company context".into());
                }
                (ExecutionContext::Project(company, project), None, None) => (project.clone(), company.clone()),
                (ExecutionContext::Project(_, _), Some(_), _) => {
                    return Err("Project parameter not needed in project context".into());
                }
                (ExecutionContext::Project(_, _), None, Some(_)) => {
                    return Err("Company parameter not needed in project context".into());
                }
            };

                let base_path = match context {
                    ExecutionContext::Root => ".".to_string(),
                    ExecutionContext::Company(_) => "../".to_string(),
                    ExecutionContext::Project(_, _) => ".".to_string(),
                };
            let project_repository = FileProjectRepository::with_base_path(base_path.into());
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

            match update_use_case.execute(&project_code, &code, args) {
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
            company,
            name,
            email,
            description,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("update", "resource") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine company code based on context
            let company_code = match (&context, company) {
                (ExecutionContext::Root, Some(company)) => company,
                (ExecutionContext::Root, None) => {
                    return Err("Company parameter required in root context for updating resources".into());
                }
                (ExecutionContext::Company(code), Some(_)) => {
                    return Err("Company parameter not needed in company context".into());
                }
                (ExecutionContext::Company(code), None) => code.clone(),
                (ExecutionContext::Project(company, _), Some(_)) => {
                    return Err("Company parameter not needed in project context".into());
                }
                (ExecutionContext::Project(company, _), None) => company.clone(),
            };

            let base_path = match context {
                ExecutionContext::Root => ".".to_string(),
                ExecutionContext::Company(_) => "../".to_string(),
                ExecutionContext::Project(_, _) => "../".to_string(),
            };
            let resource_repository = FileResourceRepository::new(base_path);
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
