use crate::domain::resource::{resource::Resource, resource_repository::ResourceRepository};
use crate::domain::shared_kernel::errors::DomainError;

pub struct CreateResourceUseCase<R: ResourceRepository> {
    repository: R,
}

impl<R: ResourceRepository> CreateResourceUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn execute(&self, name: String, resource_type: String) -> Result<(), DomainError> {
        let r = Resource::new(None, name.clone(), None, resource_type, None, None, 0);
        self.repository.save(r)?;
        println!("Recurso {} criado.", name);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::shared_kernel::errors::DomainError;
    use std::cell::RefCell;

    struct MockResourceRepository {
        should_fail: bool,
        saved_config: RefCell<Option<Resource>>,
    }

    impl MockResourceRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: Resource) -> Result<Resource, DomainError> {
            if self.should_fail {
                return Err(DomainError::Generic("Erro mockado ao salvar".to_string()));
            }
            *self.saved_config.borrow_mut() = Some(resource.clone());

            Ok(resource)
        }
    }
    #[test]
    fn test_create_project_success() {
        let mock_repo = MockResourceRepository::new(false);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John".to_string();
        let resource_type = "a simple test project".to_string();

        let result = use_case.execute(name, resource_type);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockResourceRepository::new(true);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John".to_string();
        let resource_type = "a simple test project".to_string();

        let result = use_case.execute(name, resource_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockResourceRepository::new(false);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John".to_string();
        let resource_type = "a simple test project".to_string();
        let _ = use_case.execute(name.clone(), resource_type.clone());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        assert_eq!(saved_config.as_ref().unwrap().name, name);
        assert_eq!(saved_config.as_ref().unwrap().resource_type, resource_type);
    }
}
