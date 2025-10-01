#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::{
    any_project::AnyProject,
    repository::{ProjectRepository, ProjectRepositoryWithId},
};
use std::fmt;

#[derive(Debug)]
pub enum UpdateAppError {
    ProjectNotFound(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for UpdateAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            UpdateAppError::AppError(message) => write!(f, "Domain error: {}", message),
            UpdateAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for UpdateAppError {}

impl From<AppError> for UpdateAppError {
    fn from(err: AppError) -> Self {
        UpdateAppError::RepositoryError(err)
    }
}

impl From<crate::domain::shared::errors::DomainError> for UpdateAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        UpdateAppError::RepositoryError(err.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateProjectArgs {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateProjectUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> UpdateProjectUseCase<PR, CR>
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

    pub fn execute(&self, project_code: &str, args: UpdateProjectArgs) -> Result<AnyProject, UpdateAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(UpdateAppError::RepositoryError)?;

        // 2. Load the project aggregate using ID
        let mut project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| UpdateAppError::ProjectNotFound(project_code.to_string()))?;

        // 3. Update the fields on the aggregate.
        if let Some(name) = args.name {
            project.set_name(name);
        }
        if let Some(description) = args.description {
            project.set_description(Some(description));
        }

        // 4. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        Ok(project)
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
    fn create_test_project(code: &str, name: &str, description: Option<&str>) -> AnyProject {
        ProjectBuilder::new()
            .name(name.to_string())
            .code(code.to_string())
            .company_code("COMP-001".to_string())
            .created_by("system".to_string())
            .description(description.map(|s| s.to_string()))
            .build()
            .unwrap()
            .into()
    }

    // --- Tests ---

    #[test]
    fn test_update_project_name_and_description_success() {
        let initial_project = create_test_project("PROJ-1", "Old Name", Some("Old Description"));
        let project_id = initial_project.id().to_string();

        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_id.clone(), initial_project)])),
        };

        let code_resolver = MockCodeResolver::new();
        code_resolver.add_project("PROJ-1", &project_id);

        let use_case = UpdateProjectUseCase::new(project_repo, code_resolver);

        let args = UpdateProjectArgs {
            name: Some("New Name".to_string()),
            description: Some("New Description".to_string()),
        };

        let result = use_case.execute("PROJ-1", args);

        assert!(result.is_ok());
        let updated_project = result.unwrap();
        assert_eq!(updated_project.name(), "New Name");
        assert_eq!(updated_project.description().unwrap(), "New Description");
    }

    #[test]
    fn test_update_project_fails_if_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = UpdateProjectUseCase::new(project_repo, code_resolver);

        let args = UpdateProjectArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-NONEXISTENT", args);
        assert!(matches!(result, Err(UpdateAppError::RepositoryError(_))));
    }
}
