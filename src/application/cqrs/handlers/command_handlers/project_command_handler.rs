use crate::application::cqrs::commands::project::*;
use crate::domain::project_management::project::Project;

/// Handler para comandos de projeto
pub struct ProjectCommandHandler;

impl ProjectCommandHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_create_project(&self, command: CreateProjectCommand) -> Result<Project, String> {
        // Implementação simplificada para demonstração
        let project = Project::new(
            command.code,
            command.name,
            command.company_code,
            command.description.unwrap_or_else(|| "".to_string()),
        ).map_err(|e| format!("Failed to create project: {}", e))?;
        
        Ok(project)
    }

    pub fn handle_update_project(&self, _command: UpdateProjectCommand) -> Result<Project, String> {
        // Implementação simplificada para demonstração
        Err("Update not implemented yet".to_string())
    }

    pub fn handle_cancel_project(&self, _command: CancelProjectCommand) -> Result<(), String> {
        // Implementação simplificada para demonstração
        Ok(())
    }

    pub fn handle_delete_project(&self, _command: DeleteProjectCommand) -> Result<(), String> {
        // Implementação simplificada para demonstração
        Ok(())
    }
}
