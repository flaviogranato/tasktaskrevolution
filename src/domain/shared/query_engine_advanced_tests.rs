#[cfg(test)]
mod advanced_query_tests {
    use super::super::query_engine::{QueryEngine, Queryable};
    use super::super::query_parser::{
        AggregationType, ComparisonOperator, FilterCondition, PaginationOptions, Query, QueryExpression,
        QueryValue, SortOption,
    };
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEntity {
        name: String,
        age: i32,
        active: bool,
        score: f64,
        tags: Vec<String>,
        created_at: chrono::NaiveDate,
    }

    impl Queryable for TestEntity {
        fn get_field_value(&self, field: &str) -> Option<QueryValue> {
            match field {
                "name" => Some(QueryValue::String(self.name.clone())),
                "age" => Some(QueryValue::Number(self.age as f64)),
                "active" => Some(QueryValue::Boolean(self.active)),
                "score" => Some(QueryValue::Number(self.score)),
                "tags" => Some(QueryValue::Array(
                    self.tags.iter().map(|t| QueryValue::String(t.clone())).collect()
                )),
                "created_at" => Some(QueryValue::Date(self.created_at)),
                _ => None,
            }
        }

        fn entity_type() -> &'static str {
            "test_entity"
        }
    }

    fn create_test_entities() -> Vec<TestEntity> {
        vec![
            TestEntity {
                name: "Alice Johnson".to_string(),
                age: 25,
                active: true,
                score: 85.5,
                tags: vec!["developer".to_string(), "senior".to_string()],
                created_at: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            },
            TestEntity {
                name: "Bob Smith".to_string(),
                age: 30,
                active: false,
                score: 92.0,
                tags: vec!["manager".to_string(), "lead".to_string()],
                created_at: chrono::NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
            },
            TestEntity {
                name: "Charlie Brown".to_string(),
                age: 22,
                active: true,
                score: 78.0,
                tags: vec!["junior".to_string(), "intern".to_string()],
                created_at: chrono::NaiveDate::from_ymd_opt(2024, 3, 5).unwrap(),
            },
        ]
    }

    #[test]
    fn test_regex_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::Regex,
                value: QueryValue::String("^[A-Z].*".to_string()),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 3); // All names start with capital letter
    }

    #[test]
    fn test_not_regex_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::NotRegex,
                value: QueryValue::String("^[A-Z].*".to_string()),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 0); // All names start with capital letter
    }

    #[test]
    fn test_starts_with_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::StartsWith,
                value: QueryValue::String("Alice".to_string()),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1);
        assert_eq!(result.items[0].name, "Alice Johnson");
    }

    #[test]
    fn test_ends_with_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::EndsWith,
                value: QueryValue::String("Smith".to_string()),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1);
        assert_eq!(result.items[0].name, "Bob Smith");
    }

    #[test]
    fn test_in_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "age".to_string(),
                operator: ComparisonOperator::In,
                value: QueryValue::Array(vec![
                    QueryValue::Number(25.0),
                    QueryValue::Number(30.0),
                ]),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 2);
    }

    #[test]
    fn test_not_in_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "age".to_string(),
                operator: ComparisonOperator::NotIn,
                value: QueryValue::Array(vec![
                    QueryValue::Number(25.0),
                    QueryValue::Number(30.0),
                ]),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1);
        assert_eq!(result.items[0].age, 22);
    }

    #[test]
    fn test_between_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "age".to_string(),
                operator: ComparisonOperator::Between,
                value: QueryValue::Range {
                    start: Box::new(QueryValue::Number(20.0)),
                    end: Box::new(QueryValue::Number(30.0)),
                },
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 3); // All ages are between 20 and 30
    }

    #[test]
    fn test_not_between_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "age".to_string(),
                operator: ComparisonOperator::NotBetween,
                value: QueryValue::Range {
                    start: Box::new(QueryValue::Number(25.0)),
                    end: Box::new(QueryValue::Number(30.0)),
                },
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1); // Only Charlie (age 22) is not between 25-30
        assert_eq!(result.items[0].age, 22);
    }

    #[test]
    fn test_is_null_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::IsNull,
                value: QueryValue::Null,
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 0); // No null names
    }

    #[test]
    fn test_is_not_null_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::IsNotNull,
                value: QueryValue::Null,
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 3); // All names are not null
    }

    #[test]
    fn test_array_contains_operator() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Condition(FilterCondition {
                field: "tags".to_string(),
                operator: ComparisonOperator::Contains,
                value: QueryValue::String("developer".to_string()),
            }),
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: None,
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1);
        assert_eq!(result.items[0].name, "Alice Johnson");
    }

    #[test]
    fn test_complex_query_with_multiple_operators() {
        let entities = create_test_entities();
        let query = Query {
            projection: ProjectionOptions::include_all_fields(),
            expression: QueryExpression::Logical {
                operator: crate::domain::shared::query_parser::LogicalOperator::And,
                left: Box::new(QueryExpression::Condition(FilterCondition {
                    field: "active".to_string(),
                    operator: ComparisonOperator::Equal,
                    value: QueryValue::Boolean(true),
                })),
                right: Some(Box::new(QueryExpression::Condition(FilterCondition {
                    field: "age".to_string(),
                    operator: ComparisonOperator::Between,
                    value: QueryValue::Range {
                        start: Box::new(QueryValue::Number(20.0)),
                        end: Box::new(QueryValue::Number(30.0)),
                    },
                }))),
            },
            aggregation: None,
            pagination: PaginationOptions::new_default(),
            sort: Some(SortOption {
                field: "score".to_string(),
                ascending: false,
            }),
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 2); // Alice and Charlie are active and between 20-30
        assert_eq!(result.items[0].name, "Alice Johnson"); // Sorted by score desc
        assert_eq!(result.items[1].name, "Charlie Brown");
    }
}

