use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CancelTaskError {
    #[error("Project with code '{0}' not found.")]
    ProjectNotFound(String),
    #[error("Task with code '{0}' not found in project.")]
    TaskNotFound(String),
    #[error("An unexpected domain rule was violated: {0}")]
    DomainError(String),
    #[error("A repository error occurred: {0}")]
    RepositoryError(#[from] DomainError),
}

pub struct CancelTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> CancelTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, CancelTaskError> {
        // 1. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| CancelTaskError::ProjectNotFound(project_code.to_string()))?;

        // 2. Delegate the cancellation to the project aggregate.
        project.cancel_task(task_code).map_err(CancelTaskError::DomainError)?;

        // 3. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 4. Return the updated (cancelled) task.
        let cancelled_task = project
            .tasks()
            .get(task_code)
            .cloned()
            .ok_or_else(|| CancelTaskError::TaskNotFound(task_code.to_string()))?;

        Ok(cancelled_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    #[derive(Clone)]
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), DomainError> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            Ok(self.projects.borrow().get(code).cloned())
        }
        fn load(&self) -> Result<AnyProject, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, DomainError> {
            unimplemented!()
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
        }
        .into()
    }

    fn setup_test_project(tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new("Test Project".to_string())
            .code("PROJ-1".to_string())
            .build()
            .into();
        for task in tasks {
            project.add_task(task);
        }
        project
    }

    // --- Tests ---

    #[test]
    fn test_cancel_task_fails_if_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let use_case = CancelTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1");
        assert!(matches!(result, Err(CancelTaskError::ProjectNotFound(_))));
    }

    // TODO: Enable this test once `AnyProject::cancel_task` is implemented.

    #[test]
    fn test_cancel_task_success() {
        // This requires `cancel_task` to be implemented on the real `AnyProject`
        let project = setup_test_project(vec![create_test_task("TSK-1")]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let use_case = CancelTaskUseCase::new(project_repo.clone());

        let result = use_case.execute("PROJ-1", "TSK-1");

        assert!(result.is_ok());
        let cancelled_task = result.unwrap();
        assert_eq!(cancelled_task.status(), "Cancelled");
    }
}
