use crate::application::errors::AppError;
use crate::domain::resource_management::{any_resource::AnyResource, repository::ResourceRepository};

#[derive(Debug, Clone)]
pub struct ResourceWithContext {
    pub resource: AnyResource,
    pub company_code: String,
    pub project_codes: Vec<String>,
}

pub struct ListResourcesUseCase<R: ResourceRepository> {
    repository: R,
}

impl<R: ResourceRepository> ListResourcesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<AnyResource>, AppError> {
        self.repository.find_all()
    }

    pub fn execute_by_company(&self, company_code: &str) -> Result<Vec<AnyResource>, AppError> {
        self.repository.find_by_company(company_code)
    }

    pub fn execute_with_context(&self) -> Result<Vec<ResourceWithContext>, AppError> {
        let resources_with_context = self.repository.find_all_with_context()?;
        let mut result = Vec::new();

        for (resource, company_code, project_codes) in resources_with_context {
            result.push(ResourceWithContext {
                resource,
                company_code,
                project_codes,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::{
        any_resource::AnyResource,
        resource::{Resource, ResourceScope},
    };

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
        should_fail: bool,
    }

    impl MockResourceRepository {
        fn new(resources: Vec<AnyResource>) -> Self {
            Self {
                resources,
                should_fail: false,
            }
        }

        fn new_with_failure() -> Self {
            Self {
                resources: vec![],
                should_fail: true,
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_all".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.resources.clone())
        }

        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_by_company".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            // Filter resources by company (simplified for testing)
            Ok(self.resources.clone())
        }

        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_all_with_context".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self
                .resources
                .iter()
                .map(|r| (r.clone(), "TEST-COMPANY".to_string(), vec!["PROJ-1".to_string()]))
                .collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_by_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.resources.iter().find(|r| r.code() == code).cloned())
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save_in_hierarchy".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(resource)
        }

        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(resource)
        }

        fn save_time_off(&self, _r: &str, _h: u32, _d: &str, _desc: Option<String>) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save_time_off".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            // Return a mock resource for testing
            Ok(create_test_resource("Test", "test-1", "Developer"))
        }

        fn save_vacation(
            &self,
            _r: &str,
            _s: &str,
            _e: &str,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save_vacation".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            // Return a mock resource for testing
            Ok(create_test_resource("Test", "test-1", "Developer"))
        }

        fn check_if_layoff_period(
            &self,
            _s: &chrono::DateTime<chrono::Local>,
            _e: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            false // Mock implementation
        }

        fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "get_next_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    fn create_test_resource(name: &str, code: &str, r_type: &str) -> AnyResource {
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            r_type.to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            0,
        )
        .into()
    }

    #[test]
    fn test_list_resources_success() {
        let resources = vec![
            create_test_resource("Alice", "dev-1", "Developer"),
            create_test_resource("Bob", "qa-1", "QA"),
        ];
        let mock_repo = MockResourceRepository::new(resources);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|r| r.name() == "Alice"));
        assert!(result.iter().any(|r| r.code() == "qa-1"));
    }

    #[test]
    fn test_list_resources_empty() {
        let mock_repo = MockResourceRepository::new(vec![]);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_list_resources_repository_error() {
        let mock_repo = MockResourceRepository::new_with_failure();
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_all");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_execute_by_company_success() {
        let resources = vec![
            create_test_resource("Alice", "dev-1", "Developer"),
            create_test_resource("Bob", "qa-1", "QA"),
        ];
        let mock_repo = MockResourceRepository::new(resources);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_by_company("TEST-COMPANY").unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|r| r.name() == "Alice"));
        assert!(result.iter().any(|r| r.code() == "qa-1"));
    }

    #[test]
    fn test_execute_by_company_empty() {
        let mock_repo = MockResourceRepository::new(vec![]);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_by_company("TEST-COMPANY").unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_by_company_repository_error() {
        let mock_repo = MockResourceRepository::new_with_failure();
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_by_company("TEST-COMPANY");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_by_company");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_execute_with_context_success() {
        let resources = vec![
            create_test_resource("Alice", "dev-1", "Developer"),
            create_test_resource("Bob", "qa-1", "QA"),
        ];
        let mock_repo = MockResourceRepository::new(resources);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_with_context().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|r| r.resource.name() == "Alice"));
        assert!(result.iter().any(|r| r.resource.code() == "qa-1"));
        assert!(result.iter().all(|r| r.company_code == "TEST-COMPANY"));
        assert!(result.iter().all(|r| r.project_codes == vec!["PROJ-1"]));
    }

    #[test]
    fn test_execute_with_context_empty() {
        let mock_repo = MockResourceRepository::new(vec![]);
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_with_context().unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_with_context_repository_error() {
        let mock_repo = MockResourceRepository::new_with_failure();
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute_with_context();
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_all_with_context");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_resource_with_context_structure() {
        let resource = create_test_resource("Alice", "dev-1", "Developer");
        let context = ResourceWithContext {
            resource: resource.clone(),
            company_code: "TEST-COMPANY".to_string(),
            project_codes: vec!["PROJ-1".to_string(), "PROJ-2".to_string()],
        };

        assert_eq!(context.resource.code(), resource.code());
        assert_eq!(context.company_code, "TEST-COMPANY");
        assert_eq!(context.project_codes, vec!["PROJ-1", "PROJ-2"]);
    }

    #[test]
    fn test_list_resources_use_case_creation() {
        let mock_repo = MockResourceRepository::new(vec![]);
        let _use_case = ListResourcesUseCase::new(mock_repo);

        // Test that the use case was created successfully
        // If we get here, creation succeeded
    }
}
