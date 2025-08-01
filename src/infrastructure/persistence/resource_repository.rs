use crate::{
    domain::{
        project_management::repository::ProjectRepository,
        resource_management::{AnyResource, Period, PeriodType, repository::ResourceRepository},
        shared::errors::DomainError,
    },
    infrastructure::persistence::{
        manifests::resource_manifest::ResourceManifest, project_repository::FileProjectRepository,
    },
};
use chrono::{DateTime, Local, NaiveDate, Offset};
use glob::glob;
use serde_yaml;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct FileResourceRepository {
    base_path: PathBuf,
}

impl FileResourceRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn get_resource_file_path(&self, resource_name: &str) -> PathBuf {
        self.base_path
            .join("resources")
            .join(format!("{}.yaml", resource_name.replace(' ', "_").to_lowercase()))
    }

    fn find_by_name(&self, resource_name: &str) -> Result<Option<AnyResource>, DomainError> {
        let file_path = self.get_resource_file_path(resource_name);
        if !file_path.exists() {
            return Ok(None);
        }
        let yaml =
            fs::read_to_string(&file_path).map_err(|e| DomainError::Io(format!("Error reading resource file: {e}")))?;
        let manifest: ResourceManifest = serde_yaml::from_str(&yaml)
            .map_err(|e| DomainError::Serialization(format!("Error deserializing resource: {e}")))?;
        let resource = AnyResource::try_from(manifest).map_err(DomainError::Serialization)?;
        Ok(Some(resource))
    }
}

impl ResourceRepository for FileResourceRepository {
    fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError> {
        let file_path = self.get_resource_file_path(resource.name());
        let resource_manifest = ResourceManifest::from(resource.clone());
        let yaml = serde_yaml::to_string(&resource_manifest)
            .map_err(|e| DomainError::Serialization(format!("Error serializing resource: {e}")))?;

        fs::create_dir_all(file_path.parent().unwrap())
            .map_err(|e| DomainError::Io(format!("Error creating directory: {e}")))?;

        fs::write(file_path, yaml).map_err(|e| DomainError::Io(format!("Error saving resource: {e}")))?;

        Ok(resource)
    }

    fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
        let pattern = self.base_path.join("**/resources/**/*.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;
        let mut resources = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|e| DomainError::Generic(e.to_string()))?;
            let file_path = entry.as_path();
            let yaml = fs::read_to_string(file_path)
                .map_err(|e| DomainError::Io(format!("Error reading resource file: {e}")))?;

            let resource_manifest: ResourceManifest = serde_yaml::from_str(&yaml)
                .map_err(|e| DomainError::Serialization(format!("Error deserializing resource: {e}")))?;

            resources.push(AnyResource::try_from(resource_manifest).map_err(DomainError::Serialization)?);
        }

        Ok(resources)
    }

    fn save_time_off(
        &self,
        resource_name: String,
        hours: u32,
        _date: String,
        _description: Option<String>,
    ) -> Result<AnyResource, DomainError> {
        let resource = self
            .find_by_name(&resource_name)?
            .ok_or_else(|| DomainError::NotFound("Resource not found".to_string()))?;

        let updated_resource = match resource {
            AnyResource::Available(mut r) => {
                r.time_off_balance += hours;
                AnyResource::Available(r)
            }
            AnyResource::Assigned(mut r) => {
                r.time_off_balance += hours;
                AnyResource::Assigned(r)
            }
            AnyResource::Inactive(mut r) => {
                r.time_off_balance += hours;
                AnyResource::Inactive(r)
            }
        };
        self.save(updated_resource)
    }

    fn save_vacation(
        &self,
        resource_name: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<AnyResource, DomainError> {
        let resource = self
            .find_by_name(&resource_name)?
            .ok_or_else(|| DomainError::NotFound("Resource not found".to_string()))?;

        let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| DomainError::Generic(format!("Invalid start date: {e}")))?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
            .map_err(|e| DomainError::Generic(format!("Invalid end date: {e}")))?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        if end_date < start_date {
            return Err(DomainError::Generic("End date must be after start date".to_string()));
        }

        let offset = Local::now().offset().fix();
        let start_date: DateTime<Local> = DateTime::from_naive_utc_and_offset(start_date, offset);
        let end_date: DateTime<Local> = DateTime::from_naive_utc_and_offset(end_date, offset);

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

        let add_vacation = |vacations: Option<Vec<Period>>| -> Option<Vec<Period>> {
            let mut v = vacations.unwrap_or_default();
            v.push(new_vacation);
            Some(v)
        };

        let updated_resource = match resource {
            AnyResource::Available(mut r) => {
                r.vacations = add_vacation(r.vacations);
                AnyResource::Available(r)
            }
            AnyResource::Assigned(mut r) => {
                r.vacations = add_vacation(r.vacations);
                AnyResource::Assigned(r)
            }
            AnyResource::Inactive(_) => {
                return Err(DomainError::InvalidState(
                    "Cannot add vacation to inactive resource".to_string(),
                ));
            }
        };

        self.save(updated_resource)
    }

    fn check_if_layoff_period(&self, start_date: &DateTime<Local>, end_date: &DateTime<Local>) -> bool {
        let project_repo = FileProjectRepository::new();

        if let Ok(project) = project_repo.load() {
            if let Some(vacation_rules) = project.vacation_rules() {
                if let Some(layoff_periods) = &vacation_rules.layoff_periods {
                    for layoff_period in layoff_periods {
                        if let (Ok(layoff_start), Ok(layoff_end)) = (
                            chrono::NaiveDate::parse_from_str(&layoff_period.start_date, "%Y-%m-%d")
                                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                                .map(|dt| DateTime::<Local>::from_naive_utc_and_offset(dt, *start_date.offset())),
                            chrono::NaiveDate::parse_from_str(&layoff_period.end_date, "%Y-%m-%d")
                                .map(|d| d.and_hms_opt(23, 59, 59).unwrap())
                                .map(|dt| DateTime::<Local>::from_naive_utc_and_offset(dt, *end_date.offset())),
                        ) {
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

    fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError> {
        let all_resources = self.find_all()?;
        let prefix = resource_type.to_lowercase();
        let prefix_with_dash = format!("{prefix}-");

        let max_num = all_resources
            .iter()
            .filter_map(|r| match r {
                AnyResource::Available(res) => Some((&res.code, &res.resource_type)),
                AnyResource::Assigned(res) => Some((&res.code, &res.resource_type)),
                AnyResource::Inactive(res) => Some((&res.code, &res.resource_type)),
            })
            .filter(|(_, r_type)| r_type.to_lowercase() == prefix)
            .filter_map(|(code, _)| code.strip_prefix(&prefix_with_dash))
            .filter_map(|num_str| num_str.parse::<u32>().ok())
            .max()
            .unwrap_or(0);

        Ok(format!("{}{}", prefix_with_dash, max_num + 1))
    }
}

impl Default for FileResourceRepository {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::resource::Resource;
    use crate::domain::resource_management::state::Available;
    use tempfile::tempdir;

    fn create_test_resource(name: &str, code: &str, resource_type: &str) -> Resource<Available> {
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            resource_type.to_string(),
            None,
            0,
        )
    }

    #[test]
    fn test_save_and_find_all() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource1 = create_test_resource("test1", "dev-1", "dev");
        let resource2 = create_test_resource("test2", "dev-2", "dev");

        repo.save(resource1.clone().into()).unwrap();
        repo.save(resource2.clone().into()).unwrap();

        let resources = repo.find_all().unwrap();
        assert_eq!(resources.len(), 2);
        assert!(resources.iter().any(|r| r.name() == "test1"));
        assert!(resources.iter().any(|r| r.name() == "test2"));
    }

    #[test]
    fn test_save_vacation() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("test", "dev-1", "dev");
        repo.save(resource.into()).unwrap();

        let result = repo.save_vacation(
            "test".to_string(),
            "2024-01-01".to_string(),
            "2024-01-31".to_string(),
            false,
            None,
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();

        let vacations = match updated_resource {
            AnyResource::Available(r) => r.vacations,
            AnyResource::Assigned(r) => r.vacations,
            AnyResource::Inactive(_) => None,
        };
        assert_eq!(vacations.unwrap().len(), 1);
    }

    #[test]
    fn test_save_time_off() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("test", "dev-1", "dev");
        repo.save(resource.into()).unwrap();

        let result = repo.save_time_off(
            "test".to_string(),
            10,
            "2024-01-01".to_string(),
            Some("Test time off".to_string()),
        );

        assert!(result.is_ok());
        let updated_resource = result.unwrap();
        let balance = match updated_resource {
            AnyResource::Available(r) => r.time_off_balance,
            AnyResource::Assigned(r) => r.time_off_balance,
            AnyResource::Inactive(r) => r.time_off_balance,
        };
        assert_eq!(balance, 10);
    }

    #[test]
    fn test_get_next_code() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        // Test with no resources of a type
        assert_eq!(repo.get_next_code("dev").unwrap(), "dev-1");

        // Add some resources
        repo.save(create_test_resource("res1", "dev-1", "dev").into()).unwrap();
        repo.save(create_test_resource("res2", "qa-1", "qa").into()).unwrap();
        repo.save(create_test_resource("res3", "dev-2", "dev").into()).unwrap();
        repo.save(create_test_resource("res4", "dev-5", "dev").into()) // Test with a gap
            .unwrap();

        // Test again for both types
        assert_eq!(repo.get_next_code("dev").unwrap(), "dev-6");
        assert_eq!(repo.get_next_code("qa").unwrap(), "qa-2");
        assert_eq!(repo.get_next_code("manager").unwrap(), "manager-1"); // Test new type
    }
}
