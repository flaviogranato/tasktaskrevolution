#![allow(dead_code, unused_imports)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::resource_management::{
    ResourceTypeValidator,
    any_resource::AnyResource,
    repository::{ResourceRepository, ResourceRepositoryWithId},
    resource::WipLimits,
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

pub struct UpdateResourceUseCase<RR, CR>
where
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    resource_repository: RR,
    code_resolver: CR,
    type_validator: ResourceTypeValidator,
}

impl<RR, CR> UpdateResourceUseCase<RR, CR>
where
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    pub fn new(resource_repository: RR, code_resolver: CR) -> Self {
        Self {
            resource_repository,
            code_resolver,
            type_validator: ResourceTypeValidator::new(),
        }
    }

    pub fn execute(
        &self,
        resource_code: &str,
        company_code: &str,
        args: UpdateResourceArgs,
    ) -> Result<AnyResource, UpdateAppError> {
        // 1. Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(UpdateAppError::RepositoryError)?;

        // 2. Load the resource aggregate using ID
        let mut resource = self
            .resource_repository
            .find_by_id(&resource_id)?
            .ok_or_else(|| UpdateAppError::ResourceNotFound(resource_code.to_string()))?;

        // 3. Update the fields on the aggregate.
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

        // 4. Save the updated resource aggregate using save_in_hierarchy.
        let updated_resource = self
            .resource_repository
            .save_in_hierarchy(resource, company_code, None)?;

        // 5. Return the updated resource.
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
        fn resolve_company_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn resolve_project_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("project", "Not implemented in mock"))
        }

        fn resolve_resource_code(&self, code: &str) -> Result<String, AppError> {
            self.resource_codes
                .borrow()
                .get(code)
                .cloned()
                .ok_or_else(|| AppError::validation_error("resource", format!("Resource '{}' not found", code)))
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

        fn validate_resource_code(&self, code: &str) -> Result<(), AppError> {
            self.resolve_resource_code(code)?;
            Ok(())
        }

        fn validate_task_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            self.resources
                .borrow_mut()
                .insert(resource.id().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.borrow().values().find(|r| r.code() == code).cloned())
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

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, id: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.borrow().get(id).cloned())
        }
    }

    // --- Helpers ---
    fn create_test_resource(code: &str, name: &str, email: &str, r#type: &str) -> AnyResource {
        Resource::<Available> {
            project_id: None,
            scope: ResourceScope::Company,
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
        let resource_id = initial_resource.id().to_string();

        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource_id.clone(), initial_resource)])),
        };

        let code_resolver = MockCodeResolver::new();
        code_resolver.add_resource("DEV-1", &resource_id);

        let use_case = UpdateResourceUseCase::new(resource_repo, code_resolver);

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
        let code_resolver = MockCodeResolver::new();
        let use_case = UpdateResourceUseCase::new(resource_repo, code_resolver);

        let args = UpdateResourceArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("DEV-NONEXISTENT", "TEST-001", args);

        assert!(matches!(result, Err(UpdateAppError::RepositoryError(_))));
    }
}
