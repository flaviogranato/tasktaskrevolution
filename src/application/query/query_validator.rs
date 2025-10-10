use super::query_executor::EntityType;
use crate::application::errors::AppError;
use crate::domain::shared::query_parser::{AggregationType, FilterCondition, Query, QueryExpression};

/// Validador de queries
pub struct QueryValidator;

impl QueryValidator {
    /// Valida uma query para um tipo específico de entidade
    pub fn validate_query(query: &Query, entity_type: EntityType) -> Result<(), AppError> {
        // Valida a expressão da query
        Self::validate_expression(&query.expression, entity_type)?;

        // Valida agregação se especificada
        if let Some(aggregation) = &query.aggregation {
            Self::validate_aggregation(aggregation, entity_type)?;
        }

        // Valida ordenação se especificada
        if let Some(sort) = &query.sort {
            Self::validate_sort_field(&sort.field, entity_type)?;
        }

        Ok(())
    }

    /// Valida uma expressão de query
    fn validate_expression(expression: &QueryExpression, entity_type: EntityType) -> Result<(), AppError> {
        match expression {
            QueryExpression::Condition(condition) => {
                Self::validate_condition(condition, entity_type)?;
            }
            QueryExpression::Logical { left, right, .. } => {
                Self::validate_expression(left, entity_type)?;
                if let Some(right_expr) = right {
                    Self::validate_expression(right_expr, entity_type)?;
                }
            }
            QueryExpression::Not(expr) => {
                Self::validate_expression(expr, entity_type)?;
            }
        }
        Ok(())
    }

    /// Valida uma condição de filtro
    fn validate_condition(condition: &FilterCondition, entity_type: EntityType) -> Result<(), AppError> {
        let available_fields = Self::get_available_fields(entity_type);

        if !available_fields.contains(&condition.field) {
            return Err(AppError::validation_error(
                "field",
                format!(
                    "Field '{}' is not valid for entity type '{}'. Available fields: {}",
                    condition.field,
                    entity_type,
                    available_fields.join(", ")
                ),
            ));
        }

        // Validação específica por tipo de campo
        Self::validate_field_value(&condition.field, &condition.value, entity_type)?;

        Ok(())
    }

    /// Valida uma agregação
    fn validate_aggregation(aggregation: &AggregationType, entity_type: EntityType) -> Result<(), AppError> {
        match aggregation {
            AggregationType::Count => Ok(()), // COUNT sempre é válido
            AggregationType::Sum(field)
            | AggregationType::Average(field)
            | AggregationType::Min(field)
            | AggregationType::Max(field) => {
                let available_fields = Self::get_available_fields(entity_type);
                if !available_fields.contains(&field.to_string()) {
                    return Err(AppError::validation_error(
                        "aggregation_field",
                        format!(
                            "Field '{}' is not valid for aggregation on entity type '{}'",
                            field, entity_type
                        ),
                    ));
                }

                // Verifica se o campo é numérico
                if !Self::is_numeric_field(field, entity_type) {
                    return Err(AppError::validation_error(
                        "aggregation_field",
                        format!("Field '{}' is not numeric and cannot be used for aggregation", field),
                    ));
                }

                Ok(())
            }
        }
    }

    /// Valida um campo de ordenação
    fn validate_sort_field(field: &str, entity_type: EntityType) -> Result<(), AppError> {
        let available_fields = Self::get_available_fields(entity_type);

        if !available_fields.contains(&field.to_string()) {
            return Err(AppError::validation_error(
                "sort_field",
                format!(
                    "Field '{}' is not valid for sorting on entity type '{}'. Available fields: {}",
                    field,
                    entity_type,
                    available_fields.join(", ")
                ),
            ));
        }

        Ok(())
    }

    /// Valida um valor de campo
    fn validate_field_value(
        field: &str,
        value: &crate::domain::shared::query_parser::QueryValue,
        entity_type: EntityType,
    ) -> Result<(), AppError> {
        // Validações específicas por tipo de campo
        match field {
            "status" => {
                if let crate::domain::shared::query_parser::QueryValue::String(status) = value
                    && !Self::is_valid_status(status, entity_type)
                {
                    return Err(AppError::validation_error(
                        "status",
                        format!("Invalid status '{}' for entity type '{}'", status, entity_type),
                    ));
                }
            }
            "priority" => {
                if let crate::domain::shared::query_parser::QueryValue::String(priority) = value
                    && !Self::is_valid_priority(priority, entity_type)
                {
                    return Err(AppError::validation_error(
                        "priority",
                        format!("Invalid priority '{}' for entity type '{}'", priority, entity_type),
                    ));
                }
            }
            _ => {} // Outros campos não têm validação específica
        }

        Ok(())
    }

    /// Retorna os campos disponíveis para um tipo de entidade
    fn get_available_fields(entity_type: EntityType) -> Vec<String> {
        match entity_type {
            EntityType::Project => vec![
                "id".to_string(),
                "code".to_string(),
                "name".to_string(),
                "description".to_string(),
                "status".to_string(),
                "priority".to_string(),
                "start_date".to_string(),
                "end_date".to_string(),
                "actual_start_date".to_string(),
                "actual_end_date".to_string(),
                "company_code".to_string(),
                "manager_id".to_string(),
                "created_by".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
                "task_count".to_string(),
                "resource_count".to_string(),
                "is_active".to_string(),
                "priority_weight".to_string(),
            ],
            EntityType::Task => vec![
                "id".to_string(),
                "project_code".to_string(),
                "code".to_string(),
                "name".to_string(),
                "description".to_string(),
                "status".to_string(),
                "start_date".to_string(),
                "due_date".to_string(),
                "actual_end_date".to_string(),
                "priority".to_string(),
                "category".to_string(),
                "dependency_count".to_string(),
                "assigned_resource_count".to_string(),
                "is_overdue".to_string(),
                "days_until_due".to_string(),
                "priority_weight".to_string(),
            ],
            EntityType::Resource => vec![
                "id".to_string(),
                "code".to_string(),
                "name".to_string(),
                "email".to_string(),
                "resource_type".to_string(),
                "scope".to_string(),
                "project_id".to_string(),
                "start_date".to_string(),
                "end_date".to_string(),
                "time_off_balance".to_string(),
                "vacation_count".to_string(),
                "time_off_history_count".to_string(),
                "task_assignment_count".to_string(),
                "active_task_count".to_string(),
                "current_allocation_percentage".to_string(),
                "is_wip_limits_exceeded".to_string(),
                "wip_status".to_string(),
                "is_available".to_string(),
            ],
        }
    }

    /// Verifica se um campo é numérico
    fn is_numeric_field(field: &str, entity_type: EntityType) -> bool {
        let numeric_fields = match entity_type {
            EntityType::Project => vec!["task_count", "resource_count", "priority_weight"],
            EntityType::Task => vec![
                "dependency_count",
                "assigned_resource_count",
                "days_until_due",
                "priority_weight",
            ],
            EntityType::Resource => vec![
                "time_off_balance",
                "vacation_count",
                "time_off_history_count",
                "task_assignment_count",
                "active_task_count",
                "current_allocation_percentage",
            ],
        };

        numeric_fields.contains(&field)
    }

    /// Verifica se um status é válido para o tipo de entidade
    fn is_valid_status(status: &str, entity_type: EntityType) -> bool {
        match entity_type {
            EntityType::Project => {
                matches!(
                    status,
                    "Planned" | "In Progress" | "On Hold" | "Completed" | "Cancelled"
                )
            }
            EntityType::Task => {
                matches!(
                    status,
                    "Planned" | "In Progress" | "Blocked" | "Completed" | "Cancelled"
                )
            }
            EntityType::Resource => {
                matches!(status, "Available" | "Assigned" | "Inactive")
            }
        }
    }

    /// Verifica se uma prioridade é válida para o tipo de entidade
    fn is_valid_priority(priority: &str, entity_type: EntityType) -> bool {
        match entity_type {
            EntityType::Project | EntityType::Task => {
                matches!(priority, "Low" | "Medium" | "High" | "Critical")
            }
            EntityType::Resource => {
                // Recursos não têm prioridade
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::query_parser::{
        AggregationType, ComparisonOperator, FilterCondition, Query, QueryExpression, QueryValue,
    };

    #[test]
    fn test_validate_valid_query() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_field() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "invalid_field".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("value".to_string()),
            }),
            aggregation: None,
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not valid for entity type"));
    }

    #[test]
    fn test_validate_aggregation_count() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: Some(AggregationType::Count),
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_aggregation_numeric_field() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: Some(AggregationType::Sum("task_count".to_string())),
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_aggregation_non_numeric_field() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: Some(AggregationType::Sum("name".to_string())),
            sort: None,
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not numeric"));
    }

    #[test]
    fn test_validate_sort_field() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: Some(crate::domain::shared::query_parser::SortOption {
                field: "name".to_string(),
                ascending: true,
            }),
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_sort_field() {
        let query = Query {
            expression: QueryExpression::Condition(FilterCondition {
                field: "status".to_string(),
                operator: ComparisonOperator::Equal,
                value: QueryValue::String("active".to_string()),
            }),
            aggregation: None,
            sort: Some(crate::domain::shared::query_parser::SortOption {
                field: "invalid_field".to_string(),
                ascending: true,
            }),
            pagination: crate::domain::shared::query_parser::PaginationOptions::new_default(),
        };

        let result = QueryValidator::validate_query(&query, EntityType::Project);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not valid for sorting"));
    }
}
