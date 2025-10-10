use task_task_revolution::application::query::{QueryExecutor, QueryBuilder, QueryValidator, EntityType};
use task_task_revolution::domain::shared::query_parser::{Query, QueryExpression, FilterCondition, ComparisonOperator, QueryValue, AggregationType, SortOption, PaginationOptions};
use task_task_revolution::domain::shared::query_engine::QueryEngine;
use std::collections::HashMap;

// Mock repositories for testing
struct MockProjectRepository {
    projects: Vec<MockProject>,
}

struct MockProject {
    id: String,
    code: String,
    name: String,
    status: String,
    priority: String,
    task_count: f64,
    is_active: bool,
}

impl task_task_revolution::domain::shared::query_engine::Queryable for MockProject {
    fn get_field_value(&self, field: &str) -> Option<QueryValue> {
        match field {
            "id" => Some(QueryValue::String(self.id.clone())),
            "code" => Some(QueryValue::String(self.code.clone())),
            "name" => Some(QueryValue::String(self.name.clone())),
            "status" => Some(QueryValue::String(self.status.clone())),
            "priority" => Some(QueryValue::String(self.priority.clone())),
            "task_count" => Some(QueryValue::Number(self.task_count)),
            "is_active" => Some(QueryValue::Boolean(self.is_active)),
            _ => None,
        }
    }

    fn entity_type() -> &'static str {
        "project"
    }
}

impl MockProjectRepository {
    fn new() -> Self {
        Self {
            projects: vec![
                MockProject {
                    id: "1".to_string(),
                    code: "PROJ-001".to_string(),
                    name: "Project Alpha".to_string(),
                    status: "active".to_string(),
                    priority: "high".to_string(),
                    task_count: 5.0,
                    is_active: true,
                },
                MockProject {
                    id: "2".to_string(),
                    code: "PROJ-002".to_string(),
                    name: "Project Beta".to_string(),
                    status: "completed".to_string(),
                    priority: "medium".to_string(),
                    task_count: 3.0,
                    is_active: false,
                },
                MockProject {
                    id: "3".to_string(),
                    code: "PROJ-003".to_string(),
                    name: "Project Gamma".to_string(),
                    status: "active".to_string(),
                    priority: "low".to_string(),
                    task_count: 8.0,
                    is_active: true,
                },
            ],
        }
    }

    fn find_all(&self) -> Result<Vec<MockProject>, task_task_revolution::application::errors::AppError> {
        Ok(self.projects.clone())
    }
}

#[test]
fn test_query_engine_basic_filtering() {
    let projects = vec![
        MockProject {
            id: "1".to_string(),
            code: "PROJ-001".to_string(),
            name: "Project Alpha".to_string(),
            status: "active".to_string(),
            priority: "high".to_string(),
            task_count: 5.0,
            is_active: true,
        },
        MockProject {
            id: "2".to_string(),
            code: "PROJ-002".to_string(),
            name: "Project Beta".to_string(),
            status: "completed".to_string(),
            priority: "medium".to_string(),
            task_count: 3.0,
            is_active: false,
        },
    ];

    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryEngine::execute(&query, projects).unwrap();
    assert_eq!(result.filtered_count, 1);
    assert_eq!(result.items[0].code, "PROJ-001");
}

#[test]
fn test_query_engine_aggregation() {
    let projects = vec![
        MockProject {
            id: "1".to_string(),
            code: "PROJ-001".to_string(),
            name: "Project Alpha".to_string(),
            status: "active".to_string(),
            priority: "high".to_string(),
            task_count: 5.0,
            is_active: true,
        },
        MockProject {
            id: "2".to_string(),
            code: "PROJ-002".to_string(),
            name: "Project Beta".to_string(),
            status: "active".to_string(),
            priority: "medium".to_string(),
            task_count: 3.0,
            is_active: true,
        },
    ];

    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: Some(AggregationType::Sum("task_count".to_string())),
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryEngine::execute(&query, projects).unwrap();
    assert_eq!(result.filtered_count, 2);
    assert!(result.aggregation_result.is_some());
    
    let aggregation = result.aggregation_result.unwrap();
    assert_eq!(aggregation.value, 8.0);
}

#[test]
fn test_query_engine_sorting() {
    let projects = vec![
        MockProject {
            id: "1".to_string(),
            code: "PROJ-001".to_string(),
            name: "Project Alpha".to_string(),
            status: "active".to_string(),
            priority: "high".to_string(),
            task_count: 5.0,
            is_active: true,
        },
        MockProject {
            id: "2".to_string(),
            code: "PROJ-002".to_string(),
            name: "Project Beta".to_string(),
            status: "active".to_string(),
            priority: "medium".to_string(),
            task_count: 3.0,
            is_active: true,
        },
        MockProject {
            id: "3".to_string(),
            code: "PROJ-003".to_string(),
            name: "Project Gamma".to_string(),
            status: "active".to_string(),
            priority: "low".to_string(),
            task_count: 8.0,
            is_active: true,
        },
    ];

    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: Some(SortOption {
            field: "task_count".to_string(),
            ascending: false,
        }),
        pagination: PaginationOptions::default(),
    };

    let result = QueryEngine::execute(&query, projects).unwrap();
    assert_eq!(result.filtered_count, 3);
    assert_eq!(result.items[0].code, "PROJ-003"); // Highest task_count
    assert_eq!(result.items[1].code, "PROJ-001");
    assert_eq!(result.items[2].code, "PROJ-002");
}

#[test]
fn test_query_engine_pagination() {
    let projects = vec![
        MockProject {
            id: "1".to_string(),
            code: "PROJ-001".to_string(),
            name: "Project Alpha".to_string(),
            status: "active".to_string(),
            priority: "high".to_string(),
            task_count: 5.0,
            is_active: true,
        },
        MockProject {
            id: "2".to_string(),
            code: "PROJ-002".to_string(),
            name: "Project Beta".to_string(),
            status: "active".to_string(),
            priority: "medium".to_string(),
            task_count: 3.0,
            is_active: true,
        },
        MockProject {
            id: "3".to_string(),
            code: "PROJ-003".to_string(),
            name: "Project Gamma".to_string(),
            status: "active".to_string(),
            priority: "low".to_string(),
            task_count: 8.0,
            is_active: true,
        },
    ];

    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: Some(SortOption {
            field: "code".to_string(),
            ascending: true,
        }),
        pagination: PaginationOptions::new(Some(2), Some(1)),
    };

    let result = QueryEngine::execute(&query, projects).unwrap();
    assert_eq!(result.filtered_count, 2);
    assert_eq!(result.items[0].code, "PROJ-002");
    assert_eq!(result.items[1].code, "PROJ-003");
}

#[test]
fn test_query_builder() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .filter("priority", ">", QueryValue::String("medium".to_string()))
        .sort_desc("task_count")
        .limit(10)
        .build()
        .unwrap();

    assert!(query.aggregation.is_none());
    assert!(query.sort.is_some());
    assert!(!query.sort.as_ref().unwrap().ascending);
    assert_eq!(query.pagination.limit, Some(10));
}

#[test]
fn test_query_builder_aggregation() {
    let query = QueryBuilder::new()
        .filter("status", "=", QueryValue::String("active".to_string()))
        .count()
        .build()
        .unwrap();

    assert!(query.aggregation.is_some());
    assert!(matches!(query.aggregation.unwrap(), AggregationType::Count));
}

#[test]
fn test_query_validator() {
    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryValidator::validate_query(&query, EntityType::Project);
    assert!(result.is_ok());
}

#[test]
fn test_query_validator_invalid_field() {
    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "invalid_field".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("value".to_string()),
        }),
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryValidator::validate_query(&query, EntityType::Project);
    assert!(result.is_err());
}

#[test]
fn test_query_validator_aggregation() {
    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: Some(AggregationType::Sum("task_count".to_string())),
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryValidator::validate_query(&query, EntityType::Project);
    assert!(result.is_ok());
}

#[test]
fn test_query_validator_invalid_aggregation_field() {
    let query = Query {
        expression: QueryExpression::Condition(FilterCondition {
            field: "status".to_string(),
            operator: ComparisonOperator::Equal,
            value: QueryValue::String("active".to_string()),
        }),
        aggregation: Some(AggregationType::Sum("invalid_field".to_string())),
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryValidator::validate_query(&query, EntityType::Project);
    assert!(result.is_err());
}

#[test]
fn test_query_engine_complex_expression() {
    let projects = vec![
        MockProject {
            id: "1".to_string(),
            code: "PROJ-001".to_string(),
            name: "Project Alpha".to_string(),
            status: "active".to_string(),
            priority: "high".to_string(),
            task_count: 5.0,
            is_active: true,
        },
        MockProject {
            id: "2".to_string(),
            code: "PROJ-002".to_string(),
            name: "Project Beta".to_string(),
            status: "completed".to_string(),
            priority: "high".to_string(),
            task_count: 3.0,
            is_active: false,
        },
        MockProject {
            id: "3".to_string(),
            code: "PROJ-003".to_string(),
            name: "Project Gamma".to_string(),
            status: "active".to_string(),
            priority: "low".to_string(),
            task_count: 8.0,
            is_active: true,
        },
    ];

    // Query: status = "active" AND (priority = "high" OR task_count > 7)
    let query = Query {
        expression: QueryExpression::Logical {
            operator: task_task_revolution::domain::shared::query_parser::LogicalOperator::And,
            left: Box::new(QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            })),
            right: Some(Box::new(QueryExpression::Logical {
                operator: task_task_revolution::domain::shared::query_parser::LogicalOperator::Or,
                left: Box::new(QueryExpression::Condition(FilterCondition {
                    field: "priority".to_string(),
                    operator: ComparisonOperator::Equal,
                    value: QueryValue::String("high".to_string()),
                })),
                right: Some(Box::new(QueryExpression::Condition(FilterCondition {
                    field: "task_count".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: QueryValue::Number(7.0),
                }))),
            })),
        },
        aggregation: None,
        sort: None,
        pagination: PaginationOptions::default(),
    };

    let result = QueryEngine::execute(&query, projects).unwrap();
    assert_eq!(result.filtered_count, 2);
    assert_eq!(result.items[0].code, "PROJ-001"); // status=active AND priority=high
    assert_eq!(result.items[1].code, "PROJ-003"); // status=active AND task_count>7
}
