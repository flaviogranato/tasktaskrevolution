/// A specification pattern implementation for domain validation
pub trait Specification<T> {
    /// Check if the specification is satisfied by the given item
    fn is_satisfied_by(&self, item: &T) -> bool;
    
    /// Get a description of what this specification checks
    fn description(&self) -> &str;
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
}

impl<T> Specification<T> for AndSpecification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool {
        self.specifications.iter().all(|spec| spec.is_satisfied_by(item))
    }
    
    fn description(&self) -> &str {
        "All specifications must be satisfied"
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
}

impl<T> Specification<T> for OrSpecification<T> {
    fn is_satisfied_by(&self, item: &T) -> bool {
        self.specifications.iter().any(|spec| spec.is_satisfied_by(item))
    }
    
    fn description(&self) -> &str {
        "At least one specification must be satisfied"
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
        "Negation of the inner specification"
    }
}
