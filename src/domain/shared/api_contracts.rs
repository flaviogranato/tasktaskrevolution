use crate::domain::shared::api_versioning::{ApiVersion, ApiVersionManager, CompatibilityInfo};
use crate::domain::shared::query_parser::Query;
// use crate::domain::shared::query_engine::QueryResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contrato de API para uma versão específica
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiContract {
    pub version: ApiVersion,
    pub supported_features: Vec<ApiFeature>,
    pub deprecated_features: Vec<ApiFeature>,
    pub breaking_changes: Vec<BreakingChange>,
    pub migration_guide: Option<String>,
}

/// Funcionalidade da API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiFeature {
    pub name: String,
    pub description: String,
    pub introduced_in: ApiVersion,
    pub deprecated_in: Option<ApiVersion>,
    pub removed_in: Option<ApiVersion>,
}

/// Mudança que quebra compatibilidade
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakingChange {
    pub description: String,
    pub affected_features: Vec<String>,
    pub migration_steps: Vec<String>,
    pub introduced_in: ApiVersion,
}

impl ApiContract {
    /// Cria um novo contrato de API
    pub fn new(version: ApiVersion) -> Self {
        Self {
            version,
            supported_features: Vec::new(),
            deprecated_features: Vec::new(),
            breaking_changes: Vec::new(),
            migration_guide: None,
        }
    }

    /// Adiciona uma funcionalidade suportada
    pub fn with_feature(mut self, feature: ApiFeature) -> Self {
        self.supported_features.push(feature);
        self
    }

    /// Adiciona uma funcionalidade depreciada
    pub fn with_deprecated_feature(mut self, feature: ApiFeature) -> Self {
        self.deprecated_features.push(feature);
        self
    }

    /// Adiciona uma mudança que quebra compatibilidade
    pub fn with_breaking_change(mut self, change: BreakingChange) -> Self {
        self.breaking_changes.push(change);
        self
    }

    /// Define o guia de migração
    pub fn with_migration_guide(mut self, guide: String) -> Self {
        self.migration_guide = Some(guide);
        self
    }

    /// Verifica se uma funcionalidade é suportada
    pub fn supports_feature(&self, feature_name: &str) -> bool {
        self.supported_features.iter().any(|f| f.name == feature_name)
    }

    /// Verifica se uma funcionalidade está depreciada
    pub fn is_feature_deprecated(&self, feature_name: &str) -> bool {
        self.deprecated_features.iter().any(|f| f.name == feature_name)
    }

    /// Obtém informações sobre uma funcionalidade
    pub fn get_feature_info(&self, feature_name: &str) -> Option<&ApiFeature> {
        self.supported_features
            .iter()
            .chain(self.deprecated_features.iter())
            .find(|f| f.name == feature_name)
    }
}

/// Gerenciador de contratos de API
pub struct ApiContractManager {
    contracts: HashMap<ApiVersion, ApiContract>,
    version_manager: ApiVersionManager,
}

impl ApiContractManager {
    /// Cria um novo gerenciador de contratos
    pub fn new() -> Self {
        let mut manager = Self {
            contracts: HashMap::new(),
            version_manager: ApiVersionManager::new(),
        };

        // Inicializa com contratos padrão
        manager.initialize_default_contracts();
        manager
    }

    /// Inicializa os contratos padrão
    fn initialize_default_contracts(&mut self) {
        // Contrato para v1.0.0
        let v1_0_0 = ApiContract::new(ApiVersion::new(1, 0, 0))
            .with_feature(ApiFeature {
                name: "basic_queries".to_string(),
                description: "Basic query functionality with filters".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "sorting".to_string(),
                description: "Query result sorting".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "pagination".to_string(),
                description: "Query result pagination".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            });

        self.contracts.insert(ApiVersion::new(1, 0, 0), v1_0_0);

        // Contrato para v1.1.0 (com projeções)
        let v1_1_0 = ApiContract::new(ApiVersion::new(1, 1, 0))
            .with_feature(ApiFeature {
                name: "basic_queries".to_string(),
                description: "Basic query functionality with filters".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "sorting".to_string(),
                description: "Query result sorting".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "pagination".to_string(),
                description: "Query result pagination".to_string(),
                introduced_in: ApiVersion::new(1, 0, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "projections".to_string(),
                description: "Field projection and selection".to_string(),
                introduced_in: ApiVersion::new(1, 1, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_feature(ApiFeature {
                name: "field_aliases".to_string(),
                description: "Field aliases in projections".to_string(),
                introduced_in: ApiVersion::new(1, 1, 0),
                deprecated_in: None,
                removed_in: None,
            })
            .with_migration_guide(
                "To use projections, add a 'projection' field to your query with 'fields' array and 'include_all' boolean.".to_string()
            );

        self.contracts.insert(ApiVersion::new(1, 1, 0), v1_1_0);
    }

    /// Obtém o contrato para uma versão específica
    pub fn get_contract(&self, version: &ApiVersion) -> Option<&ApiContract> {
        self.contracts.get(version)
    }

    /// Obtém o contrato mais recente
    pub fn get_latest_contract(&self) -> Option<&ApiContract> {
        self.contracts.values().max_by_key(|c| &c.version)
    }

    /// Verifica compatibilidade entre versões
    pub fn check_compatibility(&self, requested: &ApiVersion, actual: &ApiVersion) -> CompatibilityInfo {
        self.version_manager.check_compatibility(requested, actual)
    }

    /// Valida uma query contra um contrato
    pub fn validate_query(&self, query: &Query, version: &ApiVersion) -> QueryValidationResult {
        let contract = match self.get_contract(version) {
            Some(contract) => contract,
            None => return QueryValidationResult::error(format!("Unsupported API version: {}", version)),
        };

        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validar projeções se presentes
        if !query.projection.include_all {
            if !contract.supports_feature("projections") {
                errors.push("Projections are not supported in this API version".to_string());
            }

            if !query.projection.fields.is_empty() && !contract.supports_feature("field_aliases") {
                    // Verificar se há aliases
                    let has_aliases = query.projection.fields.iter().any(|f| f.alias.is_some());
                    if has_aliases {
                        errors.push("Field aliases are not supported in this API version".to_string());
                    }
            }
        }

        // Validar outros recursos conforme necessário
        if query.aggregation.is_some() && !contract.supports_feature("aggregation") {
            warnings.push("Aggregation may not be fully supported in this API version".to_string());
        }

        if errors.is_empty() {
            QueryValidationResult::success(warnings)
        } else {
            QueryValidationResult::error(errors.join("; "))
        }
    }

    /// Obtém informações sobre mudanças entre versões
    pub fn get_version_changes(&self, from: &ApiVersion, to: &ApiVersion) -> Vec<String> {
        self.version_manager.get_version_changes(from, to)
    }
}

impl Default for ApiContractManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Resultado da validação de query
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryValidationResult {
    Success { warnings: Vec<String> },
    Error { message: String },
}

impl QueryValidationResult {
    /// Cria um resultado de sucesso
    pub fn success(warnings: Vec<String>) -> Self {
        Self::Success { warnings }
    }

    /// Cria um resultado de erro
    pub fn error(message: String) -> Self {
        Self::Error { message }
    }

    /// Verifica se é sucesso
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Verifica se é erro
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Obtém as mensagens de warning
    pub fn warnings(&self) -> Vec<String> {
        match self {
            Self::Success { warnings } => warnings.clone(),
            Self::Error { .. } => Vec::new(),
        }
    }

    /// Obtém a mensagem de erro
    pub fn error_message(&self) -> Option<String> {
        match self {
            Self::Success { .. } => None,
            Self::Error { message } => Some(message.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::query_parser::ProjectionOptions;

    #[test]
    fn test_api_contract_creation() {
        let contract = ApiContract::new(ApiVersion::new(1, 0, 0));
        assert_eq!(contract.version.major, 1);
        assert_eq!(contract.version.minor, 0);
        assert_eq!(contract.version.patch, 0);
    }

    #[test]
    fn test_api_contract_features() {
        let feature = ApiFeature {
            name: "test_feature".to_string(),
            description: "Test feature".to_string(),
            introduced_in: ApiVersion::new(1, 0, 0),
            deprecated_in: None,
            removed_in: None,
        };

        let contract = ApiContract::new(ApiVersion::new(1, 0, 0)).with_feature(feature);

        assert!(contract.supports_feature("test_feature"));
        assert!(!contract.supports_feature("nonexistent"));
    }

    #[test]
    fn test_api_contract_manager() {
        let manager = ApiContractManager::new();
        let v1_0_0 = ApiVersion::new(1, 0, 0);
        let v1_1_0 = ApiVersion::new(1, 1, 0);

        assert!(manager.get_contract(&v1_0_0).is_some());
        assert!(manager.get_contract(&v1_1_0).is_some());
        assert!(manager.get_latest_contract().is_some());
    }

    #[test]
    fn test_query_validation() {
        let manager = ApiContractManager::new();
        let v1_0_0 = ApiVersion::new(1, 0, 0);
        let v1_1_0 = ApiVersion::new(1, 1, 0);

        // Query básica para v1.0.0
        let basic_query = Query {
            expression: crate::domain::shared::query_parser::QueryExpression::Condition(
                crate::domain::shared::query_parser::FilterCondition {
                    field: "status".to_string(),
                    operator: crate::domain::shared::query_parser::ComparisonOperator::Equal,
                    value: crate::domain::shared::query_parser::QueryValue::String("active".to_string()),
                },
            ),
            aggregation: None,
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
            projection: ProjectionOptions::include_all_fields(),
        };

        let result = manager.validate_query(&basic_query, &v1_0_0);
        assert!(result.is_success());

        let result = manager.validate_query(&basic_query, &v1_1_0);
        assert!(result.is_success());
    }
}
