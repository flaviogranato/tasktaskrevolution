use super::manifests::project_manifest::ProjectManifest;
use crate::domain::project::project::Project;
use crate::domain::project::project_repository::ProjectRepository;
use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::shared_kernel::errors::DomainError;
use serde_yaml::to_string;
use std::fs;
use std::path::PathBuf;

pub struct FileProjectRepository;

impl FileProjectRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn load_project(&self, path: &PathBuf) -> Result<Project, std::io::Error> {
        let project_path = path.join("project.yaml");
        if !project_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Project file not found",
            ));
        }

        let contents = std::fs::read_to_string(project_path)?;
        let manifest: ProjectManifest = serde_yaml::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(manifest.to())
    }
}

impl ProjectRepository for FileProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError> {
        let current_dir = std::env::current_dir().map_err(|e| DomainError::Generic(e.to_string()));
        let path = current_dir.unwrap().join(&project.name);
        let project_file_path = path.join("project.yaml");
        let project_manifest = <ProjectManifest as Convertable<Project>>::from(project);
        let project_yaml =
            to_string(&project_manifest).map_err(|e| DomainError::Generic(e.to_string()))?;
        fs::create_dir_all(&path).map_err(|e| DomainError::Generic(e.to_string()))?;
        fs::write(project_file_path, project_yaml)
            .map_err(|e| DomainError::Generic(e.to_string()))?;

        println!("Projeto criado em: {}", path.display());
        Ok(())
    }
}
