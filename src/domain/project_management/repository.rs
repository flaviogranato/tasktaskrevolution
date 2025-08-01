use crate::domain::project_management::AnyProject;
use crate::domain::shared::errors::DomainError;

pub trait ProjectRepository {
    fn save(&self, project: AnyProject) -> Result<(), DomainError>;
    fn load(&self) -> Result<AnyProject, DomainError>;
    fn find_all(&self) -> Result<Vec<AnyProject>, DomainError>;
    fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError>;
    fn get_next_code(&self) -> Result<String, DomainError>;
}
