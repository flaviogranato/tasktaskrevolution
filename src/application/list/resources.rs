use crate::domain::resource_management::{any_resource::AnyResource, repository::ResourceRepository};
use crate::domain::shared::errors::DomainError;

pub struct ListResourcesUseCase<R: ResourceRepository> {
    repository: R,
}

impl<R: ResourceRepository> ListResourcesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<AnyResource>, DomainError> {
        self.repository.find_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::{any_resource::AnyResource, resource::Resource};

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.clone())
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(None)
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, DomainError> {
            self.save(resource)
        }

        fn save(&self, _resource: AnyResource) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _r: &str,
            _h: u32,
            _d: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _r: &str,
            _s: &str,
            _e: &str,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(
            &self,
            _s: &chrono::DateTime<chrono::Local>,
            _e: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }
        fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    fn create_test_resource(name: &str, code: &str, r_type: &str) -> AnyResource {
        Resource::new(code.to_string(), name.to_string(), None, r_type.to_string(), None, 0).into()
    }

    #[test]
    fn test_list_resources_success() {
        let resources = vec![
            create_test_resource("Alice", "dev-1", "Developer"),
            create_test_resource("Bob", "qa-1", "QA"),
        ];
        let mock_repo = MockResourceRepository { resources };
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|r| r.name() == "Alice"));
        assert!(result.iter().any(|r| r.code() == "qa-1"));
    }

    #[test]
    fn test_list_resources_empty() {
        let mock_repo = MockResourceRepository { resources: vec![] };
        let use_case = ListResourcesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();

        assert!(result.is_empty());
    }
}
