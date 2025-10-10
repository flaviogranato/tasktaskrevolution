use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::task_management::AnyTask;

/// Repository trait for Task entity operations.
///
/// This trait defines the contract for task persistence operations,
/// following the Repository pattern from Domain-Driven Design.
/// Implementations should be provided by the infrastructure layer.
pub trait TaskRepository {
    /// Saves a task to the repository.
    ///
    /// # Arguments
    /// * `task` - The task to save
    ///
    /// # Returns
    /// * `Ok(saved_task)` - The saved task with any generated fields
    /// * `Err(DomainError)` if an error occurred during save
    fn save(&self, task: AnyTask) -> DomainResult<AnyTask>;

    /// Saves a task in the hierarchical structure (company/project/task).
    ///
    /// # Arguments
    /// * `task` - The task to save
    /// * `company_code` - The company code for the hierarchy
    /// * `project_code` - The project code for the hierarchy
    ///
    /// # Returns
    /// * `Ok(saved_task)` - The saved task with any generated fields
    /// * `Err(DomainError)` if an error occurred during save
    fn save_in_hierarchy(&self, task: AnyTask, company_code: &str, project_code: &str) -> DomainResult<AnyTask>;

    /// Retrieves all tasks from the repository.
    ///
    /// # Returns
    /// * `Ok(tasks)` - Vector of all tasks
    /// * `Err(DomainError)` if an error occurred during retrieval
    fn find_all(&self) -> DomainResult<Vec<AnyTask>>;

    /// Finds a task by its code.
    ///
    /// # Arguments
    /// * `code` - The task code to search for
    ///
    /// # Returns
    /// * `Ok(Some(task))` if the task was found
    /// * `Ok(None)` if no task was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyTask>>;

    /// Finds all tasks for a specific project.
    ///
    /// # Arguments
    /// * `project_code` - The project code to search for
    ///
    /// # Returns
    /// * `Ok(tasks)` - Vector of tasks for the project
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_project(&self, project_code: &str) -> DomainResult<Vec<AnyTask>>;

    /// Finds all tasks for a specific project within a company.
    ///
    /// # Arguments
    /// * `company_code` - The company code
    /// * `project_code` - The project code to search for
    ///
    /// # Returns
    /// * `Ok(tasks)` - Vector of tasks for the project
    /// * `Err(DomainError)` if an error occurred during search
    fn find_all_by_project(&self, company_code: &str, project_code: &str) -> DomainResult<Vec<AnyTask>>;

    /// Generates the next available task code for a project.
    ///
    /// # Arguments
    /// * `project_code` - The project code to generate a task code for
    ///
    /// # Returns
    /// * `Ok(code)` - The next available task code
    /// * `Err(DomainError)` if an error occurred during code generation
    fn get_next_code(&self, project_code: &str) -> DomainResult<String>;
}

/// Extension trait for repositories that support ID-based operations.
///
/// This trait extends the base TaskRepository with ID-based lookup capabilities,
/// which is useful for CQRS patterns and advanced querying scenarios.
pub trait TaskRepositoryWithId: TaskRepository {
    /// Finds a task by its unique identifier.
    ///
    /// # Arguments
    /// * `id` - The task ID to search for
    ///
    /// # Returns
    /// * `Ok(Some(task))` if the task was found
    /// * `Ok(None)` if no task was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyTask>>;
}
