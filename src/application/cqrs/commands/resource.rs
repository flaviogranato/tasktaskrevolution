use super::Command;
use serde::{Deserialize, Serialize};
use crate::domain::resource_management::AnyResource;
use chrono::NaiveDate;

/// Command para criar um novo recurso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceCommand {
    pub name: String,
    pub code: String,
    pub email: Option<String>,
    pub resource_type: String,
}

impl Command for CreateResourceCommand {
    type Result = Result<AnyResource, String>;
}

/// Command para atualizar um recurso existente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceCommand {
    pub code: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub resource_type: Option<String>,
}

impl Command for UpdateResourceCommand {
    type Result = Result<AnyResource, String>;
}

/// Command para criar um time off
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeOffCommand {
    pub resource_code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub description: Option<String>,
}

impl Command for CreateTimeOffCommand {
    type Result = Result<AnyResource, String>;
}

/// Command para criar uma férias
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVacationCommand {
    pub resource_code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub description: Option<String>,
}

impl Command for CreateVacationCommand {
    type Result = Result<AnyResource, String>;
}

/// Command para deletar um recurso
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResourceCommand {
    pub code: String,
}

impl Command for DeleteResourceCommand {
    type Result = Result<(), String>;
}
