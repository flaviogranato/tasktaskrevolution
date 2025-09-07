use super::super::commands::DeleteCommand;
use crate::{
    application::{
        project::cancel_project::CancelProjectUseCase, resource::deactivate_resource::DeactivateResourceUseCase,
        task::delete_task::DeleteTaskUseCase,
    },
    infrastructure::persistence::{
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};

pub fn handle_delete_command(command: DeleteCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        DeleteCommand::Project { code, company } => {
            let project_repository = FileProjectRepository::new();
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
        DeleteCommand::Task { code, project, company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let delete_use_case = DeleteTaskUseCase::new(project_repository);

            match delete_use_case.execute(&code, &project) {
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

            match deactivate_use_case.execute(&code) {
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
