use crate::domain::config::config::ConfigManifest;
use serde_yml::to_string;
use std::fs;
use std::path::PathBuf;

pub fn create_config(
    path: &Option<PathBuf>,
    manager_name: &String,
    manager_email: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_path = path.clone().unwrap_or(std::env::current_dir()?);

    if !repo_path.exists() {
        match fs::create_dir(&repo_path) {
            Ok(_) => println!("Criado o repositório de configurações"),
            Err(e) => println!("Erro ao criar diretório de resources: {}", e),
        }
    }

    let config_path = repo_path.join("config.yaml");
    let config = ConfigManifest::basic(manager_name, manager_email);

    let config_yaml = to_string(&config)?;

    fs::write(config_path, config_yaml)?;

    println!("Repositório inicializado em: {}", repo_path.display());
    Ok(())
}
