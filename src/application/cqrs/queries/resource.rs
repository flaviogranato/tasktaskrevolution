use crate::application::cqrs::Query;
use serde::{Deserialize, Serialize};
use crate::domain::resource_management::AnyResource;

/// Query para obter um recurso por código
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourceQuery {
    pub code: String,
}

impl Query for GetResourceQuery {
    type Result = Result<Option<AnyResource>, String>;
}

/// Query para listar todos os recursos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesQuery {
    pub filters: Option<ResourceFilters>,
}

impl Query for ListResourcesQuery {
    type Result = Result<Vec<AnyResource>, String>;
}

/// Filtros para consulta de recursos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceFilters {
    pub name_contains: Option<String>,
    pub code_contains: Option<String>,
    pub email_contains: Option<String>,
    pub resource_type: Option<String>,
}
