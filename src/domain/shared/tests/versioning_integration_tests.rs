use crate::domain::shared::api_versioning::{ApiVersion, ApiVersionManager, CompatibilityInfo};
use crate::domain::shared::api_contracts::{ApiContractManager, QueryValidationResult};
use crate::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, PaginationOptions, ProjectionOptions, FieldProjection};

/// Teste de integração do sistema de versionamento
#[test]
fn test_versioning_system_integration() {
    let version_manager = ApiVersionManager::new();
    let contract_manager = ApiContractManager::new();
    
    // Testar versões
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    let v2_0_0 = ApiVersion::new(2, 0, 0);
    
    // Testar compatibilidade
    let compat1 = version_manager.check_compatibility(&v1_0_0, &v1_1_0);
    assert!(compat1.is_compatible, "v1.0.0 should be compatible with v1.1.0");
    
    let compat2 = version_manager.check_compatibility(&v1_0_0, &v2_0_0);
    assert!(!compat2.is_compatible, "v1.0.0 should not be compatible with v2.0.0");
    
    // Testar contratos
    let contract_v1_0_0 = contract_manager.get_contract(&v1_0_0);
    assert!(contract_v1_0_0.is_some(), "Contract for v1.0.0 should exist");
    
    let contract_v1_1_0 = contract_manager.get_contract(&v1_1_0);
    assert!(contract_v1_1_0.is_some(), "Contract for v1.1.0 should exist");
    
    // Testar funcionalidades
    if let Some(contract) = contract_v1_1_0 {
        assert!(contract.supports_feature("projections"), "v1.1.0 should support projections");
        assert!(contract.supports_feature("field_aliases"), "v1.1.0 should support field aliases");
    }
}

/// Teste de integração com validação de queries
#[test]
fn test_query_validation_integration() {
    let contract_manager = ApiContractManager::new();
    
    // Query básica (compatível com v1.0.0)
    let basic_query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::new_default(),
        projection: Some(ProjectionOptions::include_all_fields()),
    };
    
    // Query com projeções (compatível apenas com v1.1.0+)
    let projection_query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::new_default(),
        projection: Some(ProjectionOptions::with_fields(vec![
            FieldProjection::new("name".to_string()),
            FieldProjection::with_alias("status".to_string(), "state".to_string()),
        ])),
    };
    
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    
    // Validar query básica
    let basic_validation_v1_0_0 = contract_manager.validate_query(&basic_query, &v1_0_0);
    assert!(basic_validation_v1_0_0.is_success(), "Basic query should be valid in v1.0.0");
    
    let basic_validation_v1_1_0 = contract_manager.validate_query(&basic_query, &v1_1_0);
    assert!(basic_validation_v1_1_0.is_success(), "Basic query should be valid in v1.1.0");
    
    // Validar query com projeções
    let projection_validation_v1_0_0 = contract_manager.validate_query(&projection_query, &v1_0_0);
    assert!(projection_validation_v1_0_0.is_error(), "Projection query should be invalid in v1.0.0");
    
    let projection_validation_v1_1_0 = contract_manager.validate_query(&projection_query, &v1_1_0);
    assert!(projection_validation_v1_1_0.is_success(), "Projection query should be valid in v1.1.0");
}

/// Teste de integração com mudanças de versão
#[test]
fn test_version_changes_integration() {
    let version_manager = ApiVersionManager::new();
    
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    let v2_0_0 = ApiVersion::new(2, 0, 0);
    
    // Testar mudanças entre versões
    let changes_1_0_to_1_1 = version_manager.get_version_changes(&v1_0_0, &v1_1_0);
    assert!(!changes_1_0_to_1_1.is_empty(), "Should have changes from v1.0.0 to v1.1.0");
    
    let changes_1_0_to_2_0 = version_manager.get_version_changes(&v1_0_0, &v2_0_0);
    assert!(!changes_1_0_to_2_0.is_empty(), "Should have changes from v1.0.0 to v2.0.0");
    
    // Testar compatibilidade
    let compat_1_0_to_1_1 = version_manager.check_compatibility(&v1_0_0, &v1_1_0);
    assert!(compat_1_0_to_1_1.is_compatible, "v1.0.0 should be compatible with v1.1.0");
    
    let compat_1_0_to_2_0 = version_manager.check_compatibility(&v1_0_0, &v2_0_0);
    assert!(!compat_1_0_to_2_0.is_compatible, "v1.0.0 should not be compatible with v2.0.0");
}

/// Teste de integração com funcionalidades depreciadas
#[test]
fn test_deprecated_features_integration() {
    let contract_manager = ApiContractManager::new();
    
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    
    // Testar funcionalidades suportadas
    if let Some(contract_v1_0_0) = contract_manager.get_contract(&v1_0_0) {
        assert!(contract_v1_0_0.supports_feature("basic_queries"), "v1.0.0 should support basic queries");
        assert!(contract_v1_0_0.supports_feature("sorting"), "v1.0.0 should support sorting");
        assert!(contract_v1_0_0.supports_feature("pagination"), "v1.0.0 should support pagination");
    }
    
    if let Some(contract_v1_1_0) = contract_manager.get_contract(&v1_1_0) {
        assert!(contract_v1_1_0.supports_feature("basic_queries"), "v1.1.0 should support basic queries");
        assert!(contract_v1_1_0.supports_feature("projections"), "v1.1.0 should support projections");
        assert!(contract_v1_1_0.supports_feature("field_aliases"), "v1.1.0 should support field aliases");
    }
}

/// Teste de integração com guias de migração
#[test]
fn test_migration_guide_integration() {
    let contract_manager = ApiContractManager::new();
    
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    
    if let Some(contract) = contract_manager.get_contract(&v1_1_0) {
        assert!(contract.migration_guide.is_some(), "v1.1.0 should have migration guide");
        
        if let Some(guide) = &contract.migration_guide {
            assert!(guide.contains("projection"), "Migration guide should mention projections");
            assert!(guide.contains("fields"), "Migration guide should mention fields");
        }
    }
}

/// Teste de integração com validação de versões suportadas
#[test]
fn test_supported_versions_integration() {
    let version_manager = ApiVersionManager::new();
    
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    let v2_0_0 = ApiVersion::new(2, 0, 0);
    let v0_9_0 = ApiVersion::new(0, 9, 0);
    
    // Testar versões suportadas
    assert!(version_manager.is_version_supported(&v1_0_0), "v1.0.0 should be supported");
    assert!(version_manager.is_version_supported(&v1_1_0), "v1.1.0 should be supported");
    assert!(!version_manager.is_version_supported(&v2_0_0), "v2.0.0 should not be supported yet");
    assert!(!version_manager.is_version_supported(&v0_9_0), "v0.9.0 should not be supported");
}

/// Teste de integração com parsing de versões
#[test]
fn test_version_parsing_integration() {
    // Testar parsing de strings de versão
    let version_strs = vec!["v1.0.0", "1.1.0", "v2.0.0", "1.0.0"];
    
    for version_str in version_strs {
        let version: ApiVersion = version_str.parse()
            .expect(&format!("Should parse version '{}'", version_str));
        
        assert!(version.major > 0, "Major version should be > 0 for '{}'", version_str);
    }
    
    // Testar parsing inválido
    let invalid_versions = vec!["invalid", "1.0", "v1.0.0.0", ""];
    
    for invalid_version in invalid_versions {
        let result: Result<ApiVersion, _> = invalid_version.parse();
        assert!(result.is_err(), "Should fail to parse invalid version '{}'", invalid_version);
    }
}

/// Teste de integração com comparação de versões
#[test]
fn test_version_comparison_integration() {
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    let v1_1_1 = ApiVersion::new(1, 1, 1);
    let v2_0_0 = ApiVersion::new(2, 0, 0);
    
    // Testar comparações
    assert!(v1_1_0 > v1_0_0, "v1.1.0 should be greater than v1.0.0");
    assert!(v1_1_1 > v1_1_0, "v1.1.1 should be greater than v1.1.0");
    assert!(v2_0_0 > v1_1_1, "v2.0.0 should be greater than v1.1.1");
    
    // Testar compatibilidade
    assert!(v1_0_0.is_compatible_with(&v1_1_0), "v1.0.0 should be compatible with v1.1.0");
    assert!(v1_1_0.is_compatible_with(&v1_0_0), "v1.1.0 should be compatible with v1.0.0");
    assert!(!v1_0_0.is_compatible_with(&v2_0_0), "v1.0.0 should not be compatible with v2.0.0");
    
    // Testar ordenação
    let mut versions = vec![v2_0_0.clone(), v1_0_0.clone(), v1_1_1.clone(), v1_1_0.clone()];
    versions.sort();
    
    assert_eq!(versions[0], v1_0_0, "First version should be v1.0.0");
    assert_eq!(versions[1], v1_1_0, "Second version should be v1.1.0");
    assert_eq!(versions[2], v1_1_1, "Third version should be v1.1.1");
    assert_eq!(versions[3], v2_0_0, "Fourth version should be v2.0.0");
}
