use crate::domain::{
    project::{project::Project, project_repository::ProjectRepository},
    shared_kernel::{errors::DomainError, convertable::Convertable},
};
use std::path::{Path, PathBuf};
use std::fs;
use serde_yaml;
use crate::infrastructure::persistence::manifests::project_manifest::ProjectManifest;
use std::error::Error;
use std::fmt;
use std::default::Default;

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

    fn get_project_file_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
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

    fn load_manifest(&self, path: &Path) -> Result<ProjectManifest, Box<dyn Error>> {
        let yaml = fs::read_to_string(path)
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;
        
        serde_yaml::from_str(&yaml)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
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

    fn load(&self, path: &Path) -> Result<Project, DomainError> {
        let manifest_path = path.join("project.yaml");
        
        if let Ok(manifest) = self.load_manifest(&manifest_path) {
            Ok(manifest.to())
        } else {
            Err(DomainError::Generic("Falha ao carregar o arquivo do projeto".to_string()))
        }
    }
}

impl Default for FileProjectRepository {
    fn default() -> Self {
        Self::new()
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

    fn load(&self, _path: &Path) -> Result<Project, DomainError> {
        Ok(self.project.clone())
    }
}
