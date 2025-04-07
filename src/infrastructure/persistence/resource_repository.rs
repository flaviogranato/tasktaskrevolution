use crate::domain::{
    project::repository::ProjectRepository,
    resource::{
        model::{Period, PeriodType, Resource},
        repository::ResourceRepository,
    },
    shared_kernel::{convertable::Convertable, errors::DomainError},
};
use crate::infrastructure::persistence::manifests::resource_manifest::ResourceManifest;
use chrono::{DateTime, Local, NaiveDate, Offset};
use serde_yaml;
use std::fs;
use std::path::PathBuf;

pub struct FileResourceRepository {
    base_path: PathBuf,
}

impl FileResourceRepository {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    fn get_resource_file_path(&self, resource_name: &str) -> PathBuf {
        self.base_path
            .join("resources")
            .join(format!("{}.yaml", resource_name))
    }
}

impl ResourceRepository for FileResourceRepository {
    fn save(&self, resource: Resource) -> Result<Resource, DomainError> {
        let file_path = self.get_resource_file_path(&resource.name);
        let resource_manifest = <ResourceManifest as Convertable<Resource>>::from(resource.clone());
        let yaml = serde_yaml::to_string(&resource_manifest)
            .map_err(|e| DomainError::Generic(format!("Erro ao serializar recurso: {}", e)))?;

        fs::create_dir_all(file_path.parent().unwrap())
            .map_err(|e| DomainError::Generic(format!("Erro ao criar diretório: {}", e)))?;

        fs::write(file_path, yaml)
            .map_err(|e| DomainError::Generic(format!("Erro ao salvar recurso: {}", e)))?;

        Ok(resource)
    }

    fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
        let resources_dir = self.base_path.join("resources");
        if !resources_dir.exists() {
            return Ok(Vec::new());
        }

        let mut resources = Vec::new();
        for entry in fs::read_dir(resources_dir).map_err(|e| {
            DomainError::Generic(format!("Erro ao ler diretório de recursos: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                DomainError::Generic(format!("Erro ao ler entrada do diretório: {}", e))
            })?;

            if entry
                .file_type()
                .map_err(|e| DomainError::Generic(format!("Erro ao obter tipo do arquivo: {}", e)))?
                .is_file()
            {
                let file_path = entry.path();
                if file_path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    let yaml = fs::read_to_string(&file_path).map_err(|e| {
                        DomainError::Generic(format!("Erro ao ler arquivo de recurso: {}", e))
                    })?;

                    let resource_manifest: ResourceManifest =
                        serde_yaml::from_str(&yaml).map_err(|e| {
                            DomainError::Generic(format!("Erro ao deserializar recurso: {}", e))
                        })?;

                    resources.push(<ResourceManifest as Convertable<Resource>>::to(
                        &resource_manifest,
                    ));
                }
            }
        }

        Ok(resources)
    }

    fn save_time_off(
        &self,
        resource_name: String,
        hours: u32,
        _date: String,
        _description: Option<String>,
    ) -> Result<Resource, DomainError> {
        let mut resources = self.find_all()?;
        let resource = resources
            .iter_mut()
            .find(|r| r.name == resource_name)
            .ok_or_else(|| DomainError::Generic("Recurso não encontrado".to_string()))?;

        resource.time_off_balance += hours;
        self.save(resource.clone())?;
        Ok(resource.clone())
    }

    fn save_vacation(
        &self,
        resource_name: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<Resource, DomainError> {
        let mut resources = self.find_all()?;
        let resource = resources
            .iter_mut()
            .find(|r| r.name == resource_name)
            .ok_or_else(|| DomainError::Generic("Recurso não encontrado".to_string()))?;

        let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| DomainError::Generic(format!("Data de início inválida: {}", e)))?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
            .map_err(|e| DomainError::Generic(format!("Data de fim inválida: {}", e)))?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        if end_date < start_date {
            return Err(DomainError::Generic(
                "A data de fim deve ser posterior à data de início".to_string(),
            ));
        }

        let offset = Local::now().offset().fix();
        let start_date: DateTime<Local> = DateTime::from_naive_utc_and_offset(start_date, offset);
        let end_date: DateTime<Local> = DateTime::from_naive_utc_and_offset(end_date, offset);

        // Verificar se o período coincide com algum período de layoff
        let is_layoff = self.check_if_layoff_period(&start_date, &end_date);

        let new_vacation = Period {
            start_date,
            end_date,
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation,
            compensated_hours,
            is_layoff,
        };

        let mut vacations = resource.vacations.clone().unwrap_or_default();
        vacations.push(new_vacation);
        resource.vacations = Some(vacations);

        self.save(resource.clone())?;
        Ok(resource.clone())
    }

    fn check_if_layoff_period(
        &self,
        start_date: &DateTime<Local>,
        end_date: &DateTime<Local>,
    ) -> bool {
        use crate::infrastructure::persistence::project_repository::FileProjectRepository;
        use std::path::PathBuf;

        let project_repo = FileProjectRepository::new();

        if let Ok(project) = project_repo.load(&PathBuf::from(".")) {
            if let Some(vacation_rules) = project.vacation_rules {
                if let Some(layoff_periods) = vacation_rules.layoff_periods {
                    for layoff_period in layoff_periods {
                        // Converter as datas de string para DateTime
                        if let (Ok(layoff_start), Ok(layoff_end)) = (
                            chrono::NaiveDate::parse_from_str(
                                &layoff_period.start_date,
                                "%Y-%m-%d",
                            )
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                            .map(|dt| {
                                DateTime::<Local>::from_naive_utc_and_offset(
                                    dt,
                                    *start_date.offset(),
                                )
                            }),
                            chrono::NaiveDate::parse_from_str(&layoff_period.end_date, "%Y-%m-%d")
                                .map(|d| d.and_hms_opt(23, 59, 59).unwrap())
                                .map(|dt| {
                                    DateTime::<Local>::from_naive_utc_and_offset(
                                        dt,
                                        *end_date.offset(),
                                    )
                                }),
                        ) {
                            // Verificar se os períodos se sobrepõem
                            if start_date <= &layoff_end && end_date >= &layoff_start {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }
}

impl Default for FileResourceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource::model::Resource;
    use tempfile::tempdir;

    fn create_test_resource(name: &str) -> Resource {
        Resource::new(
            Some(name.to_string()),
            name.to_string(),
            None,
            "dev".to_string(),
            None,
            None,
            0,
        )
    }

    #[test]
    fn test_save_and_find_all() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let resource1 = create_test_resource("test1");
        let resource2 = create_test_resource("test2");

        repo.save(resource1.clone()).unwrap();
        repo.save(resource2.clone()).unwrap();

        let resources = repo.find_all().unwrap();
        assert_eq!(resources.len(), 2);
        assert!(resources.iter().any(|r| r.name == "test1"));
        assert!(resources.iter().any(|r| r.name == "test2"));
    }

    #[test]
    fn test_save_vacation() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let resource = create_test_resource("test");
        repo.save(resource).unwrap();

        let result = repo.save_vacation(
            "test".to_string(),
            "2024-01-01".to_string(),
            "2024-01-31".to_string(),
            false,
            None,
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.vacations.unwrap().len(), 1);
    }

    #[test]
    fn test_save_time_off() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let resource = create_test_resource("test");
        repo.save(resource).unwrap();

        let result = repo.save_time_off(
            "test".to_string(),
            10,
            "2024-01-01".to_string(),
            Some("Test time off".to_string()),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        assert_eq!(updated_resource.time_off_balance, 10);
    }

    #[test]
    fn test_save_vacation_with_compensation() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let resource = create_test_resource("test");
        repo.save(resource).unwrap();

        let result = repo.save_vacation(
            "test".to_string(),
            "2024-01-01".to_string(),
            "2024-01-31".to_string(),
            true,
            Some(10),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        let vacation = &updated_resource.vacations.unwrap()[0];
        assert!(vacation.is_time_off_compensation);
        assert_eq!(vacation.compensated_hours, Some(10));
        assert!(!vacation.is_layoff);
    }

    #[test]
    fn test_save_vacation_invalid_dates() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let resource = create_test_resource("test");
        repo.save(resource).unwrap();

        let result = repo.save_vacation(
            "test".to_string(),
            "2024-01-31".to_string(),
            "2024-01-01".to_string(),
            false,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_save_vacation_nonexistent_resource() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let result = repo.save_vacation(
            "nonexistent".to_string(),
            "2024-01-01".to_string(),
            "2024-01-31".to_string(),
            false,
            None,
        );

        assert!(result.is_err());
    }   
}
