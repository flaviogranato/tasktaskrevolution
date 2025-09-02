use std::collections::HashMap;
use uuid7::uuid7;

/// A factory trait for creating domain entities
pub trait EntityFactory<T, P = ()> {
    /// Create a new entity with the given parameters
    fn create(&self, params: P) -> T;

    /// Create a new entity with default parameters
    fn create_default(&self) -> T
    where
        P: Default;
}

/// A factory trait for creating entities with validation
pub trait ValidatedEntityFactory<T, P = ()> {
    type Error;

    /// Create a new entity with validation
    fn create_validated(&self, params: P) -> Result<T, Self::Error>;
}

/// A factory for creating entities with specific configurations
pub trait ConfigurableEntityFactory<T, P = ()> {
    /// Create an entity with configuration
    fn create_with_config(&self, params: P, config: HashMap<String, String>) -> T;
}

/// A factory registry that can hold multiple factories
pub struct FactoryRegistry<T, P = ()> {
    factories: HashMap<String, Box<dyn EntityFactory<T, P>>>,
}

impl<T, P> FactoryRegistry<T, P> {
    /// Create a new factory registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a factory with a name
    pub fn register<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: EntityFactory<T, P> + 'static, // 'static necessário para Box<dyn>
    {
        self.factories.insert(name.into(), Box::new(factory));
    }

    /// Get a factory by name
    pub fn get(&self, name: &str) -> Option<&dyn EntityFactory<T, P>> {
        self.factories.get(name).map(|f| f.as_ref())
    }

    /// Create an entity using the specified factory
    pub fn create(&self, factory_name: &str, params: P) -> Option<T> {
        self.get(factory_name).map(|f| f.create(params))
    }
}

impl<T, P> Default for FactoryRegistry<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple factory implementation for basic entity creation
pub struct SimpleFactory<F, T, P> {
    creator: F,
    _phantom: std::marker::PhantomData<(T, P)>,
}

impl<F, T, P> SimpleFactory<F, T, P>
where
    F: Fn(P) -> T,
{
    /// Create a new simple factory
    pub fn new(creator: F) -> Self {
        Self {
            creator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T, P> EntityFactory<T, P> for SimpleFactory<F, T, P>
where
    F: Fn(P) -> T,
{
    fn create(&self, params: P) -> T {
        (self.creator)(params)
    }

    fn create_default(&self) -> T
    where
        P: Default,
    {
        (self.creator)(Default::default())
    }
}

/// A factory that creates entities with unique IDs
pub struct UniqueIdFactory<F, T, P> {
    creator: F,
    _phantom: std::marker::PhantomData<(T, P)>,
}

impl<F, T, P> UniqueIdFactory<F, T, P>
where
    F: Fn(P, uuid7::Uuid) -> T,
{
    /// Create a new unique ID factory
    pub fn new(creator: F) -> Self {
        Self {
            creator,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<F, T, P> EntityFactory<T, P> for UniqueIdFactory<F, T, P>
where
    F: Fn(P, uuid7::Uuid) -> T,
    P: Default,
{
    fn create(&self, params: P) -> T {
        (self.creator)(params, uuid7())
    }

    fn create_default(&self) -> T {
        (self.creator)(Default::default(), uuid7())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock entity types for testing
    #[derive(Debug, Clone, PartialEq)]
    struct MockEntity {
        id: String,
        name: String,
        value: u32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct MockParams {
        name: String,
        value: u32,
    }

    impl Default for MockParams {
        fn default() -> Self {
            Self {
                name: "default".to_string(),
                value: 42,
            }
        }
    }

    // Mock factory implementations for testing
    struct MockEntityFactory;

    impl EntityFactory<MockEntity, MockParams> for MockEntityFactory {
        fn create(&self, params: MockParams) -> MockEntity {
            MockEntity {
                id: "mock-001".to_string(),
                name: params.name,
                value: params.value,
            }
        }

        fn create_default(&self) -> MockEntity {
            MockEntity {
                id: "mock-default".to_string(),
                name: "default".to_string(),
                value: 42,
            }
        }
    }

    struct MockValidatedFactory;

    impl ValidatedEntityFactory<MockEntity, MockParams> for MockValidatedFactory {
        type Error = String;

        fn create_validated(&self, params: MockParams) -> Result<MockEntity, Self::Error> {
            if params.value > 100 {
                Err("Value too high".to_string())
            } else {
                Ok(MockEntity {
                    id: "validated-001".to_string(),
                    name: params.name,
                    value: params.value,
                })
            }
        }
    }

    struct MockConfigurableFactory;

    impl ConfigurableEntityFactory<MockEntity, MockParams> for MockConfigurableFactory {
        fn create_with_config(&self, params: MockParams, config: HashMap<String, String>) -> MockEntity {
            let default_prefix = "config".to_string();
            let prefix = config.get("prefix").unwrap_or(&default_prefix);
            MockEntity {
                id: format!("{}-001", prefix),
                name: params.name,
                value: params.value,
            }
        }
    }

    // Tests for EntityFactory trait
    #[test]
    fn test_entity_factory_create() {
        let factory = MockEntityFactory;
        let params = MockParams {
            name: "test".to_string(),
            value: 123,
        };

        let entity = factory.create(params);

        assert_eq!(entity.id, "mock-001");
        assert_eq!(entity.name, "test");
        assert_eq!(entity.value, 123);
    }

    #[test]
    fn test_entity_factory_create_default() {
        let factory = MockEntityFactory;
        let entity = factory.create_default();

        assert_eq!(entity.id, "mock-default");
        assert_eq!(entity.name, "default");
        assert_eq!(entity.value, 42);
    }

    // Tests for ValidatedEntityFactory trait
    #[test]
    fn test_validated_entity_factory_success() {
        let factory = MockValidatedFactory;
        let params = MockParams {
            name: "valid".to_string(),
            value: 50,
        };

        let result = factory.create_validated(params);
        assert!(result.is_ok());

        let entity = result.unwrap();
        assert_eq!(entity.id, "validated-001");
        assert_eq!(entity.name, "valid");
        assert_eq!(entity.value, 50);
    }

    #[test]
    fn test_validated_entity_factory_failure() {
        let factory = MockValidatedFactory;
        let params = MockParams {
            name: "invalid".to_string(),
            value: 150,
        };

        let result = factory.create_validated(params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Value too high");
    }

    // Tests for ConfigurableEntityFactory trait
    #[test]
    fn test_configurable_entity_factory_with_config() {
        let factory = MockConfigurableFactory;
        let params = MockParams {
            name: "config".to_string(),
            value: 75,
        };
        let mut config = HashMap::new();
        config.insert("prefix".to_string(), "custom".to_string());

        let entity = factory.create_with_config(params, config);

        assert_eq!(entity.id, "custom-001");
        assert_eq!(entity.name, "config");
        assert_eq!(entity.value, 75);
    }

    #[test]
    fn test_configurable_entity_factory_without_config() {
        let factory = MockConfigurableFactory;
        let params = MockParams {
            name: "no-config".to_string(),
            value: 25,
        };
        let config = HashMap::new();

        let entity = factory.create_with_config(params, config);

        assert_eq!(entity.id, "config-001");
        assert_eq!(entity.name, "no-config");
        assert_eq!(entity.value, 25);
    }

    // Tests for FactoryRegistry
    #[test]
    fn test_factory_registry_new() {
        let registry: FactoryRegistry<MockEntity, MockParams> = FactoryRegistry::new();
        assert_eq!(registry.factories.len(), 0);
    }

    #[test]
    fn test_factory_registry_default() {
        let registry: FactoryRegistry<MockEntity, MockParams> = FactoryRegistry::default();
        assert_eq!(registry.factories.len(), 0);
    }

    #[test]
    fn test_factory_registry_register_and_get() {
        let mut registry = FactoryRegistry::new();
        let factory = MockEntityFactory;

        registry.register("mock", factory);

        let retrieved_factory = registry.get("mock");
        assert!(retrieved_factory.is_some());

        let retrieved_factory = retrieved_factory.unwrap();
        let entity = retrieved_factory.create(MockParams {
            name: "retrieved".to_string(),
            value: 99,
        });

        assert_eq!(entity.id, "mock-001");
        assert_eq!(entity.name, "retrieved");
        assert_eq!(entity.value, 99);
    }

    #[test]
    fn test_factory_registry_get_nonexistent() {
        let registry: FactoryRegistry<MockEntity, MockParams> = FactoryRegistry::new();
        let factory = registry.get("nonexistent");
        assert!(factory.is_none());
    }

    #[test]
    fn test_factory_registry_create() {
        let mut registry = FactoryRegistry::new();
        let factory = MockEntityFactory;

        registry.register("mock", factory);

        let entity = registry.create(
            "mock",
            MockParams {
                name: "created".to_string(),
                value: 88,
            },
        );

        assert!(entity.is_some());
        let entity = entity.unwrap();
        assert_eq!(entity.id, "mock-001");
        assert_eq!(entity.name, "created");
        assert_eq!(entity.value, 88);
    }

    #[test]
    fn test_factory_registry_create_nonexistent() {
        let registry: FactoryRegistry<MockEntity, MockParams> = FactoryRegistry::new();
        let entity = registry.create(
            "nonexistent",
            MockParams {
                name: "test".to_string(),
                value: 0,
            },
        );
        assert!(entity.is_none());
    }

    // Tests for SimpleFactory
    #[test]
    fn test_simple_factory_new() {
        let creator = |params: MockParams| MockEntity {
            id: "simple".to_string(),
            name: params.name,
            value: params.value,
        };

        let factory = SimpleFactory::new(creator);
        let params = MockParams {
            name: "simple_test".to_string(),
            value: 33,
        };

        let entity = factory.create(params);
        assert_eq!(entity.id, "simple");
        assert_eq!(entity.name, "simple_test");
        assert_eq!(entity.value, 33);
    }

    #[test]
    fn test_simple_factory_create_default() {
        let creator = |params: MockParams| MockEntity {
            id: "simple_default".to_string(),
            name: params.name,
            value: params.value,
        };

        let factory = SimpleFactory::new(creator);
        let entity = factory.create_default();

        assert_eq!(entity.id, "simple_default");
        assert_eq!(entity.name, "default");
        assert_eq!(entity.value, 42);
    }

    // Tests for UniqueIdFactory
    #[test]
    fn test_unique_id_factory_new() {
        let creator = |params: MockParams, id: uuid7::Uuid| MockEntity {
            id: id.to_string(),
            name: params.name,
            value: params.value,
        };

        let factory = UniqueIdFactory::new(creator);
        let params = MockParams {
            name: "unique".to_string(),
            value: 55,
        };

        let entity = factory.create(params);
        assert!(!entity.id.is_empty());
        assert_eq!(entity.name, "unique");
        assert_eq!(entity.value, 55);
    }

    #[test]
    fn test_unique_id_factory_create_default() {
        let creator = |params: MockParams, id: uuid7::Uuid| MockEntity {
            id: id.to_string(),
            name: params.name,
            value: params.value,
        };

        let factory = UniqueIdFactory::new(creator);
        let entity = factory.create_default();

        assert!(!entity.id.is_empty());
        assert_eq!(entity.name, "default");
        assert_eq!(entity.value, 42);
    }

    #[test]
    fn test_unique_id_factory_generates_different_ids() {
        let creator = |params: MockParams, id: uuid7::Uuid| MockEntity {
            id: id.to_string(),
            name: params.name,
            value: params.value,
        };

        let factory = UniqueIdFactory::new(creator);
        let params = MockParams {
            name: "unique_test".to_string(),
            value: 10,
        };

        let entity1 = factory.create(params.clone());
        let entity2 = factory.create(params);

        assert_ne!(entity1.id, entity2.id);
        assert_eq!(entity1.name, entity2.name);
        assert_eq!(entity1.value, entity2.value);
    }

    // Tests for complex scenarios
    #[test]
    fn test_factory_registry_multiple_factories() {
        let mut registry = FactoryRegistry::new();

        // Register multiple factories
        registry.register("mock", MockEntityFactory);
        registry.register(
            "simple",
            SimpleFactory::new(|params: MockParams| MockEntity {
                id: "simple".to_string(),
                name: params.name,
                value: params.value,
            }),
        );

        // Test both factories
        let mock_entity = registry
            .create(
                "mock",
                MockParams {
                    name: "mock_test".to_string(),
                    value: 111,
                },
            )
            .unwrap();

        let simple_entity = registry
            .create(
                "simple",
                MockParams {
                    name: "simple_test".to_string(),
                    value: 222,
                },
            )
            .unwrap();

        assert_eq!(mock_entity.id, "mock-001");
        assert_eq!(simple_entity.id, "simple");
        assert_ne!(mock_entity.id, simple_entity.id);
    }

    #[test]
    fn test_factory_trait_objects() {
        let mut registry = FactoryRegistry::new();
        let factory = MockEntityFactory;

        registry.register("trait_object", factory);

        // Test that we can use the factory through trait objects
        let factory_ref = registry.get("trait_object").unwrap();
        let entity = factory_ref.create(MockParams {
            name: "trait_test".to_string(),
            value: 77,
        });

        assert_eq!(entity.id, "mock-001");
        assert_eq!(entity.name, "trait_test");
        assert_eq!(entity.value, 77);
    }

    // Tests for edge cases
    #[test]
    fn test_factory_with_empty_params() {
        let creator = |_: ()| MockEntity {
            id: "empty".to_string(),
            name: "empty_params".to_string(),
            value: 0,
        };

        let factory = SimpleFactory::new(creator);
        let entity = factory.create(());

        assert_eq!(entity.id, "empty");
        assert_eq!(entity.name, "empty_params");
        assert_eq!(entity.value, 0);
    }

    #[test]
    fn test_factory_registry_overwrite() {
        let mut registry = FactoryRegistry::new();

        // Register first factory
        registry.register("overwrite", MockEntityFactory);

        // Register second factory with same name
        let creator = |params: MockParams| MockEntity {
            id: "overwritten".to_string(),
            name: params.name,
            value: params.value,
        };
        registry.register("overwrite", SimpleFactory::new(creator));

        // Should get the second factory
        let entity = registry
            .create(
                "overwrite",
                MockParams {
                    name: "overwrite_test".to_string(),
                    value: 999,
                },
            )
            .unwrap();

        assert_eq!(entity.id, "overwritten");
        assert_eq!(entity.name, "overwrite_test");
        assert_eq!(entity.value, 999);
    }

    // Additional tests for better coverage
    #[test]
    fn test_factory_registry_clear() {
        let mut registry = FactoryRegistry::new();
        registry.register("test", MockEntityFactory);
        assert_eq!(registry.factories.len(), 1);

        // Clear all factories (this would require adding a clear method)
        // For now, we test the current behavior
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_factory_registry_iteration() {
        let mut registry = FactoryRegistry::new();
        registry.register("first", MockEntityFactory);
        registry.register("second", MockEntityFactory);

        // Test that we can access multiple factories
        let first = registry.get("first");
        let second = registry.get("second");

        assert!(first.is_some());
        assert!(second.is_some());

        // Test that both factories work correctly
        let entity1 = first.unwrap().create(MockParams {
            name: "first_test".to_string(),
            value: 100,
        });
        let entity2 = second.unwrap().create(MockParams {
            name: "second_test".to_string(),
            value: 200,
        });

        assert_eq!(entity1.name, "first_test");
        assert_eq!(entity2.name, "second_test");
        assert_ne!(entity1.name, entity2.name);
    }

    #[test]
    fn test_factory_with_complex_params() {
        #[derive(Debug, Clone, PartialEq)]
        struct ComplexParams {
            name: String,
            value: u32,
            metadata: HashMap<String, String>,
        }

        impl Default for ComplexParams {
            fn default() -> Self {
                let mut metadata = HashMap::new();
                metadata.insert("version".to_string(), "1.0".to_string());
                metadata.insert("type".to_string(), "default".to_string());

                Self {
                    name: "complex_default".to_string(),
                    value: 100,
                    metadata,
                }
            }
        }

        let creator = |params: ComplexParams| MockEntity {
            id: format!("complex-{}", params.value),
            name: params.name,
            value: params.value,
        };

        let factory = SimpleFactory::new(creator);
        let params = ComplexParams {
            name: "complex_test".to_string(),
            value: 200,
            metadata: {
                let mut m = HashMap::new();
                m.insert("version".to_string(), "2.0".to_string());
                m.insert("type".to_string(), "test".to_string());
                m
            },
        };

        let entity = factory.create(params);
        assert_eq!(entity.id, "complex-200");
        assert_eq!(entity.name, "complex_test");
        assert_eq!(entity.value, 200);
    }

    #[test]
    fn test_factory_with_unit_params() {
        let creator = |_: ()| MockEntity {
            id: "unit".to_string(),
            name: "unit_params".to_string(),
            value: 0,
        };

        let factory = SimpleFactory::new(creator);
        let entity = factory.create(());

        assert_eq!(entity.id, "unit");
        assert_eq!(entity.name, "unit_params");
        assert_eq!(entity.value, 0);
    }

    #[test]
    fn test_factory_with_reference_params() {
        #[derive(Debug, Clone, PartialEq)]
        struct RefParams<'a> {
            name: &'a str,
            value: u32,
        }

        let creator = |params: RefParams| MockEntity {
            id: format!("ref-{}", params.value),
            name: params.name.to_string(),
            value: params.value,
        };

        let factory = SimpleFactory::new(creator);
        let params = RefParams {
            name: "reference_test",
            value: 300,
        };

        let entity = factory.create(params);
        assert_eq!(entity.id, "ref-300");
        assert_eq!(entity.name, "reference_test");
        assert_eq!(entity.value, 300);
    }

    #[test]
    fn test_factory_with_optional_params() {
        #[derive(Debug, Clone, PartialEq)]
        struct OptionalParams {
            name: Option<String>,
            value: Option<u32>,
        }

        impl Default for OptionalParams {
            fn default() -> Self {
                Self {
                    name: Some("optional_default".to_string()),
                    value: Some(50),
                }
            }
        }

        let creator = |params: OptionalParams| MockEntity {
            id: "optional".to_string(),
            name: params.name.unwrap_or_else(|| "unknown".to_string()),
            value: params.value.unwrap_or(0),
        };

        let factory = SimpleFactory::new(creator);

        // Test with Some values
        let params = OptionalParams {
            name: Some("optional_test".to_string()),
            value: Some(150),
        };
        let entity = factory.create(params);
        assert_eq!(entity.name, "optional_test");
        assert_eq!(entity.value, 150);

        // Test with None values
        let params = OptionalParams {
            name: None,
            value: None,
        };
        let entity = factory.create(params);
        assert_eq!(entity.name, "unknown");
        assert_eq!(entity.value, 0);
    }

    #[test]
    fn test_factory_with_enum_params() {
        #[derive(Debug, Clone, PartialEq)]
        enum EnumParams {
            Simple { name: String },
            Complex { name: String, value: u32 },
        }

        impl Default for EnumParams {
            fn default() -> Self {
                Self::Simple {
                    name: "enum_default".to_string(),
                }
            }
        }

        let creator = |params: EnumParams| MockEntity {
            id: "enum".to_string(),
            name: match &params {
                EnumParams::Simple { name } => name.clone(),
                EnumParams::Complex { name, .. } => name.clone(),
            },
            value: match params {
                EnumParams::Simple { .. } => 0,
                EnumParams::Complex { value, .. } => value,
            },
        };

        let factory = SimpleFactory::new(creator);

        // Test Simple variant
        let params = EnumParams::Simple {
            name: "simple_enum".to_string(),
        };
        let entity = factory.create(params);
        assert_eq!(entity.name, "simple_enum");
        assert_eq!(entity.value, 0);

        // Test Complex variant
        let params = EnumParams::Complex {
            name: "complex_enum".to_string(),
            value: 400,
        };
        let entity = factory.create(params);
        assert_eq!(entity.name, "complex_enum");
        assert_eq!(entity.value, 400);
    }

    #[test]
    fn test_factory_with_generic_entity() {
        #[derive(Debug, Clone, PartialEq)]
        struct GenericEntity<T> {
            id: String,
            data: T,
        }

        #[derive(Debug, Clone, PartialEq)]
        struct GenericParams<T> {
            data: T,
        }

        impl<T: Default> Default for GenericParams<T> {
            fn default() -> Self {
                Self {
                    data: Default::default(),
                }
            }
        }

        let creator = |params: GenericParams<String>| GenericEntity {
            id: "generic".to_string(),
            data: params.data,
        };

        let factory = SimpleFactory::new(creator);
        let params = GenericParams {
            data: "generic_test".to_string(),
        };

        let entity = factory.create(params);
        assert_eq!(entity.id, "generic");
        assert_eq!(entity.data, "generic_test");
    }

    #[test]
    fn test_factory_with_phantom_data() {
        use std::marker::PhantomData;

        #[derive(Debug, Clone, PartialEq)]
        struct PhantomEntity<T> {
            id: String,
            _phantom: PhantomData<T>,
        }

        #[derive(Debug, Clone, PartialEq)]
        struct PhantomParams<T> {
            _phantom: PhantomData<T>,
        }

        impl<T> Default for PhantomParams<T> {
            fn default() -> Self {
                Self { _phantom: PhantomData }
            }
        }

        let creator = |_: PhantomParams<String>| PhantomEntity {
            id: "phantom".to_string(),
            _phantom: PhantomData::<String>,
        };

        let factory = SimpleFactory::new(creator);
        let params = PhantomParams::<String>::default();

        let entity = factory.create(params);
        assert_eq!(entity.id, "phantom");
    }

    #[test]
    fn test_factory_with_custom_error_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct CustomError {
            message: String,
            code: u32,
        }

        #[derive(Debug, Clone, PartialEq)]
        struct CustomParams {
            name: String,
            value: u32,
        }

        struct CustomValidatedFactory;

        impl ValidatedEntityFactory<MockEntity, CustomParams> for CustomValidatedFactory {
            type Error = CustomError;

            fn create_validated(&self, params: CustomParams) -> Result<MockEntity, Self::Error> {
                if params.value == 0 {
                    Err(CustomError {
                        message: "Value cannot be zero".to_string(),
                        code: 1001,
                    })
                } else if params.name.is_empty() {
                    Err(CustomError {
                        message: "Name cannot be empty".to_string(),
                        code: 1002,
                    })
                } else {
                    Ok(MockEntity {
                        id: "custom-001".to_string(),
                        name: params.name,
                        value: params.value,
                    })
                }
            }
        }

        let factory = CustomValidatedFactory;

        // Test success case
        let params = CustomParams {
            name: "custom_test".to_string(),
            value: 100,
        };
        let result = factory.create_validated(params);
        assert!(result.is_ok());

        let entity = result.unwrap();
        assert_eq!(entity.id, "custom-001");
        assert_eq!(entity.name, "custom_test");
        assert_eq!(entity.value, 100);

        // Test zero value error
        let params = CustomParams {
            name: "zero_test".to_string(),
            value: 0,
        };
        let result = factory.create_validated(params);
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.message, "Value cannot be zero");
            assert_eq!(error.code, 1001);
        }

        // Test empty name error
        let params = CustomParams {
            name: "".to_string(),
            value: 50,
        };
        let result = factory.create_validated(params);
        assert!(result.is_err());

        if let Err(error) = result {
            assert_eq!(error.message, "Name cannot be empty");
            assert_eq!(error.code, 1002);
        }
    }
}
