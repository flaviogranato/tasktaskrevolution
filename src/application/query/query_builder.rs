use crate::domain::shared::query_parser::{AggregationType, Query, QueryExpression, QueryValue, SortOption, PaginationOptions};

/// Builder para construção de queries de forma fluente
pub struct QueryBuilder {
    expression: Option<QueryExpression>,
    aggregation: Option<AggregationType>,
    sort: Option<SortOption>,
    pagination: PaginationOptions,
}

impl QueryBuilder {
    /// Cria um novo QueryBuilder
    pub fn new() -> Self {
        Self {
            expression: None,
            aggregation: None,
            sort: None,
            pagination: PaginationOptions::default(),
        }
    }

    /// Adiciona uma condição de filtro
    pub fn filter(mut self, field: &str, operator: &str, value: QueryValue) -> Self {
        let comparison_op = match operator {
            "=" | "eq" => crate::domain::shared::query_parser::ComparisonOperator::Equal,
            "!=" | "ne" => crate::domain::shared::query_parser::ComparisonOperator::NotEqual,
            ">" | "gt" => crate::domain::shared::query_parser::ComparisonOperator::GreaterThan,
            "<" | "lt" => crate::domain::shared::query_parser::ComparisonOperator::LessThan,
            ">=" | "gte" => crate::domain::shared::query_parser::ComparisonOperator::GreaterOrEqual,
            "<=" | "lte" => crate::domain::shared::query_parser::ComparisonOperator::LessOrEqual,
            "~" | "contains" => crate::domain::shared::query_parser::ComparisonOperator::Contains,
            "!~" | "not_contains" => crate::domain::shared::query_parser::ComparisonOperator::NotContains,
            _ => crate::domain::shared::query_parser::ComparisonOperator::Equal,
        };

        let condition = crate::domain::shared::query_parser::FilterCondition {
            field: field.to_string(),
            operator: comparison_op,
            value,
        };

        let new_expression = QueryExpression::Condition(condition);

        self.expression = match self.expression {
            Some(existing) => Some(QueryExpression::Logical {
                operator: crate::domain::shared::query_parser::LogicalOperator::And,
                left: Box::new(existing),
                right: Some(Box::new(new_expression)),
            }),
            None => Some(new_expression),
        };

        self
    }

    /// Adiciona uma condição OR
    pub fn or_filter(mut self, field: &str, operator: &str, value: QueryValue) -> Self {
        let comparison_op = match operator {
            "=" | "eq" => crate::domain::shared::query_parser::ComparisonOperator::Equal,
            "!=" | "ne" => crate::domain::shared::query_parser::ComparisonOperator::NotEqual,
            ">" | "gt" => crate::domain::shared::query_parser::ComparisonOperator::GreaterThan,
            "<" | "lt" => crate::domain::shared::query_parser::ComparisonOperator::LessThan,
            ">=" | "gte" => crate::domain::shared::query_parser::ComparisonOperator::GreaterOrEqual,
            "<=" | "lte" => crate::domain::shared::query_parser::ComparisonOperator::LessOrEqual,
            "~" | "contains" => crate::domain::shared::query_parser::ComparisonOperator::Contains,
            "!~" | "not_contains" => crate::domain::shared::query_parser::ComparisonOperator::NotContains,
            _ => crate::domain::shared::query_parser::ComparisonOperator::Equal,
        };

        let condition = crate::domain::shared::query_parser::FilterCondition {
            field: field.to_string(),
            operator: comparison_op,
            value,
        };

        let new_expression = QueryExpression::Condition(condition);

        self.expression = match self.expression {
            Some(existing) => Some(QueryExpression::Logical {
                operator: crate::domain::shared::query_parser::LogicalOperator::Or,
                left: Box::new(existing),
                right: Some(Box::new(new_expression)),
            }),
            None => Some(new_expression),
        };

        self
    }

    /// Adiciona agregação COUNT
    pub fn count(mut self) -> Self {
        self.aggregation = Some(AggregationType::Count);
        self
    }

    /// Adiciona agregação SUM
    pub fn sum(mut self, field: &str) -> Self {
        self.aggregation = Some(AggregationType::Sum(field.to_string()));
        self
    }

    /// Adiciona agregação AVERAGE
    pub fn average(mut self, field: &str) -> Self {
        self.aggregation = Some(AggregationType::Average(field.to_string()));
        self
    }

    /// Adiciona agregação MIN
    pub fn min(mut self, field: &str) -> Self {
        self.aggregation = Some(AggregationType::Min(field.to_string()));
        self
    }

    /// Adiciona agregação MAX
    pub fn max(mut self, field: &str) -> Self {
        self.aggregation = Some(AggregationType::Max(field.to_string()));
        self
    }

    /// Adiciona ordenação
    pub fn sort_by(mut self, field: &str, ascending: bool) -> Self {
        self.sort = Some(SortOption {
            field: field.to_string(),
            ascending,
        });
        self
    }

    /// Adiciona ordenação ascendente
    pub fn sort_asc(self, field: &str) -> Self {
        self.sort_by(field, true)
    }

    /// Adiciona ordenação descendente
    pub fn sort_desc(self, field: &str) -> Self {
        self.sort_by(field, false)
    }

    /// Adiciona paginação
    pub fn paginate(mut self, limit: Option<usize>, offset: Option<usize>) -> Self {
        self.pagination = PaginationOptions::new(limit, offset);
        self
    }

    /// Adiciona limite
    pub fn limit(mut self, limit: usize) -> Self {
        self.pagination.limit = Some(limit);
        self
    }

    /// Adiciona offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.pagination.offset = Some(offset);
        self
    }

    /// Constrói a query final
    pub fn build(self) -> Result<Query, String> {
        let expression = self.expression.ok_or("Query must have at least one filter condition")?;

        Ok(Query {
            expression,
            aggregation: self.aggregation,
            sort: self.sort,
            pagination: self.pagination,
        })
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions para criar queries comuns
impl QueryBuilder {
    /// Cria uma query para buscar por status
    pub fn status_equals(status: &str) -> Self {
        Self::new().filter("status", "=", QueryValue::String(status.to_string()))
    }

    /// Cria uma query para buscar por prioridade
    pub fn priority_equals(priority: &str) -> Self {
        Self::new().filter("priority", "=", QueryValue::String(priority.to_string()))
    }

    /// Cria uma query para buscar por nome (contains)
    pub fn name_contains(name: &str) -> Self {
        Self::new().filter("name", "~", QueryValue::String(name.to_string()))
    }

    /// Cria uma query para buscar por data de início
    pub fn start_date_after(date: chrono::NaiveDate) -> Self {
        Self::new().filter("start_date", ">", QueryValue::Date(date))
    }

    /// Cria uma query para buscar por data de fim
    pub fn end_date_before(date: chrono::NaiveDate) -> Self {
        Self::new().filter("end_date", "<", QueryValue::Date(date))
    }

    /// Cria uma query para buscar entidades ativas
    pub fn is_active() -> Self {
        Self::new().filter("is_active", "=", QueryValue::Boolean(true))
    }

    /// Cria uma query para buscar entidades com alta prioridade
    pub fn high_priority() -> Self {
        Self::new()
            .filter("priority", "=", QueryValue::String("High".to_string()))
            .or_filter("priority", "=", QueryValue::String("Critical".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_query_builder_basic() {
        let query = QueryBuilder::new()
            .filter("status", "=", QueryValue::String("active".to_string()))
            .filter("priority", ">", QueryValue::String("medium".to_string()))
            .sort_asc("name")
            .limit(10)
            .build()
            .unwrap();

        assert!(query.aggregation.is_none());
        assert!(query.sort.is_some());
        assert_eq!(query.sort.as_ref().unwrap().field, "name");
        assert!(query.sort.as_ref().unwrap().ascending);
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
    fn test_query_builder_or_conditions() {
        let query = QueryBuilder::new()
            .filter("status", "=", QueryValue::String("active".to_string()))
            .or_filter("status", "=", QueryValue::String("pending".to_string()))
            .build()
            .unwrap();

        // Verifica se a estrutura OR foi criada corretamente
        match &query.expression {
            QueryExpression::Logical { operator, .. } => {
                assert!(matches!(operator, crate::domain::shared::query_parser::LogicalOperator::Or));
            }
            _ => panic!("Expected Logical expression with OR operator"),
        }
    }

    #[test]
    fn test_query_builder_helper_functions() {
        let query = QueryBuilder::status_equals("active")
            .sort_desc("created_at")
            .limit(5)
            .build()
            .unwrap();

        assert!(query.sort.is_some());
        assert!(!query.sort.as_ref().unwrap().ascending);
        assert_eq!(query.pagination.limit, Some(5));
    }

    #[test]
    fn test_query_builder_empty_fails() {
        let result = QueryBuilder::new().build();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have at least one filter condition"));
    }

    #[test]
    fn test_query_builder_date_filters() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let query = QueryBuilder::start_date_after(date)
            .end_date_before(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap())
            .build()
            .unwrap();

        // Verifica se as condições de data foram adicionadas
        match &query.expression {
            QueryExpression::Logical { operator, .. } => {
                assert!(matches!(operator, crate::domain::shared::query_parser::LogicalOperator::And));
            }
            _ => panic!("Expected Logical expression with AND operator"),
        }
    }
}
