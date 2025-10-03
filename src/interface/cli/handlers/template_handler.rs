use super::super::commands::TemplateCommand;
use crate::{
    application::{
        create::{project::CreateProjectUseCase, resource::CreateResourceUseCase, task::CreateTaskUseCase},
        template::{
            create_from_template::CreateFromTemplateUseCase, list_templates::ListTemplatesUseCase,
            load_template::LoadTemplateUseCase,
        },
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository, config_repository::FileConfigRepository,
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
        task_repository::FileTaskRepository,
    },
};
use std::collections::HashMap;

pub fn handle_template_command(command: TemplateCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        TemplateCommand::List => {
            let list_use_case = ListTemplatesUseCase::new();
            let templates_dir = std::path::Path::new("templates/projects");

            match list_use_case.execute(templates_dir) {
                Ok(templates) => {
                    if templates.is_empty() {
                        println!("No templates found.");
                    } else {
                        println!("Available templates:");
                        for template in templates {
                            println!("  - {}: {}", template.name, template.description);
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to list templates: {}", e);
                    Err(e.into())
                }
            }
        }
        TemplateCommand::Show { name } => {
            let load_use_case = LoadTemplateUseCase::new();
            let templates_dir = std::path::Path::new("templates/projects");

            match load_use_case.load_by_name(templates_dir, &name) {
                Ok(template) => {
                    println!("Template: {}", template.metadata.name);
                    println!("Description: {}", template.metadata.description);
                    println!("Version: {}", template.metadata.version);
                    println!("Category: {}", template.metadata.category);
                    println!("Tags: {:?}", template.metadata.tags);
                    println!("Variables:");
                    for (name, variable) in &template.spec.variables {
                        println!("  - {}: {} ({})", name, variable.description, variable.r#type);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to show template: {}", e);
                    Err(e.into())
                }
            }
        }
        TemplateCommand::Create {
            template,
            name,
            code,
            company,
            params,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let resource_repository = FileResourceRepository::new(".");
            let _company_repository = FileCompanyRepository::new(".");
            let _task_repository = FileTaskRepository::new(".");

            let _code_resolver_project = crate::application::shared::code_resolver::CodeResolver::new(".");
            let _code_resolver_resource = crate::application::shared::code_resolver::CodeResolver::new(".");
            let code_resolver_task = crate::application::shared::code_resolver::CodeResolver::new(".");
            let create_project_use_case = CreateProjectUseCase::new(project_repository);
            let config_repository = FileConfigRepository::new();
            let create_resource_use_case = CreateResourceUseCase::new(resource_repository, config_repository);
            let create_task_use_case = CreateTaskUseCase::new(
                FileProjectRepository::with_base_path(".".into()),
                FileTaskRepository::new("."),
                FileResourceRepository::new("."),
                code_resolver_task,
            );

            let load_use_case = LoadTemplateUseCase::new();
            let create_use_case =
                CreateFromTemplateUseCase::new(create_project_use_case, create_resource_use_case, create_task_use_case);

            let templates_dir = std::path::Path::new("templates/projects");
            let template_data = load_use_case.load_by_name(templates_dir, &template)?;

            let mut template_params = HashMap::new();

            // Add name and code as template parameters
            template_params.insert("project_name".to_string(), name);
            template_params.insert("project_code".to_string(), code);

            for param in params {
                // Split by comma first, then by equals
                for kv_pair in param.split(',') {
                    if let Some((key, value)) = kv_pair.split_once('=') {
                        template_params.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }

            match create_use_case.execute(&template_data, &template_params, company) {
                Ok(_project) => {
                    println!("Project created from template successfully!");
                    println!("   Project created from template successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Failed to create project from template: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
