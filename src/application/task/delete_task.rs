#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
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

impl From<crate::domain::shared::errors::DomainError> for DeleteAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        DeleteAppError::RepositoryError(err.into())
    }
}

impl From<crate::domain::shared::errors::DomainError> for DeleteAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        DeleteAppError::RepositoryError(err.into())
    }
}

pub struct DeleteTaskUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> DeleteTaskUseCase<PR, CR>
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

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, DeleteAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(DeleteAppError::RepositoryError)?;

        // 2. Load the project aggregate using ID
        let mut project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| DeleteAppError::ProjectNotFound(project_code.to_string()))?;

        // 3. Cancel the task (soft delete - change status to Cancelled)
        let cancelled_task = project.cancel_task(task_code).map_err(DeleteAppError::AppError)?;

        // 4. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 5. Return the cancelled task.
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
        fn resolve_company_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn resolve_project_code(&self, code: &str) -> Result<String, AppError> {
            self.project_codes
                .borrow()
                .get(code)
                .cloned()
                .ok_or_else(|| AppError::validation_error("project", format!("Project '{}' not found", code)))
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

        fn validate_project_code(&self, code: &str) -> Result<(), AppError> {
            self.resolve_project_code(code)?;
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
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            self.projects.borrow_mut().insert(project.id().to_string(), project);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            Ok(self.projects.borrow().values().find(|p| p.code() == code).cloned())
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
        fn find_by_id(&self, id: &str) -> Result<Option<AnyProject>, AppError> {
            Ok(self.projects.borrow().get(id).cloned())
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
        let code_resolver = MockCodeResolver::new();
        let use_case = DeleteTaskUseCase::new(project_repo, code_resolver);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1");
        assert!(matches!(result, Err(DeleteAppError::RepositoryError(_))));
    }

    #[test]
    fn test_cancel_task_success() {
        // This requires `cancel_task` to be implemented on the real `AnyProject`
        let project = setup_test_project(vec![create_test_task("TSK-1")]);
        let project_id = project.id().to_string();

        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_id.clone(), project)])),
        };

        let code_resolver = MockCodeResolver::new();
        code_resolver.add_project("PROJ-1", &project_id);

        let use_case = DeleteTaskUseCase::new(project_repo.clone(), code_resolver);

        let result = use_case.execute("PROJ-1", "TSK-1");

        assert!(result.is_ok());
        let cancelled_task = result.unwrap();
        assert_eq!(cancelled_task.status(), "Cancelled");
    }
}
