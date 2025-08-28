use std::fmt;

/// A specification pattern implementation for domain validation
pub trait Specification<T> {
    /// Check if the specification is satisfied by the given item
    fn is_satisfied_by(&self, item: &T) -> bool;
    
    /// Get a description of what this specification checks
    fn description(&self) -> &str;
    
    /// Get a detailed explanation of why the specification failed
    fn explain_why_not_satisfied(&self, item: &T) -> Option<String> {
        if self.is_satisfied_by(item) {
            None
        } else {
            Some(format!("Item does not satisfy: {}", self.description()))
        }
    }
}

/// A composite specification that combines multiple specifications with AND logic
pub struct AndSpecification<T> {
    specifications: Vec<Box<dyn Specification<T>>>,
}

impl<T> AndSpecification<T> {
    pub fn new() -> Self {
        Self {
            specifications: Vec::new(),
        }
    }
    
    pub fn add(mut self, spec: Box<dyn Specification<T>>) -> Self {
        self.specifications.push(spec);
        self
    }
    
    pub fn add_all(mut self, specs: Vec<Box<dyn Specification<T>>>) -> Self {
        self.specifications.extend(specs);
        self
    }
}

impl<T> Specification<T> for AndSpecification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool {
        self.specifications.iter().all(|spec| spec.is_satisfied_by(item))
    }
    
    fn description(&self) -> &str {
        "All specifications must be satisfied"
    }
    
    fn explain_why_not_satisfied(&self, item: &T) -> Option<String> {
        let failed_specs: Vec<String> = self.specifications
            .iter()
            .filter_map(|spec| spec.explain_why_not_satisfied(item))
            .collect();
        
        if failed_specs.is_empty() {
            None
        } else {
            Some(format!("Failed specifications: {}", failed_specs.join("; ")))
        }
    }
}

impl<T> Default for AndSpecification<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A composite specification that combines multiple specifications with OR logic
pub struct OrSpecification<T> {
    specifications: Vec<Box<dyn Specification<T>>>,
}

impl<T> OrSpecification<T> {
    pub fn new() -> Self {
        Self {
            specifications: Vec::new(),
        }
    }
    
    pub fn add(mut self, spec: Box<dyn Specification<T>>) -> Self {
        self.specifications.push(spec);
        self
    }
    
    pub fn add_all(mut self, specs: Vec<Box<dyn Specification<T>>>) -> Self {
        self.specifications.extend(specs);
        self
    }
}

impl<T> Specification<T> for OrSpecification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool {
        self.specifications.iter().any(|spec| spec.is_satisfied_by(item))
    }
    
    fn description(&self) -> &str {
        "At least one specification must be satisfied"
    }
    
    fn explain_why_not_satisfied(&self, item: &T) -> Option<String> {
        if self.is_satisfied_by(item) {
            None
        } else {
            let failed_specs: Vec<String> = self.specifications
                .iter()
                .map(|spec| spec.description().to_string())
                .collect();
            Some(format!("None of the specifications were satisfied: {}", failed_specs.join(", ")))
        }
    }
}

impl<T> Default for OrSpecification<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A specification that negates another specification
pub struct NotSpecification<T> {
    specification: Box<dyn Specification<T>>,
}

impl<T> NotSpecification<T> {
    pub fn new(specification: Box<dyn Specification<T>>) -> Self {
        Self { specification }
    }
}

impl<T> Specification<T> for NotSpecification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool {
        !self.specification.is_satisfied_by(item)
    }
    
    fn description(&self) -> &str {
        "Specification must NOT be satisfied"
    }
    
    fn explain_why_not_satisfied(&self, item: &T) -> Option<String> {
        if self.is_satisfied_by(item) {
            None
        } else {
            Some(format!("Item unexpectedly satisfied: {}", self.specification.description()))
        }
    }
}

/// A specification that always returns true
pub struct AlwaysTrueSpecification;

impl<T> Specification<T> for AlwaysTrueSpecification {
    fn is_satisfied_by(&self, _item: &T) -> bool {
        true
    }
    
    fn description(&self) -> &str {
        "Always satisfied"
    }
}

/// A specification that always returns false
pub struct AlwaysFalseSpecification;

impl<T> Specification<T> for AlwaysFalseSpecification {
    fn is_satisfied_by(&self, _item: &T) -> bool {
        false
    }
    
    fn description(&self) -> &str {
        "Never satisfied"
    }
    
    fn explain_why_not_satisfied(&self, _item: &T) -> Option<String> {
        Some("This specification is never satisfied".to_string())
    }
}

/// Extension trait for easier specification composition
pub trait SpecificationExt<T>: Specification<T> + Sized + 'static {
    /// Combine this specification with another using AND logic
    fn and<S>(self, other: S) -> AndSpecification<T>
    where
        S: Specification<T> + 'static,
    {
        AndSpecification::new()
            .add(Box::new(self))
            .add(Box::new(other))
    }
    
    /// Combine this specification with another using OR logic
    fn or<S>(self, other: S) -> OrSpecification<T>
    where
        S: Specification<T> + 'static,
    {
        OrSpecification::new()
            .add(Box::new(self))
            .add(Box::new(other))
    }
    
    /// Negate this specification
    fn not(self) -> NotSpecification<T> {
        NotSpecification::new(Box::new(self))
    }
}

impl<T, S> SpecificationExt<T> for S where S: Specification<T> + 'static {}

/// A specification that checks if a value is within a range
pub struct RangeSpecification<T> {
    min: Option<T>,
    max: Option<T>,
    description: String,
}

impl<T> RangeSpecification<T>
where
    T: PartialOrd + fmt::Display,
{
    pub fn new(min: Option<T>, max: Option<T>) -> Self {
        let description = match (&min, &max) {
            (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
            (Some(min), None) => format!("Value must be at least {}", min),
            (None, Some(max)) => format!("Value must be at most {}", max),
            (None, None) => "No range constraints".to_string(),
        };
        
        Self {
            min,
            max,
            description,
        }
    }
    
    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }
    
    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }
}

impl<T> Specification<T> for RangeSpecification<T>
where
    T: PartialOrd + fmt::Display,
{
    fn is_satisfied_by(&self, item: &T) -> bool {
        let min_ok = self.min.as_ref().is_none_or(|min| item >= min);
        let max_ok = self.max.as_ref().is_none_or(|max| item <= max);
        min_ok && max_ok
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn explain_why_not_satisfied(&self, item: &T) -> Option<String> {
        if self.is_satisfied_by(item) {
            None
        } else {
            let mut reasons = Vec::new();
            
            if let Some(min) = &self.min
                && item < min {
                reasons.push(format!("Value {} is below minimum {}", item, min));
            }
            
            if let Some(max) = &self.max
                && item > max {
                reasons.push(format!("Value {} is above maximum {}", item, max));
            }
            
            Some(reasons.join("; "))
        }
    }
}
