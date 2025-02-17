use crate::domain::{
    project::project_repository::ProjectRepository,
    resource::resource_repository::ResourceRepository,
    shared_kernel::errors::DomainError,
    resource::resource::Period,
};
use chrono::Local;

pub struct ValidateVacationsUseCase<P: ProjectRepository, R: ResourceRepository> {
    project_repository: P,
    resource_repository: R,
}

impl<P: ProjectRepository, R: ResourceRepository> ValidateVacationsUseCase<P, R> {
    pub fn new(project_repository: P, resource_repository: R) -> Self {
        Self {
            project_repository,
            resource_repository,
        }
    }

    fn check_vacation_overlap(&self, period1: &Period, period2: &Period) -> bool {
        period1.start_date <= period2.end_date && period2.start_date <= period1.end_date
    }

    pub fn execute(&self) -> Result<Vec<String>, DomainError> {
        let resources = self.resource_repository.find_all()?;
        let mut mensagens = Vec::new();
        
        // Verificar sobreposição entre todos os recursos
        for (i, resource1) in resources.iter().enumerate() {
            if let Some(vacations1) = &resource1.vacations {
                // Verificar sobreposição com outros recursos
                for resource2 in resources.iter().skip(i + 1) {
                    if let Some(vacations2) = &resource2.vacations {
                        for period1 in vacations1 {
                            for period2 in vacations2 {
                                if self.check_vacation_overlap(period1, period2) {
                                    mensagens.push(format!(
                                        "⚠️ Sobreposição detectada: {} e {} têm férias sobrepostas entre {} e {}",
                                        resource1.name,
                                        resource2.name,
                                        period1.start_date.format("%d/%m/%Y"),
                                        period1.end_date.format("%d/%m/%Y")
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if mensagens.is_empty() {
            mensagens.push("✅ Não foram encontradas sobreposições de férias".to_string());
        }

        Ok(mensagens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource::resource::Resource;
    use chrono::{Duration, Local};

    struct MockProjectRepository;
    struct MockResourceRepository {
        resources: Vec<Resource>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: crate::domain::project::project::Project) -> Result<(), DomainError> {
            Ok(())
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: Resource) -> Result<Resource, DomainError> {
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.clone())
        }
    }

    #[test]
    fn test_detect_vacation_overlap() {
        let now = Local::now();
        let resource1 = Resource::new(
            None,
            "João".to_string(),
            None,
            "Dev".to_string(),
            Some(vec![Period {
                start_date: now,
                end_date: now + Duration::days(10),
                approved: true,
                period_type: crate::domain::resource::resource::PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
            }]),
            None,
            0,
        );

        let resource2 = Resource::new(
            None,
            "Maria".to_string(),
            None,
            "Dev".to_string(),
            Some(vec![Period {
                start_date: now + Duration::days(5),
                end_date: now + Duration::days(15),
                approved: true,
                period_type: crate::domain::resource::resource::PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
            }]),
            None,
            0,
        );

        let mock_project_repo = MockProjectRepository;
        let mock_resource_repo = MockResourceRepository {
            resources: vec![resource1, resource2],
        };

        let use_case = ValidateVacationsUseCase::new(mock_project_repo, mock_resource_repo);
        let result = use_case.execute().unwrap();

        assert!(result.iter().any(|msg| msg.contains("Sobreposição detectada")));
    }
} 