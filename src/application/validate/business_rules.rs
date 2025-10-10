#![allow(dead_code)]

use super::types::ValidationResult;

use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::{repository::ResourceRepository, resource::Period};
use chrono::{DateTime, FixedOffset, Local, NaiveDate, Offset};

pub struct ValidateBusinessRulesUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: &'a P,
    resource_repository: &'a R,
    company_repository: &'a C,
}

impl<'a, P, R, C> ValidateBusinessRulesUseCase<'a, P, R, C>
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

        // Validate vacation rules
        results.extend(self.validate_vacation_rules(&companies, &resources, &projects)?);

        // Validate resource allocation
        results.extend(self.validate_resource_allocation(&resources, &projects)?);

        // Validate project timeline
        results.extend(self.validate_project_timeline(&projects)?);

        // Validate cost constraints
        results.extend(self.validate_cost_constraints(&projects)?);

        Ok(results)
    }

    fn validate_vacation_rules(
        &self,
        _companies: &[crate::domain::company_management::company::Company],
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        _projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Check vacation overlaps between resources
        for (i, resource1) in resources.iter().enumerate() {
            if let Some(vacations1) = resource1.vacations() {
                // Check overlap with other resources
                for resource2 in resources.iter().skip(i + 1) {
                    if let Some(vacations2) = resource2.vacations() {
                        for period1 in vacations1 {
                            for period2 in vacations2 {
                                if self.check_vacation_overlap(period1, period2) {
                                    results.push(
                                        ValidationResult::warning("VACATION_OVERLAP".to_string(), "Vacation overlap detected".to_string())
                                            .with_entity("Resource".to_string(), resource1.code().to_string())
                                            .with_details(format!(
                                                "Resource '{}' and '{}' have overlapping vacations between {} and {}",
                                                resource1.name(),
                                                resource2.name(),
                                                period1.start_date.format("%d/%m/%Y"),
                                                period1.end_date.format("%d/%m/%Y")
                                            )),
                                    );
                                }
                            }
                        }
                    }
                }

                // TODO: Implement layoff period validation when VacationRules methods are available
                // For now, we'll skip this validation
            }
        }

        // TODO: Implement minimum resource validation when VacationRules methods are available
        // For now, we'll skip this validation

        Ok(results)
    }

    fn validate_resource_allocation(
        &self,
        _resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        for project in projects {
            for task in project.tasks().values() {
                let assigned_resources = task.assigned_resources();

                // Check if task has resources assigned
                if assigned_resources.is_empty() {
                    results.push(
                        ValidationResult::warning("TASK_NO_RESOURCES".to_string(), "Task has no assigned resources".to_string())
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details("Task may not be completed without resource assignment".to_string()),
                    );
                    continue;
                }

                // Check resource capacity vs task effort (simplified for now)
                // TODO: Implement proper effort and capacity validation when these fields are available
            }
        }

        Ok(results)
    }

    fn validate_project_timeline(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        for project in projects {
            // Check if project has start and end dates
            if let (Some(start_date), Some(end_date)) = (project.start_date(), project.end_date()) {
                if start_date >= end_date {
                    results.push(
                        ValidationResult::error("INVALID_PROJECT_TIMELINE".to_string(), "Invalid project timeline".to_string())
                            .with_entity("Project".to_string(), project.code().to_string())
                            .with_details("Project start date must be before end date".to_string()),
                    );
                }

                // Check if tasks fit within project timeline
                for task in project.tasks().values() {
                    let task_start = task.start_date();
                    let task_end = task.due_date();
                    if *task_start < start_date {
                        results.push(
                            ValidationResult::error("TASK_STARTS_BEFORE_PROJECT".to_string(), "Task starts before project".to_string())
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details("Task start date is before project start date".to_string()),
                        );
                    }
                    if *task_end > end_date {
                        results.push(
                            ValidationResult::error("TASK_EXTENDS_BEYOND_PROJECT".to_string(), "Task extends beyond project".to_string())
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details("Task due date is after project end date".to_string()),
                        );
                    }
                }
            }
        }

        Ok(results)
    }

    fn validate_cost_constraints(
        &self,
        _projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let results = Vec::new();

        // TODO: Implement cost validation when budget and cost fields are available
        // For now, this is a placeholder

        Ok(results)
    }

    // Helper methods
    fn check_vacation_overlap(&self, period1: &Period, period2: &Period) -> bool {
        period1.start_date <= period2.end_date && period2.start_date <= period1.end_date
    }

    fn check_layoff_overlap(&self, vacation_period: &Period, layoff_period: &(String, String)) -> bool {
        let layoff_start = NaiveDate::parse_from_str(&layoff_period.0, "%Y-%m-%d")
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default();
        let layoff_end = NaiveDate::parse_from_str(&layoff_period.1, "%Y-%m-%d")
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default();

        let offset = Local::now().offset().fix();
        let layoff_start: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_start, offset);
        let layoff_end: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_end, offset);

        vacation_period.start_date <= layoff_end && layoff_start <= vacation_period.end_date
    }

    fn count_available_resources_during_project(
        &self,
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        project: &crate::domain::project_management::any_project::AnyProject,
        _vacation_rules: &crate::domain::project_management::project::VacationRules,
    ) -> usize {
        // Simplified implementation - count resources not on vacation during project period
        resources
            .iter()
            .filter(|resource| {
                if let Some(vacations) = resource.vacations() {
                    // Check if resource is available during project period
                    if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
                        !vacations.iter().any(|vacation| {
                            // TODO: Fix this when Period struct is properly implemented
                            // For now, we'll use a simple date comparison
                            // Convert DateTime to NaiveDate for comparison
                            let vacation_start = vacation.start_date.date_naive();
                            let vacation_end = vacation.end_date.date_naive();
                            vacation_start <= end && start <= vacation_end
                        })
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::validate::types::{ValidationResult, ValidationSeverity};
    use crate::domain::shared::errors::{DomainError, DomainResult};
    use crate::domain::company_management::company::Company;
    use crate::domain::company_management::repository::CompanyRepository;
    use crate::domain::project_management::any_project::AnyProject;
    use crate::domain::project_management::repository::ProjectRepository;
    use crate::domain::resource_management::any_resource::AnyResource;
    use crate::domain::resource_management::repository::ResourceRepository;
    use crate::domain::resource_management::resource::Period;
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
    fn test_validate_business_rules_use_case_creation() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let _use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);
        // Should not panic
    }

    #[test]
    fn test_validate_business_rules_execute_with_empty_repositories() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);
        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_validate_business_rules_execute_returns_validation_results() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);
        let result = use_case.execute();
        assert!(result.is_ok());
        let results = result.unwrap();

        // Verify that all results are ValidationResult instances
        for validation_result in results {
            assert!(matches!(validation_result, ValidationResult { .. }));
        }
    }

    #[test]
    fn test_check_vacation_overlap_no_overlap() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let period1 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let period2 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-20T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        assert!(!use_case.check_vacation_overlap(&period1, &period2));
    }

    #[test]
    fn test_check_vacation_overlap_with_overlap() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let period1 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let period2 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-20T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        assert!(use_case.check_vacation_overlap(&period1, &period2));
    }

    #[test]
    fn test_check_vacation_overlap_exact_same_period() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let period1 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let period2 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        assert!(use_case.check_vacation_overlap(&period1, &period2));
    }

    #[test]
    fn test_check_vacation_overlap_touching_periods() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let period1 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let period2 = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-10T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-20T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        assert!(use_case.check_vacation_overlap(&period1, &period2));
    }

    #[test]
    fn test_check_layoff_overlap_with_overlap() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let vacation_period = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let layoff_period = ("2024-01-10".to_string(), "2024-01-20".to_string());

        assert!(use_case.check_layoff_overlap(&vacation_period, &layoff_period));
    }

    #[test]
    fn test_check_layoff_overlap_no_overlap() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let vacation_period = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-05T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let layoff_period = ("2024-01-10".to_string(), "2024-01-20".to_string());

        assert!(!use_case.check_layoff_overlap(&vacation_period, &layoff_period));
    }

    #[test]
    fn test_check_layoff_overlap_invalid_date_format() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        let vacation_period = Period {
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().into(),
            end_date: DateTime::parse_from_rfc3339("2024-01-15T00:00:00Z").unwrap().into(),
            approved: true,
            period_type: crate::domain::resource_management::resource::PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };

        let layoff_period = ("invalid-date".to_string(), "2024-01-20".to_string());

        // Should not panic even with invalid date format
        let _result = use_case.check_layoff_overlap(&vacation_period, &layoff_period);
    }

    #[test]
    fn test_count_available_resources_during_project_no_vacations() {
        let project_repo = MockProjectRepository { projects: vec![] };
        let resource_repo = MockResourceRepository { resources: vec![] };
        let company_repo = MockCompanyRepository { companies: vec![] };

        let use_case = ValidateBusinessRulesUseCase::new(&project_repo, &resource_repo, &company_repo);

        // Create a mock project with start and end dates
        let project = AnyProject::Project(
            crate::domain::project_management::project::Project::new(
                "PROJ-001".to_string(),
                "Test Project".to_string(),
                "COMP-001".to_string(),
                "user1".to_string(),
            )
            .unwrap(),
        );

        let resources = vec![]; // Empty resources
        let vacation_rules = crate::domain::project_management::project::VacationRules::default();

        let count = use_case.count_available_resources_during_project(&resources, &project, &vacation_rules);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_validation_result_builder_methods() {
        let result = ValidationResult::error("TEST_ERROR".to_string(), "Test error".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string())
            .with_field("name".to_string())
            .with_details("Detailed error information".to_string());

        assert_eq!(result.level, ValidationSeverity::Error);
        assert_eq!(result.message, "Test error");
        assert_eq!(result.entity_type, Some("Project".to_string()));
        assert_eq!(result.entity_code, Some("PROJ-001".to_string()));
        assert_eq!(result.field, Some("name".to_string()));
        assert_eq!(result.details, Some("Detailed error information".to_string()));
    }

    #[test]
    fn test_validation_result_warning_builder() {
        let result = ValidationResult::warning("TEST_WARNING".to_string(), "Test warning".to_string())
            .with_entity("Resource".to_string(), "RES-001".to_string())
            .with_details("Warning details".to_string());

        assert_eq!(result.level, ValidationSeverity::Warning);
        assert_eq!(result.message, "Test warning");
        assert_eq!(result.entity_type, Some("Resource".to_string()));
        assert_eq!(result.entity_code, Some("RES-001".to_string()));
        assert_eq!(result.details, Some("Warning details".to_string()));
    }

    #[test]
    fn test_validation_result_info_builder() {
        let result =
            ValidationResult::info("TEST_INFO".to_string(), "Test info".to_string()).with_entity("Task".to_string(), "TASK-001".to_string());

        assert_eq!(result.level, ValidationSeverity::Info);
        assert_eq!(result.message, "Test info");
        assert_eq!(result.entity_type, Some("Task".to_string()));
        assert_eq!(result.entity_code, Some("TASK-001".to_string()));
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }

    #[test]
    fn test_validation_result_minimal_creation() {
        let result = ValidationResult::error("SIMPLE_ERROR".to_string(), "Simple error".to_string());

        assert_eq!(result.level, ValidationSeverity::Error);
        assert_eq!(result.message, "Simple error");
        assert_eq!(result.entity_type, None);
        assert_eq!(result.entity_code, None);
        assert_eq!(result.field, None);
        assert_eq!(result.details, None);
    }

    #[test]
    fn test_validation_result_display_formatting() {
        let result = ValidationResult::error("TEST_ERROR".to_string(), "Test error".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string())
            .with_field("name".to_string())
            .with_details("Detailed info".to_string());

        let display = format!("{}", result);
        assert!(display.contains("ERROR"));
        assert!(display.contains("Test error"));
        assert!(display.contains("Project: PROJ-001"));
        assert!(display.contains("Field: name"));
        assert!(display.contains("Detailed info"));
    }

    #[test]
    fn test_validation_result_with_different_entity_types() {
        let project_result = ValidationResult::error("PROJECT_ERROR".to_string(), "Project error".to_string())
            .with_entity("Project".to_string(), "PROJ-001".to_string());

        let resource_result = ValidationResult::warning("RESOURCE_WARNING".to_string(), "Resource warning".to_string())
            .with_entity("Resource".to_string(), "RES-001".to_string());

        let task_result =
            ValidationResult::info("TASK_INFO".to_string(), "Task info".to_string()).with_entity("Task".to_string(), "TASK-001".to_string());

        assert_eq!(project_result.entity_type, Some("Project".to_string()));
        assert_eq!(resource_result.entity_type, Some("Resource".to_string()));
        assert_eq!(task_result.entity_type, Some("Task".to_string()));
    }

    #[test]
    fn test_validation_result_severity_levels() {
        let error_result = ValidationResult::error("ERROR_MESSAGE".to_string(), "Error message".to_string());
        let warning_result = ValidationResult::warning("WARNING_MESSAGE".to_string(), "Warning message".to_string());
        let info_result = ValidationResult::info("INFO_MESSAGE".to_string(), "Info message".to_string());

        assert_eq!(error_result.level, ValidationSeverity::Error);
        assert_eq!(warning_result.level, ValidationSeverity::Warning);
        assert_eq!(info_result.level, ValidationSeverity::Info);
    }
}
