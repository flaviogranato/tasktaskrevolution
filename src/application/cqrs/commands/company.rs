use super::Command;
use serde::{Deserialize, Serialize};
use crate::domain::company_management::company::Company;

/// Command para criar uma nova empresa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompanyCommand {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

impl Command for CreateCompanyCommand {
    type Result = Result<Company, String>;
}

/// Command para atualizar uma empresa existente
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCompanyCommand {
    pub code: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Command for UpdateCompanyCommand {
    type Result = Result<Company, String>;
}

/// Command para deletar uma empresa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCompanyCommand {
    pub code: String,
}

impl Command for DeleteCompanyCommand {
    type Result = Result<(), String>;
}
