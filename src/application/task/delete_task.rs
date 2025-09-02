#![allow(dead_code)]

use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::{any_task::AnyTask, Priority, Category},
};
use std::fmt;

#[derive(Debug)]
pub enum DeleteTaskError {
    ProjectNotFound(String),
    TaskNotFound(String),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for DeleteTaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeleteTaskError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            DeleteTaskError::TaskNotFound(code) => write!(f, "Task with code '{}' not found in project.", code),
            DeleteTaskError::DomainError(message) => write!(f, "Domain error: {}", message),
            DeleteTaskError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DeleteTaskError {}

impl From<DomainError> for DeleteTaskError {
    fn from(err: DomainError) -> Self {
        DeleteTaskError::RepositoryError(err)
    }
}

pub struct DeleteTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> DeleteTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, DeleteTaskError> {
        // 1. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| DeleteTaskError::ProjectNotFound(project_code.to_string()))?;

        // 2. Cancel the task (change its state to Cancelled)
        let cancelled_task = project.cancel_task(task_code).map_err(DeleteTaskError::DomainError)?;

        // 3. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 4. Return the cancelled task.
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
            priority: Priority::default(), category: Category::default(),
        }
        .into()
    }

    fn setup_test_project(tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new()
            .name("Test Project".to_string())
            .code("PROJ-1".to_string())
            .company_code("COMP-001".to_string())
            .created_by("system".to_string())
            .build()
            .unwrap()
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
        let use_case = DeleteTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1");
        assert!(matches!(result, Err(DeleteTaskError::ProjectNotFound(_))));
    }

    #[test]
    fn test_cancel_task_success() {
        // This requires `cancel_task` to be implemented on the real `AnyProject`
        let project = setup_test_project(vec![create_test_task("TSK-1")]);

        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let use_case = DeleteTaskUseCase::new(project_repo.clone());

        let result = use_case.execute("PROJ-1", "TSK-1");

        assert!(result.is_ok());
        let cancelled_task = result.unwrap();
        assert_eq!(cancelled_task.status(), "Cancelled");
    }
}
