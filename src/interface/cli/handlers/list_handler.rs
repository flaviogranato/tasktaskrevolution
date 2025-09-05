use crate::{
    application::list::{
        projects::ListProjectsUseCase,
        resources::ListResourcesUseCase,
        tasks::ListTasksUseCase,
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
        task_repository::FileTaskRepository,
    },
};
use super::super::commands::ListCommand;

pub fn handle_list_command(command: ListCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ListCommand::Projects { company } => {
            let project_repository = FileProjectRepository::new();
            let list_use_case = ListProjectsUseCase::new(project_repository);

            match list_use_case.execute(company) {
                Ok(projects) => {
                    if projects.is_empty() {
                        println!("No projects found.");
                    } else {
                        println!("Projects:");
                        for project in projects {
                            println!("  - {} ({}) - {}", project.name(), project.code(), project.company_code());
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
            let task_repository = FileTaskRepository::new(".");
            let list_use_case = ListTasksUseCase::new(task_repository);

            match list_use_case.execute(project, company) {
                Ok(tasks) => {
                    if tasks.is_empty() {
                        println!("No tasks found.");
                    } else {
                        println!("Tasks:");
                        for task in tasks {
                            println!("  - {} ({}) - {}", task.name(), task.code(), task.state());
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
                            println!("  - {} ({}) - {}", resource.name(), resource.code(), resource.email());
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
