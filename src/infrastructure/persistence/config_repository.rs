use std::{fs, path::PathBuf};

use serde_yaml::to_string;

use crate::domain::config::config::Config;
use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::{
    config::config_repository::ConfigRepository, shared_kernel::errors::DomainError,
};
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;

pub struct FileConfigRepository;

impl FileConfigRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn load_config(&self) -> Result<Config, std::io::Error> {
        let config_path = PathBuf::from("config.yaml");
        if !config_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Config file not found",
            ));
        }

        let contents = std::fs::read_to_string(config_path)?;
        let manifest: ConfigManifest = serde_yaml::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(manifest.to())
    }
}

impl Default for FileConfigRepository {
    fn default() -> Self {
        Self::new()
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
