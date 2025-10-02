use super::business_rules::ValidateBusinessRulesUseCase;
use super::data_integrity::ValidateDataIntegrityUseCase;
use super::entities::ValidateEntitiesUseCase;
use super::types::ValidationResult;
use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::errors::DomainResult;

pub struct ValidateSystemUseCase<P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: P,
    resource_repository: R,
    company_repository: C,
}

impl<P, R, C> ValidateSystemUseCase<P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    pub fn new(project_repository: P, resource_repository: R, company_repository: C) -> Self {
        Self {
            project_repository,
            resource_repository,
            company_repository,
        }
    }

    pub fn execute(&self) -> Result<Vec<ValidationResult>, AppError> {
        let mut all_results = Vec::new();

        // 1. Validate data integrity first (foundation)
        let data_integrity_results = self.validate_data_integrity()?;
        all_results.extend(data_integrity_results);

        // 2. Validate entities and relationships
        let entity_results = self.validate_entities()?;
        all_results.extend(entity_results);

        // 3. Validate business rules
        let business_results = self.validate_business_rules()?;
        all_results.extend(business_results);

        Ok(all_results)
    }

    fn validate_data_integrity(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateDataIntegrityUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_entities(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateEntitiesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_business_rules(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateBusinessRulesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::validate::types::{ValidationResult, ValidationSeverity};
    use crate::domain::company_management::repository::CompanyRepository;
    use crate::domain::project_management::repository::ProjectRepository;
    use crate::domain::resource_management::repository::ResourceRepository;
    use chrono::{DateTime, Local};

    // Mock repositories for testing
    struct MockProjectRepository;
    struct MockResourceRepository;
    struct MockCompanyRepository;

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: crate::domain::project_management::any_project::AnyProject) -> DomainResult<()> {
            Ok(())
        }

        fn load(&self) -> DomainResult<crate::domain::project_management::any_project::AnyProject> {
            Err(crate::domain::shared::errors::DomainError::ProjectNotFound {
                code: "test".to_string(),
            })
        }

        fn find_all(&self) -> DomainResult<Vec<crate::domain::project_management::any_project::AnyProject>> {
            Ok(vec![])
        }

        fn find_by_code(
            &self,
            _code: &str,
        ) -> DomainResult<Option<crate::domain::project_management::any_project::AnyProject>> {
            Ok(None)
        }

        fn get_next_code(&self) -> DomainResult<String> {
            Ok("PROJ-001".to_string())
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(
            &self,
            _resource: crate::domain::resource_management::any_resource::AnyResource,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn save_in_hierarchy(
            &self,
            _resource: crate::domain::resource_management::any_resource::AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn find_all(&self) -> DomainResult<Vec<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(vec![])
        }

        fn find_by_company(
            &self,
            _company_code: &str,
        ) -> DomainResult<Vec<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(vec![])
        }

        fn find_all_with_context(
            &self,
        ) -> DomainResult<
            Vec<(
                crate::domain::resource_management::any_resource::AnyResource,
                String,
                Vec<String>,
            )>,
        > {
            Ok(vec![])
        }

        fn find_by_code(
            &self,
            _code: &str,
        ) -> DomainResult<Option<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(None)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            Ok("RES-001".to_string())
        }
    }

    impl CompanyRepository for MockCompanyRepository {
        fn save(&self, _company: crate::domain::company_management::company::Company) -> DomainResult<()> {
            Ok(())
        }

        fn find_all(&self) -> DomainResult<Vec<crate::domain::company_management::company::Company>> {
            Ok(vec![])
        }

        fn find_by_code(
            &self,
            _code: &str,
        ) -> DomainResult<Option<crate::domain::company_management::company::Company>> {
            Ok(None)
        }

        fn find_by_id(&self, _id: &str) -> DomainResult<Option<crate::domain::company_management::company::Company>> {
            Ok(None)
        }

        fn find_by_name(
            &self,
            _name: &str,
        ) -> DomainResult<Option<crate::domain::company_management::company::Company>> {
            Ok(None)
        }

        fn update(&self, _company: crate::domain::company_management::company::Company) -> DomainResult<()> {
            Ok(())
        }

        fn delete(&self, _id: &str) -> DomainResult<()> {
            Ok(())
        }

        fn get_next_code(&self) -> DomainResult<String> {
            Ok("COMP-001".to_string())
        }

        fn code_exists(&self, _code: &str) -> DomainResult<bool> {
            Ok(false)
        }

        fn name_exists(&self, _name: &str) -> DomainResult<bool> {
            Ok(false)
        }
    }

    #[test]
    fn test_validate_system_use_case_creation() {
        let _use_case =
            ValidateSystemUseCase::new(MockProjectRepository, MockResourceRepository, MockCompanyRepository);
        // Should not panic
    }

    #[test]
    fn test_validate_system_execute_with_empty_repositories() {
        let use_case = ValidateSystemUseCase::new(MockProjectRepository, MockResourceRepository, MockCompanyRepository);

        let result = use_case.execute();
        assert!(result.is_ok());
        let _results = result.unwrap();
        // With empty repositories, we should get some validation results
        // (even if they're just "no data found" warnings)
        // results.len() is always >= 0
    }

    #[test]
    fn test_validate_system_execute_returns_validation_results() {
        let use_case = ValidateSystemUseCase::new(MockProjectRepository, MockResourceRepository, MockCompanyRepository);

        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();

        // Verify that all results are ValidationResult instances
        for validation_result in results {
            assert!(matches!(validation_result, ValidationResult { .. }));
        }
    }

    #[test]
    fn test_validate_system_categorizes_results_correctly() {
        let use_case = ValidateSystemUseCase::new(MockProjectRepository, MockResourceRepository, MockCompanyRepository);

        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();

        // Categorize results
        let errors: Vec<_> = results
            .iter()
            .filter(|r| r.level == ValidationSeverity::Error)
            .collect();
        let warnings: Vec<_> = results
            .iter()
            .filter(|r| r.level == ValidationSeverity::Warning)
            .collect();
        let info: Vec<_> = results
            .iter()
            .filter(|r| r.level == ValidationSeverity::Info)
            .collect();

        // Verify categorization
        assert_eq!(errors.len() + warnings.len() + info.len(), results.len());
    }

    #[test]
    fn test_validate_system_handles_validation_errors() {
        let use_case = ValidateSystemUseCase::new(MockProjectRepository, MockResourceRepository, MockCompanyRepository);

        // This should not panic even if individual validations fail
        let result = use_case.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_system_validation_result_structure() {
        let validation_result = ValidationResult {
            code: "TEST_CODE".to_string(),
            path: None,
            entity_type: Some("Project".to_string()),
            entity_code: Some("PROJ-001".to_string()),
            field: Some("name".to_string()),
            level: ValidationSeverity::Error,
            message: "Test validation error".to_string(),
            details: Some("Detailed error information".to_string()),
        };

        assert_eq!(validation_result.entity_type, Some("Project".to_string()));
        assert_eq!(validation_result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(validation_result.field, Some("name".to_string()));
        assert_eq!(validation_result.level, ValidationSeverity::Error);
        assert_eq!(validation_result.message, "Test validation error");
        assert_eq!(
            validation_result.details,
            Some("Detailed error information".to_string())
        );
    }

    #[test]
    fn test_validate_system_validation_result_with_minimal_fields() {
        let validation_result = ValidationResult {
            code: "TEST_CODE".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Simple warning".to_string(),
            details: None,
        };

        assert_eq!(validation_result.entity_type, None);
        assert_eq!(validation_result.entity_code, None);
        assert_eq!(validation_result.field, None);
        assert_eq!(validation_result.level, ValidationSeverity::Warning);
        assert_eq!(validation_result.message, "Simple warning");
        assert_eq!(validation_result.details, None);
    }

    #[test]
    fn test_validate_system_validation_result_severity_levels() {
        let error_result = ValidationResult {
            code: "TEST_ERROR".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Error,
            message: "Error message".to_string(),
            details: None,
        };

        let warning_result = ValidationResult {
            code: "TEST_WARNING".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Warning message".to_string(),
            details: None,
        };

        let info_result = ValidationResult {
            code: "TEST_INFO".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Info,
            message: "Info message".to_string(),
            details: None,
        };

        assert_eq!(error_result.level, ValidationSeverity::Error);
        assert_eq!(warning_result.level, ValidationSeverity::Warning);
        assert_eq!(info_result.level, ValidationSeverity::Info);
    }

    #[test]
    fn test_validate_system_validation_result_entity_types() {
        let project_result = ValidationResult {
            code: "PROJECT_TEST".to_string(),
            path: None,
            entity_type: Some("Project".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Error,
            message: "Project error".to_string(),
            details: None,
        };

        let resource_result = ValidationResult {
            code: "RESOURCE_TEST".to_string(),
            path: None,
            entity_type: Some("Resource".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Error,
            message: "Resource error".to_string(),
            details: None,
        };

        let company_result = ValidationResult {
            code: "COMPANY_TEST".to_string(),
            path: None,
            entity_type: Some("Company".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Error,
            message: "Company error".to_string(),
            details: None,
        };

        assert_eq!(project_result.entity_type, Some("Project".to_string()));
        assert_eq!(resource_result.entity_type, Some("Resource".to_string()));
        assert_eq!(company_result.entity_type, Some("Company".to_string()));
    }

    #[test]
    fn test_validation_severity_display() {
        assert_eq!(format!("{}", ValidationSeverity::Error), "ERROR");
        assert_eq!(format!("{}", ValidationSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", ValidationSeverity::Info), "INFO");
    }

    #[test]
    fn test_validation_result_display() {
        let result = ValidationResult {
            code: "TEST_RESULT".to_string(),
            path: None,
            entity_type: Some("Project".to_string()),
            entity_code: Some("PROJ-001".to_string()),
            field: Some("name".to_string()),
            level: ValidationSeverity::Error,
            message: "Test error".to_string(),
            details: Some("Detailed info".to_string()),
        };

        let display = format!("{}", result);
        assert!(display.contains("ERROR"));
        assert!(display.contains("Test error"));
        assert!(display.contains("Project: PROJ-001"));
        assert!(display.contains("Field: name"));
        assert!(display.contains("Detailed info"));
    }

    #[test]
    fn test_validation_result_builder_methods() {
        let result = ValidationResult::error("TEST_ERROR".to_string(), "Test error".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string())
            .with_field("name".to_string())
            .with_details("Detailed info".to_string());

        assert_eq!(result.level, ValidationSeverity::Error);
        assert_eq!(result.message, "Test error");
        assert_eq!(result.entity_type, Some("Project".to_string()));
        assert_eq!(result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(result.field, Some("name".to_string()));
        assert_eq!(result.details, Some("Detailed info".to_string()));
    }
}
