use std::path::Path;
use crate::domain::project::project::Project;
use crate::domain::shared_kernel::errors::DomainError;

pub trait ProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError>;
    fn load(&self, path: &Path) -> Result<Project, DomainError>;
}
