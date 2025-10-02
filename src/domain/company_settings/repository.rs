use crate::domain::company_settings::config::Config;
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::path::{Path, PathBuf};

/// Repository trait for Config entity operations.
/// 
/// This trait defines the contract for configuration persistence operations,
/// following the Repository pattern from Domain-Driven Design.
/// Implementations should be provided by the infrastructure layer.
pub trait ConfigRepository {
    /// Saves a configuration to the repository.
    /// 
    /// # Arguments
    /// * `config` - The configuration to save
    /// * `path` - The path where to save the configuration
    /// 
    /// # Returns
    /// * `Ok(())` if the configuration was saved successfully
    /// * `Err(DomainError)` if an error occurred during save
    fn save(&self, config: Config, path: &Path) -> DomainResult<()>;

    /// Creates the repository directory structure.
    /// 
    /// # Arguments
    /// * `path` - The base path where to create the repository structure
    /// 
    /// # Returns
    /// * `Ok(())` if the directory was created successfully
    /// * `Err(DomainError)` if an error occurred during creation
    fn create_repository_dir(&self, path: &Path) -> DomainResult<()>;

    /// Loads the configuration from the repository.
    /// 
    /// # Returns
    /// * `Ok((config, path))` - The loaded configuration and its path
    /// * `Err(DomainError)` if an error occurred during load
    fn load(&self) -> DomainResult<(Config, PathBuf)>;
}
