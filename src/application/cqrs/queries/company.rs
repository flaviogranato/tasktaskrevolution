use crate::application::cqrs::commands::Query;
use crate::domain::company_management::company::Company;
use serde::{Deserialize, Serialize};

/// Query para obter uma empresa por código
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetCompanyQuery {
    pub code: String,
}

impl Query for GetCompanyQuery {
    type Result = Result<Option<Company>, String>;
}

/// Query para listar todas as empresas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCompaniesQuery {
    pub filters: Option<CompanyFilters>,
}

impl Query for ListCompaniesQuery {
    type Result = Result<Vec<Company>, String>;
}

/// Filtros para consulta de empresas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFilters {
    pub name_contains: Option<String>,
    pub code_contains: Option<String>,
}
