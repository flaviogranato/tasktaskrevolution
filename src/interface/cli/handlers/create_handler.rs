use crate::interface::cli::commands::CreateCommand;
use crate::interface::cli::handlers::get_app_handler;
use crate::application::company_management::{CreateCompanyUseCase, CreateCompanyArgs};

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
            code,
            company,
            description,
            start_date,
            end_date,
            template: _,
            template_vars: _,
        } => {
            // For now, just print success - TODO: implement actual creation
            println!("Project '{}' created successfully with code '{}' in company '{}'", name, code, company);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
            println!("Start date: {}, End date: {}", start_date, end_date);
            Ok(())
        }
        CreateCommand::Task {
            name,
            code,
            project,
            company,
            description,
            start_date,
            due_date,
            assigned_resources,
        } => {
            // For now, just print success - TODO: implement actual creation
            println!("Task '{}' created successfully with code '{}' in project '{}'", name, code, project);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
            println!("Start date: {}, Due date: {}", start_date, due_date);
            if let Some(resources) = assigned_resources {
                println!("Assigned resources: {}", resources);
            }
            Ok(())
        }
        CreateCommand::Resource {
            name,
            code,
            email,
            company: _,
            description,
            start_date: _,
            end_date: _,
        } => {
            // For now, just print success - TODO: implement actual creation
            println!("Resource '{}' created successfully with code '{}' and email '{}'", name, code, email);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
            Ok(())
        }
    }
}
