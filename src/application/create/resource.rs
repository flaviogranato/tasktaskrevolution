use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::resource_management::{ResourceTypeValidator, repository::{ResourceRepository, ResourceRepositoryWithId}, resource::Resource};

#[derive(Debug, Clone)]
pub struct CreateResourceParams {
    pub name: String,
    pub resource_type: String,
    pub company_code: String,
    pub project_code: Option<String>,
    pub code: Option<String>,
    pub email: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

pub struct CreateResourceUseCase<R: ResourceRepository + ResourceRepositoryWithId, CR: CodeResolverTrait> {
    repository: R,
    code_resolver: CR,
    type_validator: ResourceTypeValidator,
}

impl<R: ResourceRepository + ResourceRepositoryWithId, CR: CodeResolverTrait> CreateResourceUseCase<R, CR> {
    pub fn new(repository: R, code_resolver: CR) -> Self {
        Self {
            repository,
            code_resolver,
            type_validator: ResourceTypeValidator::new(),
        }
    }
    pub fn execute(&self, params: CreateResourceParams) -> Result<(), AppError> {
        // Validate resource type against config
        self.type_validator
            .validate_resource_type(&params.resource_type)
            .map_err(|e| AppError::validation_error("resource_type", e))?;

        let code = match params.code {
            Some(c) => c,
            None => self.repository.get_next_code(&params.resource_type)?,
        };
        let name = params.name.clone();
        let r = Resource::new(
            code,
            params.name,
            params.email,
            params.resource_type,
            params.start_date,
            params.end_date,
            None,
            0,
        );

        // Use the new hierarchical save method
        self.repository
            .save_in_hierarchy(r.into(), &params.company_code, params.project_code.as_deref())?;

        let location = if let Some(proj_code) = params.project_code {
            format!("company {} and project {}", params.company_code, proj_code)
        } else {
            format!("company {}", params.company_code)
        };

        println!("Resource {} created in {}.", name, location);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::resource_management::AnyResource;
    use chrono::{DateTime, Local};
    use std::cell::RefCell;

    struct MockResourceRepository {
        should_fail: bool,
        saved_config: RefCell<Option<AnyResource>>,
    }

    struct MockCodeResolver {
        // Mock doesn't need to resolve anything for CreateResourceUseCase
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
            Err(AppError::validation_error("project", "Not implemented in mock"))
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
            Err(AppError::validation_error("project", "Not implemented in mock"))
        }

        fn validate_resource_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("resource", "Not implemented in mock"))
        }

        fn validate_task_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }
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

        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
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

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, _id: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.saved_config.borrow().clone())
        }
    }

    #[test]
    fn test_create_project_success() {
        let mock_repo = MockResourceRepository::new(false);
        let code_resolver = MockCodeResolver::new();
        let use_case = CreateResourceUseCase::new(mock_repo, code_resolver);
        let name = "John";
        let resource_type = "Developer";

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: None,
            start_date: None,
            end_date: None,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockResourceRepository::new(true);
        let code_resolver = MockCodeResolver::new();
        let use_case = CreateResourceUseCase::new(mock_repo, code_resolver);
        let name = "John";
        let resource_type = "Developer";

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: None,
            start_date: None,
            end_date: None,
        };
        let result = use_case.execute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockResourceRepository::new(false);
        let code_resolver = MockCodeResolver::new();
        let use_case = CreateResourceUseCase::new(mock_repo, code_resolver);
        let name = "John";
        let resource_type = "Developer";
        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: None,
            start_date: None,
            end_date: None,
        };
        let _ = use_case.execute(params);

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
