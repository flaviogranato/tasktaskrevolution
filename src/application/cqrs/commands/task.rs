use super::Command;
use crate::domain::task_management::AnyTask;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Command para criar uma nova tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskCommand {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub project_code: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub priority: Option<String>,
}

impl Command for CreateTaskCommand {
    type Result = Result<AnyTask, String>;
}

/// Command para atualizar uma tarefa existente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskCommand {
    pub code: String,
    pub project_code: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub priority: Option<String>,
}

impl Command for UpdateTaskCommand {
    type Result = Result<AnyTask, String>;
}

/// Command para adicionar dependência entre tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskDependencyCommand {
    pub from_task_code: String,
    pub to_task_code: String,
    pub project_code: String,
}

impl Command for AddTaskDependencyCommand {
    type Result = Result<(), String>;
}

/// Command para remover dependência entre tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTaskDependencyCommand {
    pub from_task_code: String,
    pub to_task_code: String,
    pub project_code: String,
}

impl Command for RemoveTaskDependencyCommand {
    type Result = Result<(), String>;
}

/// Command para deletar uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTaskCommand {
    pub code: String,
    pub project_code: String,
}

impl Command for DeleteTaskCommand {
    type Result = Result<(), String>;
}
