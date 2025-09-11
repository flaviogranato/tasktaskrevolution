use super::super::commands::ListCommand;
use crate::{
    application::{
        execution_context::ExecutionContext,
        list::{
            companies::ListCompaniesUseCase, projects::ListProjectsUseCase, resources::ListResourcesUseCase,
            tasks::ListTasksUseCase,
        },
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
    },
};

pub fn handle_list_command(command: ListCommand) -> Result<(), Box<dyn std::error::Error>> {
    // Detect the current execution context
    let context =
        ExecutionContext::detect_current().map_err(|e| format!("Failed to detect execution context: {}", e))?;

    println!("[INFO] Current context: {}", context.display_name());

    match command {
        ListCommand::Companies => {
            // Validate command in context
            if let Err(e) = context.validate_command("list", "companies") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Companies can only be listed from root context
            match &context {
                ExecutionContext::Root => {
                    let company_repository = FileCompanyRepository::new(".");
                    let list_use_case = ListCompaniesUseCase::new(company_repository);

                    match list_use_case.execute() {
                        Ok(companies) => {
                            if companies.is_empty() {
                                println!("No companies found.");
                            } else {
                                println!("Companies:");
                                for company in companies {
                                    println!(
                                        "  - {} ({}) - {}",
                                        company.name(),
                                        company.code(),
                                        company.description.as_deref().unwrap_or("No description")
                                    );
                                }
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to list companies: {}", e);
                            Err(e.into())
                        }
                    }
                }
                _ => Err("Companies can only be listed from root context".into()),
            }
        }
        ListCommand::Projects { company } => {
            // Validate command in context
            if let Err(e) = context.validate_command("list", "projects") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine company code based on context
            let company_code = match (&context, company) {
                (ExecutionContext::Root, Some(company)) => company,
                (ExecutionContext::Company(code), None) => code.clone(),
                (ExecutionContext::Company(code), Some(_)) => {
                    return Err("Company parameter not needed in company context".into());
                }
                (ExecutionContext::Project(company, _), None) => company.clone(),
                (ExecutionContext::Project(company, _), Some(_)) => {
                    return Err("Company parameter not needed in project context".into());
                }
                (ExecutionContext::Root, None) => {
                    return Err("Company parameter required in root context".into());
                }
            };

            let project_repository = FileProjectRepository::with_base_path(".".into());
            let list_use_case = ListProjectsUseCase::new(project_repository);

            match list_use_case.execute() {
                Ok(projects) => {
                    // Filter projects by company if specified
                    let filtered_projects: Vec<_> = projects
                        .into_iter()
                        .filter(|p| p.company_code() == company_code)
                        .collect();

                    if filtered_projects.is_empty() {
                        println!("No projects found for company '{}'.", company_code);
                    } else {
                        println!("Projects for company '{}':", company_code);
                        for project in filtered_projects {
                            println!(
                                "  - {} ({}) - {}",
                                project.name(),
                                project.code(),
                                project.company_code()
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list projects: {}", e);
                    Err(e.into())
                }
            }
        }
        ListCommand::Tasks { project, company } => {
            // Validate command in context
            if let Err(e) = context.validate_command("list", "tasks") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine project code and company code based on context
            let (project_code, company_code) = match (&context, project, company) {
                (ExecutionContext::Root, Some(project), Some(company)) => (project, company),
                (ExecutionContext::Root, Some(_), None) => {
                    return Err("Company parameter required in root context".into());
                }
                (ExecutionContext::Root, None, _) => {
                    return Err("Project parameter required in root context".into());
                }
                (ExecutionContext::Company(company), Some(project), None) => (project, company.clone()),
                (ExecutionContext::Company(_), Some(_), Some(_)) => {
                    return Err("Company parameter not needed in company context".into());
                }
                (ExecutionContext::Company(_), None, _) => {
                    return Err("Project parameter required in company context".into());
                }
                (ExecutionContext::Project(company, project), None, None) => (project.clone(), company.clone()),
                (ExecutionContext::Project(_, project), Some(_), _) => {
                    return Err("Project parameter not needed in project context".into());
                }
                (ExecutionContext::Project(company, _), None, Some(_)) => {
                    return Err("Company parameter not needed in project context".into());
                }
            };

            // For now, we'll use a simple approach since ListTasksUseCase doesn't support filtering
            // This will be improved in future iterations
            let base_path = match context {
                ExecutionContext::Root => ".".to_string(),
                ExecutionContext::Company(_) => "../".to_string(),
                ExecutionContext::Project(_, _) => "../../".to_string(),
            };
            let project_repository = FileProjectRepository::with_base_path(base_path.into());
            let list_use_case = ListTasksUseCase::new(project_repository);

            match list_use_case.execute(&project_code, &company_code) {
                Ok(tasks) => {
                    if tasks.is_empty() {
                        println!("No tasks found for project '{}'.", project_code);
                    } else {
                        println!("Tasks for project '{}':", project_code);
                        for task in tasks {
                            println!("  - {} ({})", task.name(), task.code());
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list tasks: {}", e);
                    Err(e.into())
                }
            }
        }
        ListCommand::Resources { company } => {
            // Validate command in context
            if let Err(e) = context.validate_command("list", "resources") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Determine company code based on context
            let company_code = match (&context, company) {
                (ExecutionContext::Root, Some(company)) => company,
                (ExecutionContext::Root, None) => {
                    return Err("Company parameter required in root context for listing resources".into());
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

            let resource_repository = FileResourceRepository::new(".");
            let list_use_case = ListResourcesUseCase::new(resource_repository);

            match list_use_case.execute() {
                Ok(resources) => {
                    if resources.is_empty() {
                        println!("No resources found for company '{}'.", company_code);
                    } else {
                        println!("Resources for company '{}':", company_code);
                        for resource in resources {
                            println!(
                                "  - {} ({}) - {}",
                                resource.name(),
                                resource.code(),
                                resource.email().unwrap_or("N/A")
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list resources: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
