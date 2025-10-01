#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainResult};

/// A trait for objects that can validate themselves
pub trait Validatable {
    /// Validate the object and return a result
    fn validate(&self) -> DomainResult<()>;

    /// Check if the object is valid without returning an error
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Get validation errors as a vector of strings
    fn validation_errors(&self) -> Vec<String> {
        match self.validate() {
            Ok(()) => Vec::new(),
            Err(err) => vec![err.to_string()],
        }
    }
}

/// A trait for objects that can validate other objects
pub trait Validator<T> {
    /// Validate the given object
    fn validate(&self, item: &T) -> DomainResult<()>;

    /// Check if the given object is valid
    fn is_valid(&self, item: &T) -> bool {
        self.validate(item).is_ok()
    }

    /// Get validation errors for the given object
    fn validation_errors(&self, item: &T) -> Vec<String> {
        match self.validate(item) {
            Ok(()) => Vec::new(),
            Err(err) => vec![err.to_string()],
        }
    }
}

/// A composite validator that combines multiple validators
pub struct CompositeValidator<T> {
    validators: Vec<Box<dyn Validator<T>>>,
}

impl<T> CompositeValidator<T> {
    pub fn new() -> Self {
        Self { validators: Vec::new() }
    }

    pub fn add_validator(mut self, validator: Box<dyn Validator<T>>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn add_all(mut self, validators: Vec<Box<dyn Validator<T>>>) -> Self {
        self.validators.extend(validators);
        self
    }
}

impl<T> Validator<T> for CompositeValidator<T> {
    fn validate(&self, item: &T) -> DomainResult<()> {
        for validator in &self.validators {
            validator.validate(item)?;
        }
        Ok(())
    }
}

impl<T> Default for CompositeValidator<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience implementation for any type that implements Validatable
impl<T: Validatable> Validator<T> for T {
    fn validate(&self, _item: &T) -> DomainResult<()> {
        self.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::errors::{DomainError, DomainResult};

    // Mock structs for testing
    #[derive(Debug, Clone, PartialEq)]
    struct MockValidatable {
        name: String,
        value: i32,
        is_valid: bool,
    }

    impl MockValidatable {
        fn new(name: &str, value: i32, is_valid: bool) -> Self {
            Self {
                name: name.to_string(),
                value,
                is_valid,
            }
        }
    }

    impl Validatable for MockValidatable {
        fn validate(&self) -> DomainResult<()> {
            if self.is_valid {
                Ok(())
            } else {
                Err(DomainError::ValidationError {
                    field: "validatable".to_string(),
                    message: format!("Validation failed for {}", self.name),
                })
            }
        }
    }

    // Mock validators for testing
    struct NameValidator;
    struct ValueValidator;
    struct CompositeTestValidator;

    impl Validator<MockValidatable> for NameValidator {
        fn validate(&self, item: &MockValidatable) -> DomainResult<()> {
            if item.name.is_empty() {
                Err(DomainError::ValidationError {
                    field: "name".to_string(),
                    message: "Name cannot be empty".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }

    impl Validator<MockValidatable> for ValueValidator {
        fn validate(&self, item: &MockValidatable) -> DomainResult<()> {
            if item.value < 0 {
                Err(DomainError::ValidationError {
                    field: "value".to_string(),
                    message: "Value must be non-negative".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }

    impl Validator<MockValidatable> for CompositeTestValidator {
        fn validate(&self, item: &MockValidatable) -> DomainResult<()> {
            if item.name == "invalid" {
                Err(DomainError::ValidationError {
                    field: "name".to_string(),
                    message: "Special invalid name".to_string(),
                })
            } else {
                Ok(())
            }
        }
    }

    // Tests for Validatable trait
    #[test]
    fn test_validatable_validate_success() {
        let valid_item = MockValidatable::new("test", 42, true);
        assert!(<MockValidatable as Validatable>::validate(&valid_item).is_ok());
    }

    #[test]
    fn test_validatable_validate_failure() {
        let invalid_item = MockValidatable::new("test", 42, false);
        let result = <MockValidatable as Validatable>::validate(&invalid_item);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Validation failed for test"));
        }
    }

    #[test]
    fn test_validatable_is_valid_success() {
        let valid_item = MockValidatable::new("test", 42, true);
        assert!(<MockValidatable as Validatable>::is_valid(&valid_item));
    }

    #[test]
    fn test_validatable_is_valid_failure() {
        let invalid_item = MockValidatable::new("test", 42, false);
        assert!(!<MockValidatable as Validatable>::is_valid(&invalid_item));
    }

    #[test]
    fn test_validatable_validation_errors_success() {
        let valid_item = MockValidatable::new("test", 42, true);
        let errors = <MockValidatable as Validatable>::validation_errors(&valid_item);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validatable_validation_errors_failure() {
        let invalid_item = MockValidatable::new("test", 42, false);
        let errors = <MockValidatable as Validatable>::validation_errors(&invalid_item);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Validation failed for test"));
    }

    // Tests for Validator trait
    #[test]
    fn test_validator_validate_success() {
        let validator = NameValidator;
        let item = MockValidatable::new("test", 42, true);
        assert!(validator.validate(&item).is_ok());
    }

    #[test]
    fn test_validator_validate_failure() {
        let validator = NameValidator;
        let item = MockValidatable::new("", 42, true);
        let result = validator.validate(&item);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Name cannot be empty"));
        }
    }

    #[test]
    fn test_validator_is_valid_success() {
        let validator = NameValidator;
        let item = MockValidatable::new("test", 42, true);
        assert!(validator.is_valid(&item));
    }

    #[test]
    fn test_validator_is_valid_failure() {
        let validator = NameValidator;
        let item = MockValidatable::new("", 42, true);
        assert!(!validator.is_valid(&item));
    }

    #[test]
    fn test_validator_validation_errors_success() {
        let validator = NameValidator;
        let item = MockValidatable::new("test", 42, true);
        let errors = validator.validation_errors(&item);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validator_validation_errors_failure() {
        let validator = NameValidator;
        let item = MockValidatable::new("", 42, true);
        let errors = validator.validation_errors(&item);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Name cannot be empty"));
    }

    // Tests for CompositeValidator struct
    #[test]
    fn test_composite_validator_new() {
        let validator = CompositeValidator::<MockValidatable>::new();
        assert!(validator.validators.is_empty());
    }

    #[test]
    fn test_composite_validator_default() {
        let validator = CompositeValidator::<MockValidatable>::default();
        assert!(validator.validators.is_empty());
    }

    #[test]
    fn test_composite_validator_add() {
        let validator = CompositeValidator::<MockValidatable>::new().add_validator(Box::new(NameValidator));
        assert_eq!(validator.validators.len(), 1);
    }

    #[test]
    fn test_composite_validator_add_all() {
        let validators = vec![
            Box::new(NameValidator) as Box<dyn Validator<MockValidatable>>,
            Box::new(ValueValidator) as Box<dyn Validator<MockValidatable>>,
        ];

        let validator = CompositeValidator::<MockValidatable>::new().add_all(validators);
        assert_eq!(validator.validators.len(), 2);
    }

    #[test]
    fn test_composite_validator_validate_success() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("test", 42, true);
        assert!(validator.validate(&item).is_ok());
    }

    #[test]
    fn test_composite_validator_validate_first_failure() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("", 42, true);
        let result = validator.validate(&item);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Name cannot be empty"));
        }
    }

    #[test]
    fn test_composite_validator_validate_second_failure() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("test", -1, true);
        let result = validator.validate(&item);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Value must be non-negative"));
        }
    }

    #[test]
    fn test_composite_validator_validate_multiple_failures() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("", -1, true);
        let result = validator.validate(&item);
        assert!(result.is_err());

        // Should fail on first validation error (NameValidator)
        if let Err(err) = result {
            assert!(err.to_string().contains("Name cannot be empty"));
        }
    }

    #[test]
    fn test_composite_validator_is_valid_success() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("test", 42, true);
        assert!(validator.is_valid(&item));
    }

    #[test]
    fn test_composite_validator_is_valid_failure() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("", 42, true);
        assert!(!validator.is_valid(&item));
    }

    #[test]
    fn test_composite_validator_validation_errors_success() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("test", 42, true);
        let errors = validator.validation_errors(&item);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_composite_validator_validation_errors_failure() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("", 42, true);
        let errors = validator.validation_errors(&item);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Name cannot be empty"));
    }

    #[test]
    fn test_composite_validator_empty() {
        let validator = CompositeValidator::<MockValidatable>::new();
        let item = MockValidatable::new("test", 42, true);
        assert!(validator.validate(&item).is_ok());
        assert!(validator.is_valid(&item));
        assert!(validator.validation_errors(&item).is_empty());
    }

    // Tests for convenience implementation
    #[test]
    fn test_validatable_as_validator() {
        let item = MockValidatable::new("test", 42, true);
        let validator: &dyn Validator<MockValidatable> = &item;

        assert!(validator.validate(&item).is_ok());
        assert!(validator.is_valid(&item));
        assert!(validator.validation_errors(&item).is_empty());
    }

    #[test]
    fn test_validatable_as_validator_failure() {
        let item = MockValidatable::new("test", 42, false);
        let validator: &dyn Validator<MockValidatable> = &item;

        let result = validator.validate(&item);
        assert!(result.is_err());
        assert!(!validator.is_valid(&item));

        let errors = validator.validation_errors(&item);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Validation failed for test"));
    }

    // Tests for complex validation scenarios
    #[test]
    fn test_composite_validator_builder_pattern() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator))
            .add_validator(Box::new(CompositeTestValidator));

        assert_eq!(validator.validators.len(), 3);

        let valid_item = MockValidatable::new("test", 42, true);
        assert!(validator.validate(&valid_item).is_ok());

        let invalid_name_item = MockValidatable::new("invalid", 42, true);
        let result = validator.validate(&invalid_name_item);
        assert!(result.is_err());

        if let Err(err) = result {
            assert!(err.to_string().contains("Special invalid name"));
        }
    }

    #[test]
    fn test_validator_trait_objects() {
        let validators: Vec<Box<dyn Validator<MockValidatable>>> =
            vec![Box::new(NameValidator), Box::new(ValueValidator)];

        let validator = CompositeValidator::<MockValidatable>::new().add_all(validators);

        let item = MockValidatable::new("test", 42, true);
        assert!(validator.validate(&item).is_ok());
    }

    #[test]
    fn test_validation_error_propagation() {
        let validator = CompositeValidator::<MockValidatable>::new()
            .add_validator(Box::new(NameValidator))
            .add_validator(Box::new(ValueValidator));

        let item = MockValidatable::new("", -1, true);
        let result = validator.validate(&item);
        assert!(result.is_err());

        // Should fail on first validation error and not continue
        if let Err(err) = result {
            assert!(err.to_string().contains("Name cannot be empty"));
            assert!(!err.to_string().contains("Value must be non-negative"));
        }
    }
}
