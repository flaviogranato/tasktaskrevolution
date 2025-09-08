use crate::interface::cli::commands::CreateCommand;
use crate::interface::cli::handlers::get_app_handler;
use crate::application::company_management::{CreateCompanyUseCase, CreateCompanyArgs};
use crate::application::create::project::CreateProjectUseCase;
use crate::application::create::resource::CreateResourceUseCase;
use crate::application::create::task::{CreateTaskUseCase, CreateTaskArgs};
use crate::application::errors::AppError;
use chrono::NaiveDate;

pub fn handle_create_command(command: CreateCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CreateCommand::Company {
            name,
            code,
            description,
        } => {
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
        CreateCommand::Project {
            name,
            code: _,
            company,
            description,
            start_date: _,
            end_date: _,
            template: _,
            template_vars: _,
        } => {
            let app = get_app_handler().get_app();
            let project_repo = &app.project_repository;
            
            let use_case = CreateProjectUseCase::new(project_repo.clone());
            match use_case.execute(&name, description.as_deref(), company.clone()) {
                Ok(project) => {
                    println!("Project '{}' created successfully with code '{}' in company '{}'", 
                        project.name(), project.code(), company);
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
            code: _,
            project,
            company,
            description: _,
            start_date,
            due_date,
            assigned_resources,
        } => {
            let app = get_app_handler().get_app();
            let project_repo = &app.project_repository;
            
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
                company_code: company,
                project_code: project.clone(),
                name: name.clone(),
                start_date: start_date_parsed,
                due_date: due_date_parsed,
                assigned_resources: assigned_resources_vec,
            };
            
            let use_case = CreateTaskUseCase::new(project_repo.clone());
            match use_case.execute(args) {
                Ok(_) => {
                    println!("Task '{}' created successfully in project '{}'", name, project);
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
            code: _,
            email: _,
            company,
            description: _,
            start_date: _,
            end_date: _,
        } => {
            let app = get_app_handler().get_app();
            let resource_repo = &app.resource_repository;
            
            let use_case = CreateResourceUseCase::new(resource_repo.clone());
            match use_case.execute(&name, "employee", company.clone(), None) {
                Ok(_) => {
                    println!("Resource '{}' created successfully in company '{}'", name, company);
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
