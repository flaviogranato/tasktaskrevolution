use crate::domain::{
    resource_management::repository::ResourceRepository,
    shared::errors::{DomainError, DomainErrorKind},
};
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

    #[allow(dead_code)]
    fn parse_date(date_str: &str) -> Result<DateTime<Local>, DomainError> {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|_| {
                DomainError::new(DomainErrorKind::Generic {
                    message: "Formato de data inválido. Use YYYY-MM-DD".to_string(),
                })
            })?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| {
                DomainError::new(DomainErrorKind::Generic {
                    message: "Erro ao converter hora".to_string(),
                })
            })?;

        Local.from_local_datetime(&naive).earliest().ok_or_else(|| {
            DomainError::new(DomainErrorKind::Generic {
                message: "Erro ao converter data local".to_string(),
            })
        })
    }

    pub fn execute(
        &self,
        resource: &str,
        hours: u32,
        date: &str,
        description: Option<&str>,
    ) -> Result<CreateTimeOffResult, Box<dyn std::error::Error>> {
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
    use crate::domain::{
        resource_management::{AnyResource, resource::Resource, state::Available},
        shared::errors::DomainError,
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
        fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError> {
            let mut resources = self.resources.borrow_mut();
            if let Some(index) = resources.iter().position(|r| r.id() == resource.id()) {
                resources[index] = resource.clone();
            } else {
                resources.push(resource.clone());
            }
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.borrow().clone())
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(None)
        }

        fn save_time_off(
            &self,
            resource_name: &str,
            hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            let mut resources = self.resources.borrow_mut();
            let resource_any = resources
                .iter_mut()
                .find(|r| r.name() == resource_name)
                .ok_or_else(|| {
                    DomainError::new(DomainErrorKind::ResourceNotFound {
                        code: "Resource not found".to_string(),
                    })
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
                    return Err(DomainError::new(DomainErrorKind::ResourceInvalidState {
                        current: "Inactive".to_string(),
                        expected: "Active".to_string(),
                    }));
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
        ) -> Result<AnyResource, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
        }

        fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    fn create_test_available_resource() -> AnyResource {
        Resource::<Available>::new(
            "developer-1".to_string(), // dummy code
            "John Doe".to_string(),
            None,
            "Developer".to_string(),
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
}
