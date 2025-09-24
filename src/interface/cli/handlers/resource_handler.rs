use super::super::commands::ResourceCommand;
use crate::{
    application::{
        create::{
            resource::{CreateResourceParams, CreateResourceUseCase},
            time_off::CreateTimeOffUseCase,
            vacation::CreateVacationUseCase,
        },
        resource::{
            deactivate_resource::DeactivateResourceUseCase,
            describe_resource::DescribeResourceUseCase,
            update_resource::{UpdateResourceArgs, UpdateResourceUseCase},
        },
    },
    infrastructure::persistence::resource_repository::FileResourceRepository,
};
use chrono::NaiveDate;

pub fn handle_resource_command(command: ResourceCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ResourceCommand::Create {
            name,
            r#type: resource_type,
            code: _,
            email,
            description: _,
        } => {
            let resource_repository = FileResourceRepository::new(".");
            let create_use_case = CreateResourceUseCase::new(resource_repository);

            let params = CreateResourceParams {
                name: name.clone(),
                resource_type: resource_type.clone(),
                company_code: "COMPANY001".to_string(),
                project_code: None,
                code: None,
                email: Some(email),
                start_date: None,
                end_date: None,
            };
            match create_use_case.execute(params) {
                Ok(_resource) => {
                    println!("Resource created successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to create resource: {}", e);
                    Err(e.into())
                }
            }
        }
        ResourceCommand::TimeOff {
            resource,
            start_date,
            end_date,
            hours,
            description,
        } => {
            let resource_repository = FileResourceRepository::new(".");
            let create_use_case = CreateTimeOffUseCase::new(resource_repository);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let _end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            match create_use_case.execute(
                &resource,
                hours,
                &start.format("%Y-%m-%d").to_string(),
                description.as_deref(),
            ) {
                Ok(_) => {
                    println!("Time off created successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to create time off: {}", e);
                    Err(e)
                }
            }
        }
        ResourceCommand::Vacation {
            resource,
            start_date,
            end_date,
            description: _,
            with_compensation,
        } => {
            let resource_repository = FileResourceRepository::new(".");
            let create_use_case = CreateVacationUseCase::new(resource_repository);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            match create_use_case.execute(
                &resource,
                &start.format("%Y-%m-%d").to_string(),
                &end.format("%Y-%m-%d").to_string(),
                with_compensation,
                None,
            ) {
                Ok(_) => {
                    println!("Vacation created successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to create vacation: {}", e);
                    Err(e)
                }
            }
        }
        ResourceCommand::Describe { code } => {
            let resource_repository = FileResourceRepository::new(".");
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let describe_use_case = DescribeResourceUseCase::new(resource_repository, code_resolver);

            match describe_use_case.execute(&code) {
                Ok(description) => {
                    println!("{:?}", description);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to describe resource: {}", e);
                    Err(e.into())
                }
            }
        }
        ResourceCommand::Update {
            code,
            name,
            r#type: resource_type,
            email,
            description: _,
        } => {
            let resource_repository = FileResourceRepository::new(".");
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let update_use_case = UpdateResourceUseCase::new(resource_repository, code_resolver);

            let args = UpdateResourceArgs {
                name,
                email,
                resource_type,
            };

            match update_use_case.execute(&code, "DEFAULT", args) {
                Ok(_) => {
                    println!("Resource updated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to update resource: {}", e);
                    Err(e.into())
                }
            }
        }
        ResourceCommand::Deactivate { code } => {
            let resource_repository = FileResourceRepository::new(".");
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let deactivate_use_case = DeactivateResourceUseCase::new(resource_repository, code_resolver);

            match deactivate_use_case.execute(&code, "DEFAULT") {
                Ok(_) => {
                    println!("Resource deactivated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to deactivate resource: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
