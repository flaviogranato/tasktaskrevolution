use super::super::commands::ValidateCommand;
use crate::{
    application::validate::{
        business_rules::ValidateBusinessRulesUseCase, data_integrity::ValidateDataIntegrityUseCase,
        entities::ValidateEntitiesUseCase, system::ValidateSystemUseCase,
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
    },
};

pub fn handle_validate_command(command: ValidateCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ValidateCommand::BusinessRules => {
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let company_repository = FileCompanyRepository::new(".");
            let validate_use_case =
                ValidateBusinessRulesUseCase::new(&project_repository, &resource_repository, &company_repository);

            match validate_use_case.execute() {
                Ok(_) => {
                    println!("✅ Business rules validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Business rules validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        ValidateCommand::DataIntegrity => {
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let company_repository = FileCompanyRepository::new(".");
            let validate_use_case =
                ValidateDataIntegrityUseCase::new(&project_repository, &resource_repository, &company_repository);

            match validate_use_case.execute() {
                Ok(_) => {
                    println!("✅ Data integrity validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Data integrity validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        ValidateCommand::Entities => {
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let company_repository = FileCompanyRepository::new(".");
            let validate_use_case =
                ValidateEntitiesUseCase::new(&project_repository, &resource_repository, &company_repository);

            match validate_use_case.execute() {
                Ok(_) => {
                    println!("✅ Entities validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Entities validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        ValidateCommand::System => {
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let company_repository = FileCompanyRepository::new(".");
            let validate_use_case =
                ValidateSystemUseCase::new(project_repository, resource_repository, company_repository);

            match validate_use_case.execute() {
                Ok(_) => {
                    println!("✅ System validation passed!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ System validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
