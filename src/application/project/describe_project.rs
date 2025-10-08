use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::{
    any_project::AnyProject,
    repository::{ProjectRepository, ProjectRepositoryWithId},
};
use std::fmt;

#[derive(Debug)]
pub enum DescribeAppError {
    ProjectNotFound(String),
    RepositoryError(AppError),
}

impl fmt::Display for DescribeAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescribeAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            DescribeAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DescribeAppError {}

impl From<AppError> for DescribeAppError {
    fn from(err: AppError) -> Self {
        DescribeAppError::RepositoryError(err)
    }
}

impl From<crate::domain::shared::errors::DomainError> for DescribeAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        DescribeAppError::RepositoryError(err.into())
    }
}

pub struct DescribeProjectUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> DescribeProjectUseCase<PR, CR>
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

    pub fn execute(&self, project_code: &str) -> Result<AnyProject, DescribeAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(|e| DescribeAppError::RepositoryError(AppError::from(e)))?;

        // 2. Use ID for internal operation
        self.project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| DescribeAppError::ProjectNotFound(project_code.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::builder::ProjectBuilder;
    use std::{cell::RefCell, collections::HashMap};

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    struct MockCodeResolver {
        project_codes: RefCell<HashMap<String, String>>, // code -> id
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                project_codes: RefCell::new(HashMap::new()),
            }
        }

        fn add_project(&self, code: &str, id: &str) {
            self.project_codes.borrow_mut().insert(code.to_string(), id.to_string());
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, code: &str) -> DomainResult<String> {
            self.project_codes.borrow().get(code).cloned().ok_or_else(|| {
                DomainError::from(AppError::validation_error(
                    "project",
                    format!("Project '{}' not found", code),
                ))
            })
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

        fn validate_project_code(&self, code: &str) -> DomainResult<()> {
            self.resolve_project_code(code)?;
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
            self.projects.borrow_mut().insert(project.id().to_string(), project);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().values().find(|p| p.code() == code).cloned())
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
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().get(id).cloned())
        }
    }

    // --- Helpers ---
    fn create_test_project(code: &str) -> AnyProject {
        ProjectBuilder::new()
            .code(code.to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .description(Some("A test project.".to_string()))
            .build()
            .unwrap()
            .into()
    }

    #[test]
    fn test_describe_project_success() {
        let project_code = "PROJ-1";
        let project = create_test_project(project_code);
        let project_id = project.id().to_string();

        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_id.clone(), project)])),
        };

        let code_resolver = MockCodeResolver::new();
        code_resolver.add_project(project_code, &project_id);

        let use_case = DescribeProjectUseCase::new(project_repo, code_resolver);

        let result = use_case.execute(project_code);

        assert!(result.is_ok());
        let found_project = result.unwrap();
        assert_eq!(found_project.code(), project_code);
        assert_eq!(found_project.name(), "Test Project");
    }

    #[test]
    fn test_describe_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DescribeProjectUseCase::new(project_repo, code_resolver);

        let result = use_case.execute("PROJ-NONEXISTENT");

        assert!(matches!(result, Err(DescribeAppError::RepositoryError(_))));
    }
}
