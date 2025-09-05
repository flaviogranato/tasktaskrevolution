use crate::{
    application::task::remove_dependency::RemoveTaskDependencyUseCase,
    infrastructure::persistence::{
        task_repository::FileTaskRepository,
        project_repository::FileProjectRepository,
    },
};
use super::super::commands::UnlinkCommand;

pub fn handle_unlink_command(command: UnlinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        UnlinkCommand::Tasks { from, to, project, company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let unlink_use_case = RemoveTaskDependencyUseCase::new(project_repository);

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
    }
}
