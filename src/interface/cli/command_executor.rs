use crate::interface::cli::commands;
use std::path::PathBuf;

pub fn execute_init(
    name: String,
    email: String,
    company_name: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::init::{InitManagerData, InitManagerUseCase};
    use crate::interface::cli::handlers::get_app_handler;

    let app = get_app_handler().get_app();
    let config_repo = &app.config_repository;

    let init_data = InitManagerData {
        name: name.clone(),
        email: email.clone(),
        timezone,
        work_hours_start,
        work_hours_end,
        work_days,
        company_name: company_name.clone(),
    };

    let init_use_case = InitManagerUseCase::new(Box::new(config_repo.clone()));

    match init_use_case.execute(init_data) {
        Ok(_config) => {
            println!("Manager/Consultant configured successfully");
            println!("Name: {}", name);
            println!("Email: {}", email);
            println!("Company: {}", company_name);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to initialize system: {}", e);
            Err(Box::new(e))
        }
    }
}

pub fn execute_build(output: PathBuf, _base_url: String) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::build_use_case::BuildUseCase;

    let current_dir = std::env::current_dir()?;
    let build_use_case = BuildUseCase::new(current_dir, output.to_str().unwrap_or("dist"))?;

    match build_use_case.execute() {
        Ok(_) => {
            println!("Static site built successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to build static site: {}", e);
            Err(e)
        }
    }
}

pub fn execute_validate(command: commands::ValidateCommand) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::validate::{
        business_rules::ValidateBusinessRulesUseCase, data_integrity::ValidateDataIntegrityUseCase,
        entities::ValidateEntitiesUseCase, system::ValidateSystemUseCase,
    };
    use crate::infrastructure::persistence::{
        company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
    };

    let project_repository = FileProjectRepository::new();
    let resource_repository = FileResourceRepository::new(".");
    let company_repository = FileCompanyRepository::new(".");

    match command {
        commands::ValidateCommand::BusinessRules => {
            let validate_use_case =
                ValidateBusinessRulesUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(_) => {
                    println!("Business rules validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Business rules validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::DataIntegrity => {
            let validate_use_case =
                ValidateDataIntegrityUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(_) => {
                    println!("Data integrity validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Data integrity validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::Entities => {
            let validate_use_case =
                ValidateEntitiesUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(_) => {
                    println!("Entities validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Entities validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::System => {
            let validate_use_case =
                ValidateSystemUseCase::new(project_repository, resource_repository, company_repository);
            match validate_use_case.execute() {
                Ok(_) => {
                    println!("System validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("System validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
