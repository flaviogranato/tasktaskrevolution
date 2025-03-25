use crate::domain::{
    resource::{
        resource::{Resource, TimeOffEntry},
        resource_repository::ResourceRepository,
    },
    shared_kernel::errors::DomainError,
};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};

pub struct CreateTimeOffUseCase<R: ResourceRepository> {
    resource_repository: R,
}

impl<R: ResourceRepository> CreateTimeOffUseCase<R> {
    pub fn new(resource_repository: R) -> Self {
        Self {
            resource_repository,
        }
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
        resource_identifier: String,
        hours: u32,
        date: String,
        description: Option<String>,
    ) -> Result<Resource, DomainError> {
        let resources = self.resource_repository.find_all()?;

        let mut resource = resources
            .into_iter()
            .find(|r| {
                r.id.as_ref().map_or(false, |id| id == &resource_identifier)
                    || r.name == resource_identifier
            })
            .ok_or_else(|| {
                DomainError::Generic(format!("Recurso não encontrado: {}", resource_identifier))
            })?;

        let entry_date = Self::parse_date(&date)?;

        // Cria uma nova entrada no histórico
        let time_off_entry = TimeOffEntry {
            date: entry_date,
            hours,
            description,
        };

        // Adiciona a entrada ao histórico
        let mut history = resource.time_off_history.unwrap_or_else(Vec::new);
        history.push(time_off_entry);
        resource.time_off_history = Some(history);

        // Atualiza o saldo total
        resource.time_off_balance += hours;

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
            8,
            "2024-01-01".to_string(),
            Some("Trabalho no feriado".to_string()),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 8);

        // Verifica se a entrada foi adicionada ao histórico
        let history = updated_resource.time_off_history.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].hours, 8);
        assert_eq!(
            history[0].description,
            Some("Trabalho no feriado".to_string())
        );
    }

    #[test]
    fn test_create_multiple_time_off_entries() {
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

        // Primeira entrada
        let result1 = use_case.execute(
            "john-doe".to_string(),
            8,
            "2024-01-01".to_string(),
            Some("Trabalho no feriado".to_string()),
        );
        assert!(result1.is_ok());

        // Segunda entrada
        let result2 = use_case.execute(
            "john-doe".to_string(),
            4,
            "2024-01-02".to_string(),
            Some("Hora extra".to_string()),
        );
        assert!(result2.is_ok());

        let final_resource = result2.unwrap();
        assert_eq!(final_resource.time_off_balance, 12); // 8 + 4

        let history = final_resource.time_off_history.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].hours, 4);
        assert_eq!(history[1].description, Some("Hora extra".to_string()));
    }
}
