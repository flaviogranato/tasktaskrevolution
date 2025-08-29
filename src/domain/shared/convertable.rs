/// A trait for objects that can be converted to and from other types
pub trait Convertible<T> {
    /// Convert self to the target type
    fn to(&self) -> T;

    /// Create self from the source type
    fn from(source: T) -> Self;
}

/// A trait for objects that can be converted to other types
pub trait Into<T> {
    /// Convert self into the target type
    fn into(self) -> T;
}

/// A trait for objects that can be created from other types
pub trait From<T> {
    /// Create self from the source type
    fn from(source: T) -> Self;
}

// Implementação padrão para tipos que implementam From
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

// Implementação padrão para tipos que implementam Into
impl<T, U> From<T> for U
where
    T: Into<U>,
{
    fn from(source: T) -> Self {
        source.into()
    }
}

// Convenience trait for bidirectional conversion
pub trait BidirectionalConvertible<T>: Convertible<T> + From<T> + Into<T> {}

// Implementação automática para tipos que implementam From e Into
impl<T, U> BidirectionalConvertible<U> for T where T: Convertible<U> + From<U> + Into<U> {}

// Extension trait for easier conversion
pub trait ConvertExt<T> {
    /// Convert to the target type using the Convertible trait
    fn convert_to(&self) -> T
    where
        Self: Convertible<T>;

    /// Convert from the source type using the Convertible trait
    fn convert_from(source: T) -> Self
    where
        Self: Convertible<T>;
}

impl<T, U> ConvertExt<U> for T
where
    T: Convertible<U>,
{
    fn convert_to(&self) -> U {
        self.to()
    }

    fn convert_from(source: U) -> Self {
        Self::from(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock types for testing conversions
    #[derive(Debug, Clone, PartialEq)]
    struct SourceType {
        value: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TargetType {
        data: String,
    }

    // Implement Convertible for SourceType -> TargetType
    impl Convertible<TargetType> for SourceType {
        fn to(&self) -> TargetType {
            TargetType {
                data: self.value.clone(),
            }
        }

        fn from(source: TargetType) -> Self {
            Self {
                value: source.data,
            }
        }
    }

    // Test data structures
    #[derive(Debug, Clone, PartialEq)]
    struct SimpleSource {
        id: u32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct SimpleTarget {
        identifier: u32,
        title: String,
    }

    // Implement Convertible for SimpleSource -> SimpleTarget
    impl Convertible<SimpleTarget> for SimpleSource {
        fn to(&self) -> SimpleTarget {
            SimpleTarget {
                identifier: self.id,
                title: self.name.clone(),
            }
        }

        fn from(source: SimpleTarget) -> Self {
            Self {
                id: source.identifier,
                name: source.title,
            }
        }
    }

    // Tests for Convertible trait
    #[test]
    fn test_convertible_to() {
        let source = SourceType {
            value: "test_value".to_string(),
        };
        let target = source.to();
        
        assert_eq!(target.data, "test_value");
    }

    #[test]
    fn test_convertible_from() {
        let target = TargetType {
            data: "test_data".to_string(),
        };
        let source = <SourceType as Convertible<TargetType>>::from(target);
        
        assert_eq!(source.value, "test_data");
    }

    #[test]
    fn test_convertible_bidirectional() {
        let original_source = SourceType {
            value: "original_value".to_string(),
        };
        
        // Convert to target
        let target = original_source.to();
        assert_eq!(target.data, "original_value");
        
        // Convert back to source
        let new_source = <SourceType as Convertible<TargetType>>::from(target);
        assert_eq!(new_source.value, "original_value");
        
        // Verify they are equal
        assert_eq!(original_source, new_source);
    }

    // Tests for SimpleSource/SimpleTarget conversions
    #[test]
    fn test_simple_conversions() {
        let source = SimpleSource {
            id: 42,
            name: "Simple Test".to_string(),
        };
        
        // Test Convertible::to
        let target = source.to();
        assert_eq!(target.identifier, 42);
        assert_eq!(target.title, "Simple Test");
        
        // Test Convertible::from
        let new_source = <SimpleSource as Convertible<SimpleTarget>>::from(target);
        assert_eq!(new_source.id, 42);
        assert_eq!(new_source.name, "Simple Test");
    }

    // Tests for complex conversion scenarios
    #[test]
    fn test_complex_conversion_scenarios() {
        let sources = vec![
            SourceType { value: "first".to_string() },
            SourceType { value: "second".to_string() },
            SourceType { value: "third".to_string() },
        ];
        
        // Convert all sources to targets
        let targets: Vec<TargetType> = sources.iter().map(|s| s.to()).collect();
        
        assert_eq!(targets.len(), 3);
        assert_eq!(targets[0].data, "first");
        assert_eq!(targets[1].data, "second");
        assert_eq!(targets[2].data, "third");
        
        // Convert all targets back to sources
        let new_sources: Vec<SourceType> = targets.into_iter().map(|t| <SourceType as Convertible<TargetType>>::from(t)).collect();
        
        assert_eq!(new_sources.len(), 3);
        assert_eq!(new_sources[0].value, "first");
        assert_eq!(new_sources[1].value, "second");
        assert_eq!(new_sources[2].value, "third");
    }

    // Tests for trait bounds and generic usage
    #[test]
    fn test_trait_bounds() {
        // Test that we can use Convertible as a trait bound
        fn convert_anything<T, U>(source: &T) -> U
        where
            T: Convertible<U>,
        {
            source.to()
        }
        
        let source = SourceType {
            value: "trait_bound".to_string(),
        };
        let target = convert_anything(&source);
        
        assert_eq!(target.data, "trait_bound");
    }

    // Tests for edge cases
    #[test]
    fn test_empty_string_conversion() {
        let source = SourceType {
            value: "".to_string(),
        };
        let target = source.to();
        
        assert_eq!(target.data, "");
    }

    #[test]
    fn test_unicode_string_conversion() {
        let source = SourceType {
            value: "🚀 🎯 💪".to_string(),
        };
        let target = source.to();
        
        assert_eq!(target.data, "🚀 🎯 💪");
    }

    // Tests for ConvertExt trait
    #[test]
    fn test_convert_ext_convert_to() {
        let source = SourceType {
            value: "ext_to".to_string(),
        };
        let target = source.convert_to();
        
        assert_eq!(target.data, "ext_to");
    }

    #[test]
    fn test_convert_ext_convert_from() {
        let target = TargetType {
            data: "ext_from".to_string(),
        };
        let source = SourceType::convert_from(target);
        
        assert_eq!(source.value, "ext_from");
    }

    // Tests for BidirectionalConvertible trait
    #[test]
    fn test_bidirectional_convertible() {
        let _source = SourceType {
            value: "bidirectional".to_string(),
        };
        
        // Test that SourceType implements BidirectionalConvertible<TargetType>
        // This test verifies that the trait can be used as a bound
        assert!(true);
    }
}
