use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::errors::{DomainError, DomainResult};

pub struct CreateVacationUseCase<R: ResourceRepository> {
    repository: R,
}

#[derive(Debug, PartialEq)]
pub struct CreateVacationResult {
    pub success: bool,
    pub message: String,
}

impl<R: ResourceRepository> CreateVacationUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn validate_dates(start_date: &str, end_date: &str) -> bool {
        if let (Ok(start), Ok(end)) = (
            chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d"),
            chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d"),
        ) {
            start <= end
        } else {
            false
        }
    }

    pub fn execute(
        &self,
        resource_name: &str,
        start_date: &str,
        end_date: &str,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<CreateVacationResult, Box<dyn std::error::Error>> {
        if !Self::validate_dates(start_date, end_date) {
            return Ok(CreateVacationResult {
                success: false,
                message: "Data de início deve ser anterior ou igual à data de fim".to_string(),
            });
        }

        match self.repository.save_vacation(
            resource_name,
            start_date,
            end_date,
            is_time_off_compensation,
            compensated_hours,
        ) {
            Ok(resource) => Ok(CreateVacationResult {
                success: true,
                message: format!("Período de férias adicionado com sucesso para {}", resource.name()),
            }),
            Err(e) => Ok(CreateVacationResult {
                success: false,
                message: format!("Erro ao adicionar período de férias: {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::resource_management::{
        AnyResource,
        resource::{Period, PeriodType, Resource, ResourceScope},
    };
    use chrono::{DateTime, Local, NaiveDateTime};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    // A mock repository that uses Rc<RefCell<...>> to share state between clones.
    struct MockResourceRepository {
        resources: Rc<RefCell<HashMap<String, AnyResource>>>,
        should_fail: bool,
    }

    impl Clone for MockResourceRepository {
        fn clone(&self) -> Self {
            Self {
                resources: self.resources.clone(), // This clones the Rc, not the data
                should_fail: self.should_fail,
            }
        }
    }

    impl MockResourceRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                resources: Rc::new(RefCell::new(HashMap::new())),
                should_fail,
            }
        }

        fn add_resource(&self, resource: AnyResource) {
            self.resources
                .borrow_mut()
                .insert(resource.name().to_string(), resource);
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(None)
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
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

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            resource_name: &str,
            start_date: &str,
            end_date: &str,
            is_time_off_compensation: bool,
            compensated_hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            if self.should_fail {
                return Err(AppError::ValidationError {
                    field: "repository".to_string(),
                    message: "Simulated repository error".to_string(),
                });
            }

            let mut resources = self.resources.borrow_mut();
            if let Some(any_resource) = resources.get_mut(resource_name) {
                let new_period = Period {
                    start_date: NaiveDateTime::parse_from_str(&format!("{start_date} 00:00:00"), "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                    end_date: NaiveDateTime::parse_from_str(&format!("{end_date} 00:00:00"), "%Y-%m-%d %H:%M:%S")
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap(),
                    approved: true,
                    period_type: PeriodType::Vacation,
                    is_time_off_compensation,
                    compensated_hours,
                    is_layoff: false,
                };

                let add_vacation = |vacations: Option<Vec<Period>>| -> Option<Vec<Period>> {
                    let mut v = vacations.unwrap_or_default();
                    v.push(new_period);
                    Some(v)
                };

                match any_resource {
                    AnyResource::Available(r) => r.vacations = add_vacation(r.vacations.clone()),
                    AnyResource::Assigned(r) => r.vacations = add_vacation(r.vacations.clone()),
                    AnyResource::Inactive(_) => {
                        return Err(AppError::ResourceInvalidState {
                            current: "Inactive".to_string(),
                            expected: "Active".to_string(),
                        });
                    }
                }
                Ok(any_resource.clone())
            } else {
                Err(AppError::ResourceNotFound {
                    code: resource_name.to_string(),
                })
            }
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    fn setup_test() -> (MockResourceRepository, AnyResource) {
        let mock_repo = MockResourceRepository::new(false);
        let resource = Resource::new(
            "res-01".to_string(),
            "John Doe".to_string(),
            None,
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            10,
        );
        let any_resource = AnyResource::Available(resource);
        mock_repo.add_resource(any_resource.clone());
        (mock_repo, any_resource)
    }

    #[test]
    fn test_create_vacation_success() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(resource.name(), "2025-07-01", "2025-07-10", false, None)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.message, "Período de férias adicionado com sucesso para John Doe");
    }

    #[test]
    fn test_create_vacation_invalid_dates() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(
                resource.name(),
                "2025-07-10", // End date
                "2025-07-01", // Start date
                false,
                None,
            )
            .unwrap();

        assert!(!result.success);
        assert_eq!(
            result.message,
            "Data de início deve ser anterior ou igual à data de fim"
        );
    }

    #[test]
    fn test_create_vacation_repository_fails() {
        let (mut mock_repo, resource) = setup_test();
        mock_repo.should_fail = true; // Configure mock to fail
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(resource.name(), "2025-08-01", "2025-08-10", false, None)
            .unwrap();

        assert!(!result.success);
        assert!(result.message.contains("Simulated repository error"));
    }

    #[test]
    fn test_create_vacation_with_compensation() {
        let (mock_repo, resource) = setup_test();
        // Clone the repo for the use case, so we can inspect the original
        let use_case = CreateVacationUseCase::new(mock_repo.clone());

        let _ = use_case
            .execute(resource.name(), "2025-09-01", "2025-09-02", true, Some(16))
            .unwrap();

        // Verify the data was saved correctly in the shared state via the original mock
        let stored_resource = mock_repo.resources.borrow().get(resource.name()).unwrap().clone();
        let vacations = stored_resource.vacations().unwrap();
        let last_vacation = vacations.last().unwrap();

        assert_eq!(vacations.len(), 1);
        assert!(last_vacation.is_time_off_compensation);
        assert_eq!(last_vacation.compensated_hours, Some(16));
    }

    #[test]
    fn test_create_vacation_validation_dates_fail() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(
                resource.name(),
                "2025-07-10", // End date
                "2025-07-01", // Start date
                false,
                None,
            )
            .unwrap();

        assert!(!result.success);
        assert_eq!(
            result.message,
            "Data de início deve ser anterior ou igual à data de fim"
        );
    }

    #[test]
    fn test_create_vacation_malformed_dates() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(
                resource.name(),
                "invalid-date", // Malformed start date
                "2025-07-10",
                false,
                None,
            )
            .unwrap();

        assert!(!result.success);
        assert_eq!(
            result.message,
            "Data de início deve ser anterior ou igual à data de fim"
        );
    }

    #[test]
    fn test_create_vacation_malformed_end_date() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(
                resource.name(),
                "2025-07-01",
                "invalid-date", // Malformed end date
                false,
                None,
            )
            .unwrap();

        assert!(!result.success);
        assert_eq!(
            result.message,
            "Data de início deve ser anterior ou igual à data de fim"
        );
    }

    #[test]
    fn test_create_vacation_both_dates_malformed() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(
                resource.name(),
                "invalid-start", // Malformed start date
                "invalid-end",   // Malformed end date
                false,
                None,
            )
            .unwrap();

        assert!(!result.success);
        assert_eq!(
            result.message,
            "Data de início deve ser anterior ou igual à data de fim"
        );
    }

    #[test]
    fn test_create_vacation_success_message_formatting() {
        let (mock_repo, resource) = setup_test();
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(resource.name(), "2025-07-01", "2025-07-10", false, None)
            .unwrap();

        assert!(result.success);
        assert_eq!(result.message, "Período de férias adicionado com sucesso para John Doe");
    }

    #[test]
    fn test_create_vacation_repository_error_message_formatting() {
        let (mut mock_repo, resource) = setup_test();
        mock_repo.should_fail = true; // Configure mock to fail
        let use_case = CreateVacationUseCase::new(mock_repo);

        let result = use_case
            .execute(resource.name(), "2025-08-01", "2025-08-10", false, None)
            .unwrap();

        assert!(!result.success);
        assert!(result.message.contains("Erro ao adicionar período de férias:"));
        assert!(result.message.contains("Simulated repository error"));
    }
}
