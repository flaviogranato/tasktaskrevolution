use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::repository::TaskRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Represents a validation error found during data validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// Represents a validation warning found during data validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

/// Represents the result of validating a single entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationResult {
    pub entity_type: String,
    pub entity_id: String,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Represents a summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationSummary {
    pub total_entities: usize,
    pub entities_with_errors: usize,
    pub entities_with_warnings: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
}

/// Represents a complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationReport {
    pub summary: DataValidationSummary,
    pub results: Vec<DataValidationResult>,
}

/// Use case for data validation
pub struct DataValidationUseCase {
    service: DataValidationService,
}

impl DataValidationUseCase {
    pub fn new(service: DataValidationService) -> Self {
        Self { service }
    }

    pub async fn validate_all(&self) -> Result<DataValidationReport, AppError> {
        let results = self.service.validate_all().await?;
        let summary = self.calculate_summary(&results);

        Ok(DataValidationReport { summary, results })
    }

    fn calculate_summary(&self, results: &[DataValidationResult]) -> DataValidationSummary {
        let total_entities = results.len();
        let entities_with_errors = results.iter().filter(|r| !r.errors.is_empty()).count();
        let entities_with_warnings = results.iter().filter(|r| !r.warnings.is_empty()).count();
        let total_errors = results.iter().map(|r| r.errors.len()).sum();
        let total_warnings = results.iter().map(|r| r.warnings.len()).sum();

        DataValidationSummary {
            total_entities,
            entities_with_errors,
            entities_with_warnings,
            total_errors,
            total_warnings,
        }
    }
}

/// Service for validating data integrity across all entities
pub struct DataValidationService {
    company_repo: Arc<dyn CompanyRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    task_repo: Arc<dyn TaskRepository>,
}

impl DataValidationService {
    pub fn new(
        company_repo: Arc<dyn CompanyRepository>,
        project_repo: Arc<dyn ProjectRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        task_repo: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            company_repo,
            project_repo,
            resource_repo,
            task_repo,
        }
    }

    /// Validates all data in the system
    pub async fn validate_all(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let mut all_results = Vec::new();

        // Validate companies
        let company_results = self.validate_companies().await?;
        all_results.extend(company_results);

        // Validate projects
        let project_results = self.validate_projects().await?;
        all_results.extend(project_results);

        // Validate resources
        let resource_results = self.validate_resources().await?;
        all_results.extend(resource_results);

        // Validate tasks
        let task_results = self.validate_tasks().await?;
        all_results.extend(task_results);

        Ok(all_results)
    }

    /// Validate company data
    async fn validate_companies(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let companies = self.company_repo.find_all()?;
        let mut results = Vec::new();

        for company in companies {
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Validate company name
            if company.name().is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Company name cannot be empty".to_string(),
                });
            }

            // Validate company code
            if company.code().is_empty() {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Company code cannot be empty".to_string(),
                });
            }

            // Check for very long descriptions
            if let Some(description) = &company.description
                && description.len() > 1000
            {
                warnings.push(ValidationWarning {
                    field: "description".to_string(),
                    message: "Company description is very long".to_string(),
                });
            }

            results.push(DataValidationResult {
                entity_type: "Company".to_string(),
                entity_id: company.code().to_string(),
                errors,
                warnings,
            });
        }

        Ok(results)
    }

    /// Validate project data
    async fn validate_projects(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let projects = self.project_repo.find_all()?;
        let mut results = Vec::new();

        for project in projects {
            let mut errors = Vec::new();
            let warnings = Vec::new();

            // Validate project name
            if project.name().is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Project name cannot be empty".to_string(),
                });
            }

            // Validate project code
            if project.code().is_empty() {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Project code cannot be empty".to_string(),
                });
            }

            results.push(DataValidationResult {
                entity_type: "Project".to_string(),
                entity_id: project.code().to_string(),
                errors,
                warnings,
            });
        }

        Ok(results)
    }

    /// Validate resource data
    async fn validate_resources(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let resources = self.resource_repo.find_all()?;
        let mut results = Vec::new();

        for resource in resources {
            let mut errors = Vec::new();
            let warnings = Vec::new();

            // Validate resource name
            if resource.name().is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Resource name cannot be empty".to_string(),
                });
            }

            // Validate resource code
            if resource.code().is_empty() {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Resource code cannot be empty".to_string(),
                });
            }

            results.push(DataValidationResult {
                entity_type: "Resource".to_string(),
                entity_id: resource.code().to_string(),
                errors,
                warnings,
            });
        }

        Ok(results)
    }

    /// Validate task data
    async fn validate_tasks(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let tasks = self.task_repo.find_all()?;
        let mut results = Vec::new();

        for task in tasks {
            let mut errors = Vec::new();
            let warnings = Vec::new();

            // Validate task name
            if task.name().is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Task name cannot be empty".to_string(),
                });
            }

            // Validate task code
            if task.code().is_empty() {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "non-empty string".to_string(),
                    actual: "empty string".to_string(),
                    message: "Task code cannot be empty".to_string(),
                });
            }

            results.push(DataValidationResult {
                entity_type: "Task".to_string(),
                entity_id: task.code().to_string(),
                errors,
                warnings,
            });
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_error_creation() {
        let error = ValidationError {
            field: "name".to_string(),
            expected: "non-empty string".to_string(),
            actual: "empty string".to_string(),
            message: "Name cannot be empty".to_string(),
        };

        assert_eq!(error.field, "name");
        assert_eq!(error.expected, "non-empty string");
        assert_eq!(error.actual, "empty string");
        assert_eq!(error.message, "Name cannot be empty");
    }

    #[tokio::test]
    async fn test_validation_warning_creation() {
        let warning = ValidationWarning {
            field: "description".to_string(),
            message: "Description is very long".to_string(),
        };

        assert_eq!(warning.field, "description");
        assert_eq!(warning.message, "Description is very long");
    }

    #[tokio::test]
    async fn test_validation_result_creation() {
        let errors = vec![ValidationError {
            field: "name".to_string(),
            expected: "non-empty string".to_string(),
            actual: "empty string".to_string(),
            message: "Name cannot be empty".to_string(),
        }];

        let warnings = vec![ValidationWarning {
            field: "description".to_string(),
            message: "Description is very long".to_string(),
        }];

        let result = DataValidationResult {
            entity_type: "Company".to_string(),
            entity_id: "COMP-001".to_string(),
            errors,
            warnings,
        };

        assert_eq!(result.entity_type, "Company");
        assert_eq!(result.entity_id, "COMP-001");
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }
}
