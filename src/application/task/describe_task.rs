#![allow(unused_imports)]
use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
use crate::domain::task_management::{Category, Priority, any_task::AnyTask};
use std::fmt;

#[derive(Debug)]
pub enum DescribeAppError {
    ProjectNotFound(String),
    TaskNotFound(String),
    RepositoryError(AppError),
}

impl fmt::Display for DescribeAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescribeAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            DescribeAppError::TaskNotFound(code) => write!(f, "Task with code '{}' not found in project.", code),
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


pub struct DescribeTaskUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> DescribeTaskUseCase<PR, CR>
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

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, DescribeAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(DescribeAppError::RepositoryError)?;

        // 2. Load the project aggregate using ID
        let project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| DescribeAppError::ProjectNotFound(project_code.to_string()))?;

        let task = project
            .tasks()
            .get(task_code)
            .cloned()
            .ok_or_else(|| DescribeAppError::TaskNotFound(task_code.to_string()))?;

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::any_project::AnyProject;
    use crate::domain::{
        project_management::builder::ProjectBuilder,
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    struct MockCodeResolver {
        // Mock doesn't need to resolve anything for DescribeTaskUseCase
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {}
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn resolve_project_code(&self, _code: &str) -> Result<String, AppError> {
            Ok("mock-project-id".to_string())
        }

        fn resolve_resource_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("resource", "Not implemented in mock"))
        }

        fn resolve_task_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }

        fn validate_company_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn validate_project_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn validate_resource_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("resource", "Not implemented in mock"))
        }

        fn validate_task_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> Result<(), AppError> {
            unimplemented!()
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

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, _id: &str) -> Result<Option<AnyProject>, AppError> {
            // For tests, we'll return the first project in the map
            Ok(self.projects.borrow().values().next().cloned())
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task.".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["dev-1".to_string()],
            priority: Priority::default(),
            category: Category::default(),
        }
        .into()
    }

    fn create_test_project(code: &str, tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new()
            .code(code.to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        for task in tasks {
            project.add_task(task);
        }
        project
    }

    #[test]
    fn test_describe_task_success() {
        let project_code = "PROJ-1";
        let task_code = "TSK-1";
        let project = create_test_project(project_code, vec![create_test_task(task_code)]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_code.to_string(), project)])),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DescribeTaskUseCase::new(project_repo, code_resolver);

        let result = use_case.execute(project_code, task_code);

        assert!(result.is_ok());
        let found_task = result.unwrap();
        assert_eq!(found_task.code(), task_code);
        assert!(!found_task.assigned_resources().is_empty());
    }

    #[test]
    fn test_describe_task_not_found() {
        let project_code = "PROJ-1";
        let project = create_test_project(project_code, vec![]); // No tasks
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_code.to_string(), project)])),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DescribeTaskUseCase::new(project_repo, code_resolver);

        let result = use_case.execute(project_code, "TSK-NONEXISTENT");

        assert!(matches!(result, Err(DescribeAppError::TaskNotFound(_))));
    }

    #[test]
    fn test_describe_task_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DescribeTaskUseCase::new(project_repo, code_resolver);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1");

        assert!(matches!(result, Err(DescribeAppError::ProjectNotFound(_))));
    }
}
