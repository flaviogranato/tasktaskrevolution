use std::path::PathBuf;

use crate::domain::config::config::ConfigManifest;
use crate::domain::shared_kernel::errors::DomainError;

pub trait ConfigRepository {
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), DomainError>;
    fn create_repository_dir(&self, path: PathBuf) -> Result<(), DomainError>;
}
