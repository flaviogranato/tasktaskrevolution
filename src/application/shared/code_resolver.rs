use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::repository::TaskRepository;
use crate::infrastructure::persistence::{
    company_repository::FileCompanyRepository,
    project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository,
    task_repository::FileTaskRepository,
};

/// Trait for resolving entity codes to IDs
pub trait CodeResolverTrait {
    fn resolve_company_code(&self, code: &str) -> Result<String, AppError>;
    fn resolve_project_code(&self, code: &str) -> Result<String, AppError>;
    fn resolve_resource_code(&self, code: &str) -> Result<String, AppError>;
    fn resolve_task_code(&self, code: &str) -> Result<String, AppError>;
    fn validate_company_code(&self, code: &str) -> Result<(), AppError>;
    fn validate_project_code(&self, code: &str) -> Result<(), AppError>;
    fn validate_resource_code(&self, code: &str) -> Result<(), AppError>;
    fn validate_task_code(&self, code: &str) -> Result<(), AppError>;
}

/// Service responsible for resolving entity codes to IDs for internal operations
pub struct CodeResolver {
    company_repository: FileCompanyRepository,
    project_repository: FileProjectRepository,
    resource_repository: FileResourceRepository,
    task_repository: FileTaskRepository,
}

impl CodeResolverTrait for CodeResolver {
    fn resolve_company_code(&self, code: &str) -> Result<String, AppError> {
        let company = self
            .company_repository
            .find_by_code(code)?
            .ok_or_else(|| {
                AppError::validation_error(
                    "company",
                    format!("Company '{}' not found", code),
                )
            })?;
        Ok(company.id)
    }

    fn resolve_project_code(&self, code: &str) -> Result<String, AppError> {
        let project = self
            .project_repository
            .find_by_code(code)?
            .ok_or_else(|| {
                AppError::validation_error(
                    "project",
                    format!("Project '{}' not found", code),
                )
            })?;
        Ok(project.id().to_string())
    }

    fn resolve_resource_code(&self, code: &str) -> Result<String, AppError> {
        let resource = self
            .resource_repository
            .find_by_code(code)?
            .ok_or_else(|| {
                AppError::validation_error(
                    "resource",
                    format!("Resource '{}' not found", code),
                )
            })?;
        Ok(resource.id().to_string())
    }

    fn resolve_task_code(&self, code: &str) -> Result<String, AppError> {
        let task = self
            .task_repository
            .find_by_code(code)?
            .ok_or_else(|| {
                AppError::validation_error(
                    "task",
                    format!("Task '{}' not found", code),
                )
            })?;
        Ok(task.id().to_string())
    }

    fn validate_company_code(&self, code: &str) -> Result<(), AppError> {
        self.resolve_company_code(code)?;
        Ok(())
    }

    fn validate_project_code(&self, code: &str) -> Result<(), AppError> {
        self.resolve_project_code(code)?;
        Ok(())
    }

    fn validate_resource_code(&self, code: &str) -> Result<(), AppError> {
        self.resolve_resource_code(code)?;
        Ok(())
    }

    fn validate_task_code(&self, code: &str) -> Result<(), AppError> {
        self.resolve_task_code(code)?;
        Ok(())
    }
}

impl CodeResolver {
    /// Create a new CodeResolver with the given base path
    pub fn new<P: AsRef<std::path::Path>>(base_path: P) -> Self {
        let base_path = base_path.as_ref().to_path_buf();
        Self {
            company_repository: FileCompanyRepository::new(&base_path),
            project_repository: FileProjectRepository::with_base_path(base_path.clone()),
            resource_repository: FileResourceRepository::new(&base_path),
            task_repository: FileTaskRepository::new(&base_path),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_code_resolver_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _resolver = CodeResolver::new(temp_dir.path());
        // Should not panic
        assert!(true);
    }

    #[test]
    fn test_resolve_nonexistent_company_code() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = CodeResolver::new(temp_dir.path());
        
        let result = resolver.resolve_company_code("NONEXISTENT");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Company 'NONEXISTENT' not found"));
    }

    #[test]
    fn test_resolve_nonexistent_project_code() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = CodeResolver::new(temp_dir.path());
        
        let result = resolver.resolve_project_code("NONEXISTENT");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Project 'NONEXISTENT' not found"));
    }

    #[test]
    fn test_resolve_nonexistent_resource_code() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = CodeResolver::new(temp_dir.path());
        
        let result = resolver.resolve_resource_code("NONEXISTENT");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Resource 'NONEXISTENT' not found"));
    }

    #[test]
    fn test_resolve_nonexistent_task_code() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = CodeResolver::new(temp_dir.path());
        
        let result = resolver.resolve_task_code("NONEXISTENT");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Task 'NONEXISTENT' not found"));
    }
}
