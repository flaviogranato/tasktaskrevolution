use crate::domain::{
    project::{project::Project, project_repository::ProjectRepository},
    shared_kernel::{errors::DomainError, convertable::Convertable},
};
use std::path::PathBuf;
use std::fs;
use serde_yaml;
use crate::infrastructure::persistence::manifests::project_manifest::ProjectManifest;
use std::error::Error;
use std::fmt;

// Implementação simples do NotFoundError
#[derive(Debug)]
struct NotFoundError {
    message: String,
}

impl NotFoundError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for NotFoundError {}

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

    fn get_project_path(&self, project_code: &str) -> PathBuf {
        self.base_path.join("projects").join(project_code)
    }
    
    pub fn load_project(&self, project_code: &str) -> Result<Project, Box<dyn Error>> {
        let project_path = self.get_project_path(project_code);
        if !project_path.exists() {
            return Err(Box::new(NotFoundError::new(&format!(
                "Projeto com código '{}' não encontrado",
                project_code
            ))));
        }

        let project_yaml = fs::read_to_string(project_path.join("project.yaml"))?;
        let project_manifest: ProjectManifest = serde_yaml::from_str(&project_yaml)?;

        Ok(<ProjectManifest as Convertable<Project>>::to(&project_manifest))
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
        
        Ok(<ProjectManifest as Convertable<Project>>::to(&project_manifest))
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
