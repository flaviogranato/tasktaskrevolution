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
        
        let entity = registry.create("mock", MockParams {
            name: "created".to_string(),
            value: 88,
        });
        
        assert!(entity.is_some());
        let entity = entity.unwrap();
        assert_eq!(entity.id, "mock-001");
        assert_eq!(entity.name, "created");
        assert_eq!(entity.value, 88);
    }

    #[test]
    fn test_factory_registry_create_nonexistent() {
        let registry: FactoryRegistry<MockEntity, MockParams> = FactoryRegistry::new();
        let entity = registry.create("nonexistent", MockParams {
            name: "test".to_string(),
            value: 0,
        });
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
        registry.register("simple", SimpleFactory::new(|params: MockParams| MockEntity {
            id: "simple".to_string(),
            name: params.name,
            value: params.value,
        }));
        
        // Test both factories
        let mock_entity = registry.create("mock", MockParams {
            name: "mock_test".to_string(),
            value: 111,
        }).unwrap();
        
        let simple_entity = registry.create("simple", MockParams {
            name: "simple_test".to_string(),
            value: 222,
        }).unwrap();
        
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
        let entity = registry.create("overwrite", MockParams {
            name: "overwrite_test".to_string(),
            value: 999,
        }).unwrap();
        
        assert_eq!(entity.id, "overwritten");
        assert_eq!(entity.name, "overwrite_test");
        assert_eq!(entity.value, 999);
    }
}
