#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::{
    any_project::AnyProject,
    repository::{ProjectRepository, ProjectRepositoryWithId},
};
use crate::domain::shared::errors::{DomainError, DomainResult};
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

impl From<crate::domain::shared::errors::DomainError> for CancelAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        CancelAppError::RepositoryError(err.into())
    }
}

pub struct CancelProjectUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> CancelProjectUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    pub fn new(project_repository: PR, code_resolver: CR) -> Self {
        Self {
            project_repository,
            code_resolver,
        }
    }

    pub fn execute(&self, project_code: &str) -> Result<AnyProject, CancelAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(|e| CancelAppError::RepositoryError(AppError::from(e)))?;

        // 2. Load the project aggregate using ID
        let project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| CancelAppError::ProjectNotFound(project_code.to_string()))?;

        // 3. Delegate the cancellation to the project aggregate.
        let cancelled_project = project.cancel().map_err(CancelAppError::AppError)?;

        // 4. Save the updated project aggregate.
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

    struct MockCodeResolver {
        // Mock doesn't need to resolve anything for CancelProjectUseCase
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {}
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, _code: &str) -> DomainResult<String> {
            Ok("mock-project-id".to_string())
        }

        fn resolve_resource_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "resource",
                "Not implemented in mock",
            )))
        }

        fn resolve_task_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }

        fn validate_company_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn validate_project_code(&self, _code: &str) -> DomainResult<()> {
            Ok(())
        }

        fn validate_resource_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "resource",
                "Not implemented in mock",
            )))
        }

        fn validate_task_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> DomainResult<()> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().get(code).cloned())
        }
        fn load(&self) -> DomainResult<AnyProject> {
            unimplemented!()
        }
        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            unimplemented!()
        }
        fn get_next_code(&self) -> DomainResult<String> {
            unimplemented!()
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, _id: &str) -> DomainResult<Option<AnyProject>> {
            // For tests, we'll return the first project in the map
            Ok(self.projects.borrow().values().next().cloned())
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
        let code_resolver = MockCodeResolver::new();
        let use_case = CancelProjectUseCase::new(project_repo, code_resolver);

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
        let code_resolver = MockCodeResolver::new();
        let use_case = CancelProjectUseCase::new(project_repo.clone(), code_resolver);

        let result = use_case.execute("PROJ-1");

        assert!(result.is_ok());
        let cancelled_project = result.unwrap();
        assert_eq!(cancelled_project.status(), ProjectStatus::Cancelled);
    }
}
