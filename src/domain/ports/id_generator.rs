//! ID generator port for domain entity IDs
//!
//! This module defines the ID generation interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::DomainResult;

/// ID generator port for generating unique identifiers
pub trait IdGeneratorPort: Send + Sync {
    /// Generate a new UUID
    fn generate_uuid(&self) -> String;

    /// Generate a new UUID v7 (time-based)
    fn generate_uuid_v7(&self) -> String;

    /// Generate a new UUID v4 (random)
    fn generate_uuid_v4(&self) -> String;

    /// Generate a new short ID
    fn generate_short_id(&self) -> String;

    /// Generate a new numeric ID
    fn generate_numeric_id(&self) -> u64;

    /// Generate a new code (e.g., "PROJ-001")
    fn generate_code(&self, prefix: &str) -> DomainResult<String>;

    /// Generate a new code with custom format
    fn generate_code_with_format(&self, format: &str) -> DomainResult<String>;

    /// Validate an ID format
    fn validate_id(&self, id: &str) -> bool;

    /// Get the ID type
    fn get_id_type(&self) -> IdType;
}

/// ID types supported by the generator
#[derive(Debug, Clone, PartialEq)]
pub enum IdType {
    Uuid,
    UuidV7,
    UuidV4,
    Short,
    Numeric,
    Code,
    Custom(String),
}
