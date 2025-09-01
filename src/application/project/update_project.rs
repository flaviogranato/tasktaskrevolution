use crate::domain::{
    project_management::{any_project::AnyProject, repository::ProjectRepository},
    shared::errors::DomainError,
};
use std::fmt;

#[derive(Debug)]
pub enum UpdateProjectError {
    ProjectNotFound(String),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for UpdateProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateProjectError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            UpdateProjectError::DomainError(message) => write!(f, "Domain error: {}", message),
            UpdateProjectError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for UpdateProjectError {}

impl From<DomainError> for UpdateProjectError {
    fn from(err: DomainError) -> Self {
        UpdateProjectError::RepositoryError(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateProjectArgs {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct UpdateProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> UpdateProjectUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str, args: UpdateProjectArgs) -> Result<AnyProject, UpdateProjectError> {
        // 1. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| UpdateProjectError::ProjectNotFound(project_code.to_string()))?;

        // 2. Update the fields on the aggregate.
        if let Some(name) = args.name {
            project.set_name(name);
        }
        if let Some(description) = args.description {
            project.set_description(Some(description));
        }

        // 3. Save the updated project aggregate.
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
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(initial_project.code().to_string(), initial_project)])),
        };
        let use_case = UpdateProjectUseCase::new(project_repo);

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
        let use_case = UpdateProjectUseCase::new(project_repo);

        let args = UpdateProjectArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-NONEXISTENT", args);
        assert!(matches!(result, Err(UpdateProjectError::ProjectNotFound(_))));
    }
}
