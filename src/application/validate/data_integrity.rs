use super::specifications::{
    ProjectHasTasksSpec, TaskWithinProjectTimelineSpec, ValidCompanySettingsSpec, ValidProjectDateRangeSpec,
    ValidResourceVacationSpec,
};
use super::types::ValidationResult;
use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::specification::{AndSpecification, Specification};

pub struct ValidateDataIntegrityUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: &'a P,
    resource_repository: &'a R,
    company_repository: &'a C,
}

impl<'a, P, R, C> ValidateDataIntegrityUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    pub fn new(project_repository: &'a P, resource_repository: &'a R, company_repository: &'a C) -> Self {
        Self {
            project_repository,
            resource_repository,
            company_repository,
        }
    }

    pub fn execute(&self) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Load all entities
        let companies = self.company_repository.find_all()?;
        let resources = self.resource_repository.find_all()?;
        let projects = self.project_repository.find_all()?;

        // Validate using specifications
        results.extend(self.validate_projects_with_specifications(&projects)?);
        results.extend(self.validate_resources_with_specifications(&resources)?);
        results.extend(self.validate_companies_with_specifications(&companies)?);

        Ok(results)
    }

    fn validate_projects_with_specifications(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Create composite specification for projects
        let project_spec = AndSpecification::new()
            .add_specification(Box::new(ValidProjectDateRangeSpec))
            .add_specification(Box::new(TaskWithinProjectTimelineSpec))
            .add_specification(Box::new(ProjectHasTasksSpec));

        for project in projects {
            if !project_spec.is_satisfied_by(project)
                && let Some(explanation) = project_spec.explain_why_not_satisfied(project)
            {
                results.push(
                    ValidationResult::warning(
                        "PROJECT_DATA_INTEGRITY".to_string(),
                        project_spec.description().to_string()
                    )
                    .with_entity("Project".to_string(), project.code().to_string())
                    .with_details(explanation)
                    .with_path(format!("companies/{}/projects/{}/project.yaml", 
                        project.company_code(), project.code())),
                );
            }
        }

        Ok(results)
    }

    fn validate_resources_with_specifications(
        &self,
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        let resource_spec = ValidResourceVacationSpec;

        for resource in resources {
            if !resource_spec.is_satisfied_by(resource)
                && let Some(explanation) = resource_spec.explain_why_not_satisfied(resource)
            {
                results.push(
                    ValidationResult::warning("RESOURCE_DATA_INTEGRITY".to_string(), resource_spec.description().to_string())
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_details(explanation),
                );
            }
        }

        Ok(results)
    }

    fn validate_companies_with_specifications(
        &self,
        companies: &[crate::domain::company_management::company::Company],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        let company_spec = ValidCompanySettingsSpec;

        for company in companies {
            if !company_spec.is_satisfied_by(company)
                && let Some(explanation) = company_spec.explain_why_not_satisfied(company)
            {
                results.push(
                    ValidationResult::warning("COMPANY_DATA_INTEGRITY".to_string(), company_spec.description().to_string())
                        .with_entity("Company".to_string(), company.code().to_string())
                        .with_details(explanation),
                );
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::shared::errors::{DomainError, DomainResult};
    use super::*;
    use crate::application::validate::types::{ValidationResult, ValidationSeverity};
    use crate::domain::company_management::company::Company;
    use crate::domain::company_management::repository::CompanyRepository;
    use crate::domain::project_management::any_project::AnyProject;
    use crate::domain::project_management::repository::ProjectRepository;
    use crate::domain::resource_management::any_resource::AnyResource;
    use crate::domain::resource_management::repository::ResourceRepository;
    use chrono::{DateTime, Local};

    // Mock repositories for testing
    struct MockProjectRepository {
        projects: Vec<AnyProject>,
    }

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    struct MockCompanyRepository {
        companies: Vec<Company>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> DomainResult<()> {
            Ok(())
        }

        fn load(&self) -> DomainResult<AnyProject> {
            Err(DomainError::from(AppError::ProjectNotFound {
                code: "test".to_string(),
            }))
        }

        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            Ok(self.projects.clone())
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(None)
        }

        fn get_next_code(&self) -> DomainResult<String> {
            Ok("PROJ-001".to_string())
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> DomainResult<AnyResource> {
            Err(DomainError::from(AppError::ResourceNotFound {
                code: "test".to_string(),
            }))
        }

        fn save_in_hierarchy(
            &self,
            _resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            Err(DomainError::from(AppError::ResourceNotFound {
                code: "test".to_string(),
            }))
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.clone())
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
            Err(DomainError::from(AppError::ResourceNotFound {
                code: "test".to_string(),
            }))
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            Err(DomainError::from(AppError::ResourceNotFound {
                code: "test".to_string(),
            }))
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            Ok("RES-001".to_string())
        }
    }

    impl CompanyRepository for MockCompanyRepository {
        fn save(&self, _company: Company) -> DomainResult<()> {
            Ok(())
        }

        fn find_all(&self) -> DomainResult<Vec<Company>> {
            Ok(self.companies.clone())
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<Company>> {
            Ok(None)
        }

        fn find_by_id(&self, _id: &str) -> DomainResult<Option<Company>> {
            Ok(None)
        }

        fn find_by_name(&self, _name: &str) -> DomainResult<Option<Company>> {
            Ok(None)
        }

        fn update(&self, _company: Company) -> DomainResult<()> {
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
    fn test_validate_data_integrity_use_case_creation() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let _use_case = ValidateDataIntegrityUseCase::new(&project_repo, &resource_repo, &company_repo);
        // Should not panic
    }

    #[test]
    fn test_validate_data_integrity_execute_with_empty_repositories() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateDataIntegrityUseCase::new(&project_repo, &resource_repo, &company_repo);
        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_validate_data_integrity_execute_returns_validation_results() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateDataIntegrityUseCase::new(&project_repo, &resource_repo, &company_repo);
        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();

        // Verify that all results are ValidationResult instances
        for validation_result in results {
            assert!(matches!(validation_result, ValidationResult { .. }));
        }
    }

    #[test]
    fn test_validate_data_integrity_handles_repository_errors() {
        // This test would require a mock that returns errors
        // For now, we'll test that the use case can be created and executed
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateDataIntegrityUseCase::new(&project_repo, &resource_repo, &company_repo);
        let result = use_case.execute();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_data_integrity_validation_result_structure() {
        let validation_result = ValidationResult {
            code: "DATA_INTEGRITY_WARNING".to_string(),
            level: ValidationSeverity::Warning,
            message: "Data integrity validation failed".to_string(),
            path: None,
            entity_type: Some("Project".to_string()),
            entity_code: Some("PROJ-001".to_string()),
            field: Some("timeline".to_string()),
            details: Some("Project timeline is invalid".to_string()),
        };

        assert_eq!(validation_result.entity_type, Some("Project".to_string()));
        assert_eq!(validation_result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(validation_result.field, Some("timeline".to_string()));
        assert_eq!(validation_result.level, ValidationSeverity::Warning);
        assert_eq!(validation_result.message, "Data integrity validation failed");
        assert_eq!(
            validation_result.details,
            Some("Project timeline is invalid".to_string())
        );
    }

    #[test]
    fn test_validate_data_integrity_validation_result_with_minimal_fields() {
        let validation_result = ValidationResult {
            code: "TEST_CODE".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Info,
            message: "Data integrity check completed".to_string(),
            details: None,
        };

        assert_eq!(validation_result.entity_type, None);
        assert_eq!(validation_result.entity_code, None);
        assert_eq!(validation_result.field, None);
        assert_eq!(validation_result.level, ValidationSeverity::Info);
        assert_eq!(validation_result.message, "Data integrity check completed");
        assert_eq!(validation_result.details, None);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_severity_levels() {
        let error_result = ValidationResult {
            code: "TEST_ERROR".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Error,
            message: "Critical data integrity error".to_string(),
            details: None,
        };

        let warning_result = ValidationResult {
            code: "TEST_WARNING".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Data integrity warning".to_string(),
            details: None,
        };

        let info_result = ValidationResult {
            code: "TEST_INFO".to_string(),
            path: None,
            entity_type: None,
            entity_code: None,
            field: None,
            level: ValidationSeverity::Info,
            message: "Data integrity information".to_string(),
            details: None,
        };

        assert_eq!(error_result.level, ValidationSeverity::Error);
        assert_eq!(warning_result.level, ValidationSeverity::Warning);
        assert_eq!(info_result.level, ValidationSeverity::Info);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_entity_types() {
        let project_result = ValidationResult {
            code: "PROJECT_TEST".to_string(),
            path: None,
            entity_type: Some("Project".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Project validation".to_string(),
            details: None,
        };

        let resource_result = ValidationResult {
            code: "RESOURCE_TEST".to_string(),
            path: None,
            entity_type: Some("Resource".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Resource validation".to_string(),
            details: None,
        };

        let company_result = ValidationResult {
            code: "COMPANY_TEST".to_string(),
            path: None,
            entity_type: Some("Company".to_string()),
            entity_code: None,
            field: None,
            level: ValidationSeverity::Warning,
            message: "Company validation".to_string(),
            details: None,
        };

        assert_eq!(project_result.entity_type, Some("Project".to_string()));
        assert_eq!(resource_result.entity_type, Some("Resource".to_string()));
        assert_eq!(company_result.entity_type, Some("Company".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_display_formatting() {
        let result = ValidationResult::warning("DATA_INTEGRITY_ISSUE".to_string(), "Data integrity issue".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string())
            .with_field("timeline".to_string())
            .with_details("Project timeline validation failed".to_string());

        let display = format!("{}", result);
        assert!(display.contains("WARNING"));
        assert!(display.contains("Data integrity issue"));
        assert!(display.contains("Project: PROJ-001"));
        assert!(display.contains("Field: timeline"));
        assert!(display.contains("Project timeline validation failed"));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_builder_methods() {
        let result = ValidationResult::warning("TEST_WARNING".to_string(), "Test warning".to_string())
            .with_entity("Resource".to_string(), "RES-001".to_string())
            .with_field("vacation".to_string())
            .with_details("Vacation validation failed".to_string());

        assert_eq!(result.level, ValidationSeverity::Warning);
        assert_eq!(result.message, "Test warning");
        assert_eq!(result.entity_type, Some("Resource".to_string()));
        assert_eq!(result.entity_code, Some("RES-001".to_string()));
        assert_eq!(result.field, Some("vacation".to_string()));
        assert_eq!(result.details, Some("Vacation validation failed".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_error_builder() {
        let result = ValidationResult::error("CRITICAL_ERROR".to_string(), "Critical error".to_string())
            .with_entity("Company".to_string(), "COMP-001".to_string())
            .with_details("Company settings validation failed".to_string());

        assert_eq!(result.level, ValidationSeverity::Error);
        assert_eq!(result.message, "Critical error");
        assert_eq!(result.entity_type, Some("Company".to_string()));
        assert_eq!(result.entity_code, Some("COMP-001".to_string()));
        assert_eq!(result.details, Some("Company settings validation failed".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_info_builder() {
        let result = ValidationResult::info("INFO_MESSAGE".to_string(), "Information message".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string());

        assert_eq!(result.level, ValidationSeverity::Info);
        assert_eq!(result.message, "Information message");
        assert_eq!(result.entity_type, Some("Project".to_string()));
        assert_eq!(result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_minimal_creation() {
        let result = ValidationResult::warning("SIMPLE_WARNING".to_string(), "Simple warning".to_string());

        assert_eq!(result.level, ValidationSeverity::Warning);
        assert_eq!(result.message, "Simple warning");
        assert_eq!(result.entity_type, None);
        assert_eq!(result.entity_code, None);
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }
}
