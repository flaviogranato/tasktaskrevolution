use crate::domain::project_management::AnyProject;
use crate::domain::shared::errors::{DomainError, DomainResult};

/// Repository trait for Project entity operations.
/// 
/// This trait defines the contract for project persistence operations,
/// following the Repository pattern from Domain-Driven Design.
/// Implementations should be provided by the infrastructure layer.
pub trait ProjectRepository {
    /// Saves a project to the repository.
    /// 
    /// # Arguments
    /// * `project` - The project to save
    /// 
    /// # Returns
    /// * `Ok(())` if the project was saved successfully
    /// * `Err(DomainError)` if an error occurred during save
    fn save(&self, project: AnyProject) -> DomainResult<()>;

    /// Loads a single project from the repository.
    /// 
    /// # Returns
    /// * `Ok(project)` if a project was found
    /// * `Err(DomainError)` if an error occurred during load
    fn load(&self) -> DomainResult<AnyProject>;

    /// Retrieves all projects from the repository.
    /// 
    /// # Returns
    /// * `Ok(projects)` - Vector of all projects
    /// * `Err(DomainError)` if an error occurred during retrieval
    fn find_all(&self) -> DomainResult<Vec<AnyProject>>;

    /// Finds a project by its code.
    /// 
    /// # Arguments
    /// * `code` - The project code to search for
    /// 
    /// # Returns
    /// * `Ok(Some(project))` if the project was found
    /// * `Ok(None)` if no project was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>>;

    /// Generates the next available project code.
    /// 
    /// # Returns
    /// * `Ok(code)` - The next available project code
    /// * `Err(DomainError)` if an error occurred during code generation
    fn get_next_code(&self) -> DomainResult<String>;
}

/// Extension trait for repositories that support ID-based operations.
/// 
/// This trait extends the base ProjectRepository with ID-based lookup capabilities,
/// which is useful for CQRS patterns and advanced querying scenarios.
pub trait ProjectRepositoryWithId: ProjectRepository {
    /// Finds a project by its unique identifier.
    /// 
    /// # Arguments
    /// * `id` - The project ID to search for
    /// 
    /// # Returns
    /// * `Ok(Some(project))` if the project was found
    /// * `Ok(None)` if no project was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>>;
}
