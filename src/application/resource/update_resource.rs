#![allow(dead_code, unused_imports)]

use crate::application::errors::AppError;
use crate::domain::resource_management::{
    ResourceTypeValidator, any_resource::AnyResource, repository::ResourceRepository, resource::WipLimits,
};
use std::fmt;

#[derive(Debug)]
pub enum UpdateAppError {
    ResourceNotFound(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for UpdateAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateAppError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            UpdateAppError::AppError(message) => write!(f, "Domain error: {}", message),
            UpdateAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for UpdateAppError {}

impl From<AppError> for UpdateAppError {
    fn from(err: AppError) -> Self {
        UpdateAppError::RepositoryError(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateResourceArgs {
    pub name: Option<String>,
    pub email: Option<String>,
    pub resource_type: Option<String>,
}

pub struct UpdateResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    resource_repository: RR,
    type_validator: ResourceTypeValidator,
}

impl<RR> UpdateResourceUseCase<RR>
where
    RR: ResourceRepository,
{
    pub fn new(resource_repository: RR) -> Self {
        Self {
            resource_repository,
            type_validator: ResourceTypeValidator::new(),
        }
    }

    pub fn execute(
        &self,
        resource_code: &str,
        company_code: &str,
        args: UpdateResourceArgs,
    ) -> Result<AnyResource, UpdateAppError> {
        // 1. Load the resource aggregate by code.
        let mut resource = self.resource_repository.find_by_code(resource_code)?
            .ok_or_else(|| UpdateAppError::ResourceNotFound(resource_code.to_string()))?;

        // 2. Update the fields on the aggregate.
        // In a more complex scenario, this would be a method on the `AnyResource`
        // aggregate to enforce invariants. For simple field updates, this is acceptable.
        if let Some(name) = args.name {
            resource.set_name(name);
        }
        if let Some(email) = args.email {
            resource.set_email(Some(email));
        }
        if let Some(resource_type) = args.resource_type {
            // Validate resource type against config
            self.type_validator
                .validate_resource_type(&resource_type)
                .map_err(UpdateAppError::AppError)?;
            resource.set_resource_type(resource_type);
        }

        // 3. Save the updated resource aggregate using save_in_hierarchy.
        let updated_resource = self
            .resource_repository
            .save_in_hierarchy(resource, company_code, None)?;

        // 4. Return the updated resource.
        Ok(updated_resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::{resource::Resource, state::Available};
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.borrow().get(code).cloned())
        }
        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            self.save(resource)
        }

        // Other methods are not needed for this test.
        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            unimplemented!()
        }
        fn get_next_code(&self, _resource_type: &str) -> Result<String, AppError> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
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
    fn create_test_resource(code: &str, name: &str, email: &str, r#type: &str) -> AnyResource {
        Resource::<Available> {
            id: uuid7(),
            code: code.to_string(),
            name: name.to_string(),
            email: Some(email.to_string()),
            resource_type: r#type.to_string(),
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

    #[test]
    fn test_update_resource_name_and_email_success() {
        let initial_resource = create_test_resource("DEV-1", "Old Name", "old@test.com", "Developer");
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(initial_resource.code().to_string(), initial_resource)])),
        };
        let use_case = UpdateResourceUseCase::new(resource_repo);

        let args = UpdateResourceArgs {
            name: Some("New Name".to_string()),
            email: Some("new@test.com".to_string()),
            resource_type: None,
        };

        let result = use_case.execute("DEV-1", "TEST-001", args);

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.name(), "New Name");
        assert_eq!(updated_resource.email().unwrap(), "new@test.com");
        assert_eq!(updated_resource.resource_type(), "Developer"); // Should not change
    }

    #[test]
    fn test_update_resource_fails_if_not_found() {
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };
        let use_case = UpdateResourceUseCase::new(resource_repo);

        let args = UpdateResourceArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("DEV-NONEXISTENT", "TEST-001", args);

        assert!(matches!(result, Err(UpdateAppError::ResourceNotFound(_))));
    }
}
