use crate::domain::{
    company_settings::{Config, repository::ConfigRepository},
    shared::{
        convertable::Convertible,
        errors::{DomainError, DomainErrorKind},
    },
};
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
use serde_yaml::to_string;
use std::{fs, path::PathBuf};

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
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), DomainError> {
        let config_yaml =
            to_string(&config).map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
        let file = path.join("config.yaml");
        fs::write(file, config_yaml)
            .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
        Ok(())
    }

    fn create_repository_dir(&self, path: PathBuf) -> Result<(), DomainError> {
        if !path.exists() {
            fs::create_dir(path).map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
            println!("Configuration repository created.");
        }
        Ok(())
    }

    fn load(&self) -> Result<(Config, PathBuf), DomainError> {
        let mut current_path = self
            .base_path
            .canonicalize()
            .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;

        loop {
            let config_path = current_path.join("config.yaml");

            if config_path.exists() {
                let file_content = fs::read_to_string(&config_path)
                    .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
                let manifest: ConfigManifest = serde_yaml::from_str(&file_content)
                    .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;

                return Ok((manifest.to(), current_path));
            }

            if !current_path.pop() {
                break;
            }
        }

        Err(DomainError::new(DomainErrorKind::Generic {
            message: "Não foi possível encontrar o arquivo 'config.yaml' nos diretórios pais.".to_string(),
        }))
    }
}
