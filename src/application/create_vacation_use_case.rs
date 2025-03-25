use crate::domain::{
    resource::{
        resource::{Period, PeriodType, Resource},
        resource_repository::ResourceRepository,
    },
    shared_kernel::errors::DomainError,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub struct CreateVacationUseCase<R: ResourceRepository> {
    resource_repository: R,
}

impl<R: ResourceRepository> CreateVacationUseCase<R> {
    pub fn new(resource_repository: R) -> Self {
        Self {
            resource_repository,
        }
    }

    fn parse_date(date_str: &str) -> Result<DateTime<Local>, DomainError> {
        let naive =
            NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
                .map_err(|e| DomainError::Generic(format!("Erro ao converter data: {}", e)))?;

        // Usando from_local em vez de from_naive_utc_and_local
        Ok(Local
            .from_local_datetime(&naive)
            .earliest()
            .ok_or_else(|| DomainError::Generic("Erro ao converter data local".to_string()))?)
    }

    pub fn execute(
        &self,
        resource_identifier: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<Resource, DomainError> {
        let resources = self.resource_repository.find_all()?;

        // Busca por código ou nome
        let mut resource = resources
            .into_iter()
            .find(|r| {
                r.id.as_ref().map_or(false, |id| id == &resource_identifier)
                    || r.name == resource_identifier
            })
            .ok_or_else(|| {
                DomainError::Generic(format!("Recurso não encontrado: {}", resource_identifier))
            })?;

        let start = Self::parse_date(&start_date)?;
        let end = Self::parse_date(&end_date)?;

        if start >= end {
            return Err(DomainError::Generic(
                "Data de início deve ser anterior à data de fim".to_string(),
            ));
        }

        let new_period = Period {
            start_date: start,
            end_date: end,
            approved: false,
            period_type: PeriodType::Vacation,
            is_time_off_compensation,
            compensated_hours,
        };

        let mut vacations = resource.vacations.unwrap_or_else(Vec::new);
        vacations.push(new_period);
        resource.vacations = Some(vacations);

        self.resource_repository.save(resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockResourceRepository {
        resources: RefCell<Vec<Resource>>,
    }

    impl MockResourceRepository {
        fn new(resources: Vec<Resource>) -> Self {
            Self {
                resources: RefCell::new(resources),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: Resource) -> Result<Resource, DomainError> {
            let mut resources = self.resources.borrow_mut();
            let index = resources
                .iter()
                .position(|r| r.id == resource.id)
                .ok_or_else(|| DomainError::Generic("Recurso não encontrado".to_string()))?;
            resources[index] = resource.clone();
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.borrow().clone())
        }
    }

    #[test]
    fn test_create_vacation_success() {
        let resource = Resource::new(
            Some("john-doe".to_string()),
            "John Doe".to_string(),
            None,
            "Developer".to_string(),
            None,
            None,
            0,
        );

        let repository = MockResourceRepository::new(vec![resource]);
        let use_case = CreateVacationUseCase::new(repository);

        let result = use_case.execute(
            "john-doe".to_string(),
            "2024-01-01".to_string(),
            "2024-01-15".to_string(),
            false,
            None,
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.vacations.unwrap().len(), 1);
    }

    #[test]
    fn test_create_vacation_invalid_dates() {
        let resource = Resource::new(
            Some("john-doe".to_string()),
            "John Doe".to_string(),
            None,
            "Developer".to_string(),
            None,
            None,
            0,
        );

        let repository = MockResourceRepository::new(vec![resource]);
        let use_case = CreateVacationUseCase::new(repository);

        let result = use_case.execute(
            "john-doe".to_string(),
            "2024-01-15".to_string(),
            "2024-01-01".to_string(),
            false,
            None,
        );

        assert!(result.is_err());
    }
}
