use crate::application::cqrs::commands::Query;
use crate::domain::project_management::project::Project;
use serde::{Deserialize, Serialize};

/// Query para obter um projeto por código
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetProjectQuery {
    pub code: String,
}

impl Query for GetProjectQuery {
    type Result = Result<Option<Project>, String>;
}

/// Query para listar projetos de uma empresa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProjectsQuery {
    pub company_code: String,
    pub filters: Option<ProjectFilters>,
}

impl Query for ListProjectsQuery {
    type Result = Result<Vec<Project>, String>;
}

/// Query para listar todos os projetos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAllProjectsQuery {
    pub filters: Option<ProjectFilters>,
}

impl Query for ListAllProjectsQuery {
    type Result = Result<Vec<Project>, String>;
}

/// Filtros para consulta de projetos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFilters {
    pub name_contains: Option<String>,
    pub code_contains: Option<String>,
    pub status: Option<String>,
}
