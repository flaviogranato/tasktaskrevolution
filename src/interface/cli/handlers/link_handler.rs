use crate::{
    application::task::link_task::LinkTaskUseCase,
    infrastructure::persistence::{
        task_repository::FileTaskRepository,
        project_repository::FileProjectRepository,
    },
};
use super::super::commands::LinkCommand;

pub fn handle_link_command(command: LinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        LinkCommand::Tasks { from, to, project, company } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let link_use_case = LinkTaskUseCase::new(project_repository);

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
    }
}
