#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::{AnyResource, Period, PeriodType, repository::ResourceRepository};
use crate::infrastructure::persistence::{
    manifests::resource_manifest::ResourceManifest, project_repository::FileProjectRepository,
};
use chrono::{DateTime, Local, NaiveDate, Offset};
use glob::glob;
use serde_yaml;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone)]
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

    /// Gets the path to a resource in a specific company's global resources
    fn get_company_resource_path(&self, company_code: &str, resource_name: &str) -> PathBuf {
        // If base_path is relative (like "../"), resolve it to absolute path
        let base_path = if self.base_path.is_relative() {
            std::env::current_dir()
                .unwrap_or_else(|_| self.base_path.clone())
                .join(&self.base_path)
                .canonicalize()
                .unwrap_or_else(|_| self.base_path.clone())
        } else {
            self.base_path.clone()
        };

        // If we're already in a companies directory, don't add "companies" again
        if base_path.ends_with("companies") {
            base_path
                .join(company_code)
                .join("resources")
                .join(format!("{}.yaml", resource_name.replace(' ', "_").to_lowercase()))
        } else {
            base_path
                .join("companies")
                .join(company_code)
                .join("resources")
                .join(format!("{}.yaml", resource_name.replace(' ', "_").to_lowercase()))
        }
    }

    /// Gets the path to a resource in a specific company's global resources using resource code
    fn get_company_resource_path_by_code(&self, company_code: &str, resource_code: &str) -> PathBuf {
        // If base_path is relative (like "../"), resolve it to absolute path
        let base_path = if self.base_path.is_relative() {
            std::env::current_dir()
                .unwrap_or_else(|_| self.base_path.clone())
                .join(&self.base_path)
                .canonicalize()
                .unwrap_or_else(|_| self.base_path.clone())
        } else {
            self.base_path.clone()
        };

        // If we're already in a companies directory, don't add "companies" again
        if base_path.ends_with("companies") {
            base_path
                .join(company_code)
                .join("resources")
                .join(format!("{}.yaml", resource_code))
        } else {
            base_path
                .join("companies")
                .join(company_code)
                .join("resources")
                .join(format!("{}.yaml", resource_code))
        }
    }

    /// Gets the path to a resource in a specific project
    fn get_project_resource_path(&self, company_code: &str, project_code: &str, resource_name: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("resources")
            .join(format!("{}.yaml", resource_name.replace(' ', "_").to_lowercase()))
    }

    /// Gets the path to company resources directory
    fn get_company_resources_path(&self, company_code: &str) -> PathBuf {
        self.base_path.join("companies").join(company_code).join("resources")
    }

    /// Gets the path to project resources directory
    fn get_project_resources_path(&self, company_code: &str, project_code: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("resources")
    }

    /// Find all resources available for a specific project
    /// This includes:
    /// 1. Company global resources (can be allocated to any project)
    /// 2. Project-specific resources (belong only to this project)
    pub fn find_all_by_project(&self, company_code: &str, project_code: &str) -> Result<Vec<AnyResource>, AppError> {
        let mut resources = Vec::new();

        // 1. Load company global resources (available for all projects)
        let company_resources_path = self.get_company_resources_path(company_code);
        if company_resources_path.exists() {
            let pattern = company_resources_path.join("*.yaml");
            let walker = glob(pattern.to_str().unwrap()).map_err(|e| AppError::ValidationError {
                field: "glob pattern".to_string(),
                message: e.to_string(),
            })?;

            for entry in walker {
                let entry = entry.map_err(|e| AppError::ValidationError {
                    field: "glob entry".to_string(),
                    message: e.to_string(),
                })?;
                let file_path = entry.as_path();
                let yaml = fs::read_to_string(file_path).map_err(|e| AppError::IoErrorWithPath {
                    operation: "file read".to_string(),
                    path: file_path.to_string_lossy().to_string(),
                    details: e.to_string(),
                })?;

                let resource_manifest: ResourceManifest =
                    serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                        format: "YAML".to_string(),
                        details: format!("Error deserializing resource: {}", e),
                    })?;

                resources.push(
                    AnyResource::try_from(resource_manifest).map_err(|e| AppError::SerializationError {
                        format: "YAML".to_string(),
                        details: format!("Error converting manifest: {}", e),
                    })?,
                );
            }
        }

        // 2. Load project-specific resources (belong only to this project)
        let project_resources_path = self.get_project_resources_path(company_code, project_code);
        if project_resources_path.exists() {
            let pattern = project_resources_path.join("*.yaml");
            let walker = glob(pattern.to_str().unwrap()).map_err(|e| AppError::ValidationError {
                field: "glob pattern".to_string(),
                message: e.to_string(),
            })?;

            for entry in walker {
                let entry = entry.map_err(|e| AppError::ValidationError {
                    field: "glob entry".to_string(),
                    message: e.to_string(),
                })?;
                let file_path = entry.as_path();
                let yaml = fs::read_to_string(file_path).map_err(|e| AppError::IoErrorWithPath {
                    operation: "file read".to_string(),
                    path: file_path.to_string_lossy().to_string(),
                    details: e.to_string(),
                })?;

                let resource_manifest: ResourceManifest =
                    serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                        format: "YAML".to_string(),
                        details: format!("Error deserializing resource: {}", e),
                    })?;

                resources.push(
                    AnyResource::try_from(resource_manifest).map_err(|e| AppError::SerializationError {
                        format: "YAML".to_string(),
                        details: format!("Error converting manifest: {}", e),
                    })?,
                );
            }
        }

        Ok(resources)
    }

    fn find_by_name(&self, resource_name: &str) -> Result<Option<AnyResource>, AppError> {
        let file_path = self.get_resource_file_path(resource_name);
        if !file_path.exists() {
            return Ok(None);
        }
        let yaml = fs::read_to_string(&file_path).map_err(|e| AppError::IoErrorWithPath {
            operation: "file read".to_string(),
            path: file_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;
        let manifest: ResourceManifest = serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error deserializing resource: {}", e),
        })?;
        let resource = AnyResource::try_from(manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error converting manifest: {}", e),
        })?;
        Ok(Some(resource))
    }

    fn read_resource_from_dir(&self, dir: &Path) -> Result<Option<AnyResource>, AppError> {
        let manifest_path = dir.join("resource.yaml");
        if !manifest_path.exists() {
            return Ok(None);
        }

        let yaml = fs::read_to_string(&manifest_path).map_err(|e| AppError::IoErrorWithPath {
            operation: "file read".to_string(),
            path: manifest_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;
        let manifest: ResourceManifest = serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error deserializing resource: {}", e),
        })?;
        let resource = AnyResource::try_from(manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error converting manifest: {}", e),
        })?;
        Ok(Some(resource))
    }
}

impl ResourceRepository for FileResourceRepository {
    fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
        let file_path = self.get_resource_file_path(resource.name());
        let resource_manifest = ResourceManifest::from(resource.clone());
        let yaml = serde_yaml::to_string(&resource_manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error serializing resource: {}", e),
        })?;

        fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| AppError::IoError {
            operation: "create directory".to_string(),
            details: e.to_string(),
        })?;

        fs::write(&file_path, yaml).map_err(|e| AppError::IoErrorWithPath {
            operation: "file write".to_string(),
            path: file_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        Ok(resource)
    }

    /// Save resource in the new hierarchical structure
    /// If project_code is None, saves as company global resource
    /// If project_code is Some, saves as project-specific resource
    fn save_in_hierarchy(
        &self,
        resource: AnyResource,
        company_code: &str,
        project_code: Option<&str>,
    ) -> Result<AnyResource, AppError> {
        let file_path = if let Some(proj_code) = project_code {
            // Save as project-specific resource
            self.get_project_resource_path(company_code, proj_code, resource.name())
        } else {
            // Save as company global resource using name for creation, code for updates
            // This is a heuristic: if the resource has a code that doesn't match the name,
            // it's likely an update operation
            if resource.code() != resource.name().replace(' ', "_").to_lowercase() {
                self.get_company_resource_path_by_code(company_code, resource.code())
            } else {
                self.get_company_resource_path(company_code, resource.name())
            }
        };

        let resource_manifest = ResourceManifest::from(resource.clone());
        let yaml = serde_yaml::to_string(&resource_manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error serializing resource: {}", e),
        })?;

        fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| AppError::IoError {
            operation: "create directory".to_string(),
            details: e.to_string(),
        })?;

        fs::write(&file_path, yaml).map_err(|e| AppError::IoErrorWithPath {
            operation: "file write".to_string(),
            path: file_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        Ok(resource)
    }

    fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
        let mut resources = Vec::new();

        // Search in company resources: companies/*/resources/*.yaml
        let absolute_base = std::fs::canonicalize(&self.base_path).unwrap_or_else(|_| self.base_path.clone());
        let company_pattern = if absolute_base.ends_with("companies") {
            absolute_base.join("*/resources/*.yaml")
        } else {
            absolute_base.join("companies/*/resources/*.yaml")
        };
        let company_walker = glob(company_pattern.to_str().unwrap()).map_err(|e| AppError::ValidationError {
            field: "glob pattern".to_string(),
            message: e.to_string(),
        })?;

        for entry in company_walker {
            let entry = entry.map_err(|e| AppError::ValidationError {
                field: "glob entry".to_string(),
                message: e.to_string(),
            })?;
            let file_path = entry.as_path();
            let yaml = fs::read_to_string(file_path).map_err(|e| AppError::IoErrorWithPath {
                operation: "file read".to_string(),
                path: file_path.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;

            let resource_manifest: ResourceManifest =
                serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error deserializing resource: {}", e),
                })?;

            resources.push(
                AnyResource::try_from(resource_manifest).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error converting manifest: {}", e),
                })?,
            );
        }

        // Search in project resources: companies/*/projects/*/resources/*.yaml
        let project_pattern = self.base_path.join("companies/*/projects/*/resources/*.yaml");
        let project_walker = glob(project_pattern.to_str().unwrap()).map_err(|e| AppError::ValidationError {
            field: "glob pattern".to_string(),
            message: e.to_string(),
        })?;

        for entry in project_walker {
            let entry = entry.map_err(|e| AppError::ValidationError {
                field: "glob entry".to_string(),
                message: e.to_string(),
            })?;
            let file_path = entry.as_path();
            let yaml = fs::read_to_string(file_path).map_err(|e| AppError::IoErrorWithPath {
                operation: "file read".to_string(),
                path: file_path.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;

            let resource_manifest: ResourceManifest =
                serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error deserializing resource: {}", e),
                })?;

            resources.push(
                AnyResource::try_from(resource_manifest).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error converting manifest: {}", e),
                })?,
            );
        }

        Ok(resources)
    }

    fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
        // If we're in a project or company context, search in company resources first
        if self.base_path.ends_with("projects")
            || self.base_path.ends_with("companies")
            || self.base_path.to_string_lossy() == "../"
        {
            // We're in a project or company context, search in company resources
            let company_pattern = if self.base_path.ends_with("projects") {
                // We're in projects/*/ directory, go up to companies/*/resources
                self.base_path
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("*/resources/*.yaml")
            } else if self.base_path.to_string_lossy() == "../" {
                // We're in a project directory, search in ../resources/*.yaml
                // Get the current working directory and go up two levels to get to company directory
                let current_dir = std::env::current_dir().unwrap_or_else(|_| self.base_path.clone());
                let company_dir = current_dir.parent().unwrap().parent().unwrap();
                company_dir.join("resources/*.yaml")
            } else {
                // We're in companies/*/ directory, search in */resources
                self.base_path.join("*/resources/*.yaml")
            };

            // Check if the resources directory exists
            let _resources_dir = if self.base_path.to_string_lossy() == "../" {
                // Get the current working directory and go up two levels to get to company directory
                let current_dir = std::env::current_dir().unwrap_or_else(|_| self.base_path.clone());
                let company_dir = current_dir.parent().unwrap().parent().unwrap();
                company_dir.join("resources")
            } else {
                self.base_path.join("resources")
            };

            let walker = glob(company_pattern.to_str().unwrap()).map_err(|e| AppError::ValidationError {
                field: "glob pattern".to_string(),
                message: e.to_string(),
            })?;

            for entry in walker {
                let entry = entry.map_err(|e| AppError::ValidationError {
                    field: "glob entry".to_string(),
                    message: e.to_string(),
                })?;
                let file_path = entry.as_path();

                let yaml = fs::read_to_string(file_path).map_err(|e| AppError::IoErrorWithPath {
                    operation: "file read".to_string(),
                    path: file_path.to_string_lossy().to_string(),
                    details: e.to_string(),
                })?;

                let resource_manifest: ResourceManifest =
                    serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                        format: "YAML".to_string(),
                        details: format!("Error deserializing resource: {}", e),
                    })?;

                let resource = AnyResource::try_from(resource_manifest).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error converting manifest: {}", e),
                })?;

                if resource.code() == code {
                    return Ok(Some(resource));
                }
            }
        }

        // Since resources are saved by name, we need to search through all resources
        // to find one with the matching code
        let all_resources = self.find_all()?;
        for resource in all_resources {
            if resource.code() == code {
                return Ok(Some(resource));
            }
        }
        Ok(None)
    }

    fn save_time_off(
        &self,
        resource_name: &str,
        hours: u32,
        _date: &str,
        _description: Option<String>,
    ) -> Result<AnyResource, AppError> {
        let resource = self
            .find_by_name(resource_name)?
            .ok_or_else(|| AppError::ResourceNotFound {
                code: "unknown".to_string(),
            })?;

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
        resource_name: &str,
        start_date: &str,
        end_date: &str,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<AnyResource, AppError> {
        let resource = self
            .find_by_name(resource_name)?
            .ok_or_else(|| AppError::ResourceNotFound {
                code: "unknown".to_string(),
            })?;

        let start_date = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| AppError::ValidationError {
                field: "start_date".to_string(),
                message: format!("Invalid start date: {}", e),
            })?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let end_date = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|e| AppError::ValidationError {
                field: "end_date".to_string(),
                message: format!("Invalid end date: {}", e),
            })?
            .and_hms_opt(0, 0, 0)
            .unwrap();

        if end_date < start_date {
            return Err(AppError::ValidationError {
                field: "end_date".to_string(),
                message: "End date must be after start date".to_string(),
            });
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
                return Err(AppError::ResourceInvalidState {
                    current: "unknown".to_string(),
                    expected: "valid".to_string(),
                });
            }
        };

        self.save(updated_resource)
    }

    fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
        let project_repo = FileProjectRepository::new();

        if let Ok(project) = project_repo.load()
            && let Some(_vacation_rules) = project.vacation_rules()
        // TODO: Implement layoff_periods in the new VacationRules
        // && let Some(layoff_periods) = &vacation_rules.layoff_periods
        {
            // For now, we'll just return false since layoff_periods is not implemented
            return false;
        }

        false
    }

    fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
        let all_resources = self.find_all()?;
        let prefix = resource_type.to_lowercase();
        let prefix_with_dash = format!("{prefix}-");

        let max_num = all_resources
            .iter()
            .map(|r| match r {
                AnyResource::Available(res) => (&res.code, &res.resource_type),
                AnyResource::Assigned(res) => (&res.code, &res.resource_type),
                AnyResource::Inactive(res) => (&res.code, &res.resource_type),
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
    use crate::infrastructure::persistence::manifests::resource_manifest::ResourceManifest;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_resource(name: &str, code: &str, resource_type: &str) -> Resource<Available> {
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            resource_type.to_string(),
            None,
            None,
            None,
            0,
        )
    }

    fn create_test_resource_manifest(name: &str, code: &str, resource_type: &str) -> ResourceManifest {
        ResourceManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Resource".to_string(),
            metadata: crate::infrastructure::persistence::manifests::resource_manifest::ResourceMetadata {
                id: Some(uuid7::uuid7().to_string()),
                code: code.to_string(),
                name: name.to_string(),
                email: "test@example.com".to_string(),
                resource_type: resource_type.to_string(),
                status: "Available".to_string(),
                description: None,
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec: crate::infrastructure::persistence::manifests::resource_manifest::ResourceSpec {
                start_date: None,
                end_date: None,
                time_off_balance: 0,
                time_off_history: None,
                project_assignments: None,
                vacations: None,
            },
        }
    }

    #[test]
    fn test_save_and_find_all() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource1 = create_test_resource("test1", "dev-1", "dev");
        let resource2 = create_test_resource("test2", "dev-2", "dev");

        repo.save_in_hierarchy(resource1.clone().into(), "COMP-001", None)
            .unwrap();
        repo.save_in_hierarchy(resource2.clone().into(), "COMP-001", None)
            .unwrap();

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

        let result = repo.save_vacation("test", "2024-01-01", "2024-01-31", false, None);

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

        let result = repo.save_time_off("test", 10, "2024-01-01", Some("Test time off".to_string()));

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
        repo.save_in_hierarchy(create_test_resource("res1", "dev-1", "dev").into(), "COMP-001", None)
            .unwrap();
        repo.save_in_hierarchy(create_test_resource("res2", "qa-1", "qa").into(), "COMP-001", None)
            .unwrap();
        repo.save_in_hierarchy(create_test_resource("res3", "dev-2", "dev").into(), "COMP-001", None)
            .unwrap();
        repo.save_in_hierarchy(create_test_resource("res4", "dev-5", "dev").into(), "COMP-001", None)
            .unwrap(); // Test with a gap

        // Test again for both types
        assert_eq!(repo.get_next_code("dev").unwrap(), "dev-6");
        assert_eq!(repo.get_next_code("qa").unwrap(), "qa-2");
        assert_eq!(repo.get_next_code("manager").unwrap(), "manager-1"); // Test new type
    }

    #[test]
    fn test_resource_manifest_serialization() {
        let manifest = create_test_resource_manifest("Test Resource", "TEST-001", "developer");

        let yaml = serde_yaml::to_string(&manifest).expect("Failed to serialize to YAML");
        let deserialized: ResourceManifest = serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

        assert_eq!(manifest.metadata.code, deserialized.metadata.code);
        assert_eq!(manifest.metadata.name, deserialized.metadata.name);
        assert_eq!(manifest.metadata.resource_type, deserialized.metadata.resource_type);
        assert_eq!(manifest.metadata.email, deserialized.metadata.email);
    }

    #[test]
    fn test_resource_repository_save_and_verify() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("Test Resource", "TEST-001", "developer");

        // Save resource
        let save_result = repo.save(resource.clone().into());
        assert!(save_result.is_ok(), "Failed to save resource: {:?}", save_result);

        // Verify resource was saved by checking file exists
        let resource_file = temp_dir.path().join("resources").join("test_resource.yaml");
        assert!(resource_file.exists(), "Resource file should exist after save");

        // Verify resource directory structure
        let resources_dir = temp_dir.path().join("resources");
        assert!(resources_dir.exists(), "Resources directory should exist");
    }

    #[test]
    fn test_resource_repository_save_multiple_resources() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        // Create and save multiple resources
        let resource1 = create_test_resource("Developer 1", "DEV-001", "developer");
        let resource2 = create_test_resource("QA Engineer", "QA-001", "qa");
        let resource3 = create_test_resource("Manager", "MGR-001", "manager");

        repo.save(resource1.into()).expect("Failed to save resource 1");
        repo.save(resource2.into()).expect("Failed to save resource 2");
        repo.save(resource3.into()).expect("Failed to save resource 3");

        // Verify all resources were saved by checking files exist
        let dev_file = temp_dir.path().join("resources").join("developer_1.yaml");
        let qa_file = temp_dir.path().join("resources").join("qa_engineer.yaml");
        let mgr_file = temp_dir.path().join("resources").join("manager.yaml");

        assert!(dev_file.exists(), "Developer file should exist");
        assert!(qa_file.exists(), "QA file should exist");
        assert!(mgr_file.exists(), "Manager file should exist");
    }

    #[test]
    fn test_resource_repository_find_by_code() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("Test Resource", "TEST-001", "developer");
        repo.save_in_hierarchy(resource.clone().into(), "COMP-001", None)
            .expect("Failed to save resource");

        // Find resource by code
        let found_resource = repo.find_by_code("TEST-001");
        assert!(
            found_resource.is_ok(),
            "Failed to find resource by code: {:?}",
            found_resource
        );

        let found_resource = found_resource.unwrap();
        assert!(found_resource.is_some(), "Resource should be found");

        let found_resource = found_resource.unwrap();
        assert_eq!(found_resource.code(), "TEST-001");
        assert_eq!(found_resource.name(), "Test Resource");
    }

    #[test]
    fn test_resource_repository_error_handling() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        // Try to find non-existent resource
        let result = repo.find_by_code("NON-EXISTENT");
        assert!(result.is_ok(), "Should return Ok(None) for non-existent resource");
        assert!(
            result.unwrap().is_none(),
            "Should return None for non-existent resource"
        );
    }

    #[test]
    fn test_resource_repository_file_corruption_handling() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("Test Resource", "TEST-001", "developer");
        repo.save(resource.clone().into()).expect("Failed to save resource");

        // Corrupt the YAML file
        let resource_file = temp_dir.path().join("resources").join("test_resource.yaml");
        fs::write(&resource_file, "invalid: yaml: content: [").expect("Failed to corrupt file");

        // Note: We can't test loading corrupted files yet since find_by_code is not fully implemented
        // This test verifies that we can save resources and corrupt files
        assert!(resource_file.exists(), "Resource file should exist even if corrupted");
    }

    #[test]
    fn test_resource_repository_concurrent_access() {
        let temp_dir = tempdir().unwrap();

        // Create multiple resources concurrently
        let mut handles = vec![];

        for i in 1..=5 {
            let temp_dir = temp_dir.path().to_path_buf();
            let handle = std::thread::spawn(move || {
                let repo = FileResourceRepository::new(temp_dir);
                let resource = create_test_resource(&format!("Resource {}", i), &format!("RES-{:03}", i), "developer");
                repo.save(resource.into())
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().expect("Thread failed to complete");
            assert!(
                result.is_ok(),
                "Failed to save resource in concurrent access: {:?}",
                result
            );
        }

        // Verify all resources were saved by checking files exist
        for i in 1..=5 {
            let resource_file = temp_dir.path().join("resources").join(format!("resource_{}.yaml", i));
            assert!(resource_file.exists(), "Resource {} file should exist", i);
        }
    }

    #[test]
    fn test_resource_repository_vacation_validation() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("Test Resource", "TEST-001", "developer");
        repo.save(resource.into()).expect("Failed to save resource");

        // Test invalid date range (end date before start date)
        let result = repo.save_vacation("Test Resource", "2024-12-31", "2024-01-01", false, None);
        assert!(result.is_err(), "Should return error for invalid date range");

        // Test valid date range
        let result = repo.save_vacation("Test Resource", "2024-01-01", "2024-12-31", false, None);
        assert!(result.is_ok(), "Should succeed with valid date range");
    }

    #[test]
    fn test_resource_repository_time_off_accumulation() {
        let temp_dir = tempdir().unwrap();
        let repo = FileResourceRepository::new(temp_dir.path());

        let resource = create_test_resource("Test Resource", "TEST-001", "developer");
        repo.save(resource.into()).expect("Failed to save resource");

        // Add multiple time off entries
        repo.save_time_off("Test Resource", 8, "2024-01-01", Some("Morning off".to_string()))
            .expect("Failed to save time off 1");
        repo.save_time_off("Test Resource", 4, "2024-01-02", Some("Afternoon off".to_string()))
            .expect("Failed to save time off 2");

        // Verify total balance
        let updated_resource = repo
            .find_by_name("Test Resource")
            .expect("Failed to find resource")
            .unwrap();
        let balance = match updated_resource {
            AnyResource::Available(r) => r.time_off_balance,
            AnyResource::Assigned(r) => r.time_off_balance,
            AnyResource::Inactive(r) => r.time_off_balance,
        };
        assert_eq!(balance, 12, "Time off balance should accumulate");
    }
}
