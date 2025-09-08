#![allow(dead_code)]

use crate::domain::project_management::{any_project::AnyProject, repository::ProjectRepository};
use crate::application::errors::AppError;
use std::fmt;

#[derive(Debug)]
pub enum CancelAppError {
    ProjectNotFound(String),
    ProjectAlreadyCancelled(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for CancelAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            CancelAppError::ProjectAlreadyCancelled(code) => write!(f, "Project '{}' is already cancelled.", code),
            CancelAppError::AppError(message) => write!(f, "Domain error: {}", message),
            CancelAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for CancelAppError {}

impl From<AppError> for CancelAppError {
    fn from(err: AppError) -> Self {
        CancelAppError::RepositoryError(err)
    }
}

pub struct CancelProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> CancelProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str) -> Result<AnyProject, CancelAppError> {
        // 1. Load the project aggregate.
        let project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| CancelAppError::ProjectNotFound(project_code.to_string()))?;

        // 2. Delegate the cancellation to the project aggregate.
        let cancelled_project = project.cancel().map_err(CancelAppError::AppError)?;

        // 3. Save the updated project aggregate.
        self.project_repository.save(cancelled_project.clone())?;

        Ok(cancelled_project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::builder::ProjectBuilder;
    use crate::domain::project_management::project::ProjectStatus;
    use std::{cell::RefCell, collections::HashMap};

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
    fn create_test_project(code: &str) -> AnyProject {
        ProjectBuilder::new()
            .code(code.to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into()
    }

    // --- Tests ---
    #[test]
    fn test_cancel_project_fails_if_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let use_case = CancelProjectUseCase::new(project_repo);

        let result = use_case.execute("PROJ-NONEXISTENT");
        assert!(matches!(result, Err(CancelAppError::ProjectNotFound(_))));
    }

    // TODO: Enable this test once `AnyProject::cancel` is implemented.

    #[test]
    fn test_cancel_project_success() {
        let initial_project = create_test_project("PROJ-1");
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(initial_project.code().to_string(), initial_project)])),
        };
        let use_case = CancelProjectUseCase::new(project_repo.clone());

        let result = use_case.execute("PROJ-1");

        assert!(result.is_ok());
        let cancelled_project = result.unwrap();
        assert_eq!(cancelled_project.status(), ProjectStatus::Cancelled);
    }
}
