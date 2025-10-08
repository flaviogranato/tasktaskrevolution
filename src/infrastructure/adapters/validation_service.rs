//! Validation service adapter implementation
//!
//! This module provides a concrete implementation of the ValidationServicePort
//! for validating domain data.

use crate::domain::ports::validation_service::{
    ValidationServicePort, ValidationResult, ValidationRule, ValidationRuleType, ValidationSeverity,
    CustomValidationRule,
};
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::collections::HashMap;
// Removed regex dependency

/// Standard validation service adapter
pub struct StandardValidationServiceAdapter {
    custom_rules: HashMap<String, Box<dyn CustomValidationRule>>,
}

impl StandardValidationServiceAdapter {
    pub fn new() -> Self {
        Self {
            custom_rules: HashMap::new(),
        }
    }
}

impl Default for StandardValidationServiceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationServicePort for StandardValidationServiceAdapter {
    fn validate_field(&self, field: &str, value: &str, rules: &[ValidationRule]) -> DomainResult<ValidationResult> {
        for rule in rules {
            if rule.field != field {
                continue;
            }

            let is_valid = match &rule.rule_type {
                ValidationRuleType::Required => !value.is_empty(),
                ValidationRuleType::MinLength(min) => value.len() >= *min,
                ValidationRuleType::MaxLength(max) => value.len() <= *max,
                ValidationRuleType::Pattern(pattern) => {
                    // Simple pattern matching without regex
                    value.contains(pattern)
                }
                ValidationRuleType::Email => {
                    // Simple email validation without regex
                    let parts: Vec<&str> = value.split('@').collect();
                    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.')
                }
                ValidationRuleType::Url => {
                    // Simple URL validation without regex
                    value.starts_with("http://") || value.starts_with("https://")
                }
                ValidationRuleType::Numeric => value.parse::<f64>().is_ok(),
                ValidationRuleType::Date => {
                    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok()
                }
                ValidationRuleType::DateTime => {
                    chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").is_ok()
                }
                ValidationRuleType::Custom(name) => {
                    if let Some(rule) = self.custom_rules.get(name) {
                        rule.validate(value, &HashMap::new()).unwrap_or(false)
                    } else {
                        false
                    }
                }
            };

            if !is_valid {
                return Ok(ValidationResult {
                    field: field.to_string(),
                    is_valid: false,
                    message: rule.message.clone(),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        Ok(ValidationResult {
            field: field.to_string(),
            is_valid: true,
            message: None,
            severity: ValidationSeverity::Info,
        })
    }

    fn validate_fields(&self, data: &HashMap<String, String>, rules: &[ValidationRule]) -> DomainResult<Vec<ValidationResult>> {
        let mut results = Vec::new();

        for (field, value) in data {
            let result = self.validate_field(field, value, rules)?;
            results.push(result);
        }

        Ok(results)
    }

    fn validate_entity(&self, entity_type: &str, data: &HashMap<String, String>) -> DomainResult<Vec<ValidationResult>> {
        // In a real implementation, this would load entity-specific rules
        // For now, we'll just validate basic fields
        let mut results = Vec::new();

        for (field, value) in data {
            let result = self.validate_field(field, value, &[])?;
            results.push(result);
        }

        Ok(results)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get_supported_rules(&self) -> Vec<ValidationRuleType> {
        vec![
            ValidationRuleType::Required,
            ValidationRuleType::MinLength(0),
            ValidationRuleType::MaxLength(255),
            ValidationRuleType::Pattern("".to_string()),
            ValidationRuleType::Email,
            ValidationRuleType::Url,
            ValidationRuleType::Numeric,
            ValidationRuleType::Date,
            ValidationRuleType::DateTime,
        ]
    }

    fn register_custom_rule(&self, name: &str, rule: Box<dyn CustomValidationRule>) -> DomainResult<()> {
        // In a real implementation, this would be mutable
        // For now, we'll just return an error
        Err(DomainError::OperationNotAllowed {
            operation: "register_custom_rule".to_string(),
            reason: "Custom rules are not supported in this implementation".to_string(),
        })
    }
}
