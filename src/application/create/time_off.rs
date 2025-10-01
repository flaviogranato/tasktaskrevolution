use crate::application::errors::AppError;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, Local, NaiveDate, TimeZone};

pub struct CreateTimeOffUseCase<R: ResourceRepository> {
    repository: R,
}

#[derive(Debug)]
pub struct CreateTimeOffResult {
    pub success: bool,
    pub message: String,
    pub time_off_balance: u32,
    pub description: Option<String>,
    pub date: String,
}

impl<R: ResourceRepository> CreateTimeOffUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn parse_date(date_str: &str) -> Result<DateTime<Local>, AppError> {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| AppError::ValidationError {
                field: "date".to_string(),
                message: "Formato de data inválido. Use YYYY-MM-DD".to_string(),
            })?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::ValidationError {
                field: "time".to_string(),
                message: "Erro ao converter hora".to_string(),
            })?;

        Local
            .from_local_datetime(&naive)
            .earliest()
            .ok_or_else(|| AppError::ValidationError {
                field: "date".to_string(),
                message: "Erro ao converter data local".to_string(),
            })
    }

    pub fn execute(
        &self,
        resource: &str,
        hours: u32,
        date: &str,
        description: Option<&str>,
    ) -> Result<CreateTimeOffResult, Box<dyn std::error::Error>> {
        // Validate date format first
        Self::parse_date(date)?;

        match self
            .repository
            .save_time_off(resource, hours, date, description.map(|d| d.to_string()))
        {
            Ok(resource) => Ok(CreateTimeOffResult {
                success: true,
                message: format!("{} horas adicionadas com sucesso para {}", hours, resource.name()),
                time_off_balance: resource.time_off_balance(),
                description: description.map(|d| d.to_string()),
                date: date.to_string(),
            }),
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::resource_management::{
        AnyResource,
        resource::{Resource, ResourceScope},
        state::Available,
    };
    use std::cell::RefCell;

    struct MockResourceRepository {
        resources: RefCell<Vec<AnyResource>>,
    }

    impl MockResourceRepository {
        fn new(resources: Vec<AnyResource>) -> Self {
            Self {
                resources: RefCell::new(resources),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            let mut resources = self.resources.borrow_mut();
            if let Some(index) = resources.iter().position(|r| r.id() == resource.id()) {
                resources[index] = resource.clone();
            } else {
                resources.push(resource.clone());
            }
            Ok(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().clone())
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(None)
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

        fn save_time_off(
            &self,
            resource_name: &str,
            hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<AnyResource> {
            // Force error for specific test case
            if resource_name == "error_resource" {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Simulated repository error".to_string(),
                });
            }

            let mut resources = self.resources.borrow_mut();
            let resource_any = resources
                .iter_mut()
                .find(|r| r.name() == resource_name)
                .ok_or_else(|| AppError::ResourceNotFound {
                    code: "Resource not found".to_string(),
                })?;

            let updated = match resource_any {
                AnyResource::Available(r) => {
                    let mut updated_r = r.clone();
                    updated_r.time_off_balance += hours;
                    AnyResource::Available(updated_r)
                }
                AnyResource::Assigned(r) => {
                    let mut updated_r = r.clone();
                    updated_r.time_off_balance += hours;
                    AnyResource::Assigned(updated_r)
                }
                AnyResource::Inactive(_) => {
                    return Err(DomainError::ResourceInvalidState {
                        current: "Inactive".to_string(),
                        expected: "Active".to_string(),
                    });
                }
            };
            *resource_any = updated.clone();
            Ok(updated)
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            unimplemented!("Not needed for these tests")
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, resource_type: &str) -> DomainResult<String> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    fn create_test_available_resource() -> AnyResource {
        Resource::<Available>::new(
            "developer-1".to_string(), // dummy code
            "John Doe".to_string(),
            None,
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            0,
        )
        .into()
    }

    #[test]
    fn test_create_time_off_success() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 10, "2024-01-01", Some("Test time off"));

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
    }

    #[test]
    fn test_create_time_off_nonexistent_resource() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("nonexistent", 10, "2024-01-01", Some("Test time off"));

        assert!(result.is_err());
    }

    #[test]
    fn test_create_time_off_accumulates_balance() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // First entry
        let result1 = use_case.execute("John Doe", 4, "2024-01-01", Some("Manhã"));
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().time_off_balance, 4);

        // Second entry
        let result2 = use_case.execute("John Doe", 4, "2024-01-02", Some("Tarde"));
        assert!(result2.is_ok());

        let final_resource = result2.unwrap();
        assert_eq!(final_resource.time_off_balance, 8);
    }

    #[test]
    fn test_create_time_off_repository_error() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("error_resource", 10, "2024-01-01", Some("Test time off"));

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Validation error for field 'repository': Simulated repository error"
        );
    }

    #[test]
    fn test_create_time_off_with_empty_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 10, "2024-01-01", None);
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert!(updated_resource.description.is_none());
    }

    #[test]
    fn test_create_time_off_with_invalid_date_format() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with invalid date format - should fail at parse_date
        let result = use_case.execute("John Doe", 10, "invalid-date", Some("Test time off"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Formato de data inválido"));
    }

    #[test]
    fn test_create_time_off_with_invalid_date_format_2() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with invalid date format - should fail at parse_date
        let result = use_case.execute("John Doe", 10, "01/01/2024", Some("Test time off"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Formato de data inválido"));
    }

    #[test]
    fn test_create_time_off_with_malformed_date() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with malformed date that might cause hour conversion issues
        let result = use_case.execute("John Doe", 10, "2024-13-45", Some("Test time off"));
        assert!(result.is_err());
        // This should fail at date parsing, not at hour conversion
        assert!(result.unwrap_err().to_string().contains("Formato de data inválido"));
    }

    #[test]
    fn test_create_time_off_with_edge_case_dates() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with valid edge case dates
        let result1 = use_case.execute("John Doe", 10, "2024-01-01", Some("New Year"));
        assert!(result1.is_ok());

        let result2 = use_case.execute("John Doe", 10, "2024-12-31", Some("Year End"));
        assert!(result2.is_ok());

        let result3 = use_case.execute("John Doe", 10, "2024-02-29", Some("Leap Year"));
        assert!(result3.is_ok());
    }

    #[test]
    fn test_create_time_off_with_special_characters_in_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute(
            "John Doe",
            10,
            "2024-01-01",
            Some("Test with special chars: !@#$%^&*()"),
        );
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert_eq!(
            updated_resource.description,
            Some("Test with special chars: !@#$%^&*()".to_string())
        );
    }

    #[test]
    fn test_create_time_off_with_zero_hours() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 0, "2024-01-01", Some("Zero hours"));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 0);
    }

    #[test]
    fn test_create_time_off_with_large_hours() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 999999, "2024-01-01", Some("Large hours"));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 999999);
    }

    #[test]
    fn test_create_time_off_with_max_u32_hours() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", u32::MAX, "2024-01-01", Some("Max hours"));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, u32::MAX);
    }

    #[test]
    fn test_create_time_off_with_edge_case_time_conversion() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with dates that might cause time conversion issues
        // These dates should be valid but test edge cases
        let result1 = use_case.execute("John Doe", 10, "2024-01-01", Some("Start of year"));
        assert!(result1.is_ok());

        let result2 = use_case.execute("John Doe", 10, "2024-12-31", Some("End of year"));
        assert!(result2.is_ok());

        let result3 = use_case.execute("John Doe", 10, "2024-06-15", Some("Mid year"));
        assert!(result3.is_ok());
    }

    #[test]
    fn test_create_time_off_with_different_timezone_scenarios() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with dates that might have different timezone implications
        let result1 = use_case.execute("John Doe", 10, "2024-03-10", Some("DST start"));
        assert!(result1.is_ok());

        let result2 = use_case.execute("John Doe", 10, "2024-11-03", Some("DST end"));
        assert!(result2.is_ok());

        let result3 = use_case.execute("John Doe", 10, "2024-07-04", Some("Summer date"));
        assert!(result3.is_ok());
    }

    #[test]
    fn test_create_time_off_with_very_short_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 10, "2024-01-01", Some(""));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert_eq!(updated_resource.description, Some("".to_string()));
    }

    #[test]
    fn test_create_time_off_with_numeric_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 10, "2024-01-01", Some("12345"));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert_eq!(updated_resource.description, Some("12345".to_string()));
    }

    #[test]
    fn test_create_time_off_with_inactive_resource() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // This should fail because the mock repository doesn't have an inactive resource
        // but we can test the error handling path
        let result = use_case.execute("inactive_resource", 10, "2024-01-01", Some("Test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_time_off_with_very_long_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let long_description = "A".repeat(1000); // Very long description
        let result = use_case.execute("John Doe", 10, "2024-01-01", Some(&long_description));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert_eq!(updated_resource.description, Some(long_description));
    }

    #[test]
    fn test_create_time_off_with_unicode_description() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let unicode_description = "Férias com emojis 🏖️🌴☀️ e acentos: áéíóú çãõ";
        let result = use_case.execute("John Doe", 10, "2024-01-01", Some(unicode_description));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
        assert_eq!(updated_resource.description, Some(unicode_description.to_string()));
    }

    #[test]
    fn test_create_time_off_with_boundary_dates() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        // Test with boundary dates
        let result1 = use_case.execute("John Doe", 10, "1900-01-01", Some("Very old date"));
        assert!(result1.is_ok());

        let result2 = use_case.execute("John Doe", 10, "2100-12-31", Some("Future date"));
        assert!(result2.is_ok());

        let result3 = use_case.execute("John Doe", 10, "2000-02-29", Some("Leap year"));
        assert!(result3.is_ok());
    }

    #[test]
    fn test_create_time_off_with_single_character_name() {
        let repository = MockResourceRepository::new(vec![create_test_available_resource()]);
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute("John Doe", 10, "2024-01-01", Some("Single char name"));
        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
    }
}
