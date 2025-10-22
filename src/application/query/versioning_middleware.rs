use crate::domain::shared::api_contracts::ApiContractManager;
use crate::domain::shared::api_versioning::ApiVersion;
use crate::domain::shared::query_parser::Query;
use serde::{Deserialize, Serialize};

/// Middleware para versionamento de API
pub struct VersioningMiddleware {
    contract_manager: ApiContractManager,
    default_version: ApiVersion,
}

impl VersioningMiddleware {
    /// Cria um novo middleware de versionamento
    pub fn new() -> Self {
        Self {
            contract_manager: ApiContractManager::new(),
            default_version: ApiVersion::current(),
        }
    }

    /// Cria middleware com versão padrão específica
    pub fn with_default_version(version: ApiVersion) -> Self {
        Self {
            contract_manager: ApiContractManager::new(),
            default_version: version,
        }
    }

    /// Processa uma query com versionamento
    pub fn process_query(&self, request: &VersionedQueryRequest) -> VersionedQueryResponse {
        let api_version = request.api_version.as_ref().unwrap_or(&self.default_version);

        // Validar a query contra o contrato da versão
        let validation_result = self.contract_manager.validate_query(&request.query, api_version);

        if validation_result.is_error() {
            return VersionedQueryResponse {
                success: false,
                error: Some(validation_result.error_message().unwrap()),
                warnings: Vec::new(),
                api_version: api_version.clone(),
                query: request.query.clone(),
                result: None,
            };
        }

        // Query é válida, processar normalmente
        VersionedQueryResponse {
            success: true,
            error: None,
            warnings: validation_result.warnings(),
            api_version: api_version.clone(),
            query: request.query.clone(),
            result: None, // Será preenchido pelo processador de query
        }
    }

    /// Obtém informações sobre uma versão da API
    pub fn get_version_info(&self, version: &ApiVersion) -> Option<VersionInfo> {
        self.contract_manager.get_contract(version).map(|contract| VersionInfo {
            version: version.clone(),
            supported_features: contract.supported_features.iter().map(|f| f.name.clone()).collect(),
            deprecated_features: contract.deprecated_features.iter().map(|f| f.name.clone()).collect(),
            migration_guide: contract.migration_guide.clone(),
        })
    }

    /// Obtém informações sobre compatibilidade entre versões
    pub fn check_compatibility(
        &self,
        requested: &ApiVersion,
        actual: &ApiVersion,
    ) -> crate::domain::shared::api_versioning::CompatibilityInfo {
        self.contract_manager.check_compatibility(requested, actual)
    }

    /// Obtém a versão padrão
    pub fn default_version(&self) -> &ApiVersion {
        &self.default_version
    }

    /// Obtém o gerenciador de contratos
    pub fn contract_manager(&self) -> &ApiContractManager {
        &self.contract_manager
    }
}

impl Default for VersioningMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Request de query versionada
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedQueryRequest {
    pub query: Query,
    pub api_version: Option<ApiVersion>,
    pub client_info: Option<ClientInfo>,
}

impl VersionedQueryRequest {
    /// Cria um novo request versionado
    pub fn new(query: Query) -> Self {
        Self {
            query,
            api_version: None,
            client_info: None,
        }
    }

    /// Adiciona versão da API
    pub fn with_api_version(mut self, version: ApiVersion) -> Self {
        self.api_version = Some(version);
        self
    }

    /// Adiciona informações do cliente
    pub fn with_client_info(mut self, client_info: ClientInfo) -> Self {
        self.client_info = Some(client_info);
        self
    }
}

/// Response de query versionada
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedQueryResponse {
    pub success: bool,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub api_version: ApiVersion,
    pub query: Query,
    pub result: Option<serde_json::Value>, // Será preenchido com QueryResult serializado
}

/// Informações do cliente
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
    pub user_agent: Option<String>,
}

impl ClientInfo {
    /// Cria informações do cliente
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            user_agent: None,
        }
    }

    /// Adiciona user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

/// Informações sobre uma versão da API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: ApiVersion,
    pub supported_features: Vec<String>,
    pub deprecated_features: Vec<String>,
    pub migration_guide: Option<String>,
}

/// Informações de compatibilidade
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub is_compatible: bool,
    pub warnings: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::query_parser::{
        ComparisonOperator, FilterCondition, PaginationOptions, ProjectionOptions, QueryExpression, QueryValue,
    };

    #[test]
    fn test_versioning_middleware_creation() {
        let middleware = VersioningMiddleware::new();
        assert_eq!(middleware.default_version().major, 1);
    }

    #[test]
    fn test_versioned_query_request() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        };

        let request = VersionedQueryRequest::new(query).with_api_version(ApiVersion::new(1, 1, 0));

        assert!(request.api_version.is_some());
        assert_eq!(request.api_version.unwrap().major, 1);
    }

    #[test]
    fn test_versioned_query_response() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        };

        let response = VersionedQueryResponse {
            success: true,
            error: None,
            warnings: Vec::new(),
            api_version: ApiVersion::new(1, 1, 0),
            query,
            result: None,
        };

        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_client_info() {
        let client_info = ClientInfo::new("TestClient".to_string(), "1.0.0".to_string())
            .with_user_agent("TestClient/1.0.0".to_string());

        assert_eq!(client_info.name, "TestClient");
        assert_eq!(client_info.version, "1.0.0");
        assert_eq!(client_info.user_agent, Some("TestClient/1.0.0".to_string()));
    }

    #[test]
    fn test_middleware_processing() {
        let middleware = VersioningMiddleware::new();
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        };

        let request = VersionedQueryRequest::new(query).with_api_version(ApiVersion::new(1, 1, 0));

        let response = middleware.process_query(&request);
        assert!(response.success);
    }
}
