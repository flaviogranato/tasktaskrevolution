use crate::domain::{
    resource::{
        resource::Resource,
        resource_repository::ResourceRepository,
    },
    shared_kernel::errors::DomainError,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub struct CreateTimeOffUseCase<R: ResourceRepository> {
    repository: R,
}

#[derive(Debug)]
pub struct CreateTimeOffResult {
    pub success: bool,
    pub message: String,
    pub resource_name: String,
    pub hours: u32,
    pub time_off_balance: u32,
    pub description: Option<String>,
    pub date: String,
}

impl<R: ResourceRepository> CreateTimeOffUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn parse_date(date_str: &str) -> Result<DateTime<Local>, DomainError> {
        let naive =
            NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
                .map_err(|e| DomainError::Generic(format!("Erro ao converter data: {}", e)))?;

        Ok(Local
            .from_local_datetime(&naive)
            .earliest()
            .ok_or_else(|| DomainError::Generic("Erro ao converter data local".to_string()))?)
    }

    pub fn execute(
        &self,
        resource: String,
        hours: u32,
        date: String,
        description: Option<String>,
    ) -> Result<CreateTimeOffResult, Box<dyn std::error::Error>> {
        match self.repository.save_time_off(resource.clone(), hours, date.clone(), description.clone()) {
            Ok(resource) => Ok(CreateTimeOffResult {
                success: true,
                message: format!("{} horas adicionadas com sucesso para {}", hours, resource.name),
                resource_name: resource.name,
                hours,
                time_off_balance: resource.time_off_balance,
                description,
                date,
            }),
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use crate::domain::resource::resource::Resource;

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

        fn save_time_off(&self, resource_name: String, hours: u32, _date: String, _description: Option<String>) -> Result<Resource, DomainError> {
            let mut resources = self.resources.borrow_mut();
            let resource = resources.iter_mut()
                .find(|r| r.id == Some(resource_name.clone()))
                .ok_or_else(|| DomainError::Generic("Recurso não encontrado".to_string()))?;
            
            resource.time_off_balance += hours;
            Ok(resource.clone())
        }

        fn save_vacation(&self, _resource_name: String, _start_date: String, _end_date: String, _is_time_off_compensation: bool, _compensated_hours: Option<u32>) -> Result<Resource, DomainError> {
            unimplemented!("Not needed for these tests")
        }
    }

    #[test]
    fn test_create_time_off_success() {
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
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute(
            "john-doe".to_string(),
            10,
            "2024-01-01".to_string(),
            Some("Test time off".to_string()),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
    }

    #[test]
    fn test_create_time_off_nonexistent_resource() {
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
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute(
            "nonexistent".to_string(),
            10,
            "2024-01-01".to_string(),
            Some("Test time off".to_string()),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_create_time_off_with_description() {
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
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute(
            "john-doe".to_string(),
            8,
            "2024-01-01".to_string(),
            Some("Dia de folga".to_string()),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 8);
    }

    #[test]
    fn test_create_time_off_without_description() {
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
        let use_case = CreateTimeOffUseCase::new(repository);

        let result = use_case.execute(
            "john-doe".to_string(),
            4,
            "2024-01-01".to_string(),
            None,
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 4);
    }

    #[test]
    fn test_create_time_off_accumulates_balance() {
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
        let use_case = CreateTimeOffUseCase::new(repository);

        // Primeiro registro
        let result1 = use_case.execute(
            "john-doe".to_string(),
            4,
            "2024-01-01".to_string(),
            Some("Manhã".to_string()),
        );
        assert!(result1.is_ok());

        // Segundo registro
        let result2 = use_case.execute(
            "john-doe".to_string(),
            4,
            "2024-01-02".to_string(),
            Some("Tarde".to_string()),
        );
        assert!(result2.is_ok());

        let final_resource = result2.unwrap();
        assert_eq!(final_resource.time_off_balance, 8);
    }
}
