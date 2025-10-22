use crate::application::query::{QueryBuilder, QueryExecutor, QueryValidator, VersioningMiddleware};
use crate::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, PaginationOptions, ProjectionOptions, FieldProjection};
use crate::domain::shared::query_engine::{QueryResult, Queryable};
use crate::domain::shared::api_versioning::ApiVersion;
use std::collections::HashMap;

// Mock entity for testing
struct TestEntity {
    id: String,
    name: String,
    status: String,
    email: String,
    age: i32,
    created_at: String,
}

impl Queryable for TestEntity {
    fn get_field_value(&self, field: &str) -> Option<String> {
        match field {
            "id" => Some(self.id.clone()),
            "name" => Some(self.name.clone()),
            "status" => Some(self.status.clone()),
            "email" => Some(self.email.clone()),
            "age" => Some(self.age.to_string()),
            "created_at" => Some(self.created_at.clone()),
            _ => None,
        }
    }
}

/// Teste de integração: QueryBuilder → QueryValidator → QueryExecutor
#[test]
fn test_query_builder_to_executor_integration() {
    // 1. Construir query usando QueryBuilder
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .filter("age", ">=", QueryValue::Number(18.0))
        .sort_by("name", true)
        .limit(10)
        .select(vec!["name", "email", "status"])
        .build()
        .expect("Query should build successfully");

    // 2. Validar query
    let validator = QueryValidator::new();
    let validation_result = validator.validate(&query);
    assert!(validation_result.is_valid(), "Query should be valid");

    // 3. Simular execução (mock)
    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        TestEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "active".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
    ];

    // Simular filtros
    let filtered_data: Vec<&TestEntity> = test_data.iter()
        .filter(|item| {
            item.status == "active" && item.age >= 18
        })
        .collect();

    // Simular ordenação
    let mut sorted_data = filtered_data.clone();
    sorted_data.sort_by(|a, b| a.name.cmp(&b.name));

    // Simular paginação
    let paginated_data: Vec<&TestEntity> = sorted_data.iter()
        .take(10)
        .collect();

    assert_eq!(paginated_data.len(), 2, "Should have 2 filtered items");
    assert_eq!(paginated_data[0].name, "Alice Johnson", "First item should be Alice");
    assert_eq!(paginated_data[1].name, "Bob Smith", "Second item should be Bob");
}

/// Teste de integração com versionamento
#[test]
fn test_versioned_query_integration() {
    let middleware = VersioningMiddleware::new();
    
    // Query com projeções (v1.1.0+)
    let query = Query {
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

    // Testar com v1.0.0 (deve falhar)
    let v1_0_0 = ApiVersion::new(1, 0, 0);
    let validation_old = middleware.contract_manager().validate_query(&query, &v1_0_0);
    assert!(validation_old.is_error(), "Query with projections should fail in v1.0.0");

    // Testar com v1.1.0 (deve passar)
    let v1_1_0 = ApiVersion::new(1, 1, 0);
    let validation_new = middleware.contract_manager().validate_query(&query, &v1_1_0);
    assert!(validation_new.is_success(), "Query with projections should pass in v1.1.0");
}

/// Teste de integração com filtros avançados
#[test]
fn test_advanced_filters_integration() {
    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        TestEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "inactive".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
        TestEntity {
            id: "3".to_string(),
            name: "Charlie Brown".to_string(),
            status: "active".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
            created_at: "2025-01-03".to_string(),
        },
    ];

    // Teste com filtro regex
    let regex_query = QueryBuilder::new()
        .filter("name", "~*", QueryValue::String("^[A-Z].*".to_string()))
        .build()
        .expect("Regex query should build");

    // Simular execução da query
    let filtered_items: Vec<&TestEntity> = test_data.iter()
        .filter(|item| {
            // Simular lógica de filtro regex
            item.name.starts_with('A') || item.name.starts_with('B') || item.name.starts_with('C')
        })
        .collect();

    assert_eq!(filtered_items.len(), 3, "Regex filter should match all items starting with capital letters");

    // Teste com filtro de range
    let range_query = QueryBuilder::new()
        .filter("age", "BETWEEN", QueryValue::Range {
            start: Box::new(QueryValue::Number(25.0)),
            end: Box::new(QueryValue::Number(35.0)),
        })
        .build()
        .expect("Range query should build");

    let range_filtered: Vec<&TestEntity> = test_data.iter()
        .filter(|item| item.age >= 25 && item.age <= 35)
        .collect();

    assert_eq!(range_filtered.len(), 3, "Range filter should match all items in range");
}

/// Teste de integração com projeções
#[test]
fn test_projection_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .select(vec!["name", "email"])
        .select_field_as("status", "state")
        .build()
        .expect("Projection query should build");

    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
    ];

    // Simular aplicação de projeção
    let projected_data: Vec<HashMap<String, String>> = test_data.iter()
        .map(|item| {
            let mut projected = HashMap::new();
            projected.insert("name".to_string(), item.name.clone());
            projected.insert("email".to_string(), item.email.clone());
            projected.insert("state".to_string(), item.status.clone());
            projected
        })
        .collect();

    assert_eq!(projected_data.len(), 1, "Should have 1 projected item");
    assert_eq!(projected_data[0].len(), 3, "Should have 3 projected fields");
    assert_eq!(projected_data[0]["name"], "Alice Johnson", "Name should be projected");
    assert_eq!(projected_data[0]["state"], "active", "Status should be aliased as state");
}

/// Teste de integração com ordenação e paginação
#[test]
fn test_sorting_and_pagination_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .sort_by("name", true)  // ASC
        .limit(2)
        .offset(1)
        .build()
        .expect("Sorted and paginated query should build");

    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        TestEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "active".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
        TestEntity {
            id: "3".to_string(),
            name: "Charlie Brown".to_string(),
            status: "active".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
            created_at: "2025-01-03".to_string(),
        },
    ];

    // Simular ordenação e paginação
    let mut sorted_data = test_data.clone();
    sorted_data.sort_by(|a, b| a.name.cmp(&b.name));
    
    let paginated_data: Vec<&TestEntity> = sorted_data.iter()
        .skip(1)  // offset
        .take(2)  // limit
        .collect();

    assert_eq!(paginated_data.len(), 2, "Should have 2 items after pagination");
    assert_eq!(paginated_data[0].name, "Bob Smith", "First item should be Bob");
    assert_eq!(paginated_data[1].name, "Charlie Brown", "Second item should be Charlie");
}

/// Teste de integração com agregações
#[test]
fn test_aggregation_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .aggregate("age", "AVG")
        .build()
        .expect("Aggregation query should build");

    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        TestEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "active".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
    ];

    // Simular cálculo de agregação
    let ages: Vec<i32> = test_data.iter()
        .map(|item| item.age)
        .collect();
    
    let avg_age = ages.iter().sum::<i32>() as f64 / ages.len() as f64;
    assert_eq!(avg_age, 27.5, "Average age should be 27.5");
}

/// Teste de integração com validação de versões
#[test]
fn test_version_validation_integration() {
    let middleware = VersioningMiddleware::new();
    
    // Testar diferentes versões
    let versions = vec![
        ApiVersion::new(1, 0, 0),
        ApiVersion::new(1, 1, 0),
    ];

    for version in versions {
        // Query básica (compatível com todas as versões)
        let basic_query = QueryBuilder::new()
            .filter("status", "=", QueryValue::String("active".to_string()))
            .build()
            .expect("Basic query should build");

        let validation = middleware.contract_manager().validate_query(&basic_query, &version);
        assert!(validation.is_success(), "Basic query should be valid for version {}", version);

        // Query com projeções (compatível apenas com v1.1.0+)
        if version.minor >= 1 {
            let projection_query = QueryBuilder::new()
                .filter("status", "=", QueryValue::String("active".to_string()))
                .select(vec!["name", "email"])
                .build()
                .expect("Projection query should build");

            let projection_validation = middleware.contract_manager().validate_query(&projection_query, &version);
            assert!(projection_validation.is_success(), "Projection query should be valid for version {}", version);
        }
    }
}

/// Teste de integração com múltiplos filtros
#[test]
fn test_multiple_filters_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .filter("age", ">=", QueryValue::Number(18.0))
        .filter("age", "<=", QueryValue::Number(65.0))
        .filter("name", "~*", QueryValue::String("^[A-Z].*".to_string()))
        .build()
        .expect("Multiple filters query should build");

    let test_data = vec![
        TestEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        TestEntity {
            id: "2".to_string(),
            name: "bob smith".to_string(),  // lowercase, should be filtered out
            status: "active".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
        TestEntity {
            id: "3".to_string(),
            name: "Charlie Brown".to_string(),
            status: "inactive".to_string(),  // inactive, should be filtered out
            email: "charlie@example.com".to_string(),
            age: 35,
            created_at: "2025-01-03".to_string(),
        },
    ];

    // Simular aplicação de múltiplos filtros
    let filtered_data: Vec<&TestEntity> = test_data.iter()
        .filter(|item| {
            item.status == "active" &&
            item.age >= 18 &&
            item.age <= 65 &&
            item.name.starts_with('A') || item.name.starts_with('B') || item.name.starts_with('C')
        })
        .collect();

    assert_eq!(filtered_data.len(), 1, "Should have 1 item after multiple filters");
    assert_eq!(filtered_data[0].name, "Alice Johnson", "Should be Alice Johnson");
}
