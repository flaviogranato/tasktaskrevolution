use crate::application::errors::AppError;
use crate::domain::company_settings::repository::ConfigRepository;
use crate::domain::resource_management::{
    ResourceTypeValidator,
    repository::ResourceRepository,
    resource::{Resource, ResourceScope},
};
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::infrastructure::persistence::config_repository::FileConfigRepository;

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
    pub scope: ResourceScope,
}

pub struct CreateResourceUseCase<R: ResourceRepository> {
    repository: R,
    type_validator: ResourceTypeValidator,
}

impl<R: ResourceRepository> CreateResourceUseCase<R> {
    pub fn new<C: ConfigRepository + 'static>(repository: R, _config_repository: C) -> Self {
        Self {
            repository,
            type_validator: ResourceTypeValidator::new(),
        }
    }
    pub fn execute(&self, params: CreateResourceParams) -> Result<(), AppError> {
        // Validate resource type against config
        let config_repo = FileConfigRepository::new();
        self.type_validator
            .validate_resource_type(&params.resource_type, &config_repo)
            .map_err(|e| AppError::validation_error("resource_type", e))?;

        // Basic scope validation
        self.validate_resource_scope(&params)?;

        let code = match params.code {
            Some(c) => c,
            None => self.repository.get_next_code(&params.resource_type)?,
        };
        let name = params.name.clone();
        let project_id = match params.scope {
            ResourceScope::Company => None,
            ResourceScope::Project => params.project_code.clone(),
        };

        let r = Resource::new(
            code,
            params.name,
            params.email,
            params.resource_type,
            params.scope,
            project_id,
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

        Ok(())
    }

    /// Validates resource scope constraints
    fn validate_resource_scope(&self, params: &CreateResourceParams) -> Result<(), AppError> {
        match params.scope {
            ResourceScope::Company => {
                // Company-scoped resources don't need project validation
                Ok(())
            }
            ResourceScope::Project => {
                // Project-scoped resources must have a valid project
                if params.project_code.is_none() {
                    return Err(AppError::validation_error(
                        "project_code",
                        "Project-scoped resources must specify a project",
                    ));
                }

                // Validate business rules for project-scoped resources
                self.validate_scope_business_rules(params)?;
                Ok(())
            }
        }
    }

    /// Validates resource scope against business rules
    fn validate_scope_business_rules(&self, params: &CreateResourceParams) -> Result<(), AppError> {
        match params.scope {
            ResourceScope::Company => {
                // Company-scoped resources can be any type
                Ok(())
            }
            ResourceScope::Project => {
                // Some resource types might be restricted to company scope
                match params.resource_type.to_lowercase().as_str() {
                    "manager" | "director" | "executive" => {
                        Err(AppError::validation_error(
                            "resource_scope",
                            format!("Resource type '{}' should be company-scoped, not project-scoped", params.resource_type),
                        ))
                    }
                    _ => Ok(()),
                }
            }
        }
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

    impl MockResourceRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Mocked save error".to_string(),
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
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(vec![])
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(vec![])
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(None)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<AnyResource> {
            unimplemented!("Not needed for these tests")
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            unimplemented!("Not needed for these tests")
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, resource_type: &str) -> DomainResult<String> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    struct MockConfigRepository;

    impl MockConfigRepository {
        fn new() -> Self {
            Self
        }
    }

    impl ConfigRepository for MockConfigRepository {
        fn save(
            &self,
            _config: crate::domain::company_settings::config::Config,
            _path: &std::path::Path,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn create_repository_dir(&self, _path: &std::path::Path) -> DomainResult<()> {
            Ok(())
        }

        fn load(&self) -> DomainResult<(crate::domain::company_settings::config::Config, std::path::PathBuf)> {
            let config = crate::domain::company_settings::config::Config::new(
                "Test Manager".to_string(),
                "test@example.com".to_string(),
                "UTC".to_string(),
            );
            Ok((config, std::path::PathBuf::from("/test/path")))
        }
    }

    #[test]
    fn test_create_project_success() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
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
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockResourceRepository::new(true);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
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
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
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
            scope: ResourceScope::Company,
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

    #[test]
    fn test_create_resource_with_custom_code() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Custom Resource";
        let resource_type = "Designer";
        let custom_code = "DES-001".to_string();

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: Some(custom_code.clone()),
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.code, custom_code);
        } else {
            panic!("Expected Available resource");
        }
    }

    #[test]
    fn test_create_resource_with_email() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Resource with Email";
        let resource_type = "Manager";
        let email = "test@example.com".to_string();

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: Some(email.clone()),
            start_date: None,
            end_date: None,
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.email, Some(email));
        } else {
            panic!("Expected Available resource");
        }
    }

    #[test]
    fn test_create_resource_with_dates() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Resource with Dates";
        let resource_type = "Business Analyst";
        let start_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: None,
            start_date: Some(start_date),
            end_date: Some(end_date),
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.start_date, Some(start_date));
            assert_eq!(r.end_date, Some(end_date));
        } else {
            panic!("Expected Available resource");
        }
    }

    #[test]
    fn test_create_resource_with_project_code() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Project Resource";
        let resource_type = "Developer";
        let project_code = "PROJ-001".to_string();

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: Some(project_code),
            code: None,
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Project,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
    }

    #[test]
    fn test_create_resource_use_case_creation() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let _use_case = CreateResourceUseCase::new(mock_repo, config_repo);

        // Test that the use case was created successfully
        // If we get here, creation succeeded
    }

    #[test]
    fn test_create_resource_repository_error() {
        let mock_repo = MockResourceRepository::new(true);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Test Resource";
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
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::ValidationError { field, message } => {
                assert_eq!(field, "repository");
                assert_eq!(message, "Mocked save error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_create_resource_minimal_parameters() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Minimal Resource";
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
            scope: ResourceScope::Company,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

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

    #[test]
    fn test_create_resource_with_all_parameters() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);
        let name = "Complete Resource";
        let resource_type = "Developer";
        let custom_code = "DEV-001".to_string();
        let email = "manager@example.com".to_string();
        let project_code = "PROJ-001".to_string();
        let start_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let params = CreateResourceParams {
            name: name.to_string(),
            resource_type: resource_type.to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: Some(project_code),
            code: Some(custom_code.clone()),
            email: Some(email.clone()),
            start_date: Some(start_date),
            end_date: Some(end_date),
            scope: ResourceScope::Project,
        };
        let result = use_case.execute(params);
        assert!(result.is_ok());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        let any_resource = saved_config.as_ref().unwrap();
        assert_eq!(any_resource.name(), name);
        if let AnyResource::Available(r) = any_resource {
            assert_eq!(r.code, custom_code);
            assert_eq!(r.resource_type, resource_type);
            assert_eq!(r.email, Some(email));
            assert_eq!(r.start_date, Some(start_date));
            assert_eq!(r.end_date, Some(end_date));
        } else {
            panic!("Expected Available resource");
        }
    }

    #[test]
    fn test_validate_resource_scope_project_without_project_code() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);

        let params = CreateResourceParams {
            name: "Test Resource".to_string(),
            resource_type: "Developer".to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None, // No project code for project-scoped resource
            code: None,
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Project,
        };

        let result = use_case.execute(params);
        assert!(result.is_err());
        
        if let Err(AppError::ValidationError { field, message }) = result {
            assert_eq!(field, "project_code");
            assert!(message.contains("must specify a project"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_resource_scope_manager_project_scope() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);

        let params = CreateResourceParams {
            name: "Test Manager".to_string(),
            resource_type: "Manager".to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: Some("PROJ-001".to_string()),
            code: None,
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Project, // Manager should be company-scoped
        };

        let result = use_case.execute(params);
        assert!(result.is_err());
        
        if let Err(AppError::ValidationError { field, message }) = result {
            assert_eq!(field, "resource_scope");
            assert!(message.contains("should be company-scoped"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_validate_resource_scope_company_scope_success() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);

        let params = CreateResourceParams {
            name: "Test Manager".to_string(),
            resource_type: "Manager".to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: None,
            code: None,
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Company, // Manager should be company-scoped
        };

        let result = use_case.execute(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_resource_scope_developer_project_scope_success() {
        let mock_repo = MockResourceRepository::new(false);
        let config_repo = MockConfigRepository::new();
        let use_case = CreateResourceUseCase::new(mock_repo, config_repo);

        let params = CreateResourceParams {
            name: "Test Developer".to_string(),
            resource_type: "Developer".to_string(),
            company_code: "TEST_COMPANY".to_string(),
            project_code: Some("PROJ-001".to_string()),
            code: None,
            email: None,
            start_date: None,
            end_date: None,
            scope: ResourceScope::Project, // Developer can be project-scoped
        };

        let result = use_case.execute(params);
        assert!(result.is_ok());
    }
}
