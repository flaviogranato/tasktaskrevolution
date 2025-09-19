use crate::application::execution_context::ExecutionContext;
use crate::domain::company_management::repository::CompanyRepository;
use crate::infrastructure::persistence::{
    company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
};

/// Centralized context management for CLI operations
pub struct ContextManager {
    context: ExecutionContext,
    base_dir: std::path::PathBuf,
}

impl ContextManager {
    /// Detect current execution context
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;
        let context =
            ExecutionContext::detect_current().map_err(|e| format!("Failed to detect execution context: {}", e))?;
        Ok(Self {
            context,
            base_dir: current_dir,
        })
    }

    /// Detect current execution context with specific base directory
    pub fn new_with_base_dir<P: AsRef<std::path::Path>>(base_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir = base_dir.as_ref().to_path_buf();
        let context =
            ExecutionContext::detect(&base_dir).map_err(|e| format!("Failed to detect execution context: {}", e))?;
        Ok(Self { context, base_dir })
    }

    /// Create ContextManager with specific base directory
    pub fn with_base_dir<P: AsRef<std::path::Path>>(base_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir = base_dir.as_ref().to_path_buf();
        let context =
            ExecutionContext::detect(&base_dir).map_err(|e| format!("Failed to detect execution context: {}", e))?;
        Ok(Self { context, base_dir })
    }

    /// Create ContextManager with specific context
    pub fn with_context(context: ExecutionContext) -> Self {
        let base_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        Self { context, base_dir }
    }

    /// Get current context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Validate command in current context
    pub fn validate_command(&self, command: &str, subcommand: &str) -> Result<(), String> {
        self.context
            .validate_command(command, subcommand)
            .map_err(|e| e.to_string())
    }

    /// Get company code based on context and parameter
    pub fn resolve_company_code(&self, company_param: Option<String>) -> Result<String, String> {
        match (&self.context, company_param) {
            (ExecutionContext::Root, Some(company)) => Ok(company),
            (ExecutionContext::Root, None) => Ok("ALL".to_string()), // Allow global listing
            (ExecutionContext::Company(code), None) => Ok(code.clone()),
            (ExecutionContext::Company(company), Some(company_param)) => {
                if company_param == *company {
                    Ok(company.clone())
                } else {
                    Err(format!(
                        "Company parameter '{}' does not match current context '{}'",
                        company_param, company
                    ))
                }
            }
            (ExecutionContext::Project(company, _), None) => Ok(company.clone()),
            (ExecutionContext::Project(_, _), Some(_)) => {
                Err("Company parameter not needed in project context".to_string())
            }
        }
    }

    /// Get project and company codes based on context and parameters
    pub fn resolve_project_codes(
        &self,
        project_param: Option<String>,
        company_param: Option<String>,
    ) -> Result<(String, String), String> {
        match (&self.context, project_param, company_param) {
            (ExecutionContext::Root, Some(project), Some(company)) => {
                // Validate that the company exists
                let company_repo = FileCompanyRepository::new(self.get_base_path());
                match company_repo.find_by_code(&company) {
                    Ok(Some(_)) => Ok((project, company)),
                    Ok(None) => Err(format!("Company '{}' not found", company)),
                    Err(e) => Err(format!("Error validating company '{}': {}", company, e)),
                }
            }
            (ExecutionContext::Root, None, _) => Err("Project parameter required in root context".to_string()),
            (ExecutionContext::Root, Some(_), None) => Err("Company parameter required in root context".to_string()),
            (ExecutionContext::Company(company), Some(project), None) => Ok((project, company.clone())),
            (ExecutionContext::Company(_), None, _) => Err("Project parameter required in company context".to_string()),
            (ExecutionContext::Company(company), Some(project), Some(company_param)) => {
                if company_param == *company {
                    Ok((project, company.clone()))
                } else {
                    Err(format!(
                        "Company parameter '{}' does not match current context '{}'",
                        company_param, company
                    ))
                }
            }
            (ExecutionContext::Project(company, project), None, None) => Ok((project.clone(), company.clone())),
            (ExecutionContext::Project(_, _), Some(_), _) => {
                Err("Project parameter not needed in project context".to_string())
            }
            (ExecutionContext::Project(_, _), None, Some(_)) => {
                Err("Company parameter not needed in project context".to_string())
            }
        }
    }

    /// Get base path for file operations based on context
    pub fn get_base_path(&self) -> String {
        match self.context {
            ExecutionContext::Root => self.base_dir.to_string_lossy().to_string(),
            ExecutionContext::Company(_) => {
                // In company context, go up one level to reach the root directory
                // From: /path/companies/COMPANY
                // To:   /path
                self.base_dir
                    .parent()
                    .unwrap_or(&self.base_dir)
                    .to_string_lossy()
                    .to_string()
            }
            ExecutionContext::Project(_, _) => {
                // In project context, go up three levels to reach the root directory
                // From: /path/companies/COMPANY/projects/PROJECT
                // To:   /path
                self.base_dir
                    .parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                    .unwrap_or(&self.base_dir)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }

    /// Create project repository with correct base path
    pub fn create_project_repository(&self) -> FileProjectRepository {
        let base_path = self.get_base_path();
        FileProjectRepository::with_base_path(base_path.into())
    }

    /// Get project repository with correct base path (alias for create_project_repository)
    pub fn get_project_repository(&self) -> FileProjectRepository {
        self.create_project_repository()
    }

    /// Create resource repository with correct base path
    pub fn create_resource_repository(&self) -> FileResourceRepository {
        let base_path = self.get_base_path();
        FileResourceRepository::new(base_path)
    }

    /// Create company repository
    pub fn create_company_repository(&self) -> FileCompanyRepository {
        FileCompanyRepository::new(".")
    }

    /// Create task repository with correct base path
    pub fn create_task_repository(&self) -> FileTaskRepository {
        let base_path = self.get_base_path();
        FileTaskRepository::new(base_path)
    }
}
