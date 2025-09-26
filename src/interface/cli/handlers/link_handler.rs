use super::super::commands::LinkCommand;
use crate::{
    application::task::link_task::LinkTaskUseCase,
    infrastructure::persistence::project_repository::FileProjectRepository,
};

pub fn handle_link_command(command: LinkCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        LinkCommand::Tasks {
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
    }
}
