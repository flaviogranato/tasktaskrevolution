use super::super::commands::ListCommand;
use crate::{
    application::list::{projects::ListProjectsUseCase, resources::ListResourcesUseCase, tasks::ListTasksUseCase},
    infrastructure::persistence::{
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};

pub fn handle_list_command(command: ListCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ListCommand::Projects { company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let list_use_case = ListProjectsUseCase::new(project_repository);

            match list_use_case.execute() {
                Ok(projects) => {
                    if projects.is_empty() {
                        println!("No projects found.");
                    } else {
                        println!("Projects:");
                        for project in projects {
                            println!(
                                "  - {} ({}) - {}",
                                project.name(),
                                project.code(),
                                project.company_code()
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list projects: {}", e);
                    Err(e.into())
                }
            }
        }
        ListCommand::Tasks { project, company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let list_use_case = ListTasksUseCase::new(project_repository);

            match list_use_case.execute() {
                Ok(tasks) => {
                    if tasks.is_empty() {
                        println!("No tasks found.");
                    } else {
                        println!("Tasks:");
                        for task in tasks {
                            println!("  - {} ({})", task.name(), task.code());
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list tasks: {}", e);
                    Err(e.into())
                }
            }
        }
        ListCommand::Resources => {
            let resource_repository = FileResourceRepository::new(".");
            let list_use_case = ListResourcesUseCase::new(resource_repository);

            match list_use_case.execute() {
                Ok(resources) => {
                    if resources.is_empty() {
                        println!("No resources found.");
                    } else {
                        println!("Resources:");
                        for resource in resources {
                            println!(
                                "  - {} ({}) - {}",
                                resource.name(),
                                resource.code(),
                                resource.email().unwrap_or("N/A")
                            );
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to list resources: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
