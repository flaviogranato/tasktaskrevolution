use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::{
    company_settings::{Config, repository::ConfigRepository},
    shared::convertable::Convertible,
};
use serde_yaml::to_string;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct FileConfigRepository {
    base_path: PathBuf,
}

impl FileConfigRepository {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    pub fn with_base_path(path: PathBuf) -> Self {
        Self { base_path: path }
    }
}

impl Default for FileConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigRepository for FileConfigRepository {
    fn save(&self, config: Config, path: PathBuf) -> DomainResult<()> {
        let config_yaml = to_string(&config).map_err(|e| DomainError::SerializationError {
            operation: "YAML serialization".to_string(),
            details: e.to_string(),
        })?;
        let file = path.join("config.yaml");
        fs::write(&file, config_yaml).map_err(|e| DomainError::IoErrorWithPath {
            operation: "file write".to_string(),
            path: file.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;
        Ok(())
    }

    fn create_repository_dir(&self, path: PathBuf) -> DomainResult<()> {
        if !path.exists() {
            fs::create_dir(&path).map_err(|e| DomainError::IoErrorWithPath {
                operation: "create directory".to_string(),
                path: path.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;
            println!("Configuration repository created.");
        }
        Ok(())
    }

    fn load(&self) -> DomainResult<(Config, PathBuf)> {
        let mut current_path = self.base_path.canonicalize().map_err(|e| DomainError::IoErrorWithPath {
            operation: "canonicalize path".to_string(),
            path: self.base_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        loop {
            let config_path = current_path.join("config.yaml");

            if config_path.exists() {
                let file_content = fs::read_to_string(&config_path).map_err(|e| DomainError::IoErrorWithPath {
                    operation: "file read".to_string(),
                    path: config_path.to_string_lossy().to_string(),
                    details: e.to_string(),
                })?;
                let manifest: Config =
                    serde_yaml::from_str(&file_content).map_err(|e| DomainError::SerializationError {
                        operation: "YAML serialization".to_string(),
                        details: e.to_string(),
                    })?;

                return Ok((manifest, current_path));
            }

            if !current_path.pop() {
                break;
            }
        }

        Err(DomainError::ValidationError {
            field: "config file".to_string(),
            message: "Não foi possível encontrar o arquivo 'config.yaml' nos diretórios pais.".to_string(),
        })
    }
}
