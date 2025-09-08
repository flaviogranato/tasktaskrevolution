use super::super::commands::DeleteCommand;
use crate::{
    application::{
        execution_context::ExecutionContext,
        project::cancel_project::CancelProjectUseCase, 
        resource::deactivate_resource::DeactivateResourceUseCase,
        task::delete_task::DeleteTaskUseCase,
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};

pub fn handle_delete_command(command: DeleteCommand) -> Result<(), Box<dyn std::error::Error>> {
    // Detect the current execution context
    let context = ExecutionContext::detect_current()
        .map_err(|e| format!("Failed to detect execution context: {}", e))?;

    println!("[INFO] Current context: {}", context.display_name());

    match command {
        DeleteCommand::Project { code, company } => {
            // Validate command in context
            if let Err(e) = context.validate_command("delete", "project") {
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

            let project_repository = FileProjectRepository::new();
            let cancel_use_case = CancelProjectUseCase::new(project_repository);

            match cancel_use_case.execute(&code) {
                Ok(_) => {
                    println!("✅ Project cancelled successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to cancel project: {}", e);
                    Err(e.into())
                }
            }
        }
        DeleteCommand::Task { code, project, company } => {
            // Validate command in context
            if let Err(e) = context.validate_command("delete", "task") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine project and company codes based on context
            let (project_code, company_code) = match (&context, project, company) {
                (ExecutionContext::Root, Some(project), Some(company)) => (project, company),
                (ExecutionContext::Root, None, _) => {
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

            let project_repository = FileProjectRepository::with_base_path(".".into());
            let delete_use_case = DeleteTaskUseCase::new(project_repository);

            match delete_use_case.execute(&code, &project_code) {
                Ok(_) => {
                    println!("✅ Task deleted successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to delete task: {}", e);
                    Err(e.into())
                }
            }
        }
        DeleteCommand::Resource { code } => {
            // Validate command in context
            if let Err(e) = context.validate_command("delete", "resource") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Resources can be deleted from company or project context
            match &context {
                ExecutionContext::Root => {
                    return Err("Resource deletion not allowed in root context".into());
                }
                ExecutionContext::Company(_) | ExecutionContext::Project(_, _) => {
                    // Resource deletion is allowed in company or project context
                }
            }

            let resource_repository = FileResourceRepository::new(".");
            let deactivate_use_case = DeactivateResourceUseCase::new(resource_repository);

            match deactivate_use_case.execute(&code) {
                Ok(_) => {
                    println!("✅ Resource deactivated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to deactivate resource: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
