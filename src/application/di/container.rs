use super::traits::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

trait Factory: Send + Sync {
    fn create(&self) -> Box<dyn Any + Send + Sync>;
}

struct FactoryWrapper<T, F> {
    factory: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> FactoryWrapper<T, F>
where
    T: Injectable + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    fn new(factory: F) -> Self {
        Self {
            factory,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F> Factory for FactoryWrapper<T, F>
where
    T: Injectable + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    fn create(&self) -> Box<dyn Any + Send + Sync> {
        Box::new((self.factory)())
    }
}

/// Container de Dependency Injection thread-safe
#[derive(Default)]
pub struct DIContainer {
    singletons: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
    factories: Arc<RwLock<HashMap<TypeId, Box<dyn Factory>>>>,
}

impl DIContainer {
    /// Cria um novo container vazio
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria um container com configurações padrão
    pub fn with_defaults() -> Self {
        // Aqui podemos registrar serviços padrão
        Self::new()
    }
}

impl ServiceRegistrar for DIContainer {
    fn register<T, F>(&mut self, factory: F) -> Result<(), String>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory = Box::new(FactoryWrapper::new(factory)) as Box<dyn Factory>;

        self.factories
            .write()
            .map_err(|_| "Failed to acquire write lock".to_string())?
            .insert(type_id, factory);

        Ok(())
    }

    fn register_singleton<T, F>(&mut self, factory: F) -> Result<(), String>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let instance = factory();
        let instance = Arc::new(instance) as Arc<dyn Any + Send + Sync>;

        self.singletons
            .write()
            .map_err(|_| "Failed to acquire write lock".to_string())?
            .insert(type_id, instance);

        Ok(())
    }

    fn register_instance<T>(&mut self, instance: T) -> Result<(), String>
    where
        T: Injectable,
    {
        let type_id = TypeId::of::<T>();
        let instance = Arc::new(instance) as Arc<dyn Any + Send + Sync>;

        self.singletons
            .write()
            .map_err(|_| "Failed to acquire write lock".to_string())?
            .insert(type_id, instance);

        Ok(())
    }
}

impl ServiceResolver for DIContainer {
    fn resolve<T>(&self) -> Result<Arc<T>, String>
    where
        T: Injectable + 'static,
    {
        self.try_resolve::<T>()
            .ok_or_else(|| format!("Service {} not found", std::any::type_name::<T>()))
    }

    fn try_resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Injectable + 'static,
    {
        let type_id = TypeId::of::<T>();

        // Primeiro tenta resolver de singletons
        if let Ok(singletons) = self.singletons.read()
            && let Some(service) = singletons.get(&type_id)
        {
            // Tenta fazer downcast direto
            if let Some(typed_service) = service.downcast_ref::<T>() {
                // Cria um novo Arc<T> usando unsafe para evitar Clone
                unsafe {
                    let raw_ptr = service.as_ref() as *const dyn Any as *const T;
                    return Some(Arc::from_raw(raw_ptr));
                }
            }
        }

        // Depois tenta resolver de factories
        if let Ok(factories) = self.factories.read()
            && let Some(factory) = factories.get(&type_id)
        {
            let instance = factory.create();
            if let Ok(typed_instance) = instance.downcast::<T>() {
                return Some(Arc::new(*typed_instance));
            }
        }

        None
    }
}

impl Clone for DIContainer {
    fn clone(&self) -> Self {
        Self {
            singletons: Arc::clone(&self.singletons),
            factories: Arc::clone(&self.factories),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct MockService {
        value: String,
    }

    impl Injectable for MockService {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl Clone for MockService {
        fn clone(&self) -> Self {
            Self {
                value: self.value.clone(),
            }
        }
    }

    #[test]
    fn test_register_and_resolve_singleton() {
        let mut container = DIContainer::new();

        let service = MockService {
            value: "test".to_string(),
        };

        container.register_instance(service).unwrap();

        let resolved: Arc<MockService> = container.resolve().unwrap();
        assert_eq!(resolved.value, "test");
    }

    #[test]
    fn test_register_and_resolve_factory() {
        let mut container = DIContainer::new();

        container
            .register(|| MockService {
                value: "factory".to_string(),
            })
            .unwrap();

        let resolved: Arc<MockService> = container.resolve().unwrap();
        assert_eq!(resolved.value, "factory");
    }

    #[test]
    fn test_resolve_nonexistent_service() {
        let container = DIContainer::new();

        let result: Result<Arc<MockService>, _> = container.resolve();
        assert!(result.is_err());
    }
}
