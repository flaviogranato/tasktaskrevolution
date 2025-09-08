use crate::domain::resource_management::{repository::ResourceRepository, resource::Resource};
use crate::application::errors::AppError;

pub struct CreateResourceUseCase<R: ResourceRepository> {
    repository: R,
}

impl<R: ResourceRepository> CreateResourceUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn execute(
        &self,
        name: &str,
        resource_type: &str,
        company_code: String,
        project_code: Option<String>,
    ) -> Result<(), AppError> {
        let code = self.repository.get_next_code(resource_type)?;
        let r = Resource::new(code, name.to_string(), None, resource_type.to_string(), None, 0);

        // Use the new hierarchical save method
        self.repository
            .save_in_hierarchy(r.into(), &company_code, project_code.as_deref())?;

        let location = if let Some(proj_code) = project_code {
            format!("company {} and project {}", company_code, proj_code)
        } else {
            format!("company {}", company_code)
        };

        println!("Resource {name} created in {location}.");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::resource_management::AnyResource;
    use crate::application::errors::AppError;
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
        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::ValidationError {
                    field: "repository".to_string(),
                    message: "Erro mockado ao salvar".to_string(),
                });
            }
            *self.saved_config.borrow_mut() = Some(resource.clone());

            Ok(resource)
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            self.save(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(None)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!("Not needed for these tests")
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!("Not needed for these tests")
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    #[test]
    fn test_create_project_success() {
        let mock_repo = MockResourceRepository::new(false);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John";
        let resource_type = "Developer";

        let result = use_case.execute(name, resource_type, "TEST_COMPANY".to_string(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockResourceRepository::new(true);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John";
        let resource_type = "Developer";

        let result = use_case.execute(name, resource_type, "TEST_COMPANY".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockResourceRepository::new(false);
        let use_case = CreateResourceUseCase::new(mock_repo);
        let name = "John";
        let resource_type = "Developer";
        let _ = use_case.execute(name, resource_type, "TEST_COMPANY".to_string(), None);

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.resource_type, resource_type);
            assert_eq!(r.code, "developer-1");
        } else {
            panic!("Expected Available resource");
        }
    }
}
