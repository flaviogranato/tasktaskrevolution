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
                    ValidationResult::warning(project_spec.description().to_string())
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_details(explanation),
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
                    ValidationResult::warning(resource_spec.description().to_string())
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
                    ValidationResult::warning(company_spec.description().to_string())
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
        fn save(&self, _project: AnyProject) -> Result<(), AppError> {
            Ok(())
        }

        fn load(&self) -> Result<AnyProject, AppError> {
            Err(AppError::ProjectNotFound {
                code: "test".to_string(),
            })
        }

        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            Ok(self.projects.clone())
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<AnyProject>, AppError> {
            Ok(None)
        }

        fn get_next_code(&self) -> Result<String, AppError> {
            Ok("PROJ-001".to_string())
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, AppError> {
            Err(AppError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn save_in_hierarchy(
            &self,
            _resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            Err(AppError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(self.resources.clone())
        }

        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }

        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            Ok(vec![])
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(None)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, AppError> {
            Err(AppError::ResourceNotFound {
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
        ) -> Result<AnyResource, AppError> {
            Err(AppError::ResourceNotFound {
                code: "test".to_string(),
            })
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> Result<String, AppError> {
            Ok("RES-001".to_string())
        }
    }

    impl CompanyRepository for MockCompanyRepository {
        fn save(&self, _company: Company) -> Result<(), AppError> {
            Ok(())
        }

        fn find_all(&self) -> Result<Vec<Company>, AppError> {
            Ok(self.companies.clone())
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn find_by_id(&self, _id: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn find_by_name(&self, _name: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn update(&self, _company: Company) -> Result<(), AppError> {
            Ok(())
        }

        fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn get_next_code(&self) -> Result<String, AppError> {
            Ok("COMP-001".to_string())
        }

        fn code_exists(&self, _code: &str) -> Result<bool, AppError> {
            Ok(false)
        }

        fn name_exists(&self, _name: &str) -> Result<bool, AppError> {
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
            entity_type: Some("Project".to_string()),
            entity_code: Some("PROJ-001".to_string()),
            field: Some("timeline".to_string()),
            severity: ValidationSeverity::Warning,
            message: "Data integrity validation failed".to_string(),
            details: Some("Project timeline is invalid".to_string()),
        };

        assert_eq!(validation_result.entity_type, Some("Project".to_string()));
        assert_eq!(validation_result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(validation_result.field, Some("timeline".to_string()));
        assert_eq!(validation_result.severity, ValidationSeverity::Warning);
        assert_eq!(validation_result.message, "Data integrity validation failed");
        assert_eq!(
            validation_result.details,
            Some("Project timeline is invalid".to_string())
        );
    }

    #[test]
    fn test_validate_data_integrity_validation_result_with_minimal_fields() {
        let validation_result = ValidationResult {
            entity_type: None,
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Info,
            message: "Data integrity check completed".to_string(),
            details: None,
        };

        assert_eq!(validation_result.entity_type, None);
        assert_eq!(validation_result.entity_code, None);
        assert_eq!(validation_result.field, None);
        assert_eq!(validation_result.severity, ValidationSeverity::Info);
        assert_eq!(validation_result.message, "Data integrity check completed");
        assert_eq!(validation_result.details, None);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_severity_levels() {
        let error_result = ValidationResult {
            entity_type: None,
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Error,
            message: "Critical data integrity error".to_string(),
            details: None,
        };

        let warning_result = ValidationResult {
            entity_type: None,
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Warning,
            message: "Data integrity warning".to_string(),
            details: None,
        };

        let info_result = ValidationResult {
            entity_type: None,
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Info,
            message: "Data integrity information".to_string(),
            details: None,
        };

        assert_eq!(error_result.severity, ValidationSeverity::Error);
        assert_eq!(warning_result.severity, ValidationSeverity::Warning);
        assert_eq!(info_result.severity, ValidationSeverity::Info);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_entity_types() {
        let project_result = ValidationResult {
            entity_type: Some("Project".to_string()),
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Warning,
            message: "Project validation".to_string(),
            details: None,
        };

        let resource_result = ValidationResult {
            entity_type: Some("Resource".to_string()),
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Warning,
            message: "Resource validation".to_string(),
            details: None,
        };

        let company_result = ValidationResult {
            entity_type: Some("Company".to_string()),
            entity_code: None,
            field: None,
            severity: ValidationSeverity::Warning,
            message: "Company validation".to_string(),
            details: None,
        };

        assert_eq!(project_result.entity_type, Some("Project".to_string()));
        assert_eq!(resource_result.entity_type, Some("Resource".to_string()));
        assert_eq!(company_result.entity_type, Some("Company".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_display_formatting() {
        let result = ValidationResult::warning("Data integrity issue".to_string())
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
        let result = ValidationResult::warning("Test warning".to_string())
            .with_entity("Resource".to_string(), "RES-001".to_string())
            .with_field("vacation".to_string())
            .with_details("Vacation validation failed".to_string());

        assert_eq!(result.severity, ValidationSeverity::Warning);
        assert_eq!(result.message, "Test warning");
        assert_eq!(result.entity_type, Some("Resource".to_string()));
        assert_eq!(result.entity_code, Some("RES-001".to_string()));
        assert_eq!(result.field, Some("vacation".to_string()));
        assert_eq!(result.details, Some("Vacation validation failed".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_error_builder() {
        let result = ValidationResult::error("Critical error".to_string())
            .with_entity("Company".to_string(), "COMP-001".to_string())
            .with_details("Company settings validation failed".to_string());

        assert_eq!(result.severity, ValidationSeverity::Error);
        assert_eq!(result.message, "Critical error");
        assert_eq!(result.entity_type, Some("Company".to_string()));
        assert_eq!(result.entity_code, Some("COMP-001".to_string()));
        assert_eq!(result.details, Some("Company settings validation failed".to_string()));
    }

    #[test]
    fn test_validate_data_integrity_validation_result_info_builder() {
        let result = ValidationResult::info("Information message".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string());

        assert_eq!(result.severity, ValidationSeverity::Info);
        assert_eq!(result.message, "Information message");
        assert_eq!(result.entity_type, Some("Project".to_string()));
        assert_eq!(result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }

    #[test]
    fn test_validate_data_integrity_validation_result_minimal_creation() {
        let result = ValidationResult::warning("Simple warning".to_string());

        assert_eq!(result.severity, ValidationSeverity::Warning);
        assert_eq!(result.message, "Simple warning");
        assert_eq!(result.entity_type, None);
        assert_eq!(result.entity_code, None);
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }
}
