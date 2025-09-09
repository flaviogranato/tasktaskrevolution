use crate::interface::cli::commands::CreateCommand;
use crate::interface::cli::handlers::get_app_handler;
use crate::application::{
    execution_context::ExecutionContext,
    company_management::{CreateCompanyUseCase, CreateCompanyArgs},
    create::project::CreateProjectUseCase,
    create::resource::CreateResourceUseCase,
    create::task::{CreateTaskUseCase, CreateTaskArgs},
    errors::AppError,
};
use crate::infrastructure::persistence::project_repository::FileProjectRepository;
use chrono::NaiveDate;

pub fn handle_create_command(command: CreateCommand) -> Result<(), Box<dyn std::error::Error>> {
    // Detect the current execution context
    let context = ExecutionContext::detect_current()
        .map_err(|e| format!("Failed to detect execution context: {}", e))?;

    println!("[INFO] Current context: {}", context.display_name());

    match command {
        CreateCommand::Company {
            name,
            code,
            description,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("create", "company") {
                return Err(format!("Command not valid in current context: {}", e).into());
            }

            // Companies can only be created from root context
            match &context {
                ExecutionContext::Root => {
                    let app = get_app_handler().get_app();
                    let company_repo = &app.company_repository;
                    
                    let args = CreateCompanyArgs {
                        code,
                        name: name.clone(),
                        description,
                        tax_id: None,
                        address: None,
                        email: None,
                        phone: None,
                        website: None,
                        industry: None,
                        created_by: "CLI".to_string(),
                    };
                    
                    let use_case = CreateCompanyUseCase::new(company_repo.clone());
                    match use_case.execute(args) {
                        Ok(company) => {
                            println!("Company '{}' created successfully with code '{}'", company.name(), company.code());
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to create company: {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                _ => {
                    Err("Companies can only be created from root context".into())
                }
            }
        }
        CreateCommand::Project {
            name,
            code,
            company,
            description,
            start_date: _,
            end_date: _,
            template: _,
            template_vars: _,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("create", "project") {
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

            let app = get_app_handler().get_app();
            let project_repo = &app.project_repository;
            
            let use_case = CreateProjectUseCase::new(project_repo.clone());
            match use_case.execute(&name, description.as_deref(), company_code.clone(), code) {
                Ok(project) => {
                    println!("Project '{}' created successfully with code '{}' in company '{}'", 
                        project.name(), project.code(), company_code);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create project: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        CreateCommand::Task {
            name,
            code,
            project,
            company,
            description: _,
            start_date,
            due_date,
            assigned_resources,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("create", "task") {
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

            let base_path = context.asset_path_prefix();
            let project_repo = FileProjectRepository::with_base_path(base_path.into());
            
            // Parse dates
            let start_date_parsed = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| AppError::ValidationError {
                    field: "start_date".to_string(),
                    message: format!("Invalid start date format: {}", e),
                })?;
            
            let due_date_parsed = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
                .map_err(|e| AppError::ValidationError {
                    field: "due_date".to_string(),
                    message: format!("Invalid due date format: {}", e),
                })?;
            
            // Parse assigned resources
            let assigned_resources_vec = if let Some(resources) = assigned_resources {
                resources.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                Vec::new()
            };
            
            let args = CreateTaskArgs {
                company_code: company_code.clone(),
                project_code: project_code.clone(),
                name: name.clone(),
                code,
                start_date: start_date_parsed,
                due_date: due_date_parsed,
                assigned_resources: assigned_resources_vec,
            };
            
            let use_case = CreateTaskUseCase::new(project_repo.clone());
            match use_case.execute(args) {
                Ok(_) => {
                    println!("Task '{}' created successfully in project '{}'", name, project_code);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create task: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        CreateCommand::Resource {
            name,
            code,
            email: _,
            company,
            description: _,
            start_date: _,
            end_date: _,
        } => {
            // Validate command in context
            if let Err(e) = context.validate_command("create", "resource") {
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

            let app = get_app_handler().get_app();
            let resource_repo = &app.resource_repository;
            
            let use_case = CreateResourceUseCase::new(resource_repo.clone());
            match use_case.execute(&name, "employee", company_code.clone(), None, code) {
                Ok(_) => {
                    println!("Resource '{}' created successfully in company '{}'", name, company_code);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create resource: {}", e);
                    Err(Box::new(e))
                }
            }
        }
    }
}
