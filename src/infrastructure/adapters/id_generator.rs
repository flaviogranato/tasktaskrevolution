//! ID generator adapter implementation
//!
//! This module provides a concrete implementation of the IdGeneratorPort
//! using various ID generation strategies.

use crate::domain::ports::id_generator::{IdGeneratorPort, IdType};
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::sync::atomic::{AtomicU64, Ordering};

/// Standard ID generator adapter
pub struct StandardIdGeneratorAdapter {
    counter: AtomicU64,
}

impl StandardIdGeneratorAdapter {
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }
}

impl Default for StandardIdGeneratorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGeneratorPort for StandardIdGeneratorAdapter {
    fn generate_uuid(&self) -> String {
        uuid7::uuid7().to_string()
    }

    fn generate_uuid_v7(&self) -> String {
        uuid7::uuid7().to_string()
    }

    fn generate_uuid_v4(&self) -> String {
        uuid7::uuid7().to_string() // Using v7 as fallback since v4 is not available
    }

    fn generate_short_id(&self) -> String {
        // Simple short ID generation without external dependencies
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut hasher = DefaultHasher::new();
        timestamp.hash(&mut hasher);
        format!("{:x}", hasher.finish())[..8].to_string()
    }

    fn generate_numeric_id(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    fn generate_code(&self, prefix: &str) -> DomainResult<String> {
        let numeric_id = self.generate_numeric_id();
        Ok(format!("{}-{:03}", prefix, numeric_id))
    }

    fn generate_code_with_format(&self, format: &str) -> DomainResult<String> {
        // Simple implementation - in a real scenario, this would be more sophisticated
        if format.contains("{}") {
            let numeric_id = self.generate_numeric_id();
            Ok(format.replace("{}", &numeric_id.to_string()))
        } else {
            Err(DomainError::ValidationError {
                field: "format".to_string(),
                message: "Format must contain {} placeholder".to_string(),
            })
        }
    }

    fn validate_id(&self, id: &str) -> bool {
        // Basic validation - check if it's not empty and has reasonable length
        !id.is_empty() && id.len() <= 255
    }

    fn get_id_type(&self) -> IdType {
        IdType::UuidV7
    }
}
