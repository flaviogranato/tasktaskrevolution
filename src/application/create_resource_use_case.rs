use crate::domain::{
    resource_management::{AnyResource, repository::ResourceRepository, resource::Resource},
    shared::errors::DomainError,
};

pub struct CreateResourceUseCase<R: ResourceRepository> {
    repository: R,
}

impl<R: ResourceRepository> CreateResourceUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn execute(&self, name: String, resource_type: String) -> Result<(), DomainError> {
        let r = Resource::new(None, name.clone(), None, resource_type, None, 0);
        self.repository.save(r.into())?;
        println!("Recurso {name} criado.");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::shared::errors::DomainError;
    use chrono::{DateTime, Local};
    use std::cell::RefCell;

    struct MockResourceRepository {
        should_fail: bool,
        saved_config: RefCell<Option<AnyResource>>,
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
        fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError> {
            if self.should_fail {
                return Err(DomainError::Generic("Erro mockado ao salvar".to_string()));
            }
            *self.saved_config.borrow_mut() = Some(resource.clone());

            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(vec![])
        }

        fn save_time_off(
            &self,
            _resource_name: String,
            _hours: u32,
            _date: String,
            _description: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn save_vacation(
            &self,
            _resource_name: String,
            _start_date: String,
            _end_date: String,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
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
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.resource_type, resource_type);
        } else {
            panic!("Expected Available resource");
        }
    }
}
