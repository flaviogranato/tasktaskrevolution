use crate::domain::shared::errors::DomainError;
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
use std::path::PathBuf;

pub trait ConfigRepository {
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), DomainError>;
    fn create_repository_dir(&self, path: PathBuf) -> Result<(), DomainError>;
}
