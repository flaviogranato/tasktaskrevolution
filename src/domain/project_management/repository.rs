use crate::domain::project_management::AnyProject;
use crate::domain::shared::errors::DomainError;

pub trait ProjectRepository {
    fn save(&self, project: AnyProject) -> Result<(), DomainError>;
    fn load(&self) -> Result<AnyProject, DomainError>;
}
