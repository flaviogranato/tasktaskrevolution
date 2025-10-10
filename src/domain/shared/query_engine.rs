use crate::domain::shared::query_parser::{AggregationType, ComparisonOperator, FilterCondition, Query, QueryExpression, QueryValue, SortOption};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Trait para entidades que podem ser consultadas
pub trait Queryable {
    /// Obtém o valor de um campo da entidade
    fn get_field_value(&self, field: &str) -> Option<QueryValue>;

    /// Retorna o tipo da entidade (para validação de campos)
    fn entity_type() -> &'static str;
}

/// Resultado de uma consulta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub items: Vec<T>,
    pub total_count: usize,
    pub filtered_count: usize,
    pub aggregation_result: Option<AggregationResult>,
}

/// Resultado de uma agregação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregationResult {
    pub aggregation_type: AggregationType,
    pub value: f64,
}

impl<T> QueryResult<T> {
    pub fn new(items: Vec<T>) -> Self {
        let total_count = items.len();
        Self {
            filtered_count: total_count,
            total_count,
            items,
            aggregation_result: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total_count: 0,
            filtered_count: 0,
            aggregation_result: None,
        }
    }
    
    pub fn with_aggregation(mut self, aggregation_result: AggregationResult) -> Self {
        self.aggregation_result = Some(aggregation_result);
        self
    }
}

/// Erro de execução de consulta
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryExecutionError {
    InvalidField(String),
    TypeMismatch(String, String),
    UnsupportedOperator(String, String),
    ParseError(String),
}

impl fmt::Display for QueryExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryExecutionError::InvalidField(field) => write!(f, "Invalid field: {}", field),
            QueryExecutionError::TypeMismatch(expected, actual) => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
            QueryExecutionError::UnsupportedOperator(op, field) => {
                write!(f, "Unsupported operator {} for field {}", op, field)
            }
            QueryExecutionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for QueryExecutionError {}

/// Engine de execução de consultas
pub struct QueryEngine;

impl QueryEngine {
    /// Executa uma consulta sobre uma lista de entidades
    pub fn execute<T: Queryable + Clone>(query: &Query, items: Vec<T>) -> Result<QueryResult<T>, QueryExecutionError> {
        let total_count = items.len();
        let mut filtered_items = Vec::new();

        // 1. Filtrar itens baseado na expressão
        for item in items {
            match Self::evaluate_expression(&query.expression, &item) {
                Ok(true) => filtered_items.push(item),
                Ok(false) => {}          // Item doesn't match, skip it
                Err(e) => return Err(e), // Propagate error
            }
        }

        // 2. Aplicar ordenação se especificada
        if let Some(sort) = &query.sort {
            Self::sort_items(&mut filtered_items, sort)?;
        }

        // 3. Aplicar paginação
        let paginated_items = Self::apply_pagination(filtered_items, &query.pagination);

        // 4. Calcular agregação se especificada
        let mut result = QueryResult {
            total_count,
            filtered_count: paginated_items.len(),
            items: paginated_items,
            aggregation_result: None,
        };

        if let Some(aggregation) = &query.aggregation {
            let aggregation_result = Self::calculate_aggregation(&result.items, aggregation)?;
            result.aggregation_result = Some(aggregation_result);
        }

        Ok(result)
    }

    /// Avalia uma expressão de consulta contra uma entidade
    fn evaluate_expression<T: Queryable>(expression: &QueryExpression, item: &T) -> Result<bool, QueryExecutionError> {
        match expression {
            QueryExpression::Condition(condition) => Self::evaluate_condition(condition, item),
            QueryExpression::Logical { operator, left, right } => {
                let left_result = Self::evaluate_expression(left, item)?;

                match operator {
                    crate::domain::shared::query_parser::LogicalOperator::And => {
                        if !left_result {
                            return Ok(false);
                        }
                        if let Some(right) = right {
                            Ok(left_result && Self::evaluate_expression(right, item)?)
                        } else {
                            Ok(left_result)
                        }
                    }
                    crate::domain::shared::query_parser::LogicalOperator::Or => {
                        if left_result {
                            return Ok(true);
                        }
                        if let Some(right) = right {
                            Ok(left_result || Self::evaluate_expression(right, item)?)
                        } else {
                            Ok(left_result)
                        }
                    }
                    crate::domain::shared::query_parser::LogicalOperator::Not => Ok(!left_result),
                }
            }
            QueryExpression::Not(expr) => Ok(!Self::evaluate_expression(expr, item)?),
        }
    }

    /// Avalia uma condição de filtro contra uma entidade
    fn evaluate_condition<T: Queryable>(condition: &FilterCondition, item: &T) -> Result<bool, QueryExecutionError> {
        let field_value = item
            .get_field_value(&condition.field)
            .ok_or_else(|| QueryExecutionError::InvalidField(condition.field.clone()))?;

        Self::compare_values(&field_value, &condition.operator, &condition.value)
    }

    /// Compara dois valores usando um operador
    fn compare_values(
        left: &QueryValue,
        operator: &ComparisonOperator,
        right: &QueryValue,
    ) -> Result<bool, QueryExecutionError> {
        match operator {
            ComparisonOperator::Equal => Ok(Self::values_equal(left, right)),
            ComparisonOperator::NotEqual => Ok(!Self::values_equal(left, right)),
            ComparisonOperator::GreaterThan => Self::compare_numeric(left, right, |a, b| a > b),
            ComparisonOperator::LessThan => Self::compare_numeric(left, right, |a, b| a < b),
            ComparisonOperator::GreaterOrEqual => Self::compare_numeric(left, right, |a, b| a >= b),
            ComparisonOperator::LessOrEqual => Self::compare_numeric(left, right, |a, b| a <= b),
            ComparisonOperator::Contains => Self::compare_string(left, right, |a, b| a.contains(b)),
            ComparisonOperator::NotContains => Self::compare_string(left, right, |a, b| !a.contains(b)),
        }
    }

    /// Verifica se dois valores são iguais
    fn values_equal(left: &QueryValue, right: &QueryValue) -> bool {
        match (left, right) {
            (QueryValue::String(a), QueryValue::String(b)) => a == b,
            (QueryValue::Number(a), QueryValue::Number(b)) => (a - b).abs() < f64::EPSILON,
            (QueryValue::Boolean(a), QueryValue::Boolean(b)) => a == b,
            (QueryValue::Date(a), QueryValue::Date(b)) => a == b,
            (QueryValue::DateTime(a), QueryValue::DateTime(b)) => a == b,
            _ => false,
        }
    }

    /// Compara valores numéricos
    fn compare_numeric<F>(left: &QueryValue, right: &QueryValue, compare_fn: F) -> Result<bool, QueryExecutionError>
    where
        F: Fn(f64, f64) -> bool,
    {
        let left_num = Self::extract_number(left)?;
        let right_num = Self::extract_number(right)?;
        Ok(compare_fn(left_num, right_num))
    }

    /// Compara valores de string
    fn compare_string<F>(left: &QueryValue, right: &QueryValue, compare_fn: F) -> Result<bool, QueryExecutionError>
    where
        F: Fn(&str, &str) -> bool,
    {
        let left_str = Self::extract_string(left)?;
        let right_str = Self::extract_string(right)?;
        Ok(compare_fn(&left_str, &right_str))
    }

    /// Extrai um número de um QueryValue
    fn extract_number(value: &QueryValue) -> Result<f64, QueryExecutionError> {
        match value {
            QueryValue::Number(n) => Ok(*n),
            QueryValue::String(s) => s
                .parse::<f64>()
                .map_err(|_| QueryExecutionError::TypeMismatch("number".to_string(), "string".to_string())),
            _ => Err(QueryExecutionError::TypeMismatch(
                "number".to_string(),
                format!("{:?}", value),
            )),
        }
    }

    /// Extrai uma string de um QueryValue
    fn extract_string(value: &QueryValue) -> Result<String, QueryExecutionError> {
        match value {
            QueryValue::String(s) => Ok(s.clone()),
            QueryValue::Number(n) => Ok(n.to_string()),
            QueryValue::Boolean(b) => Ok(b.to_string()),
            QueryValue::Date(d) => Ok(d.format("%Y-%m-%d").to_string()),
            QueryValue::DateTime(dt) => Ok(dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }

    /// Aplica ordenação aos itens
    fn sort_items<T: Queryable>(items: &mut Vec<T>, sort: &SortOption) -> Result<(), QueryExecutionError> {
        items.sort_by(|a, b| {
            let a_value = a.get_field_value(&sort.field);
            let b_value = b.get_field_value(&sort.field);
            
            match (a_value, b_value) {
                (Some(a_val), Some(b_val)) => {
                    let comparison = Self::compare_values_for_sorting(&a_val, &b_val);
                    if sort.ascending {
                        comparison
                    } else {
                        comparison.reverse()
                    }
                }
                (Some(_), None) => if sort.ascending { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater },
                (None, Some(_)) => if sort.ascending { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less },
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        Ok(())
    }

    /// Aplica paginação aos itens
    fn apply_pagination<T>(items: Vec<T>, pagination: &crate::domain::shared::query_parser::PaginationOptions) -> Vec<T> {
        let start = pagination.offset.unwrap_or(0);
        let end = if let Some(limit) = pagination.limit {
            start + limit
        } else {
            items.len()
        };
        
        items.into_iter().skip(start).take(end - start).collect()
    }

    /// Calcula agregação sobre os itens
    fn calculate_aggregation<T: Queryable>(items: &[T], aggregation: &AggregationType) -> Result<AggregationResult, QueryExecutionError> {
        match aggregation {
            AggregationType::Count => Ok(AggregationResult {
                aggregation_type: aggregation.clone(),
                value: items.len() as f64,
            }),
            AggregationType::Sum(field) | AggregationType::Average(field) | AggregationType::Min(field) | AggregationType::Max(field) => {
                let values: Vec<f64> = items.iter()
                    .filter_map(|item| {
                        if let Some(QueryValue::Number(n)) = item.get_field_value(field) {
                            Some(n)
                        } else {
                            None
                        }
                    })
                    .collect();

                if values.is_empty() {
                    return Err(QueryExecutionError::InvalidField(format!("No numeric values found for field: {}", field)));
                }

                let result_value = match aggregation {
                    AggregationType::Sum(_) => values.iter().sum(),
                    AggregationType::Average(_) => values.iter().sum::<f64>() / values.len() as f64,
                    AggregationType::Min(_) => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                    AggregationType::Max(_) => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                    _ => unreachable!(),
                };

                Ok(AggregationResult {
                    aggregation_type: aggregation.clone(),
                    value: result_value,
                })
            }
        }
    }

    /// Compara dois valores para ordenação
    fn compare_values_for_sorting(a: &QueryValue, b: &QueryValue) -> std::cmp::Ordering {
        match (a, b) {
            (QueryValue::String(a), QueryValue::String(b)) => a.cmp(b),
            (QueryValue::Number(a), QueryValue::Number(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (QueryValue::Boolean(a), QueryValue::Boolean(b)) => a.cmp(b),
            (QueryValue::Date(a), QueryValue::Date(b)) => a.cmp(b),
            (QueryValue::DateTime(a), QueryValue::DateTime(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::query_parser::{
        ComparisonOperator, FilterCondition, LogicalOperator, Query, QueryExpression,
    };

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        name: String,
        age: i32,
        active: bool,
        score: f64,
    }

    impl Queryable for TestEntity {
        fn get_field_value(&self, field: &str) -> Option<QueryValue> {
            match field {
                "name" => Some(QueryValue::String(self.name.clone())),
                "age" => Some(QueryValue::Number(self.age as f64)),
                "active" => Some(QueryValue::Boolean(self.active)),
                "score" => Some(QueryValue::Number(self.score)),
                _ => None,
            }
        }

        fn entity_type() -> &'static str {
            "test_entity"
        }
    }

    #[test]
    fn test_simple_filter() {
        let entities = vec![
            TestEntity {
                name: "Alice".to_string(),
                age: 25,
                active: true,
                score: 85.5,
            },
            TestEntity {
                name: "Bob".to_string(),
                age: 30,
                active: false,
                score: 92.0,
            },
            TestEntity {
                name: "Charlie".to_string(),
                age: 35,
                active: true,
                score: 78.0,
            },
        ];

        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "active".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::Boolean(true),
            }),
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 2);
        assert_eq!(result.items.len(), 2);
        assert!(result.items.iter().all(|e| e.active));
    }

    #[test]
    fn test_numeric_comparison() {
        let entities = vec![
            TestEntity {
                name: "Alice".to_string(),
                age: 25,
                active: true,
                score: 85.5,
            },
            TestEntity {
                name: "Bob".to_string(),
                age: 30,
                active: false,
                score: 92.0,
            },
            TestEntity {
                name: "Charlie".to_string(),
                age: 35,
                active: true,
                score: 78.0,
            },
        ];

        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "score".to_string(),
                operator: ComparisonOperator::GreaterThan,
                value: QueryValue::Number(80.0),
            }),
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 2);
        assert!(result.items.iter().all(|e| e.score > 80.0));
    }

    #[test]
    fn test_string_contains() {
        let entities = vec![
            TestEntity {
                name: "Alice".to_string(),
                age: 25,
                active: true,
                score: 85.5,
            },
            TestEntity {
                name: "Bob".to_string(),
                age: 30,
                active: false,
                score: 92.0,
            },
            TestEntity {
                name: "Charlie".to_string(),
                age: 35,
                active: true,
                score: 78.0,
            },
        ];

        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "name".to_string(),
                operator: ComparisonOperator::Contains,
                value: QueryValue::String("li".to_string()),
            }),
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 2);
        assert!(result.items.iter().all(|e| e.name.contains("li")));
    }

    #[test]
    fn test_logical_and() {
        let entities = vec![
            TestEntity {
                name: "Alice".to_string(),
                age: 25,
                active: true,
                score: 85.5,
            },
            TestEntity {
                name: "Bob".to_string(),
                age: 30,
                active: false,
                score: 92.0,
            },
            TestEntity {
                name: "Charlie".to_string(),
                age: 35,
                active: true,
                score: 78.0,
            },
        ];

        let query = Query {
            expression: QueryExpression::Logical {
                operator: LogicalOperator::And,
                left: Box::new(QueryExpression::Condition(FilterCondition {
                    field: "active".to_string(),
                    operator: ComparisonOperator::Equal,
                    value: QueryValue::Boolean(true),
                })),
                right: Some(Box::new(QueryExpression::Condition(FilterCondition {
                    field: "score".to_string(),
                    operator: ComparisonOperator::GreaterThan,
                    value: QueryValue::Number(80.0),
                }))),
            },
        };

        let result = QueryEngine::execute(&query, entities).unwrap();
        assert_eq!(result.filtered_count, 1);
        assert_eq!(result.items[0].name, "Alice");
    }

    #[test]
    fn test_invalid_field() {
        let entities = vec![TestEntity {
            name: "Alice".to_string(),
            age: 25,
            active: true,
            score: 85.5,
        }];

        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "invalid_field".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("value".to_string()),
            }),
        };

        let result = QueryEngine::execute(&query, entities);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryExecutionError::InvalidField(_)));
    }
}
