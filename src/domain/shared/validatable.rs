use crate::domain::shared::errors::DomainError;

/// A trait for objects that can validate themselves
pub trait Validatable {
    /// Validate the object and return a result
    fn validate(&self) -> Result<(), DomainError>;

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
    fn validate(&self, item: &T) -> Result<(), DomainError>;

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

    pub fn add(mut self, validator: Box<dyn Validator<T>>) -> Self {
        self.validators.push(validator);
        self
    }

    pub fn add_all(mut self, validators: Vec<Box<dyn Validator<T>>>) -> Self {
        self.validators.extend(validators);
        self
    }
}

impl<T> Validator<T> for CompositeValidator<T> {
    fn validate(&self, item: &T) -> Result<(), DomainError> {
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
    fn validate(&self, _item: &T) -> Result<(), DomainError> {
        self.validate()
    }
}
