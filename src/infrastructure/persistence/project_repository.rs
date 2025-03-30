use crate::domain::{
    project::{project::Project, project_repository::ProjectRepository},
    shared_kernel::{errors::DomainError, convertable::Convertable},
};
use std::path::PathBuf;
use std::fs;
use serde_yaml;
use crate::infrastructure::persistence::manifests::project_manifest::ProjectManifest;

pub struct FileProjectRepository {
    base_path: PathBuf,
}

impl FileProjectRepository {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    fn get_project_file_path(&self, path: &PathBuf) -> PathBuf {
        path.join("project.yaml")
    }
}

impl ProjectRepository for FileProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError> {
        let file_path = self.get_project_file_path(&PathBuf::from(&project.name));
        let project_manifest = <ProjectManifest as Convertable<Project>>::from(project);
        let yaml = serde_yaml::to_string(&project_manifest)
            .map_err(|e| DomainError::Generic(format!("Erro ao serializar projeto: {}", e)))?;
        
        fs::create_dir_all(file_path.parent().unwrap())
            .map_err(|e| DomainError::Generic(format!("Erro ao criar diretório: {}", e)))?;
        
        fs::write(file_path, yaml)
            .map_err(|e| DomainError::Generic(format!("Erro ao salvar projeto: {}", e)))?;
        
        Ok(())
    }

    fn load(&self, path: &PathBuf) -> Result<Project, DomainError> {
        let file_path = self.get_project_file_path(path);
        let yaml = fs::read_to_string(&file_path)
            .map_err(|e| DomainError::Generic(format!("Erro ao ler arquivo de projeto: {}", e)))?;
        
        let project_manifest: ProjectManifest = serde_yaml::from_str(&yaml)
            .map_err(|e| DomainError::Generic(format!("Erro ao deserializar projeto: {}", e)))?;
        
        Ok(<ProjectManifest as Convertable<Project>>::to(project_manifest))
    }
}

#[cfg(test)]
pub struct MockProjectRepository {
    project: Project,
}

#[cfg(test)]
impl MockProjectRepository {
    pub fn new(project: Project) -> Self {
        Self { project }
    }
}

#[cfg(test)]
impl ProjectRepository for MockProjectRepository {
    fn save(&self, _project: Project) -> Result<(), DomainError> {
        Ok(())
    }

    fn load(&self, _path: &PathBuf) -> Result<Project, DomainError> {
        Ok(self.project.clone())
    }
}
