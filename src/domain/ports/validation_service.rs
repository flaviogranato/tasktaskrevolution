//! Validation service port for domain validation
//!
//! This module defines the validation interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::DomainResult;
use std::collections::HashMap;

/// Validation result for a field
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub field: String,
    pub is_valid: bool,
    pub message: Option<String>,
    pub severity: ValidationSeverity,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Validation rule for a field
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, String>,
    pub message: Option<String>,
}

/// Types of validation rules
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationRuleType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Email,
    Url,
    Numeric,
    Date,
    DateTime,
    Custom(String),
}

/// Validation service port for validating domain data
pub trait ValidationServicePort: Send + Sync {
    /// Validate a single field
    fn validate_field(&self, field: &str, value: &str, rules: &[ValidationRule]) -> DomainResult<ValidationResult>;

    /// Validate multiple fields
    fn validate_fields(
        &self,
        data: &HashMap<String, String>,
        rules: &[ValidationRule],
    ) -> DomainResult<Vec<ValidationResult>>;

    /// Validate an entity
    fn validate_entity(&self, entity_type: &str, data: &HashMap<String, String>)
    -> DomainResult<Vec<ValidationResult>>;

    /// Check if validation is available
    fn is_available(&self) -> bool;

    /// Get supported validation rules
    fn get_supported_rules(&self) -> Vec<ValidationRuleType>;

    /// Register a custom validation rule
    fn register_custom_rule(&self, name: &str, rule: Box<dyn CustomValidationRule>) -> DomainResult<()>;
}

/// Custom validation rule
pub trait CustomValidationRule: Send + Sync {
    /// Validate a value
    fn validate(&self, value: &str, parameters: &HashMap<String, String>) -> DomainResult<bool>;

    /// Get the rule name
    fn name(&self) -> &str;

    /// Get the rule description
    fn description(&self) -> &str;
}
