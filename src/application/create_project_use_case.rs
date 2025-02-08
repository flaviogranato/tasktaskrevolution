use crate::domain::{
    project::{
        project::{Project, ProjectStatus},
        project_repository::ProjectRepository,
    },
    shared_kernel::errors::DomainError,
};

pub struct CreateProjectUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> CreateProjectUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, name: String, description: Option<String>) -> Result<(), DomainError> {
        let project = Project::new(
            None,
            name.to_string(),
            description.clone(),
            None,
            None,
            ProjectStatus::Planned,
            None,
        );

        self.repository.save(project)?;
        println!("Projeto {} criado", name);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::shared_kernel::errors::DomainError;
    use std::cell::RefCell;

    struct MockProjectRepository {
        should_fail: bool,
        saved_config: RefCell<Option<Project>>,
    }

    impl MockProjectRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: Project) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::Generic("Erro mockado ao salvar".to_string()));
            }
            *self.saved_config.borrow_mut() = Some(project.clone());

            Ok(())
        }
    }
    #[test]
    fn test_create_project_success() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John".to_string();
        let description = Some("a simple test project".to_string());

        let result = use_case.execute(name, description);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockProjectRepository::new(true);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John".to_string();
        let description = Some("a simple test project".to_string());

        let result = use_case.execute(name, description);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John".to_string();
        let description = Some("a simple test project".to_string());
        let _ = use_case.execute(name.clone(), description.clone());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        assert_eq!(saved_config.as_ref().unwrap().name, name);
        assert_eq!(saved_config.as_ref().unwrap().description, description);
    }
}
