use crate::{
    application::{
        project::cancel_project::CancelProjectUseCase,
        task::delete_task::DeleteTaskUseCase,
        resource::deactivate_resource::DeactivateResourceUseCase,
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository,
        task_repository::FileTaskRepository,
        resource_repository::FileResourceRepository,
    },
};
use super::super::commands::DeleteCommand;

pub fn handle_delete_command(command: DeleteCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        DeleteCommand::Project { code, company } => {
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
        DeleteCommand::Task { code, project, company } => {
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
        DeleteCommand::Resource { code } => {
            let resource_repository = FileResourceRepository::new(".");
            let deactivate_use_case = DeactivateResourceUseCase::new(resource_repository);

            match deactivate_use_case.execute(code) {
                Ok(_) => {
                    println!("✅ Resource deactivated successfully!");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to deactivate resource: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
