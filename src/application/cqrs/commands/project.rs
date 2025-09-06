use super::Command;
use serde::{Deserialize, Serialize};
use crate::domain::project_management::project::Project;

/// Command para criar um novo projeto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectCommand {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub company_code: String,
}

impl Command for CreateProjectCommand {
    type Result = Result<Project, String>;
}

/// Command para atualizar um projeto existente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectCommand {
    pub code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub company_code: Option<String>,
}

impl Command for UpdateProjectCommand {
    type Result = Result<Project, String>;
}

/// Command para cancelar um projeto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelProjectCommand {
    pub code: String,
}

impl Command for CancelProjectCommand {
    type Result = Result<(), String>;
}

/// Command para deletar um projeto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProjectCommand {
    pub code: String,
}

impl Command for DeleteProjectCommand {
    type Result = Result<(), String>;
}
