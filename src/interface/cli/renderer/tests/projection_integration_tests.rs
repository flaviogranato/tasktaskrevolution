use crate::interface::cli::renderer::{QueryConverter, UnifiedRenderer, OutputFormat, RenderableData};
use crate::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, PaginationOptions, ProjectionOptions, FieldProjection};
use crate::domain::shared::query_engine::{QueryResult, Queryable};
use std::collections::HashMap;

// Mock entity for testing
struct MockEntity {
    id: String,
    name: String,
    status: String,
    created_at: String,
}

impl Queryable for MockEntity {
    fn get_field_value(&self, field: &str) -> Option<String> {
        match field {
            "id" => Some(self.id.clone()),
            "name" => Some(self.name.clone()),
            "status" => Some(self.status.clone()),
            "created_at" => Some(self.created_at.clone()),
            _ => None,
        }
    }
}

#[test]
fn test_json_output_with_projection() {
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

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Test Entity".to_string(),
                status: "active".to_string(),
                created_at: "2025-01-01".to_string(),
            }),
        ],
        total_count: 1,
        filtered_count: 1,
    };

    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "entity");
    let renderer = UnifiedRenderer::new(OutputFormat::Json);
    let json_output = renderer.render(&renderable_data).unwrap();

    // Parse the JSON to verify structure
    let json_value: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    
    // Verify projection information is included
    assert!(json_value["projection"].is_object());
    assert_eq!(json_value["projection"]["include_all"], false);
    assert!(json_value["projection"]["fields"].is_array());
    
    // Verify data structure
    assert!(json_value["data"].is_array());
    assert_eq!(json_value["data"].as_array().unwrap().len(), 1);
    
    // Verify metadata
    assert!(json_value["metadata"].is_object());
    assert_eq!(json_value["metadata"]["entity_type"], "entity");
    assert_eq!(json_value["metadata"]["total_count"], 1);
    assert_eq!(json_value["metadata"]["filtered_count"], 1);
}

#[test]
fn test_json_output_without_projection() {
    let query = Query {
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

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Test Entity".to_string(),
                status: "active".to_string(),
                created_at: "2025-01-01".to_string(),
            }),
        ],
        total_count: 1,
        filtered_count: 1,
    };

    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "entity");
    let renderer = UnifiedRenderer::new(OutputFormat::Json);
    let json_output = renderer.render(&renderable_data).unwrap();

    // Parse the JSON to verify structure
    let json_value: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    
    // Verify projection information shows include_all = true
    assert!(json_value["projection"].is_object());
    assert_eq!(json_value["projection"]["include_all"], true);
    
    // Verify data structure
    assert!(json_value["data"].is_array());
    assert_eq!(json_value["data"].as_array().unwrap().len(), 1);
}

#[test]
fn test_table_output_with_projection() {
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

    let result = QueryResult {
        items: vec![
            Box::new(MockEntity {
                id: "1".to_string(),
                name: "Test Entity".to_string(),
                status: "active".to_string(),
                created_at: "2025-01-01".to_string(),
            }),
        ],
        total_count: 1,
        filtered_count: 1,
    };

    let renderable_data = QueryConverter::to_renderable_data(&query, &result, "entity");
    let renderer = UnifiedRenderer::new(OutputFormat::Table);
    let table_output = renderer.render(&renderable_data).unwrap();

    // Verify table output contains only projected columns
    assert!(table_output.contains("name"));
    assert!(table_output.contains("state"));
    assert!(!table_output.contains("id"));
    assert!(!table_output.contains("created_at"));
}

#[test]
fn test_renderable_data_projection_application() {
    let mut data = RenderableData::new(
        vec!["id".to_string(), "name".to_string(), "status".to_string(), "created_at".to_string()],
        vec![
            vec!["1".to_string(), "Test".to_string(), "active".to_string(), "2025-01-01".to_string()],
        ],
    );

    let projection = ProjectionOptions::with_fields(vec![
        FieldProjection::new("name".to_string()),
        FieldProjection::with_alias("status".to_string(), "state".to_string()),
    ]);

    data = data.with_projection(projection).apply_projection();

    // Verify headers are filtered
    assert_eq!(data.headers, vec!["name", "state"]);
    
    // Verify rows are filtered
    assert_eq!(data.rows.len(), 1);
    assert_eq!(data.rows[0], vec!["Test", "active"]);
}
