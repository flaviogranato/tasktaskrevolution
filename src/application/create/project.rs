use crate::domain::{
    project_management::{builder::ProjectBuilder, repository::ProjectRepository},
    shared::errors::DomainError,
};

pub struct CreateProjectUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> CreateProjectUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, name: &str, description: Option<&str>) -> Result<(), DomainError> {
        let code = self.repository.get_next_code()?;
        
        // Use the typestate builder for type safety
        let mut project = ProjectBuilder::new(name.to_string())
            .code(code)
            .end_date("2024-12-31".to_string()); // Required for WithDates state
        
        // Add description if provided
        if let Some(desc) = description {
            project = project.description(Some(desc.to_string()));
        }
        
        let project = project.build(); // This returns Project<Planned> (legacy method)

        self.repository.save(project.into())?;
        println!("Projeto {name} criado");
        Ok(())
    }

    #[allow(dead_code)]
    #[cfg(test)]
    pub fn get_repository(&self) -> &R {
        &self.repository
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::domain::shared::errors::DomainError;
    use crate::domain::shared::errors::DomainErrorKind;
    use std::cell::RefCell;

    struct MockProjectRepository {
        should_fail: bool,
        saved_config: RefCell<Option<AnyProject>>,
        project: AnyProject,
    }

    impl MockProjectRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
                project: ProjectBuilder::new("John".to_string())
                    .code("proj-1".to_string())
                    .end_date("2024-12-31".to_string())
                    .build()
                    .into(),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::new(DomainErrorKind::Generic { 
                    message: "Erro mockado ao salvar".to_string() 
                }));
            }
            *self.saved_config.borrow_mut() = Some(project);
            Ok(())
        }

        fn load(&self) -> Result<AnyProject, DomainError> {
            Ok(self.project.clone())
        }

        fn get_next_code(&self) -> Result<String, DomainError> {
            Ok("proj-1".to_string()) // Always return a fixed code for tests
        }

        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            Ok(vec![self.project.clone()])
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            if self.project.code() == code {
                Ok(Some(self.project.clone()))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_create_project_success() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");

        let result = use_case.execute(name, description);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockProjectRepository::new(true);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");

        let result = use_case.execute(name, description);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");
        let _ = use_case.execute(name, description);

        let saved_config = use_case.get_repository().saved_config.borrow();
        assert!(saved_config.is_some());
        let any_project = saved_config.as_ref().unwrap();
        assert_eq!(any_project.name(), name);
        if let AnyProject::Planned(p) = any_project {
            assert_eq!(p.description.as_deref(), description);
        } else {
            panic!("Expected Planned project");
        }
    }
}
