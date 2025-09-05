use crate::{
    application::task::remove_dependency::RemoveDependencyUseCase,
    infrastructure::persistence::task_repository::FileTaskRepository,
};
use super::super::commands::UnlinkCommand;

pub fn handle_unlink_command(command: UnlinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        UnlinkCommand::Tasks { from, to, project, company } => {
            let task_repository = FileTaskRepository::new(".");
            let unlink_use_case = RemoveDependencyUseCase::new(task_repository);

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
    }
}
