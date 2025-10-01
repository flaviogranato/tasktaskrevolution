use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::AnyProject;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::AnyResource;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::AnyTask;
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
    pub valid_entities: usize,
    pub invalid_entities: usize,
    pub entities_with_errors: usize,
    pub entities_with_warnings: usize,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub success_rate: f64,
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
        let valid_entities = results.iter().filter(|r| r.errors.is_empty()).count();
        let invalid_entities = total_entities - valid_entities;
        let entities_with_errors = results.iter().filter(|r| !r.errors.is_empty()).count();
        let entities_with_warnings = results.iter().filter(|r| !r.warnings.is_empty()).count();
        let total_errors = results.iter().map(|r| r.errors.len()).sum();
        let total_warnings = results.iter().map(|r| r.warnings.len()).sum();
        let success_rate = if total_entities > 0 {
            (valid_entities as f64 / total_entities as f64) * 100.0
        } else {
            0.0
        };

        DataValidationSummary {
            total_entities,
            valid_entities,
            invalid_entities,
            entities_with_errors,
            entities_with_warnings,
            total_errors,
            total_warnings,
            success_rate,
        }
    }
}

/// Service for validating data integrity across all entities
pub struct DataValidationService {
    company_repo: Arc<dyn CompanyRepository>,
    project_repo: Arc<dyn ProjectRepository>,
    resource_repo: Arc<dyn ResourceRepository>,
    task_repo: Arc<dyn TaskRepository>,
    /// Optional code resolver to resolve and validate codes against repositories
    code_resolver: Option<crate::application::shared::code_resolver::CodeResolver>,
}

impl DataValidationService {
    fn normalize_code(&self, s: &str) -> String {
        s.trim().to_lowercase()
    }
    pub fn new(
        company_repo: Arc<dyn CompanyRepository>,
        project_repo: Arc<dyn ProjectRepository>,
        resource_repo: Arc<dyn ResourceRepository>,
        task_repo: Arc<dyn TaskRepository>,
        code_resolver: Option<crate::application::shared::code_resolver::CodeResolver>,
    ) -> Self {
        Self {
            company_repo,
            project_repo,
            resource_repo,
            task_repo,
            code_resolver,
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

    /// Validate company by specific ID
    pub async fn validate_company_by_id(&self, id: &str) -> Result<Vec<DataValidationResult>, AppError> {
        let companies = self.company_repo.find_all()?;
        let company =
            companies
                .iter()
                .find(|c| c.id == id || c.code == id)
                .ok_or_else(|| AppError::ValidationError {
                    field: "id".to_string(),
                    message: format!("Company with ID '{}' not found", id),
                })?;

        let mut results = Vec::new();
        let validation_result = self.validate_single_company(company);
        results.push(validation_result);
        Ok(results)
    }

    /// Validate project by specific ID
    pub async fn validate_project_by_id(&self, id: &str) -> Result<Vec<DataValidationResult>, AppError> {
        let projects = self.project_repo.find_all()?;
        let project = projects
            .iter()
            .find(|p| p.id() == id || p.code() == id)
            .ok_or_else(|| AppError::ValidationError {
                field: "id".to_string(),
                message: format!("Project with ID '{}' not found", id),
            })?;

        let mut results = Vec::new();
        let validation_result = self.validate_single_any_project(project);
        results.push(validation_result);
        Ok(results)
    }

    /// Validate task by specific ID
    pub async fn validate_task_by_id(&self, id: &str) -> Result<Vec<DataValidationResult>, AppError> {
        let tasks = self.task_repo.find_all()?;
        let task = tasks
            .iter()
            .find(|t| t.id().to_string() == id || t.code() == id)
            .ok_or_else(|| AppError::ValidationError {
                field: "id".to_string(),
                message: format!("Task with ID '{}' not found", id),
            })?;

        let mut results = Vec::new();
        let validation_result = self.validate_single_task(task);
        results.push(validation_result);
        Ok(results)
    }

    /// Validate resource by specific ID
    pub async fn validate_resource_by_id(&self, id: &str) -> Result<Vec<DataValidationResult>, AppError> {
        let resources = self.resource_repo.find_all()?;
        let resource = resources
            .iter()
            .find(|r| r.id().to_string() == id || r.code() == id)
            .ok_or_else(|| AppError::ValidationError {
                field: "id".to_string(),
                message: format!("Resource with ID '{}' not found", id),
            })?;

        let mut results = Vec::new();
        let validation_result = self.validate_single_resource(resource);
        results.push(validation_result);
        Ok(results)
    }

    /// Validate company data
    pub async fn validate_companies(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let companies = self.company_repo.find_all()?;
        let mut results = Vec::new();

        for company in &companies {
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

            // Validate code format (should be alphanumeric with hyphens)
            if !company.code().is_empty() && !self.is_valid_code_format(company.code()) {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "alphanumeric format with hyphens".to_string(),
                    actual: company.code().to_string(),
                    message: "Company code should contain only letters, numbers, and hyphens".to_string(),
                });
            }

            // Validate name length
            if company.name().len() < 2 {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    expected: "at least 2 characters".to_string(),
                    actual: format!("{} characters", company.name().len()),
                    message: "Company name should be at least 2 characters long".to_string(),
                });
            }

            // Check for duplicate codes
            let duplicate_code = companies
                .iter()
                .filter(|c| c.id != company.id && c.code() == company.code())
                .count()
                > 0;
            if duplicate_code {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "unique code".to_string(),
                    actual: "duplicate code".to_string(),
                    message: "Company code must be unique".to_string(),
                });
            }

            // Validate email format if provided
            if let Some(email) = &company.email
                && !self.is_valid_email(email)
            {
                errors.push(ValidationError {
                    field: "email".to_string(),
                    expected: "valid email format".to_string(),
                    actual: email.clone(),
                    message: "Invalid email format".to_string(),
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
    pub async fn validate_projects(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let projects = self.project_repo.find_all()?;
        let companies = self.company_repo.find_all()?;
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

            // Additional format validation
            if !project.code().is_empty() && !self.is_valid_code_format(project.code()) {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "alphanumeric format with hyphens".to_string(),
                    actual: project.code().to_string(),
                    message: "Project code should contain only letters, numbers, and hyphens".to_string(),
                });
            }

            // Relationship validation
            errors.extend(self.validate_project_relationships(&project, &companies));

            results.push(DataValidationResult {
                entity_type: "Project".to_string(),
                entity_id: project.code().to_string(),
                errors,
                warnings,
            });
        }

        // Consistency: duplicate code check across projects
        let mut seen_codes = std::collections::HashSet::new();
        let mut duplicate_codes = std::collections::HashSet::new();
        for project in self.project_repo.find_all()? {
            let code = project.code().to_string();
            if !seen_codes.insert(code.clone()) {
                duplicate_codes.insert(code);
            }
        }
        if !duplicate_codes.is_empty() {
            for dup in duplicate_codes {
                results.push(DataValidationResult {
                    entity_type: "Project".to_string(),
                    entity_id: dup.clone(),
                    errors: vec![ValidationError {
                        field: "code".to_string(),
                        expected: "unique code".to_string(),
                        actual: dup,
                        message: "Duplicate project code found".to_string(),
                    }],
                    warnings: vec![],
                });
            }
        }

        Ok(results)
    }

    /// Validate resource data
    pub async fn validate_resources(&self) -> Result<Vec<DataValidationResult>, AppError> {
        // Use context-aware listing to enable relationship checks
        let resources_with_ctx = self.resource_repo.find_all_with_context()?;
        let companies = self.company_repo.find_all()?;
        // Build normalized set of company codes
        use std::collections::HashSet;
        let company_code_set: HashSet<String> = companies.iter().map(|c| self.normalize_code(c.code())).collect();
        let mut results = Vec::new();

        for (resource, company_code, _projects) in resources_with_ctx.clone() {
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

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

            // Additional format validation
            if !resource.code().is_empty() && !self.is_valid_code_format(resource.code()) {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "alphanumeric format with hyphens".to_string(),
                    actual: resource.code().to_string(),
                    message: "Resource code should contain only letters, numbers, and hyphens".to_string(),
                });
            }

            // Relationship: company exists for this resource
            let norm_company = self.normalize_code(&company_code);
            let company_exists = if company_code_set.contains(&norm_company) {
                true
            } else if let Some(resolver) = &self.code_resolver {
                // Fallback to resolver to honor repo semantics (code->ID mapping) and base_path
                resolver.validate_company_code(&company_code).is_ok()
            } else {
                false
            };
            if !company_exists && company_code != "UNKNOWN" {
                errors.push(ValidationError {
                    field: "company_code".to_string(),
                    expected: "existing company code".to_string(),
                    actual: company_code.clone(),
                    message: "Resource references non-existent company".to_string(),
                });
            } else if company_code == "UNKNOWN" {
                warnings.push(ValidationWarning {
                    field: "company_code".to_string(),
                    message: "Resource company could not be inferred from path (context unknown)".to_string(),
                });
            }

            results.push(DataValidationResult {
                entity_type: "Resource".to_string(),
                entity_id: resource.code().to_string(),
                errors,
                warnings,
            });
        }

        // Consistency: duplicate resource code check (merge into existing result)
        use std::collections::HashMap;
        // Group by code -> unique IDs observed
        let mut code_to_ids: HashMap<String, HashSet<String>> = HashMap::new();
        for (resource, _c, _p) in resources_with_ctx {
            code_to_ids
                .entry(resource.code().to_string())
                .or_default()
                .insert(resource.id().to_string());
        }
        let duplicates: Vec<String> = code_to_ids
            .into_iter()
            .filter_map(|(code, ids)| if ids.len() > 1 { Some(code) } else { None })
            .collect();
        if !duplicates.is_empty() {
            let mut idx_map: HashMap<String, usize> = HashMap::new();
            for (i, r) in results.iter().enumerate() {
                idx_map.entry(r.entity_id.clone()).or_insert(i);
            }
            for dup in duplicates {
                if let Some(&i) = idx_map.get(&dup) {
                    results[i].errors.push(ValidationError {
                        field: "code".to_string(),
                        expected: "unique code".to_string(),
                        actual: dup.clone(),
                        message: "Duplicate resource code found".to_string(),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Validate task data
    pub async fn validate_tasks(&self) -> Result<Vec<DataValidationResult>, AppError> {
        let tasks = self.task_repo.find_all()?;
        let projects = self.project_repo.find_all()?;
        let resources = self.resource_repo.find_all()?;
        let mut results = Vec::new();

        use std::collections::HashSet;
        let mut seen_task_codes: HashSet<String> = HashSet::new();
        for task in tasks {
            let norm_task_code = self.normalize_code(task.code());
            if !seen_task_codes.insert(norm_task_code) {
                continue; // skip duplicate task entries
            }
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

            // Additional format validation
            if !task.code().is_empty() && !self.is_valid_code_format(task.code()) {
                errors.push(ValidationError {
                    field: "code".to_string(),
                    expected: "alphanumeric format with hyphens".to_string(),
                    actual: task.code().to_string(),
                    message: "Task code should contain only letters, numbers, and hyphens".to_string(),
                });
            }

            // Relationship validation
            errors.extend(self.validate_task_relationships(&task, &projects, &resources));

            results.push(DataValidationResult {
                entity_type: "Task".to_string(),
                entity_id: task.code().to_string(),
                errors,
                warnings,
            });
        }

        // Consistency: duplicate task code check
        let mut seen_codes = std::collections::HashSet::new();
        let mut duplicate_codes = std::collections::HashSet::new();
        for task in self.task_repo.find_all()? {
            let code = task.code().to_string();
            if !seen_codes.insert(code.clone()) {
                duplicate_codes.insert(code);
            }
        }
        if !duplicate_codes.is_empty() {
            for dup in duplicate_codes {
                results.push(DataValidationResult {
                    entity_type: "Task".to_string(),
                    entity_id: dup.clone(),
                    errors: vec![ValidationError {
                        field: "code".to_string(),
                        expected: "unique code".to_string(),
                        actual: dup,
                        message: "Duplicate task code found".to_string(),
                    }],
                    warnings: vec![],
                });
            }
        }

        Ok(results)
    }

    /// Validate code format (alphanumeric with hyphens)
    fn is_valid_code_format(&self, code: &str) -> bool {
        code.chars().all(|c| c.is_alphanumeric() || c == '-')
    }

    /// Validate email format
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && !email.starts_with('@') && !email.ends_with('@')
    }

    /// Validate date format
    #[allow(dead_code)]
    fn is_valid_date(&self, date_str: &str) -> bool {
        chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
    }

    /// Validate URL format
    #[allow(dead_code)]
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Validate project relationships
    fn validate_project_relationships(
        &self,
        project: &AnyProject,
        companies: &[crate::domain::company_management::company::Company],
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check if company exists (by code)
        let company_exists = companies.iter().any(|c| c.code == project.company_code());
        if !company_exists {
            errors.push(ValidationError {
                field: "company_id".to_string(),
                expected: "existing company ID".to_string(),
                actual: project.company_code().to_string(),
                message: "Project references non-existent company".to_string(),
            });
        }

        errors
    }

    /// Validate task relationships
    fn validate_task_relationships(
        &self,
        task: &AnyTask,
        projects: &[AnyProject],
        resources: &[AnyResource],
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check if project exists
        let project_exists = projects.iter().any(|p| p.code() == task.project_code());
        if !project_exists {
            errors.push(ValidationError {
                field: "project_id".to_string(),
                expected: "existing project ID".to_string(),
                actual: task.project_code().to_string(),
                message: "Task references non-existent project".to_string(),
            });
        }

        // Check if assigned resources exist
        for resource_id in task.assigned_resources() {
            let resource_exists = resources.iter().any(|r| r.code() == *resource_id);
            if !resource_exists {
                errors.push(ValidationError {
                    field: "assigned_resources".to_string(),
                    expected: "existing resource ID".to_string(),
                    actual: resource_id.clone(),
                    message: "Task references non-existent resource".to_string(),
                });
            }
        }

        errors
    }

    /// Validate resource relationships
    #[allow(dead_code)]
    fn validate_resource_relationships(
        &self,
        _resource: &AnyResource,
        _companies: &[crate::domain::company_management::company::Company],
    ) -> Vec<ValidationError> {
        // Nota: `AnyResource` não expõe company_code diretamente.
        // Uma validação mais rica pode usar `resource_repo.find_all_with_context()`
        // para obter o contexto (company_code) do arquivo. Por ora, não há
        // validações de relacionamento específicas do recurso aqui.
        Vec::new()
    }

    /// Validate a single company
    fn validate_single_company(
        &self,
        company: &crate::domain::company_management::company::Company,
    ) -> DataValidationResult {
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

        // Validate code format (should be alphanumeric with hyphens)
        if !company.code().is_empty() && !self.is_valid_code_format(company.code()) {
            errors.push(ValidationError {
                field: "code".to_string(),
                expected: "alphanumeric format with hyphens".to_string(),
                actual: company.code().to_string(),
                message: "Company code should contain only letters, numbers, and hyphens".to_string(),
            });
        }

        // Validate name length
        if company.name().len() < 2 {
            errors.push(ValidationError {
                field: "name".to_string(),
                expected: "at least 2 characters".to_string(),
                actual: format!("{} characters", company.name().len()),
                message: "Company name should be at least 2 characters long".to_string(),
            });
        }

        // Validate email format if provided
        if let Some(email) = &company.email
            && !self.is_valid_email(email)
        {
            errors.push(ValidationError {
                field: "email".to_string(),
                expected: "valid email format".to_string(),
                actual: email.clone(),
                message: "Invalid email format".to_string(),
            });
        }

        DataValidationResult {
            entity_type: "Company".to_string(),
            entity_id: company.code().to_string(),
            errors,
            warnings,
        }
    }

    /// Validate a single project
    #[allow(dead_code)]
    fn validate_single_project(
        &self,
        project: &crate::domain::project_management::project::Project,
    ) -> DataValidationResult {
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

        DataValidationResult {
            entity_type: "Project".to_string(),
            entity_id: project.code().to_string(),
            errors,
            warnings,
        }
    }

    /// Validate a single any project
    fn validate_single_any_project(&self, project: &AnyProject) -> DataValidationResult {
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

        DataValidationResult {
            entity_type: "Project".to_string(),
            entity_id: project.code().to_string(),
            errors,
            warnings,
        }
    }

    /// Validate a single task
    fn validate_single_task(&self, task: &AnyTask) -> DataValidationResult {
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

        DataValidationResult {
            entity_type: "Task".to_string(),
            entity_id: task.code().to_string(),
            errors,
            warnings,
        }
    }

    /// Validate a single resource
    fn validate_single_resource(&self, resource: &AnyResource) -> DataValidationResult {
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

        DataValidationResult {
            entity_type: "Resource".to_string(),
            entity_id: resource.code().to_string(),
            errors,
            warnings,
        }
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
