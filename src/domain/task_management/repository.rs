use super::AnyTask;
use crate::domain::shared::errors::DomainError;
use std::path::Path;

/// Trait defining the contract for task persistence.
/// It works with `AnyTask` to handle tasks in any state.
pub trait TaskRepository {
    /// Saves a task, regardless of its state.
    fn save(&self, task: AnyTask) -> Result<(), DomainError>;

    /// Loads a single task from a specific file path.
    #[allow(dead_code)]
    fn load(&self, path: &Path) -> Result<AnyTask, DomainError>;

    /// Finds a task by its unique code.
    fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError>;

    /// Finds a task by its unique ID.
    fn find_by_id(&self, id: &str) -> Result<Option<AnyTask>, DomainError>;

    /// Returns all tasks from the repository.
    fn find_all(&self) -> Result<Vec<AnyTask>, DomainError>;

    /// Deletes a task by its unique ID.
    #[allow(dead_code)]
    fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Finds all tasks assigned to a specific resource.
    #[allow(dead_code)]
    fn find_by_assignee(&self, assignee: &str) -> Result<Vec<AnyTask>, DomainError>;

    /// Finds all tasks that are active within a given date range.
    #[allow(dead_code)]
    fn find_by_date_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<AnyTask>, DomainError>;

    fn get_next_code(&self) -> Result<String, DomainError>;
}
