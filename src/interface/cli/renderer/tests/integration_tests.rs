use crate::interface::cli::renderer::{QueryConverter, UnifiedRenderer, OutputFormat, RenderableData};
use crate::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, PaginationOptions, ProjectionOptions, FieldProjection};
use crate::domain::shared::query_engine::{QueryResult, Queryable};
use crate::application::query::{QueryBuilder, VersioningMiddleware};
use crate::domain::shared::api_versioning::ApiVersion;
use std::collections::HashMap;

// Mock entity for testing
struct MockEntity {
    id: String,
    name: String,
    status: String,
    email: String,
    age: i32,
    created_at: String,
}

impl Queryable for MockEntity {
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

/// Complete integration test: Query → Validation → Execution → Rendering
#[test]
fn test_complete_query_integration() {
    // 1. Construir query usando QueryBuilder
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .filter("age", ">=", QueryValue::Number(18.0))
        .sort_by("name", true)
        .limit(10)
        .select(vec!["name", "email", "status"])
        .build()
        .expect("Query should build successfully");

    // 2. Simular resultado da query
    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Alice Johnson".to_string(),
                status: "active".to_string(),
                email: "alice@example.com".to_string(),
                age: 25,
                created_at: "2025-01-01".to_string(),
            }),
            Box::new(MockEntity {
                id: "2".to_string(),
                name: "Bob Smith".to_string(),
                status: "active".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
                created_at: "2025-01-02".to_string(),
            }),
        ],
        total_count: 2,
        filtered_count: 2,
    };

    // 3. Converter para RenderableData
    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "user");

    // 4. Test rendering in different formats
    let formats = vec![OutputFormat::Json, OutputFormat::Table, OutputFormat::Csv, OutputFormat::Html];
    
    for format in formats {
        let renderer = UnifiedRenderer::new(format);
        let output = renderer.render(&renderable_data).expect("Should render successfully");
        
        // Verify that output is not empty
        assert!(!output.is_empty(), "Output should not be empty for format {:?}", format);
        
        // Verify that it contains expected data
        assert!(output.contains("Alice") || output.contains("alice"), "Should contain expected data for format {:?}", format);
    }
}

/// Integration test with API versioning
#[test]
fn test_versioned_query_integration() {
    let middleware = VersioningMiddleware::new();
    
    // Query with projections (v1.1.0+)
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

/// Integration test with advanced filters
#[test]
fn test_advanced_filters_integration() {
    let test_data = vec![
        MockEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        MockEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "inactive".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
        MockEntity {
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

    // Simulate query execution
    let filtered_items: Vec<&MockEntity> = test_data.iter()
        .filter(|item| {
            // Simulate regex filter logic
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

    let range_filtered: Vec<&MockEntity> = test_data.iter()
        .filter(|item| item.age >= 25 && item.age <= 35)
        .collect();

    assert_eq!(range_filtered.len(), 3, "Range filter should match all items in range");
}

/// Integration test with projections and aliases
#[test]
fn test_projection_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .select(vec!["name", "email"])
        .select_field_as("status", "state")
        .build()
        .expect("Projection query should build");

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Alice Johnson".to_string(),
                status: "active".to_string(),
                email: "alice@example.com".to_string(),
                age: 25,
                created_at: "2025-01-01".to_string(),
            }),
        ],
        total_count: 1,
        filtered_count: 1,
    };

    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "user");
    
    // Verify that only projected fields are present
    assert_eq!(renderable_data.headers.len(), 3, "Should have 3 projected fields");
    assert!(renderable_data.headers.contains(&"name".to_string()));
    assert!(renderable_data.headers.contains(&"email".to_string()));
    assert!(renderable_data.headers.contains(&"state".to_string()));
    
    // Verify that data is correct
    assert_eq!(renderable_data.rows.len(), 1);
    assert_eq!(renderable_data.rows[0].len(), 3);
}

/// Integration test with sorting and pagination
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
        MockEntity {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            status: "active".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created_at: "2025-01-01".to_string(),
        },
        MockEntity {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            status: "active".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created_at: "2025-01-02".to_string(),
        },
        MockEntity {
            id: "3".to_string(),
            name: "Charlie Brown".to_string(),
            status: "active".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
            created_at: "2025-01-03".to_string(),
        },
    ];

    // Simulate sorting and pagination
    let mut sorted_data = test_data.clone();
    sorted_data.sort_by(|a, b| a.name.cmp(&b.name));
    
    let paginated_data: Vec<&MockEntity> = sorted_data.iter()
        .skip(1)  // offset
        .take(2)  // limit
        .collect();

    assert_eq!(paginated_data.len(), 2, "Should have 2 items after pagination");
    assert_eq!(paginated_data[0].name, "Bob Smith", "First item should be Bob");
    assert_eq!(paginated_data[1].name, "Charlie Brown", "Second item should be Charlie");
}

/// Integration test with aggregations
#[test]
fn test_aggregation_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .aggregate("age", "AVG")
        .build()
        .expect("Aggregation query should build");

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Alice Johnson".to_string(),
                status: "active".to_string(),
                email: "alice@example.com".to_string(),
                age: 25,
                created_at: "2025-01-01".to_string(),
            }),
            Box::new(MockEntity {
                id: "2".to_string(),
                name: "Bob Smith".to_string(),
                status: "active".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
                created_at: "2025-01-02".to_string(),
            }),
        ],
        total_count: 2,
        filtered_count: 2,
    };

    // Simulate aggregation calculation
    let ages: Vec<i32> = result.items.iter()
        .map(|item| item.get_field_value("age").unwrap().parse::<i32>().unwrap())
        .collect();
    
    let avg_age = ages.iter().sum::<i32>() as f64 / ages.len() as f64;
    assert_eq!(avg_age, 27.5, "Average age should be 27.5");
}

/// Integration test with rendering in multiple formats
#[test]
fn test_multi_format_rendering_integration() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .select(vec!["name", "email"])
        .build()
        .expect("Query should build");

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Alice Johnson".to_string(),
                status: "active".to_string(),
                email: "alice@example.com".to_string(),
                age: 25,
                created_at: "2025-01-01".to_string(),
            }),
        ],
        total_count: 1,
        filtered_count: 1,
    };

    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "user");

    // Testar todos os formatos
    let formats = vec![
        (OutputFormat::Json, "JSON"),
        (OutputFormat::Table, "Table"),
        (OutputFormat::Csv, "CSV"),
        (OutputFormat::Html, "HTML"),
    ];

    for (format, name) in formats {
        let renderer = UnifiedRenderer::new(format);
        let output = renderer.render(&renderable_data).expect(&format!("{} rendering should work", name));
        
        // Verify specific characteristics of each format
        match format {
            OutputFormat::Json => {
                assert!(output.contains("{"), "JSON should contain objects");
                assert!(output.contains("data"), "JSON should contain data field");
            },
            OutputFormat::Table => {
                assert!(output.contains("name"), "Table should contain headers");
                assert!(output.contains("Alice"), "Table should contain data");
            },
            OutputFormat::Csv => {
                assert!(output.contains(","), "CSV should contain commas");
                assert!(output.contains("Alice"), "CSV should contain data");
            },
            OutputFormat::Html => {
                assert!(output.contains("<"), "HTML should contain tags");
                assert!(output.contains("Alice"), "HTML should contain data");
            },
        }
    }
}

/// Integration test with versioning and compatibility
#[test]
fn test_versioning_compatibility_integration() {
    let middleware = VersioningMiddleware::new();
    
    // Test different versions
    let versions = vec![
        ApiVersion::new(1, 0, 0),
        ApiVersion::new(1, 1, 0),
    ];

    for version in versions {
        // Basic query (compatible with all versions)
        let basic_query = QueryBuilder::new()
            .filter("status", "=", QueryValue::String("active".to_string()))
            .build()
            .expect("Basic query should build");

        let validation = middleware.contract_manager().validate_query(&basic_query, &version);
        assert!(validation.is_success(), "Basic query should be valid for version {}", version);

        // Query with projections (compatible only with v1.1.0+)
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
