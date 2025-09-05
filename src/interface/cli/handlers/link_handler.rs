use crate::{
    application::task::link_task::LinkTaskUseCase,
    infrastructure::persistence::task_repository::FileTaskRepository,
};
use super::super::commands::LinkCommand;

pub fn handle_link_command(command: LinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        LinkCommand::Tasks { from, to, project, company } => {
            let task_repository = FileTaskRepository::new(".");
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
    }
}
