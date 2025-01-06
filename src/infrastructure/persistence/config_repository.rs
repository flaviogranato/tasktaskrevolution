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
        // if !repo_path.exists() {
        //     match fs::create_dir(&repo_path) {
        //         Ok(_) => println!("Criado o repositório de configurações"),
        //         Err(e) => println!("Erro ao criar diretório de resources: {}", e),
        //     }
        // }

        let config_yaml = to_string(&config).map_err(|e| DomainError::Generic(e.to_string()))?;
        fs::write(path, config_yaml).map_err(|e| DomainError::Generic(e.to_string()))?;
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
