use crate::application::app::App;
use std::sync::{Arc, OnceLock};

/// Simple app handler to replace the complex DI container
pub struct AppHandler {
    app: Arc<App>,
}

impl Default for AppHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl AppHandler {
    pub fn new() -> Self {
        Self {
            app: Arc::new(App::new()),
        }
    }

    pub fn with_base_path(base_path: String) -> Self {
        Self {
            app: Arc::new(App::with_base_path(base_path)),
        }
    }

    pub fn get_app(&self) -> &Arc<App> {
        &self.app
    }
}

/// Global app handler instance
pub static APP_HANDLER: OnceLock<AppHandler> = OnceLock::new();

/// Initialize the app handler
pub fn init_app_handler() -> Result<(), String> {
    let handler = AppHandler::new();
    APP_HANDLER
        .set(handler)
        .map_err(|_| "App handler already initialized".to_string())?;
    Ok(())
}

/// Get the app handler
pub fn get_app_handler() -> &'static AppHandler {
    APP_HANDLER.get().expect("App handler not initialized")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_handler_new() {
        let handler = AppHandler::new();
        let _app = handler.get_app();
        // If we get here, the handler was created successfullyully
    }

    #[test]
    fn test_app_handler_with_base_path() {
        let handler = AppHandler::with_base_path("/tmp/test".to_string());
        let _app = handler.get_app();
        // If we get here, the handler was created successfullyully
    }

    #[test]
    fn test_init_and_get_app_handler() {
        let _ = init_app_handler();
        let handler = get_app_handler();
        let _app = handler.get_app();
        // If we get here, the handler was initialized and retrieved successfully
    }
}
