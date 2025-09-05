use crate::{
    application::template::{
        create_from_template::CreateFromTemplateUseCase,
        list_templates::ListTemplatesUseCase,
        load_template::LoadTemplateUseCase,
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository,
        company_repository::FileCompanyRepository,
    },
};
use super::super::commands::TemplateCommand;
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
                    eprintln!("❌ Failed to list templates: {}", e);
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
                    println!("Parameters:");
                    for param in template.metadata.parameters {
                        println!("  - {}: {}", param.name, param.description);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to show template: {}", e);
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
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let company_repository = FileCompanyRepository::new(".");
            let task_repository = FileTaskRepository::new(".");
            
            let create_project_use_case = CreateProjectUseCase::new(project_repository, company_repository);
            let create_resource_use_case = CreateResourceUseCase::new(resource_repository);
            let create_task_use_case = CreateTaskUseCase::new(task_repository, FileProjectRepository::new());
            
            let load_use_case = LoadTemplateUseCase::new();
            let create_use_case = CreateFromTemplateUseCase::new(create_project_use_case, create_resource_use_case, create_task_use_case);

            let templates_dir = std::path::Path::new("templates/projects");
            let template_data = load_use_case.load_by_name(templates_dir, &template)?;

            let mut template_params = HashMap::new();
            for param in params {
                if let Some((key, value)) = param.split_once('=') {
                    template_params.insert(key.to_string(), value.to_string());
                }
            }

            match create_use_case.execute(&template_data, &template_params, &code) {
                Ok(project) => {
                    println!("✅ Project created from template successfully!");
                    println!("   Name: {}", project.name);
                    println!("   Code: {}", project.code);
                    println!("   Company: {}", project.company_code);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create project from template: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
