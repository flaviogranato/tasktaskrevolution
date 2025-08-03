use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LinkTaskError {
    #[error("Project with code '{0}' not found.")]
    ProjectNotFound(String),
    #[error("Task with code '{0}' not found in the project.")]
    TaskNotFound(String),
    #[error("Dependency task with code '{0}' not found in the project.")]
    DependencyNotFound(String),
    #[error("A task cannot depend on itself.")]
    SelfDependencyError,
    #[error("Circular dependency detected: adding this link would create a loop.")]
    CircularDependencyError,
    #[error("An unexpected domain rule was violated: {0}")]
    DomainError(String),
    #[error("A repository error occurred: {0}")]
    RepositoryError(#[from] DomainError),
}

impl From<String> for LinkTaskError {
    fn from(err: String) -> Self {
        LinkTaskError::DomainError(err)
    }
}

/// `LinkTaskUseCase` is responsible for creating a dependency between two tasks.
pub struct LinkTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> LinkTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        dependency_code: &str,
    ) -> Result<AnyTask, LinkTaskError> {
        if task_code == dependency_code {
            return Err(LinkTaskError::SelfDependencyError);
        }

        // 1. Load the project aggregate that contains the tasks.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| LinkTaskError::ProjectNotFound(project_code.to_string()))?;

        // 2. Ensure both tasks exist within the project.
        if !project.tasks().contains_key(task_code) {
            return Err(LinkTaskError::TaskNotFound(task_code.to_string()));
        }
        if !project.tasks().contains_key(dependency_code) {
            return Err(LinkTaskError::DependencyNotFound(dependency_code.to_string()));
        }

        // TODO: Add circular dependency check logic here.

        // 3. Add the dependency to the task.
        // This requires a new method on the Project aggregate.
        let updated_task = project.add_dependency_to_task(task_code, dependency_code)?;

        // 4. Save the entire project aggregate with the modified task.
        self.project_repository.save(project)?;

        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added once the domain logic is in place.
}
