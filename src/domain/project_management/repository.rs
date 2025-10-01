use crate::domain::project_management::AnyProject;
use crate::domain::shared::errors::{DomainError, DomainResult};

pub trait ProjectRepository {
    fn save(&self, project: AnyProject) -> DomainResult<()>;
    fn load(&self) -> DomainResult<AnyProject>;
    fn find_all(&self) -> DomainResult<Vec<AnyProject>>;
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>>;
    fn get_next_code(&self) -> DomainResult<String>;
}

/// Extension trait for repositories that support ID-based operations
pub trait ProjectRepositoryWithId: ProjectRepository {
    fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>>;
}
