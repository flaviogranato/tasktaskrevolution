use crate::domain::shared::query_parser::{Query, QueryResult, ProjectionOptions};
use crate::interface::cli::renderer::types::RenderableData;
use std::collections::HashMap;

/// Converter for transforming query results into renderable data
pub struct QueryConverter;

impl QueryConverter {
    /// Convert a query result to renderable data with projection support
    pub fn to_renderable_data(
        query: &Query,
        result: &QueryResult,
        entity_type: &str,
    ) -> RenderableData {
        let mut headers = Vec::new();
        let mut rows = Vec::new();

        // Determine headers based on projection
        if let Some(projection) = &query.projection {
            if projection.include_all {
                // Include all available fields
                headers = Self::extract_all_headers(result);
            } else {
                // Include only projected fields
                headers = projection.fields.iter()
                    .map(|field| {
                        field.alias.as_ref().unwrap_or(&field.field).clone()
                    })
                    .collect();
            }
        } else {
            // Default: include all fields
            headers = Self::extract_all_headers(result);
        }

        // Extract data rows
        for item in &result.items {
            let mut row = Vec::new();
            
            if let Some(projection) = &query.projection {
                if projection.include_all {
                    // Include all fields
                    row = Self::extract_all_values(item);
                } else {
                    // Include only projected fields
                    for field in &projection.fields {
                        if let Some(value) = Self::extract_field_value(item, &field.field) {
                            row.push(value);
                        } else {
                            row.push(String::new());
                        }
                    }
                }
            } else {
                // Default: include all values
                row = Self::extract_all_values(item);
            }
            
            rows.push(row);
        }

        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("total_count".to_string(), serde_json::Value::Number(serde_json::Number::from(result.total_count)));
        metadata.insert("filtered_count".to_string(), serde_json::Value::Number(serde_json::Number::from(result.filtered_count)));
        metadata.insert("entity_type".to_string(), serde_json::Value::String(entity_type.to_string()));

        // Add query information
        if let Some(aggregation) = &query.aggregation {
            metadata.insert("aggregation".to_string(), serde_json::Value::String(format!("{:?}", aggregation)));
        }

        if let Some(sort) = &query.sort {
            metadata.insert("sort".to_string(), serde_json::Value::String(format!("{} {}", sort.field, if sort.ascending { "ASC" } else { "DESC" })));
        }

        if let Some(limit) = query.pagination.limit {
            metadata.insert("limit".to_string(), serde_json::Value::Number(serde_json::Number::from(limit)));
        }

        if let Some(offset) = query.pagination.offset {
            metadata.insert("offset".to_string(), serde_json::Value::Number(serde_json::Number::from(offset)));
        }

        RenderableData::with_metadata(headers, rows, metadata)
            .with_projection(query.projection.clone())
            .apply_projection()
    }

    /// Extract all available headers from query result
    fn extract_all_headers(result: &QueryResult) -> Vec<String> {
        if result.items.is_empty() {
            return Vec::new();
        }

        // Get headers from the first item
        let first_item = &result.items[0];
        Self::extract_all_field_names(first_item)
    }

    /// Extract all field names from an entity
    fn extract_all_field_names(entity: &dyn crate::domain::shared::query_engine::Queryable) -> Vec<String> {
        // This is a simplified implementation
        // In a real implementation, you would introspect the entity to get all field names
        vec![
            "id".to_string(),
            "name".to_string(),
            "status".to_string(),
            "created_at".to_string(),
        ]
    }

    /// Extract all values from an entity
    fn extract_all_values(entity: &dyn crate::domain::shared::query_engine::Queryable) -> Vec<String> {
        // This is a simplified implementation
        // In a real implementation, you would extract all field values from the entity
        vec![
            entity.get_field_value("id").unwrap_or_default(),
            entity.get_field_value("name").unwrap_or_default(),
            entity.get_field_value("status").unwrap_or_default(),
            entity.get_field_value("created_at").unwrap_or_default(),
        ]
    }

    /// Extract a specific field value from an entity
    fn extract_field_value(entity: &dyn crate::domain::shared::query_engine::Queryable, field: &str) -> Option<String> {
        entity.get_field_value(field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, PaginationOptions, ProjectionOptions, FieldProjection};
    use crate::domain::shared::query_engine::{QueryResult, Queryable};

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
    fn test_query_converter_with_projection() {
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

        assert_eq!(renderable_data.headers, vec!["name", "state"]);
        assert_eq!(renderable_data.rows.len(), 1);
        assert_eq!(renderable_data.rows[0], vec!["Test Entity", "active"]);
        assert!(renderable_data.projection.is_some());
    }

    #[test]
    fn test_query_converter_without_projection() {
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

        assert_eq!(renderable_data.headers.len(), 4);
        assert_eq!(renderable_data.rows.len(), 1);
        assert_eq!(renderable_data.rows[0].len(), 4);
    }
}
