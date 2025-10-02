#![allow(unused_imports)]
use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::resource_management::{
    any_resource::AnyResource,
    repository::{ResourceRepository, ResourceRepositoryWithId},
    resource::{ResourceScope, WipLimits},
};
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::fmt;

#[derive(Debug)]
pub enum DescribeAppError {
    ResourceNotFound(String),
    RepositoryError(AppError),
}

impl fmt::Display for DescribeAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DescribeAppError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            DescribeAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DescribeAppError {}

impl From<AppError> for DescribeAppError {
    fn from(err: AppError) -> Self {
        DescribeAppError::RepositoryError(err)
    }
}

impl From<crate::domain::shared::errors::DomainError> for DescribeAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        DescribeAppError::RepositoryError(err.into())
    }
}

pub struct DescribeResourceUseCase<RR, CR>
where
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    resource_repository: RR,
    code_resolver: CR,
}

impl<RR, CR> DescribeResourceUseCase<RR, CR>
where
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    pub fn new(resource_repository: RR, code_resolver: CR) -> Self {
        Self {
            resource_repository,
            code_resolver,
        }
    }

    pub fn execute(&self, resource_code: &str) -> Result<AnyResource, DescribeAppError> {
        // 1. Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(|e| DescribeAppError::RepositoryError(AppError::from(e)))?;

        // 2. Use ID for internal operation
        self.resource_repository
            .find_by_id(&resource_id)?
            .ok_or_else(|| DescribeAppError::ResourceNotFound(resource_code.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::resource_management::{resource::Resource, state::Available};
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    struct MockCodeResolver {
        resource_codes: RefCell<HashMap<String, String>>, // code -> id
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                resource_codes: RefCell::new(HashMap::new()),
            }
        }

        fn add_resource(&self, code: &str, id: &str) {
            self.resource_codes
                .borrow_mut()
                .insert(code.to_string(), id.to_string());
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
        }

        fn resolve_resource_code(&self, code: &str) -> DomainResult<String> {
            self.resource_codes.borrow().get(code).cloned().ok_or_else(|| {
                DomainError::from(AppError::validation_error(
                    "resource",
                    format!("Resource '{}' not found", code),
                ))
            })
        }

        fn resolve_task_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }

        fn validate_company_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn validate_project_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
        }

        fn validate_resource_code(&self, code: &str) -> DomainResult<()> {
            self.resolve_resource_code(code)?;
            Ok(())
        }

        fn validate_task_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            unimplemented!()
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().get(code).cloned())
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        // Other methods are not needed for this test.
        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }
        fn check_if_layoff_period(
            &self,
            _start: &chrono::DateTime<chrono::Local>,
            _end: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }
    }

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().get(id).cloned())
        }
    }

    // --- Helpers ---
    fn create_test_resource(code: &str) -> AnyResource {
        Resource::<Available> {
            project_id: None,
            scope: ResourceScope::Company,
            id: uuid7(),
            code: code.to_string(),
            name: "Test Resource".to_string(),
            email: Some("test@resource.com".to_string()),
            resource_type: "Test".to_string(),
            start_date: None,
            end_date: None,
            vacations: None,
            time_off_balance: 16,
            time_off_history: None,
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        }
        .into()
    }

    #[test]
    fn test_describe_resource_success() {
        let resource_code = "RES-1";
        let resource = create_test_resource(resource_code);
        let resource_id = resource.id().to_string();

        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource_id.clone(), resource)])),
        };

        let code_resolver = MockCodeResolver::new();
        code_resolver.add_resource(resource_code, &resource_id);

        let use_case = DescribeResourceUseCase::new(resource_repo, code_resolver);

        let result = use_case.execute(resource_code);

        assert!(result.is_ok());
        let found_resource = result.unwrap();
        assert_eq!(found_resource.code(), resource_code);
        assert_eq!(found_resource.name(), "Test Resource");
    }

    #[test]
    fn test_describe_resource_not_found() {
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DescribeResourceUseCase::new(resource_repo, code_resolver);

        let result = use_case.execute("RES-NONEXISTENT");

        assert!(matches!(result, Err(DescribeAppError::RepositoryError(_))));
    }
}
