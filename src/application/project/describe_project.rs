use crate::application::errors::AppError;
use crate::domain::project_management::{any_project::AnyProject, repository::ProjectRepository};
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

pub struct DescribeProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> DescribeProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str) -> Result<AnyProject, DescribeAppError> {
        self.project_repository
            .find_by_code(project_code)?
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
            .description(Some("A test project.".to_string()))
            .build()
            .unwrap()
            .into()
    }

    #[test]
    fn test_describe_project_success() {
        let project_code = "PROJ-1";
        let project = create_test_project(project_code);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let use_case = DescribeProjectUseCase::new(project_repo);

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
        let use_case = DescribeProjectUseCase::new(project_repo);

        let result = use_case.execute("PROJ-NONEXISTENT");

        assert!(matches!(result, Err(DescribeAppError::ProjectNotFound(_))));
    }
}
