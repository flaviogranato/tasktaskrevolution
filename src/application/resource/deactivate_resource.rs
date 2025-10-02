#![allow(dead_code, unused_imports)]

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
pub enum DeactivateAppError {
    ResourceNotFound(String),
    ResourceAlreadyDeactivated(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for DeactivateAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeactivateAppError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            DeactivateAppError::ResourceAlreadyDeactivated(code) => {
                write!(f, "Resource '{}' is already deactivated.", code)
            }
            DeactivateAppError::AppError(message) => write!(f, "Domain error: {}", message),
            DeactivateAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DeactivateAppError {}

impl From<AppError> for DeactivateAppError {
    fn from(err: AppError) -> Self {
        DeactivateAppError::RepositoryError(err)
    }
}

impl From<crate::domain::shared::errors::DomainError> for DeactivateAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        DeactivateAppError::RepositoryError(err.into())
    }
}

pub struct DeactivateResourceUseCase<RR, CR>
where
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    resource_repository: RR,
    code_resolver: CR,
}

impl<RR, CR> DeactivateResourceUseCase<RR, CR>
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

    pub fn execute(&self, resource_code: &str, company_code: &str) -> Result<AnyResource, DeactivateAppError> {
        // 1. Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(|e| DeactivateAppError::RepositoryError(AppError::from(e)))?;

        // 2. Find the resource from the repository using ID
        let resource = self
            .resource_repository
            .find_by_id(&resource_id)?
            .ok_or_else(|| DeactivateAppError::ResourceNotFound(resource_code.to_string()))?;

        // 3. Call the domain logic to deactivate the resource.
        // This consumes the resource and returns a new one in the `Inactive` state.
        let deactivated_resource = resource.deactivate().map_err(DeactivateAppError::AppError)?;

        // 4. Save the now-inactive resource back to the repository using save_in_hierarchy.
        let saved_resource = self
            .resource_repository
            .save_in_hierarchy(deactivated_resource, company_code, None)?;

        Ok(saved_resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::{resource::Resource, state::Available};
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    #[derive(Clone)]
    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    struct MockCodeResolver {
        // Mock doesn't need to resolve anything for DeactivateResourceUseCase
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {}
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

        fn resolve_resource_code(&self, _code: &str) -> DomainResult<String> {
            Ok("mock-resource-id".to_string())
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

        fn validate_resource_code(&self, _code: &str) -> DomainResult<()> {
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
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
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

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            unimplemented!()
        }
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
        fn find_by_id(&self, _id: &str) -> DomainResult<Option<AnyResource>> {
            // For tests, we'll return the first resource in the map
            Ok(self.resources.borrow().values().next().cloned())
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
            email: None,
            resource_type: "Test".to_string(),
            start_date: None,
            end_date: None,
            vacations: None,
            time_off_balance: 0,
            time_off_history: None,
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        }
        .into()
    }

    // --- Tests ---
    // TODO: Enable this test once `AnyResource::deactivate` and `AnyResource::status` are implemented.

    #[test]
    fn test_deactivate_resource_success() {
        let initial_resource = create_test_resource("RES-1");
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(initial_resource.code().to_string(), initial_resource)])),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DeactivateResourceUseCase::new(resource_repo.clone(), code_resolver);

        let result = use_case.execute("RES-1", "TEST-001");

        assert!(result.is_ok());

        let deactivated_resource = result.unwrap();
        assert_eq!(deactivated_resource.status(), "Inactive");
    }

    #[test]
    fn test_deactivate_resource_fails_if_not_found() {
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };
        let code_resolver = MockCodeResolver::new();
        let use_case = DeactivateResourceUseCase::new(resource_repo, code_resolver);

        let result = use_case.execute("RES-NONEXISTENT", "TEST-001");

        assert!(matches!(result, Err(DeactivateAppError::ResourceNotFound(_))));
    }
}
