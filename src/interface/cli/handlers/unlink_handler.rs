use super::super::commands::UnlinkCommand;
use crate::{
    application::task::remove_dependency::RemoveTaskDependencyUseCase,
    infrastructure::persistence::project_repository::FileProjectRepository,
};

pub fn handle_unlink_command(command: UnlinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        UnlinkCommand::Tasks {
            from,
            to,
            project,
            company: _,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(".");
            let unlink_use_case = RemoveTaskDependencyUseCase::new(project_repository, code_resolver);

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
