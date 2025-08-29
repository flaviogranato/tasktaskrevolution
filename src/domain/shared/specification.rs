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
        let failed_specs: Vec<String> = self
            .specifications
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
            let failed_specs: Vec<String> = self
                .specifications
                .iter()
                .map(|spec| spec.description().to_string())
                .collect();
            Some(format!(
                "None of the specifications were satisfied: {}",
                failed_specs.join(", ")
            ))
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
            Some(format!(
                "Item unexpectedly satisfied: {}",
                self.specification.description()
            ))
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
    // 'static necessário para Box<dyn>
    /// Combine this specification with another using AND logic
    fn and<S>(self, other: S) -> AndSpecification<T>
    where
        S: Specification<T> + 'static, // 'static necessário para Box<dyn>
    {
        AndSpecification::new().add(Box::new(self)).add(Box::new(other))
    }

    /// Combine this specification with another using OR logic
    fn or<S>(self, other: S) -> OrSpecification<T>
    where
        S: Specification<T> + 'static, // 'static necessário para Box<dyn>
    {
        OrSpecification::new().add(Box::new(self)).add(Box::new(other))
    }

    /// Negate this specification
    fn not(self) -> NotSpecification<T> {
        NotSpecification::new(Box::new(self))
    }
}

impl<T, S> SpecificationExt<T> for S where S: Specification<T> + 'static {} // 'static necessário para Box<dyn>

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

        Self { min, max, description }
    }

    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self.description = match (&self.min, &self.max) {
            (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
            (Some(min), None) => format!("Value must be at least {}", min),
            (None, Some(max)) => format!("Value must be at most {}", max),
            (None, None) => "No range constraints".to_string(),
        };
        self
    }

    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self.description = match (&self.min, &self.max) {
            (Some(min), Some(max)) => format!("Value must be between {} and {}", min, max),
            (Some(min), None) => format!("Value must be at least {}", min),
            (None, Some(max)) => format!("Value must be at most {}", max),
            (None, None) => "No range constraints".to_string(),
        };
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
                && item < min
            {
                reasons.push(format!("Value {} is below minimum {}", item, min));
            }

            if let Some(max) = &self.max
                && item > max
            {
                reasons.push(format!("Value {} is above maximum {}", item, max));
            }

            Some(reasons.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock entity for testing
    #[derive(Debug, Clone, PartialEq)]
    struct MockEntity {
        id: u32,
        name: String,
        value: i32,
        active: bool,
    }

    impl MockEntity {
        fn new(id: u32, name: &str, value: i32, active: bool) -> Self {
            Self {
                id,
                name: name.to_string(),
                value,
                active,
            }
        }
    }

    // Mock specifications for testing
    struct IdGreaterThanSpec {
        threshold: u32,
        description: String,
    }

    impl IdGreaterThanSpec {
        fn new(threshold: u32) -> Self {
            Self { 
                threshold,
                description: format!("ID must be greater than {}", threshold)
            }
        }
    }

    impl Specification<MockEntity> for IdGreaterThanSpec {
        fn is_satisfied_by(&self, item: &MockEntity) -> bool {
            item.id > self.threshold
        }

        fn description(&self) -> &str {
            &self.description
        }
    }

    struct NameContainsSpec {
        substring: String,
        description: String,
    }

    impl NameContainsSpec {
        fn new(substring: &str) -> Self {
            Self {
                substring: substring.to_string(),
                description: format!("Name must contain '{}'", substring)
            }
        }
    }

    impl Specification<MockEntity> for NameContainsSpec {
        fn is_satisfied_by(&self, item: &MockEntity) -> bool {
            item.name.contains(&self.substring)
        }

        fn description(&self) -> &str {
            &self.description
        }
    }

    struct ValueInRangeSpec {
        min: i32,
        max: i32,
        description: String,
    }

    impl ValueInRangeSpec {
        fn new(min: i32, max: i32) -> Self {
            Self { 
                min, 
                max,
                description: format!("Value must be between {} and {}", min, max)
            }
        }
    }

    impl Specification<MockEntity> for ValueInRangeSpec {
        fn is_satisfied_by(&self, item: &MockEntity) -> bool {
            item.value >= self.min && item.value <= self.max
        }

        fn description(&self) -> &str {
            &self.description
        }
    }

    struct ActiveSpec;

    impl Specification<MockEntity> for ActiveSpec {
        fn is_satisfied_by(&self, item: &MockEntity) -> bool {
            item.active
        }

        fn description(&self) -> &str {
            "Entity must be active"
        }
    }

    // Tests for basic Specification trait
    #[test]
    fn test_specification_trait_basic_functionality() {
        let spec = IdGreaterThanSpec::new(5);
        let entity1 = MockEntity::new(10, "test", 42, true);
        let entity2 = MockEntity::new(3, "test", 42, true);

        assert!(spec.is_satisfied_by(&entity1));
        assert!(!spec.is_satisfied_by(&entity2));
        assert_eq!(spec.description(), "ID must be greater than 5");
    }

    #[test]
    fn test_specification_explain_why_not_satisfied() {
        let spec = IdGreaterThanSpec::new(5);
        let entity = MockEntity::new(3, "test", 42, true);

        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, Some("Item does not satisfy: ID must be greater than 5".to_string()));

        let entity_satisfied = MockEntity::new(10, "test", 42, true);
        let explanation_satisfied = spec.explain_why_not_satisfied(&entity_satisfied);
        assert_eq!(explanation_satisfied, None);
    }

    // Tests for AndSpecification
    #[test]
    fn test_and_specification_new() {
        let spec = AndSpecification::<MockEntity>::new();
        assert_eq!(spec.specifications.len(), 0);
    }

    #[test]
    fn test_and_specification_default() {
        let spec = AndSpecification::<MockEntity>::default();
        assert_eq!(spec.specifications.len(), 0);
    }

    #[test]
    fn test_and_specification_add() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        assert_eq!(spec.specifications.len(), 2);
    }

    #[test]
    fn test_and_specification_add_all() {
        let specs = vec![
            Box::new(IdGreaterThanSpec::new(5)) as Box<dyn Specification<MockEntity>>,
            Box::new(NameContainsSpec::new("test")),
        ];

        let spec = AndSpecification::new().add_all(specs);
        assert_eq!(spec.specifications.len(), 2);
    }

    #[test]
    fn test_and_specification_is_satisfied_by_all_true() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(10, "test entity", 42, true);
        assert!(spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_and_specification_is_satisfied_by_some_false() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(10, "other entity", 42, true);
        assert!(!spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_and_specification_is_satisfied_by_all_false() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(3, "other entity", 42, true);
        assert!(!spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_and_specification_description() {
        let spec = AndSpecification::<MockEntity>::new();
        assert_eq!(spec.description(), "All specifications must be satisfied");
    }

    #[test]
    fn test_and_specification_explain_why_not_satisfied() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(3, "other entity", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("Failed specifications"));
    }

    #[test]
    fn test_and_specification_explain_why_not_satisfied_all_passed() {
        let spec = AndSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(10, "test entity", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, None);
    }

    // Tests for OrSpecification
    #[test]
    fn test_or_specification_new() {
        let spec = OrSpecification::<MockEntity>::new();
        assert_eq!(spec.specifications.len(), 0);
    }

    #[test]
    fn test_or_specification_default() {
        let spec = OrSpecification::<MockEntity>::default();
        assert_eq!(spec.specifications.len(), 0);
    }

    #[test]
    fn test_or_specification_add() {
        let spec = OrSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        assert_eq!(spec.specifications.len(), 2);
    }

    #[test]
    fn test_or_specification_add_all() {
        let specs = vec![
            Box::new(IdGreaterThanSpec::new(5)) as Box<dyn Specification<MockEntity>>,
            Box::new(NameContainsSpec::new("test")),
        ];

        let spec = OrSpecification::new().add_all(specs);
        assert_eq!(spec.specifications.len(), 2);
    }

    #[test]
    fn test_or_specification_is_satisfied_by_any_true() {
        let spec = OrSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(10, "other entity", 42, true);
        assert!(spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_or_specification_is_satisfied_by_all_false() {
        let spec = OrSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(3, "other entity", 42, true);
        assert!(!spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_or_specification_description() {
        let spec = OrSpecification::<MockEntity>::new();
        assert_eq!(spec.description(), "At least one specification must be satisfied");
    }

    #[test]
    fn test_or_specification_explain_why_not_satisfied() {
        let spec = OrSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(3, "other entity", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("None of the specifications were satisfied"));
    }

    #[test]
    fn test_or_specification_explain_why_not_satisfied_some_passed() {
        let spec = OrSpecification::new()
            .add(Box::new(IdGreaterThanSpec::new(5)))
            .add(Box::new(NameContainsSpec::new("test")));

        let entity = MockEntity::new(10, "other entity", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, None);
    }

    // Tests for NotSpecification
    #[test]
    fn test_not_specification_new() {
        let inner_spec = IdGreaterThanSpec::new(5);
        let spec = NotSpecification::new(Box::new(inner_spec));
        assert!(spec.specification.is_satisfied_by(&MockEntity::new(10, "test", 42, true)));
    }

    #[test]
    fn test_not_specification_is_satisfied_by() {
        let inner_spec = IdGreaterThanSpec::new(5);
        let spec = NotSpecification::new(Box::new(inner_spec));

        let entity_above = MockEntity::new(10, "test", 42, true);
        let entity_below = MockEntity::new(3, "test", 42, true);

        assert!(!spec.is_satisfied_by(&entity_above)); // inner spec is true, so not spec is false
        assert!(spec.is_satisfied_by(&entity_below));   // inner spec is false, so not spec is true
    }

    #[test]
    fn test_not_specification_description() {
        let inner_spec = IdGreaterThanSpec::new(5);
        let spec = NotSpecification::new(Box::new(inner_spec));
        assert_eq!(spec.description(), "Specification must NOT be satisfied");
    }

    #[test]
    fn test_not_specification_explain_why_not_satisfied() {
        let inner_spec = IdGreaterThanSpec::new(5);
        let spec = NotSpecification::new(Box::new(inner_spec));

        let entity = MockEntity::new(10, "test", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("Item unexpectedly satisfied"));
    }

    #[test]
    fn test_not_specification_explain_why_not_satisfied_when_satisfied() {
        let inner_spec = IdGreaterThanSpec::new(5);
        let spec = NotSpecification::new(Box::new(inner_spec));

        let entity = MockEntity::new(3, "test", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, None);
    }

    // Tests for AlwaysTrueSpecification
    #[test]
    fn test_always_true_specification_is_satisfied_by() {
        let spec: AlwaysTrueSpecification = AlwaysTrueSpecification;
        let entity = MockEntity::new(1, "test", 42, true);
        assert!(spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_always_true_specification_description() {
        let spec: AlwaysTrueSpecification = AlwaysTrueSpecification;
        assert_eq!(<AlwaysTrueSpecification as Specification<MockEntity>>::description(&spec), "Always satisfied");
    }

    #[test]
    fn test_always_true_specification_explain_why_not_satisfied() {
        let spec: AlwaysTrueSpecification = AlwaysTrueSpecification;
        let entity = MockEntity::new(1, "test", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, None);
    }

    // Tests for AlwaysFalseSpecification
    #[test]
    fn test_always_false_specification_is_satisfied_by() {
        let spec: AlwaysFalseSpecification = AlwaysFalseSpecification;
        let entity = MockEntity::new(1, "test", 42, true);
        assert!(!spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_always_false_specification_description() {
        let spec: AlwaysFalseSpecification = AlwaysFalseSpecification;
        assert_eq!(<AlwaysFalseSpecification as Specification<MockEntity>>::description(&spec), "Never satisfied");
    }

    #[test]
    fn test_always_false_specification_explain_why_not_satisfied() {
        let spec: AlwaysFalseSpecification = AlwaysFalseSpecification;
        let entity = MockEntity::new(1, "test", 42, true);
        let explanation = spec.explain_why_not_satisfied(&entity);
        assert_eq!(explanation, Some("This specification is never satisfied".to_string()));
    }

    // Tests for SpecificationExt trait
    #[test]
    fn test_specification_ext_and() {
        let spec1 = IdGreaterThanSpec::new(5);
        let spec2 = NameContainsSpec::new("test");
        let combined = spec1.and(spec2);

        let entity = MockEntity::new(10, "test entity", 42, true);
        assert!(combined.is_satisfied_by(&entity));
    }

    #[test]
    fn test_specification_ext_or() {
        let spec1 = IdGreaterThanSpec::new(5);
        let spec2 = NameContainsSpec::new("test");
        let combined = spec1.or(spec2);

        let entity = MockEntity::new(10, "other entity", 42, true);
        assert!(combined.is_satisfied_by(&entity));
    }

    #[test]
    fn test_specification_ext_not() {
        let spec = IdGreaterThanSpec::new(5);
        let negated = spec.not();

        let entity = MockEntity::new(3, "test", 42, true);
        assert!(negated.is_satisfied_by(&entity));
    }

    // Tests for RangeSpecification
    #[test]
    fn test_range_specification_new_with_min_and_max() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert_eq!(spec.description(), "Value must be between 5 and 10");
    }

    #[test]
    fn test_range_specification_new_with_min_only() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), None);
        assert_eq!(spec.description(), "Value must be at least 5");
    }

    #[test]
    fn test_range_specification_new_with_max_only() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(None, Some(10));
        assert_eq!(spec.description(), "Value must be at most 10");
    }

    #[test]
    fn test_range_specification_new_with_no_constraints() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(None, None);
        assert_eq!(spec.description(), "No range constraints");
    }

    #[test]
    fn test_range_specification_min() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(None, None).min(5);
        assert_eq!(spec.description(), "Value must be at least 5");
    }

    #[test]
    fn test_range_specification_max() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(None, None).max(10);
        assert_eq!(spec.description(), "Value must be at most 10");
    }

    #[test]
    fn test_range_specification_is_satisfied_by_within_range() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert!(spec.is_satisfied_by(&7));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_at_min() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert!(spec.is_satisfied_by(&5));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_at_max() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert!(spec.is_satisfied_by(&10));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_below_min() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert!(!spec.is_satisfied_by(&3));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_above_max() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        assert!(!spec.is_satisfied_by(&15));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_min_only() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), None);
        assert!(spec.is_satisfied_by(&7));
    }

    #[test]
    fn test_range_specification_is_satisfied_by_max_only() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(None, Some(10));
        assert!(spec.is_satisfied_by(&7));
    }

    #[test]
    fn test_range_specification_explain_why_not_satisfied_below_min() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        let explanation = spec.explain_why_not_satisfied(&3);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("Value 3 is below minimum 5"));
    }

    #[test]
    fn test_range_specification_explain_why_not_satisfied_above_max() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        let explanation = spec.explain_why_not_satisfied(&15);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("Value 15 is above maximum 10"));
    }

    #[test]
    fn test_range_specification_explain_why_not_satisfied_both_violations() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        let explanation = spec.explain_why_not_satisfied(&3);
        assert!(explanation.is_some());
        assert!(explanation.unwrap().contains("Value 3 is below minimum 5"));
    }

    #[test]
    fn test_range_specification_explain_why_not_satisfied_when_satisfied() {
        let spec: RangeSpecification<i32> = RangeSpecification::new(Some(5), Some(10));
        let explanation = spec.explain_why_not_satisfied(&7);
        assert_eq!(explanation, None);
    }

    // Complex composition tests
    #[test]
    fn test_complex_specification_composition() {
        let spec = IdGreaterThanSpec::new(5)
            .and(ActiveSpec)
            .or(NameContainsSpec::new("admin"))
            .not();

        let entity1 = MockEntity::new(10, "user", 42, true);
        let entity2 = MockEntity::new(3, "admin", 42, false);
        let entity3 = MockEntity::new(10, "admin", 42, true);

        // entity1: id > 5 AND active = true, so (true AND true) OR false = true, so NOT true = false
        assert!(!spec.is_satisfied_by(&entity1));
        
        // entity2: id > 5 AND active = false, so (false AND false) OR true = true, so NOT true = false
        assert!(!spec.is_satisfied_by(&entity2));
        
        // entity3: id > 5 AND active = true, so (true AND true) OR true = true, so NOT true = false
        assert!(!spec.is_satisfied_by(&entity3));
    }

    #[test]
    fn test_empty_and_specification() {
        let spec = AndSpecification::<MockEntity>::new();
        let entity = MockEntity::new(1, "test", 42, true);
        // Empty AND specification should be satisfied (vacuous truth)
        assert!(spec.is_satisfied_by(&entity));
    }

    #[test]
    fn test_empty_or_specification() {
        let spec = OrSpecification::<MockEntity>::new();
        let entity = MockEntity::new(1, "test", 42, true);
        // Empty OR specification should not be satisfied
        assert!(!spec.is_satisfied_by(&entity));
    }
}
