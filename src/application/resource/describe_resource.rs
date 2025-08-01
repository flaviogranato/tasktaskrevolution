use crate::domain::{
    resource_management::{any_resource::AnyResource, repository::ResourceRepository},
    shared::errors::DomainError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DescribeResourceError {
    #[error("Resource with code '{0}' not found.")]
    ResourceNotFound(String),
    #[error("A repository error occurred: {0}")]
    RepositoryError(#[from] DomainError),
}

pub struct DescribeResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    resource_repository: RR,
}

impl<RR> DescribeResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    pub fn new(resource_repository: RR) -> Self {
        Self { resource_repository }
    }

    pub fn execute(&self, resource_code: &str) -> Result<AnyResource, DescribeResourceError> {
        self.resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| DescribeResourceError::ResourceNotFound(resource_code.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        resource_management::{resource::Resource, state::Available},
        shared::errors::DomainError,
    };
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            unimplemented!()
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(self.resources.borrow().get(code).cloned())
        }

        // Other methods are not needed for this test.
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
            email: Some("test@resource.com".to_string()),
            resource_type: "Test".to_string(),
            vacations: None,
            time_off_balance: 16,
            time_off_history: None,
            state: Available,
        }
        .into()
    }

    #[test]
    fn test_describe_resource_success() {
        let resource_code = "RES-1";
        let resource = create_test_resource(resource_code);
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource)])),
        };
        let use_case = DescribeResourceUseCase::new(resource_repo);

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
        let use_case = DescribeResourceUseCase::new(resource_repo);

        let result = use_case.execute("RES-NONEXISTENT");

        assert!(matches!(result, Err(DescribeResourceError::ResourceNotFound(_))));
    }
}
