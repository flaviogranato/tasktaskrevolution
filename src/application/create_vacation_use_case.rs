use crate::domain::{
    project::project_repository::ProjectRepository,
    resource::{
        resource::Resource,
        resource_repository::ResourceRepository,
    },
    shared_kernel::errors::DomainError,
};
use chrono::{NaiveDate, DateTime, Local};

pub struct CreateVacationUseCase<R: ResourceRepository> {
    repository: R,
}

#[derive(Debug)]
pub struct CreateVacationResult {
    pub success: bool,
    pub message: String,
    pub resource_name: String,
}

impl<R: ResourceRepository> CreateVacationUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn validate_dates(start_date: &str, end_date: &str) -> bool {
        if let (Ok(start), Ok(end)) = (
            NaiveDate::parse_from_str(start_date, "%Y-%m-%d"),
            NaiveDate::parse_from_str(end_date, "%Y-%m-%d"),
        ) {
            start <= end
        } else {
            false
        }
    }

    pub fn execute(
        &self,
        resource: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<CreateVacationResult, Box<dyn std::error::Error>> {
        if !Self::validate_dates(&start_date, &end_date) {
            return Ok(CreateVacationResult {
                success: false,
                message: "Data de início deve ser anterior à data de fim".to_string(),
                resource_name: String::new(),
            });
        }

        match self.repository.save_vacation(
            resource,
            start_date,
            end_date,
            is_time_off_compensation,
            compensated_hours,
        ) {
            Ok(resource) => Ok(CreateVacationResult {
                success: true,
                message: format!("Período de férias adicionado com sucesso para {}", resource.name),
                resource_name: resource.name,
            }),
            Err(e) => Ok(CreateVacationResult {
                success: false,
                message: format!("Erro ao adicionar período de férias: {}", e),
                resource_name: String::new(),
            }),
        }
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

        fn save_time_off(&self, _resource_name: String, _hours: u32, _date: String, _description: Option<String>) -> Result<Resource, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn save_vacation(&self, resource_name: String, _start_date: String, _end_date: String, _is_time_off_compensation: bool, _compensated_hours: Option<u32>) -> Result<Resource, DomainError> {
            let mut resources = self.resources.borrow_mut();
            let resource = resources.iter_mut()
                .find(|r| r.id == Some(resource_name.clone()))
                .ok_or_else(|| DomainError::Generic("Recurso não encontrado".to_string()))?;
            
            // Aqui você pode adicionar a lógica para salvar as férias no recurso
            // Por enquanto, apenas retornamos o recurso sem modificações
            Ok(resource.clone())
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            false
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
        assert_eq!(updated_resource.resource_name, "John Doe");
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

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.resource_name, "");
    }
}
