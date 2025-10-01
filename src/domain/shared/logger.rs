//! Domain logging abstraction
//!
//! This module provides a logging interface that the domain layer can use
//! without depending on the interface layer.

/// Trait for domain logging operations
pub trait DomainLogger {
    /// Log a debug message
    fn debug(&self, message: &str);
    
    /// Log an info message
    fn info(&self, message: &str);
    
    /// Log a warning message
    fn warn(&self, message: &str);
    
    /// Log an error message
    fn error(&self, message: &str);
}

/// Default implementation that does nothing (no-op logger)
pub struct NoOpLogger;

impl DomainLogger for NoOpLogger {
    fn debug(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
}

/// Convenience functions for logging
pub fn debug(message: &str) {
    // For now, just print to stderr if debug is enabled
    if std::env::var("TTR_DEBUG").is_ok() {
        eprintln!("[DEBUG] {}", message);
    }
}

pub fn info(message: &str) {
    // For now, just print to stderr
    eprintln!("[INFO] {}", message);
}

pub fn warn(message: &str) {
    // For now, just print to stderr
    eprintln!("[WARN] {}", message);
}

pub fn error(message: &str) {
    // For now, just print to stderr
    eprintln!("[ERROR] {}", message);
}

/// Debug formatting helper
pub fn debug_fmt<F>(f: F) 
where 
    F: FnOnce() -> String 
{
    let message = f();
    debug(&message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::collections::VecDeque;

    /// Test logger that captures messages
    struct TestLogger {
        messages: Arc<Mutex<VecDeque<String>>>,
    }

    impl TestLogger {
        fn new() -> Self {
            Self {
                messages: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.lock().unwrap().iter().cloned().collect()
        }
    }

    impl DomainLogger for TestLogger {
        fn debug(&self, message: &str) {
            self.messages.lock().unwrap().push_back(format!("DEBUG: {}", message));
        }

        fn info(&self, message: &str) {
            self.messages.lock().unwrap().push_back(format!("INFO: {}", message));
        }

        fn warn(&self, message: &str) {
            self.messages.lock().unwrap().push_back(format!("WARN: {}", message));
        }

        fn error(&self, message: &str) {
            self.messages.lock().unwrap().push_back(format!("ERROR: {}", message));
        }
    }

    #[test]
    fn test_no_op_logger() {
        let logger = NoOpLogger;
        logger.debug("test");
        logger.info("test");
        logger.warn("test");
        logger.error("test");
        // Should not panic
    }

    #[test]
    fn test_logger_functions() {
        // Test that logger functions don't panic
        debug("debug message");
        info("info message");
        warn("warn message");
        error("error message");
        
        // Test debug_fmt
        debug_fmt(|| "formatted message".to_string());
        
        // These tests just verify the functions don't panic
        // In a real implementation, we'd capture the output
    }
}
