use std::path::PathBuf;

use crate::domain::shared_kernel::errors::DomainError;
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;

pub trait ConfigRepository {
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), DomainError>;
    fn create_repository_dir(&self, path: PathBuf) -> Result<(), DomainError>;
}
