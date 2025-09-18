#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::task_management::{Category, Priority, any_task::AnyTask};
use std::fmt;

#[derive(Debug)]
pub enum DeleteAppError {
    ProjectNotFound(String),
    TaskNotFound(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for DeleteAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeleteAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            DeleteAppError::TaskNotFound(code) => write!(f, "Task with code '{}' not found in project.", code),
            DeleteAppError::AppError(message) => write!(f, "Domain error: {}", message),
            DeleteAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DeleteAppError {}

impl From<AppError> for DeleteAppError {
    fn from(err: AppError) -> Self {
        DeleteAppError::RepositoryError(err)
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

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, DeleteAppError> {
        // 1. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| DeleteAppError::ProjectNotFound(project_code.to_string()))?;

        // 2. Cancel the task (soft delete - change status to Cancelled)
        let cancelled_task = project.cancel_task(task_code).map_err(DeleteAppError::AppError)?;

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
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            Ok(self.projects.borrow().get(code).cloned())
        }
        fn load(&self) -> Result<AnyProject, AppError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, AppError> {
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
            priority: Priority::default(),
            category: Category::default(),
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
        assert!(matches!(result, Err(DeleteAppError::ProjectNotFound(_))));
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
