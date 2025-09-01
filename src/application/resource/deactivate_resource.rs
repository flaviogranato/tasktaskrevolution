use crate::domain::{
    resource_management::{any_resource::AnyResource, repository::ResourceRepository},
    shared::errors::DomainError,
};
use std::fmt;

#[derive(Debug)]
pub enum DeactivateResourceError {
    ResourceNotFound(String),
    ResourceAlreadyDeactivated(String),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for DeactivateResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeactivateResourceError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            DeactivateResourceError::ResourceAlreadyDeactivated(code) => write!(f, "Resource '{}' is already deactivated.", code),
            DeactivateResourceError::DomainError(message) => write!(f, "Domain error: {}", message),
            DeactivateResourceError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for DeactivateResourceError {}

impl From<DomainError> for DeactivateResourceError {
    fn from(err: DomainError) -> Self {
        DeactivateResourceError::RepositoryError(err)
    }
}

pub struct DeactivateResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    resource_repository: RR,
}

impl<RR> DeactivateResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    pub fn new(resource_repository: RR) -> Self {
        Self { resource_repository }
    }

    pub fn execute(&self, resource_code: &str) -> Result<AnyResource, DeactivateResourceError> {
        // 1. Find the resource from the repository.
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| DeactivateResourceError::ResourceNotFound(resource_code.to_string()))?;

        // 2. Call the domain logic to deactivate the resource.
        // This consumes the resource and returns a new one in the `Inactive` state.
        let deactivated_resource = resource.deactivate().map_err(DeactivateResourceError::DomainError)?;

        // 3. Save the now-inactive resource back to the repository.
        let saved_resource = self.resource_repository.save(deactivated_resource)?;

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

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(self.resources.borrow().get(code).cloned())
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            unimplemented!()
        }
        fn get_next_code(&self, _resource_type: &str) -> Result<String, DomainError> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
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

    // --- Helpers ---
    fn create_test_resource(code: &str) -> AnyResource {
        Resource::<Available> {
            id: uuid7(),
            code: code.to_string(),
            name: "Test Resource".to_string(),
            email: None,
            resource_type: "Test".to_string(),
            vacations: None,
            time_off_balance: 0,
            time_off_history: None,
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
        let use_case = DeactivateResourceUseCase::new(resource_repo.clone());

        let result = use_case.execute("RES-1");

        assert!(result.is_ok());

        let deactivated_resource = result.unwrap();
        assert_eq!(deactivated_resource.status(), "Inactive");
    }

    #[test]
    fn test_deactivate_resource_fails_if_not_found() {
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };
        let use_case = DeactivateResourceUseCase::new(resource_repo);

        let result = use_case.execute("RES-NONEXISTENT");

        assert!(matches!(result, Err(DeactivateResourceError::ResourceNotFound(_))));
    }
}
