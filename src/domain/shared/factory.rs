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
        F: EntityFactory<T, P> + 'static,
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
