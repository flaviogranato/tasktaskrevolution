use crate::domain::{
    project::{project_repository::ProjectRepository, vacation_rules::VacationRules},
    resource::resource::Period,
    resource::resource_repository::ResourceRepository,
    shared_kernel::errors::DomainError,
};
use chrono::{DateTime, Local, NaiveDate, FixedOffset, Offset};
use std::path::Path;

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

    fn check_layoff_overlap(&self, vacation_period: &Period, layoff_period: &(String, String)) -> bool {
        let layoff_start = NaiveDate::parse_from_str(&layoff_period.0, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let layoff_end = NaiveDate::parse_from_str(&layoff_period.1, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        
        let offset = Local::now().offset().fix();
        let layoff_start: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_start, offset);
        let layoff_end: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_end, offset);

        vacation_period.start_date <= layoff_end && layoff_start <= vacation_period.end_date
    }

    fn has_valid_layoff_vacation(&self, vacations: &[Period], vacation_rules: &VacationRules) -> bool {
        if let Some(layoff_periods) = &vacation_rules.layoff_periods {
            if let Some(require_layoff) = vacation_rules.require_layoff_vacation_period {
                if require_layoff {
                    // Verifica se pelo menos uma férias coincide com algum período de layoff
                    for vacation in vacations {
                        for layoff_period in layoff_periods {
                            if self.check_layoff_overlap(vacation, &(layoff_period.start_date.clone(), layoff_period.end_date.clone())) {
                                return true;
                            }
                        }
                    }
                    return false;
                }
            }
        }
        true
    }

    pub fn execute(&self) -> Result<Vec<String>, DomainError> {
        let resources = self.resource_repository.find_all()?;
        let project = self.project_repository.load(&std::path::PathBuf::from("."))?;
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

                // Verificar se há férias durante o período de layoff quando necessário
                if let Some(vacation_rules) = &project.vacation_rules {
                    if !self.has_valid_layoff_vacation(vacations1, vacation_rules) {
                        mensagens.push(format!(
                            "⚠️ {} não possui férias durante nenhum período de layoff",
                            resource1.name
                        ));
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
    use crate::domain::{
        project::{layoff_period::LayoffPeriod, vacation_rules::VacationRules},
        resource::resource::{PeriodType, Resource},
    };
    use chrono::{Duration, Local};
    use std::path::Path;

    struct MockProjectRepository {
        vacation_rules: Option<VacationRules>,
    }

    struct MockResourceRepository {
        resources: Vec<Resource>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(
            &self,
            _project: crate::domain::project::project::Project,
        ) -> Result<(), DomainError> {
            Ok(())
        }

        fn load(&self, _path: &Path) -> Result<crate::domain::project::project::Project, DomainError> {
            Ok(crate::domain::project::project::Project {
                id: None,
                name: "Test Project".to_string(),
                description: None,
                start_date: None,
                end_date: None,
                status: crate::domain::project::project::ProjectStatus::InProgress,
                vacation_rules: self.vacation_rules.clone(),
            })
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: Resource) -> Result<Resource, DomainError> {
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.clone())
        }

        fn save_time_off(&self, _resource_name: String, _hours: u32, _date: String, _description: Option<String>) -> Result<Resource, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn save_vacation(&self, _resource_name: String, _start_date: String, _end_date: String, _is_time_off_compensation: bool, _compensated_hours: Option<u32>) -> Result<Resource, DomainError> {
            unimplemented!("Not needed for these tests")
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
                period_type: PeriodType::Vacation,
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
                period_type: PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
            }]),
            None,
            0,
        );

        let mock_project_repo = MockProjectRepository {
            vacation_rules: None,
        };
        let mock_resource_repo = MockResourceRepository {
            resources: vec![resource1, resource2],
        };

        let use_case = ValidateVacationsUseCase::new(mock_project_repo, mock_resource_repo);
        let result = use_case.execute().unwrap();

        assert!(result
            .iter()
            .any(|msg| msg.contains("Sobreposição detectada")));
    }

    #[test]
    fn test_layoff_vacation_validation() {
        let now = Local::now();
        let layoff_start = now.format("%Y-%m-%d").to_string();
        let layoff_end = (now + Duration::days(30)).format("%Y-%m-%d").to_string();

        let vacation_rules = VacationRules::new(
            None,
            Some(true),
            Some(true),
            Some(vec![LayoffPeriod::new(layoff_start.clone(), layoff_end.clone())]),
        );

        // Recurso sem férias durante o layoff
        let resource1 = Resource::new(
            None,
            "João".to_string(),
            None,
            "Dev".to_string(),
            Some(vec![Period {
                start_date: now + Duration::days(40),
                end_date: now + Duration::days(50),
                approved: true,
                period_type: PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
            }]),
            None,
            0,
        );

        // Recurso com férias durante o layoff
        let resource2 = Resource::new(
            None,
            "Maria".to_string(),
            None,
            "Dev".to_string(),
            Some(vec![Period {
                start_date: now + Duration::days(5),
                end_date: now + Duration::days(15),
                approved: true,
                period_type: PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
            }]),
            None,
            0,
        );

        let mock_project_repo = MockProjectRepository {
            vacation_rules: Some(vacation_rules),
        };
        let mock_resource_repo = MockResourceRepository {
            resources: vec![resource1, resource2],
        };

        let use_case = ValidateVacationsUseCase::new(mock_project_repo, mock_resource_repo);
        let result = use_case.execute().unwrap();

        assert!(result
            .iter()
            .any(|msg| msg.contains("não possui férias durante nenhum período de layoff")));
        assert!(!result
            .iter()
            .any(|msg| msg.contains("Maria") && msg.contains("layoff")));
    }
}
