use crate::domain::project_management::project::Project;
use crate::domain::shared::errors::DomainError;
use std::path::Path;

pub trait ProjectRepository {
    fn save(&self, project: Project) -> Result<(), DomainError>;
    fn load(&self) -> Result<Project, DomainError>;
}
