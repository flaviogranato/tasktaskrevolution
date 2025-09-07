use crate::application::cqrs::commands::Query;
use crate::domain::task_management::AnyTask;
use serde::{Deserialize, Serialize};

/// Query para obter uma tarefa por código
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskQuery {
    pub code: String,
    pub project_code: String,
}

impl Query for GetTaskQuery {
    type Result = Result<Option<AnyTask>, String>;
}

/// Query para listar tarefas de um projeto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTasksQuery {
    pub project_code: String,
    pub filters: Option<TaskFilters>,
}

impl Query for ListTasksQuery {
    type Result = Result<Vec<AnyTask>, String>;
}

/// Query para listar todas as tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAllTasksQuery {
    pub filters: Option<TaskFilters>,
}

impl Query for ListAllTasksQuery {
    type Result = Result<Vec<AnyTask>, String>;
}

/// Filtros para consulta de tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilters {
    pub name_contains: Option<String>,
    pub code_contains: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
}
