use crate::domain::resource::{resource::Resource, resource_repository::ResourceRepository};
use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::shared_kernel::errors::DomainError;
use crate::infrastructure::persistence::manifests::resource_manifest::ResourceManifest;
use serde_yml::to_string;
use std::{fs, path::Path};

pub struct FileResourceRepository;

impl FileResourceRepository {
    pub fn new() -> Self {
        Self
    }
}

impl ResourceRepository for FileResourceRepository {
    fn save(&self, r: Resource) -> Result<Resource, DomainError> {
        let file_name = format!("{}.yaml", r.name.clone());
        let path = Path::new("resources");

        if !path.exists() {
            match fs::create_dir(path) {
                Ok(_) => println!("Criado o diretório de resources"),
                Err(e) => println!("Erro ao criar diretório de resources: {}", e),
            }
        }

        let resource_path = path.join(&file_name);
        let resource = <ResourceManifest as Convertable<Resource>>::from(r.clone());

        let resource_yaml =
            to_string(&resource).map_err(|e| DomainError::Generic(e.to_string()))?;
        let _ = fs::write(resource_path, resource_yaml)
            .map_err(|e| DomainError::Generic(e.to_string()));

        println!("Recurso {} criado.", r.name);
        Ok(r)
    }
}
