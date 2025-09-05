use crate::{
    application::{
        create::project::CreateProjectUseCase,
        project::{
            assign_resource_to_task::AssignResourceToTaskUseCase,
            cancel_project::CancelProjectUseCase,
            describe_project::DescribeProjectUseCase,
            update_project::{UpdateProjectArgs, UpdateProjectUseCase},
        },
        template::{
            create_from_template::CreateFromTemplateUseCase,
            load_template::LoadTemplateUseCase,
        },
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository,
        project_repository::FileProjectRepository,
    },
};
use super::super::commands::ProjectCommand;
use chrono::NaiveDate;
use std::collections::HashMap;

pub fn handle_project_command(command: ProjectCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ProjectCommand::Create {
            name,
            code,
            company,
            description,
            start_date,
            end_date,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let create_use_case = CreateProjectUseCase::new(project_repository);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            match create_use_case.execute(&name, &code, &company, description.as_deref(), start, end) {
                Ok(_) => {
                    println!("✅ Project created successfully!");
                    println!("   Name: {}", name);
                    println!("   Code: {}", code);
                    println!("   Company: {}", company);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create project: {}", e);
                    Err(e.into())
                }
            }
        }
        ProjectCommand::FromTemplate {
            template,
            name,
            code,
            company,
            params,
        } => {
            // Este comando foi movido para template_handler.rs
            return Err("FromTemplate command should be handled by template_handler".into());
        }
        ProjectCommand::Describe { code, company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let describe_use_case = DescribeProjectUseCase::new(project_repository);

            match describe_use_case.execute(&code) {
                Ok(description) => {
                    println!("{:?}", description);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to describe project: {}", e);
                    Err(e.into())
                }
            }
        }
        ProjectCommand::Update {
            code,
            company,
            name,
            description,
            start_date,
            end_date,
        } => {
            let project_repository = FileProjectRepository::with_base_path("."into());
            let update_use_case = UpdateProjectUseCase::new(project_repository);

            let start = start_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let end = end_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            let args = UpdateProjectArgs {
                name,
                description,
            };

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
        ProjectCommand::Cancel { code, company } => {
            let project_repository = FileProjectRepository::with_base_path("."into());
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
        ProjectCommand::AssignResource {
            project,
            company,
            task,
            resource,
        } => {
            let project_repository = FileProjectRepository::with_base_path("."into());
            let resource_repository = crate::infrastructure::persistence::resource_repository::FileResourceRepository::new(".");
            let assign_use_case = AssignResourceToTaskUseCase::new(project_repository, resource_repository);

            match assign_use_case.execute(&project, &task, &resource) {
                Ok(_) => {
                    println!("✅ Resource assigned to task successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to assign resource to task: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
