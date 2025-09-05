use crate::{
    application::{
        create::task::{CreateTaskArgs, CreateTaskUseCase},
        task::{
            assign_resource::AssignResourceUseCase,
            delete_task::DeleteTaskUseCase,
            describe_task::DescribeTaskUseCase,
            link_task::LinkTaskUseCase,
            update_task::{UpdateTaskArgs, UpdateTaskUseCase},
        },
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository,
        task_repository::FileTaskRepository,
    },
};
use super::super::commands::TaskCommand;
use chrono::NaiveDate;

pub fn handle_task_command(command: TaskCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        TaskCommand::Create {
            name,
            code,
            project,
            company,
            description,
            start_date,
            due_date,
            assigned_resources,
        } => {
            let task_repository = FileTaskRepository::new(".");
            let project_repository = FileProjectRepository::new();
            let create_use_case = CreateTaskUseCase::new(task_repository, project_repository);

            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let due = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid due date format: {}", e))?;

            let assigned_resources_vec = assigned_resources
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let args = CreateTaskArgs {
                name,
                code,
                project_code: project,
                company_code: company,
                description,
                start_date: start,
                due_date: due,
                assigned_resources: assigned_resources_vec,
            };

            match create_use_case.execute(args) {
                Ok(task) => {
                    println!("✅ Task created successfully!");
                    println!("   Name: {}", task.name());
                    println!("   Code: {}", task.code());
                    println!("   Project: {}", project);
                    println!("   Start: {}", task.start_date());
                    println!("   Due: {}", task.due_date());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Describe { code, project, company } => {
            let task_repository = FileTaskRepository::new(".");
            let describe_use_case = DescribeTaskUseCase::new(task_repository);

            match describe_use_case.execute(code, project, company) {
                Ok(description) => {
                    println!("{}", description);
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
            company,
            name,
            description,
            start_date,
            due_date,
        } => {
            let task_repository = FileTaskRepository::new(".");
            let update_use_case = UpdateTaskUseCase::new(task_repository);

            let start = start_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid start date format: {}", e))?;
            let due = due_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                .transpose()
                .map_err(|e| format!("Invalid due date format: {}", e))?;

            let args = UpdateTaskArgs {
                name,
                description,
                start_date: start,
                due_date: due,
            };

            match update_use_case.execute(code, project, company, args) {
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
        TaskCommand::Delete { code, project, company } => {
            let task_repository = FileTaskRepository::new(".");
            let delete_use_case = DeleteTaskUseCase::new(task_repository);

            match delete_use_case.execute(code, project, company) {
                Ok(_) => {
                    println!("✅ Task deleted successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to delete task: {}", e);
                    Err(e.into())
                }
            }
        }
        TaskCommand::Link { from, to, project, company } => {
            let project_repository = FileProjectRepository::new();
            let link_use_case = LinkTaskUseCase::new(project_repository);

            match link_use_case.execute(from, to, project, company) {
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
        TaskCommand::Unlink { from, to, project, company } => {
            let task_repository = FileTaskRepository::new(".");
            let unlink_use_case = crate::application::task::remove_dependency::RemoveDependencyUseCase::new(task_repository);

            match unlink_use_case.execute(from, to, project, company) {
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
        TaskCommand::AssignResource { task, project, company, resource } => {
            let task_repository = FileTaskRepository::new(".");
            let assign_use_case = AssignResourceUseCase::new(task_repository);

            match assign_use_case.execute(task, project, company, resource) {
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
