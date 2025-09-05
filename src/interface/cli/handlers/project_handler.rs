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
            let project_repository = FileProjectRepository::new();
            let company_repository = FileCompanyRepository::new();
            let create_use_case = CreateProjectUseCase::new(project_repository, company_repository);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid end date format: {}", e))?;

            match create_use_case.execute(name, code, company, description, start, end) {
                Ok(project) => {
                    println!("✅ Project created successfully!");
                    println!("   Name: {}", project.name());
                    println!("   Code: {}", project.code());
                    println!("   Company: {}", project.company_code());
                    println!("   Start: {}", project.start_date());
                    println!("   End: {}", project.end_date());
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
            let project_repository = FileProjectRepository::new();
            let company_repository = FileCompanyRepository::new();
            let load_use_case = LoadTemplateUseCase::new();
            let create_use_case = CreateFromTemplateUseCase::new(project_repository, company_repository);

            let templates_dir = std::path::Path::new("templates/projects");
            let template_data = load_use_case.load_by_name(templates_dir, &template)?;

            let mut template_params = HashMap::new();
            for param in params {
                if let Some((key, value)) = param.split_once('=') {
                    template_params.insert(key.to_string(), value.to_string());
                }
            }

            match create_use_case.execute(template_data, name, code, company, template_params) {
                Ok(project) => {
                    println!("✅ Project created from template successfully!");
                    println!("   Name: {}", project.name());
                    println!("   Code: {}", project.code());
                    println!("   Company: {}", project.company_code());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create project from template: {}", e);
                    Err(e.into())
                }
            }
        }
        ProjectCommand::Describe { code, company } => {
            let project_repository = FileProjectRepository::new();
            let describe_use_case = DescribeProjectUseCase::new(project_repository);

            match describe_use_case.execute(code, company) {
                Ok(description) => {
                    println!("{}", description);
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
            let project_repository = FileProjectRepository::new();
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
                start_date: start,
                end_date: end,
            };

            match update_use_case.execute(code, company, args) {
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
            let project_repository = FileProjectRepository::new();
            let cancel_use_case = CancelProjectUseCase::new(project_repository);

            match cancel_use_case.execute(code, company) {
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
            let project_repository = FileProjectRepository::new();
            let assign_use_case = AssignResourceToTaskUseCase::new(project_repository);

            match assign_use_case.execute(project, company, task, resource) {
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
