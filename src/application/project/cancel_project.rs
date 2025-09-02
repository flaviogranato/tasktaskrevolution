use crate::domain::{
    project_management::{any_project::AnyProject, repository::ProjectRepository},
    shared::errors::DomainError,
};
use std::fmt;

#[derive(Debug)]
pub enum CancelProjectError {
    ProjectNotFound(String),
    ProjectAlreadyCancelled(String),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for CancelProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CancelProjectError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            CancelProjectError::ProjectAlreadyCancelled(code) => write!(f, "Project '{}' is already cancelled.", code),
            CancelProjectError::DomainError(message) => write!(f, "Domain error: {}", message),
            CancelProjectError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for CancelProjectError {}

impl From<DomainError> for CancelProjectError {
    fn from(err: DomainError) -> Self {
        CancelProjectError::RepositoryError(err)
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

    pub fn execute(&self, project_code: &str) -> Result<AnyProject, CancelProjectError> {
        // 1. Load the project aggregate.
        let project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| CancelProjectError::ProjectNotFound(project_code.to_string()))?;

        // 2. Delegate the cancellation to the project aggregate.
        let cancelled_project = project.cancel().map_err(CancelProjectError::DomainError)?;

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
        assert!(matches!(result, Err(CancelProjectError::ProjectNotFound(_))));
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
