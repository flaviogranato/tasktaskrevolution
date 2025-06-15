use crate::domain::{
    project_management::{project::Project, repository::ProjectRepository},
    shared::{convertable::Convertable, errors::DomainError},
};
use crate::infrastructure::persistence::manifests::project_manifest::ProjectManifest;
use serde_yaml;
use std::default::Default;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FileProjectRepository;

impl FileProjectRepository {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn get_project_file_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn load_manifest(&self, path: &Path) -> Result<ProjectManifest, Box<dyn Error>> {
        let yaml = fs::read_to_string(path).map_err(|e| Box::new(e) as Box<dyn Error>)?;

        serde_yaml::from_str(&yaml).map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

impl ProjectRepository for FileProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError> {
        let file_path = self.get_project_file_path(&PathBuf::from(&project.name));
        let project_manifest = <ProjectManifest as Convertable<Project>>::from(project);
        let yaml = serde_yaml::to_string(&project_manifest).map_err(|e| DomainError::Generic(format!("Erro ao serializar projeto: {}", e)))?;

        fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| DomainError::Generic(format!("Erro ao criar diretório: {}", e)))?;

        fs::write(file_path, yaml).map_err(|e| DomainError::Generic(format!("Erro ao salvar projeto: {}", e)))?;

        Ok(())
    }

    fn load(&self, path: &Path) -> Result<Project, DomainError> {
        let manifest_path = path.join("project.yaml");

        if let Ok(manifest) = self.load_manifest(&manifest_path) {
            Ok(manifest.to())
        } else {
            Err(DomainError::Generic(
                "Falha ao carregar o arquivo do projeto".to_string(),
            ))
        }
    }
}

#[cfg(test)]
pub struct MockProjectRepository {
    project: Project,
}

#[cfg(test)]
impl ProjectRepository for MockProjectRepository {
    fn save(&self, _project: Project) -> Result<(), DomainError> {
        Ok(())
    }

    fn load(&self, _path: &Path) -> Result<Project, DomainError> {
        Ok(self.project.clone())
    }
}
