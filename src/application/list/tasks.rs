#![allow(unused_imports)]
use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::task_management::{Category, Priority, any_task::AnyTask};

pub struct ListTasksUseCase<R: ProjectRepository + ProjectRepositoryWithId, CR: CodeResolverTrait> {
    repository: R,
    code_resolver: CR,
}

impl<R: ProjectRepository + ProjectRepositoryWithId, CR: CodeResolverTrait> ListTasksUseCase<R, CR> {
    pub fn new(repository: R, code_resolver: CR) -> Self {
        Self {
            repository,
            code_resolver,
        }
    }

    pub fn execute(&self, project_code: &str, company_code: &str) -> Result<Vec<AnyTask>, AppError> {
        // 1. Resolve project code to ID
        let project_id = self.code_resolver.resolve_project_code(project_code)?;

        // 2. Use ID for internal operation
        let project = self
            .repository
            .find_by_id(&project_id)?
            .ok_or_else(|| AppError::ProjectNotFound {
                code: project_code.to_string(),
            })?;

        // 3. Verify the project belongs to the correct company
        if project.company_code() == company_code {
            let tasks = project.tasks().values().cloned().collect::<Vec<_>>();
            Ok(tasks)
        } else {
            Err(AppError::ProjectNotFound {
                code: project_code.to_string(),
            })
        }
    }

    pub fn execute_all_by_company(&self, company_code: &str) -> Result<Vec<AnyTask>, AppError> {
        let projects = self.repository.find_all()?;
        let mut all_tasks = Vec::new();

        for project in projects {
            if project.company_code() == company_code {
                let tasks = project.tasks().values().cloned().collect::<Vec<_>>();
                all_tasks.extend(tasks);
            }
        }

        Ok(all_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::domain::task_management::{state::Planned, task::Task};
    use chrono::NaiveDate;

    use uuid7::uuid7;

    struct MockProjectRepository {
        project: Option<AnyProject>,
        projects: Vec<AnyProject>,
        should_fail: bool,
    }

    struct MockCodeResolver {
        should_fail: bool,
    }

    impl MockProjectRepository {
        fn new_with_project(project: AnyProject) -> Self {
            Self {
                project: Some(project),
                projects: vec![],
                should_fail: false,
            }
        }

        fn new_with_projects(projects: Vec<AnyProject>) -> Self {
            Self {
                project: None,
                projects,
                should_fail: false,
            }
        }

        fn new_with_failure() -> Self {
            Self {
                project: None,
                projects: vec![],
                should_fail: true,
            }
        }
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self { should_fail: false }
        }

        fn new_with_failure() -> Self {
            Self { should_fail: true }
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("company", "Mock failure")))
            } else {
                Ok("company-id".to_string())
            }
        }

        fn resolve_project_code(&self, _code: &str) -> DomainResult<String> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("project", "Mock failure")))
            } else {
                Ok("project-id".to_string())
            }
        }

        fn resolve_resource_code(&self, _code: &str) -> DomainResult<String> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error(
                    "resource",
                    "Mock failure",
                )))
            } else {
                Ok("resource-id".to_string())
            }
        }

        fn resolve_task_code(&self, _code: &str) -> DomainResult<String> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("task", "Mock failure")))
            } else {
                Ok("task-id".to_string())
            }
        }

        fn validate_company_code(&self, _code: &str) -> DomainResult<()> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("company", "Mock failure")))
            } else {
                Ok(())
            }
        }

        fn validate_project_code(&self, _code: &str) -> DomainResult<()> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("project", "Mock failure")))
            } else {
                Ok(())
            }
        }

        fn validate_resource_code(&self, _code: &str) -> DomainResult<()> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error(
                    "resource",
                    "Mock failure",
                )))
            } else {
                Ok(())
            }
        }

        fn validate_task_code(&self, _code: &str) -> DomainResult<()> {
            if self.should_fail {
                Err(DomainError::from(AppError::validation_error("task", "Mock failure")))
            } else {
                Ok(())
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn load(&self) -> DomainResult<AnyProject> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "load".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            self.project.clone().ok_or(DomainError::ProjectNotFound {
                code: "not-found".to_string(),
            })
        }

        fn save(&self, _project: AnyProject) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "save".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(())
        }

        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_all".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.projects.clone())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_by_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }

            if let Some(ref project) = self.project
                && project.code() == code
            {
                return Ok(Some(project.clone()));
            }

            for project in &self.projects {
                if project.code() == code {
                    return Ok(Some(project.clone()));
                }
            }

            Ok(None)
        }

        fn get_next_code(&self) -> DomainResult<String> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "get_next_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok("PROJ-NEXT".to_string())
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_by_id".to_string(),
                    details: "Mock failure".to_string(),
                });
            }

            // For testing, we'll map the fixed ID "project-id" to the first project
            if id == "project-id" {
                if let Some(ref project) = self.project {
                    return Ok(Some(project.clone()));
                }
                if let Some(project) = self.projects.first() {
                    return Ok(Some(project.clone()));
                }
            }

            Ok(None)
        }
    }

    fn create_test_task(code: &str, name: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: name.to_string(),
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

    fn create_project_with_tasks(tasks: Vec<AnyTask>) -> AnyProject {
        let mut builder = ProjectBuilder::new()
            .code("PROJ-1".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string());

        for task in tasks {
            builder = builder.add_task(task);
        }

        builder.build().unwrap().into()
    }

    #[test]
    fn test_list_tasks_success() {
        let tasks = vec![
            create_test_task("TSK-1", "First task"),
            create_test_task("TSK-2", "Second task"),
        ];
        let project = create_project_with_tasks(tasks);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "COMP-001").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "First task"));
        assert!(result.iter().any(|t| t.code() == "TSK-2"));
    }

    #[test]
    fn test_list_tasks_empty() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "COMP-001").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_list_tasks_project_not_found() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let code_resolver = MockCodeResolver::new_with_failure();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute("NONEXISTENT", "COMP-001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationError { field, message } => {
                assert_eq!(field, "project");
                assert_eq!(message, "Mock failure");
            }
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_by_id");
                assert_eq!(details, "Mock failure");
            }
            other => panic!("Expected ValidationError or IoError, got: {:?}", other),
        }
    }

    #[test]
    fn test_list_tasks_wrong_company() {
        let tasks = vec![create_test_task("TSK-1", "First task")];
        let project = create_project_with_tasks(tasks);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "WRONG-COMPANY");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ProjectNotFound { code } => assert_eq!(code, "PROJ-1"),
            _ => panic!("Expected ProjectNotFound error"),
        }
    }

    #[test]
    fn test_list_tasks_repository_error() {
        let mock_repo = MockProjectRepository::new_with_failure();
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "COMP-001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_by_id");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_execute_all_by_company_success() {
        let tasks1 = vec![create_test_task("TSK-1", "Task 1")];
        let tasks2 = vec![create_test_task("TSK-2", "Task 2")];
        let project1 = create_project_with_tasks(tasks1);
        let project2 = create_project_with_tasks(tasks2);

        let projects = vec![project1, project2];
        let mock_repo = MockProjectRepository::new_with_projects(projects);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute_all_by_company("COMP-001").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "Task 1"));
        assert!(result.iter().any(|t| t.name() == "Task 2"));
    }

    #[test]
    fn test_execute_all_by_company_empty() {
        let mock_repo = MockProjectRepository::new_with_projects(vec![]);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute_all_by_company("COMP-001").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_all_by_company_wrong_company() {
        let tasks = vec![create_test_task("TSK-1", "Task 1")];
        let project = create_project_with_tasks(tasks);
        let projects = vec![project];
        let mock_repo = MockProjectRepository::new_with_projects(projects);
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute_all_by_company("WRONG-COMPANY").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_all_by_company_repository_error() {
        let mock_repo = MockProjectRepository::new_with_failure();
        let code_resolver = MockCodeResolver::new();
        let use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        let result = use_case.execute_all_by_company("COMP-001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_all");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_list_tasks_use_case_creation() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let code_resolver = MockCodeResolver::new();
        let _use_case = ListTasksUseCase::new(mock_repo, code_resolver);

        // Test that the use case was created successfully
        // If we get here, creation succeeded
    }
}
