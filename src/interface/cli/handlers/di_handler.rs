use crate::application::di::{DIContainer, DIFactory};
use std::sync::{Arc, RwLock, OnceLock};

/// Handler global para Dependency Injection
pub struct DIHandler {
    container: Arc<RwLock<Option<DIContainer>>>,
}

impl DIHandler {
    fn new() -> Self {
        Self {
            container: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Obtém o container de DI, inicializando se necessário
    pub fn get_container(&self) -> Result<DIContainer, String> {
        // Tenta ler o container existente
        if let Ok(container_guard) = self.container.read() {
            if let Some(container) = container_guard.as_ref() {
                return Ok(container.clone());
            }
        }
        
        // Se não existe, cria um novo
        let new_container = DIFactory::create_container()?;
        
        // Tenta escrever o novo container
        if let Ok(mut container_guard) = self.container.write() {
            *container_guard = Some(new_container.clone());
            return Ok(new_container);
        }
        
        Err("Failed to acquire write lock for DI container".to_string())
    }
}

/// Instância global do handler de DI
pub static DI_HANDLER: OnceLock<DIHandler> = OnceLock::new();

/// Inicializa o handler de DI
pub fn init_di_handler() -> Result<(), String> {
    let handler = DIHandler::new();
    DI_HANDLER.set(handler).map_err(|_| "DI handler already initialized".to_string())?;
    Ok(())
}

/// Obtém o handler de DI
pub fn get_di_handler() -> &'static DIHandler {
    DI_HANDLER.get().expect("DI handler not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_di_handler_initialization() {
        let result = init_di_handler();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_get_container() {
        // Inicializa o handler se não estiver inicializado
        let _ = init_di_handler();
        let handler = get_di_handler();
        let container = handler.get_container();
        assert!(container.is_ok());
    }
}
