use crate::domain::project::model::Project;
use crate::domain::shared_kernel::errors::DomainError;
use std::path::Path;

pub trait ProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError>;
    fn load(&self, path: &Path) -> Result<Project, DomainError>;
}
