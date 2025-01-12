use std::{fs, path::PathBuf};

use serde_yml::to_string;

use crate::domain::{
    config::{config::ConfigManifest, config_repository::ConfigRepository},
    shared_kernel::errors::DomainError,
};

pub struct FileConfigRepository;

impl FileConfigRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigRepository for FileConfigRepository {
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), DomainError> {
        let config_yaml = to_string(&config).map_err(|e| DomainError::Generic(e.to_string()))?;
        let file = path.join("config.yaml");
        fs::write(file, config_yaml).map_err(|e| DomainError::Generic(e.to_string()))?;
        Ok(())
    }

    fn create_repository_dir(&self, path: PathBuf) -> Result<(), DomainError> {
        if !path.exists() {
            fs::create_dir(path).map_err(|e| DomainError::Generic(e.to_string()))?;
            println!("Criado o repositório de configurações.");
        }
        Ok(())
    }
}
