use crate::domain::resource::{resource::Resource, resource_repository::ResourceRepository};
use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::shared_kernel::errors::DomainError;
use crate::infrastructure::persistence::manifests::resource_manifest::ResourceManifest;
use serde_yml::to_string;
use serde_yml::from_str;
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

    fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
        let path = Path::new("resources");
        if !path.exists() {
            return Ok(vec![]);
        }

        let mut resources = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| DomainError::Generic(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::Generic(e.to_string()))?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(entry.path())
                    .map_err(|e| DomainError::Generic(e.to_string()))?;
                let manifest: ResourceManifest = from_str(&content)
                    .map_err(|e| DomainError::Generic(e.to_string()))?;
                resources.push(manifest.to());
            }
        }

        Ok(resources)
    }
}
