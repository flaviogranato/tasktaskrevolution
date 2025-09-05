use std::any::Any;
use std::sync::Arc;

/// Trait para serviços que podem ser injetados
pub trait Injectable: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

/// Trait para provedores de serviços
pub trait ServiceProvider<T> {
    fn provide(&self) -> T;
}

/// Trait para registradores de serviços
pub trait ServiceRegistrar {
    fn register<T, F>(&mut self, factory: F) -> Result<(), String>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static;
    
    fn register_singleton<T, F>(&mut self, factory: F) -> Result<(), String>
    where
        T: Injectable,
        F: Fn() -> T + Send + Sync + 'static;
    
    fn register_instance<T>(&mut self, instance: T) -> Result<(), String>
    where
        T: Injectable;
}

/// Trait para resolvedores de serviços
pub trait ServiceResolver {
    fn resolve<T>(&self) -> Result<Arc<T>, String>
    where
        T: Injectable + 'static;
    
    fn try_resolve<T>(&self) -> Option<Arc<T>>
    where
        T: Injectable + 'static;
}
