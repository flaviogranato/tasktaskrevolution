use crate::application::errors::AppError;
use crate::domain::project_management::AnyProject;

pub trait ProjectRepository {
    fn save(&self, project: AnyProject) -> Result<(), AppError>;
    fn load(&self) -> Result<AnyProject, AppError>;
    fn find_all(&self) -> Result<Vec<AnyProject>, AppError>;
    fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError>;
    fn get_next_code(&self) -> Result<String, AppError>;
}

/// Extension trait for repositories that support ID-based operations
pub trait ProjectRepositoryWithId: ProjectRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<AnyProject>, AppError>;
}
