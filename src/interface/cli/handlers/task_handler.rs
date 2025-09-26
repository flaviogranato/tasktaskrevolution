use super::super::commands::TaskCommand;
use crate::{
    application::{
        create::task::{CreateTaskArgs, CreateTaskUseCase},
        task::{
            assign_resource::AssignResourceToTaskUseCase,
            delete_task::DeleteTaskUseCase,
            describe_task::DescribeTaskUseCase,
            link_task::LinkTaskUseCase,
            update_task::{UpdateTaskArgs, UpdateTaskUseCase},
        },
    },
    infrastructure::persistence::{project_repository::FileProjectRepository, task_repository::FileTaskRepository},
};
use chrono::NaiveDate;

pub fn handle_task_command(command: TaskCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        TaskCommand::Create {
            name,
            code: _,
            project,
            company,
            description: _,
            start_date,
            due_date,
            assigned_resources,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let task_repository = FileTaskRepository::new(".");
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let create_use_case = CreateTaskUseCase::new(project_repository, task_repository, code_resolver);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let due = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid due date format: {}", e))?;

            let assigned_resources_vec = assigned_resources
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let args = CreateTaskArgs {
                name: name.clone(),
                project_code: project.clone(),
                company_code: company,
                code: None, // Auto-generate code
                start_date: start,
                due_date: due,
                assigned_resources: assigned_resources_vec,
            };

            match create_use_case.execute(args) {
                Ok(_) => {
                    println!("✅ Task created successfully!");
                    println!("   Name: {}", name);
                    println!("   Project: {}", project);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Describe {
            code,
            project,
            company: _,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let describe_use_case = DescribeTaskUseCase::new(project_repository, code_resolver);

            match describe_use_case.execute(&project, &code) {
                Ok(description) => {
                    println!("{:?}", description);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to describe task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Update {
            code,
            project,
            company: _,
            name,
            description,
            start_date,
            due_date,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let task_repository = FileTaskRepository::new(".");
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let update_use_case = UpdateTaskUseCase::new(project_repository, task_repository, code_resolver);

            let start = start_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let due = due_date
                .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid due date format: {}", e))?;

            let args = UpdateTaskArgs {
                name,
                description,
                start_date: start,
                due_date: due,
            };

            match update_use_case.execute(&code, &project, args) {
                Ok(_) => {
                    println!("✅ Task updated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to update task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Delete {
            code,
            project,
            company: _,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let delete_use_case = DeleteTaskUseCase::new(project_repository, code_resolver);

            match delete_use_case.execute(&code, &project) {
                Ok(_) => {
                    println!("✅ Task cancelled successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to cancel task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Link {
            from,
            to,
            project,
            company: _,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let link_use_case = LinkTaskUseCase::new(project_repository, code_resolver);

            match link_use_case.execute(&project, &from, &to) {
                Ok(_) => {
                    println!("✅ Tasks linked successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to link tasks: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Unlink {
            from,
            to,
            project,
            company: _,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let unlink_use_case =
                crate::application::task::remove_dependency::RemoveTaskDependencyUseCase::new(project_repository, code_resolver);

            match unlink_use_case.execute(&project, &from, &to) {
                Ok(_) => {
                    println!("✅ Task link removed successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to remove task link: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::AssignResource {
            task,
            project,
            company: _,
            resource,
        } => {
            let task_repository = FileTaskRepository::new(".");
            let resource_repository =
                crate::infrastructure::persistence::resource_repository::FileResourceRepository::new(".");
            let assign_use_case = AssignResourceToTaskUseCase::new(task_repository, resource_repository);

            match assign_use_case.execute(&task, &resource, &project, None) {
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
