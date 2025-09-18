use crate::application::errors::AppError;
use serde_yaml;
use std::path::Path;

/// Represents the different execution contexts where CLI commands can be executed
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionContext {
    /// Root context: config.yaml is in the current directory
    Root,
    /// Company context: company.yaml is in the current directory
    Company(String), // company code
    /// Project context: project.yaml is in the current directory
    Project(String, String), // (company_code, project_code)
}

/// Error types for execution context detection
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionContextError {
    NoContextFound { path: String },
    InvalidYaml { file: String, error: String },
    IoError { error: String },
}

impl std::fmt::Display for ExecutionContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionContextError::NoContextFound { path } => {
                write!(f, "No valid execution context found in directory: {}", path)
            }
            ExecutionContextError::InvalidYaml { file, error } => {
                write!(f, "Invalid YAML file: {} - {}", file, error)
            }
            ExecutionContextError::IoError { error } => {
                write!(f, "IO error: {}", error)
            }
        }
    }
}

impl std::error::Error for ExecutionContextError {}

impl From<ExecutionContextError> for AppError {
    fn from(error: ExecutionContextError) -> Self {
        AppError::ValidationError {
            field: "execution_context".to_string(),
            message: error.to_string(),
        }
    }
}

/// Metadata structure for company.yaml files
#[derive(Debug, serde::Deserialize)]
struct CompanyMetadata {
    code: String,
}

/// Metadata structure for project.yaml files
#[derive(Debug, serde::Deserialize)]
struct ProjectMetadata {
    code: String,
}

/// Manifest structure for company.yaml files
#[derive(Debug, serde::Deserialize)]
struct CompanyManifest {
    metadata: CompanyMetadata,
}

/// Manifest structure for project.yaml files
#[derive(Debug, serde::Deserialize)]
struct ProjectManifest {
    metadata: ProjectMetadata,
}

impl ExecutionContext {
    /// Detects the execution context by analyzing the current directory structure
    pub fn detect(path: &Path) -> Result<Self, ExecutionContextError> {
        let path_str = path.to_string_lossy().to_string();

        // Check for config.yaml (root context)
        if path.join("config.yaml").exists() {
            return Ok(ExecutionContext::Root);
        }

        // Check for project.yaml first (project context)
        if let Some(project_code) = Self::find_project_yaml(path)? {
            // Try to find the company by looking at the directory structure
            if let Some(company_code) = Self::find_company_from_path(path)? {
                return Ok(ExecutionContext::Project(company_code, project_code));
            }
        }

        // Check for company.yaml (company context)
        if let Some(company_code) = Self::find_company_yaml(path)? {
            return Ok(ExecutionContext::Company(company_code));
        }

        Err(ExecutionContextError::NoContextFound { path: path_str })
    }

    /// Detects the execution context from the current working directory
    pub fn detect_current() -> Result<Self, ExecutionContextError> {
        let current_dir = std::env::current_dir().map_err(|e| ExecutionContextError::IoError {
            error: format!("Failed to get current directory: {}", e),
        })?;
        Self::detect(&current_dir)
    }

    /// Finds and parses company.yaml file in the given path
    fn find_company_yaml(path: &Path) -> Result<Option<String>, ExecutionContextError> {
        let company_yaml_path = path.join("company.yaml");

        if !company_yaml_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&company_yaml_path).map_err(|e| ExecutionContextError::IoError {
            error: format!("Failed to read company.yaml: {}", e),
        })?;

        let manifest: CompanyManifest =
            serde_yaml::from_str(&content).map_err(|e| ExecutionContextError::InvalidYaml {
                file: "company.yaml".to_string(),
                error: e.to_string(),
            })?;

        Ok(Some(manifest.metadata.code))
    }

    /// Finds and parses project.yaml file in the given path
    fn find_project_yaml(path: &Path) -> Result<Option<String>, ExecutionContextError> {
        let project_yaml_path = path.join("project.yaml");

        if !project_yaml_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&project_yaml_path).map_err(|e| ExecutionContextError::IoError {
            error: format!("Failed to read project.yaml: {}", e),
        })?;

        let manifest: ProjectManifest =
            serde_yaml::from_str(&content).map_err(|e| ExecutionContextError::InvalidYaml {
                file: "project.yaml".to_string(),
                error: e.to_string(),
            })?;

        Ok(Some(manifest.metadata.code))
    }

    /// Tries to find the company code from the directory path structure
    fn find_company_from_path(path: &Path) -> Result<Option<String>, ExecutionContextError> {
        // Look for companies/*/projects/*/ structure
        let path_components: Vec<_> = path.components().collect();

        // Find the pattern: .../companies/{company_code}/projects/{project_code}/...
        for (i, component) in path_components.iter().enumerate() {
            if component.as_os_str() == "companies" && i + 1 < path_components.len() {
                let company_component = &path_components[i + 1];
                if let Some(company_code) = company_component.as_os_str().to_str() {
                    return Ok(Some(company_code.to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Returns the display name for the context
    pub fn display_name(&self) -> String {
        match self {
            ExecutionContext::Root => "Root (Global)".to_string(),
            ExecutionContext::Company(code) => format!("Company: {}", code),
            ExecutionContext::Project(company, project) => format!("Project: {} in {}", project, company),
        }
    }

    /// Returns the company code if available
    pub fn company_code(&self) -> Option<String> {
        match self {
            ExecutionContext::Root => None,
            ExecutionContext::Company(code) => Some(code.clone()),
            ExecutionContext::Project(company, _) => Some(company.clone()),
        }
    }

    /// Returns the project code if available
    pub fn project_code(&self) -> Option<String> {
        match self {
            ExecutionContext::Root => None,
            ExecutionContext::Company(_) => None,
            ExecutionContext::Project(_, project) => Some(project.clone()),
        }
    }

    /// Returns the required parameters for commands based on the context
    pub fn get_required_params(&self, command: &str) -> Vec<String> {
        match (self, command) {
            // Root context
            (ExecutionContext::Root, "list") => vec![],
            (ExecutionContext::Root, "create") => vec![],
            (ExecutionContext::Root, "update") => vec![],
            (ExecutionContext::Root, "delete") => vec![],

            // Company context
            (ExecutionContext::Company(_), "list") => vec![],
            (ExecutionContext::Company(_), "create") => vec![],
            (ExecutionContext::Company(_), "update") => vec![],
            (ExecutionContext::Company(_), "delete") => vec![],

            // Project context
            (ExecutionContext::Project(_, _), "list") => vec![],
            (ExecutionContext::Project(_, _), "create") => vec![],
            (ExecutionContext::Project(_, _), "update") => vec![],
            (ExecutionContext::Project(_, _), "delete") => vec![],

            _ => vec![],
        }
    }

    /// Validates if the context is valid for the given command
    pub fn validate_command(&self, command: &str, entity: &str) -> Result<(), ExecutionContextError> {
        match (self, command, entity) {
            // Root context - can do everything but needs parameters
            (ExecutionContext::Root, _, _) => Ok(()),

            // Company context - can manage projects and resources
            (ExecutionContext::Company(_), "create", "project") => Ok(()),
            (ExecutionContext::Company(_), "create", "resource") => Ok(()),
            (ExecutionContext::Company(_), "list", "projects") => Ok(()),
            (ExecutionContext::Company(_), "list", "resources") => Ok(()),
            (ExecutionContext::Company(_), "update", "project") => Ok(()),
            (ExecutionContext::Company(_), "update", "resource") => Ok(()),
            (ExecutionContext::Company(_), "update", "task") => Ok(()),
            (ExecutionContext::Company(_), "delete", "project") => Ok(()),
            (ExecutionContext::Company(_), "delete", "resource") => Ok(()),
            (ExecutionContext::Company(_), "delete", "task") => Ok(()),

            // Project context - can manage tasks and projects
            (ExecutionContext::Project(_, _), "create", "task") => Ok(()),
            (ExecutionContext::Project(_, _), "list", "tasks") => Ok(()),
            (ExecutionContext::Project(_, _), "update", "task") => Ok(()),
            (ExecutionContext::Project(_, _), "update", "project") => Ok(()),
            (ExecutionContext::Project(_, _), "update", "resource") => Ok(()),
            (ExecutionContext::Project(_, _), "delete", "task") => Ok(()),
            (ExecutionContext::Project(_, _), "delete", "resource") => Ok(()),
            (ExecutionContext::Project(_, _), "delete", "project") => Ok(()),

            // Invalid combinations
            _ => Err(ExecutionContextError::NoContextFound {
                path: format!(
                    "Command '{} {}' not valid in context: {}",
                    command,
                    entity,
                    self.display_name()
                ),
            }),
        }
    }

    /// Returns the relative path prefix for assets based on context
    pub fn asset_path_prefix(&self) -> String {
        match self {
            ExecutionContext::Root => "".to_string(),
            ExecutionContext::Company(_) => "../".to_string(),
            ExecutionContext::Project(_, _) => "../../".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_test_environment() -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create config.yaml
        let mut config_file = File::create(root.join("config.yaml")).unwrap();
        writeln!(config_file, "apiVersion: tasktaskrevolution.io/v1alpha1\nkind: Config").unwrap();

        // Create company directory structure
        let company_dir = root.join("companies").join("test-company");
        std::fs::create_dir_all(&company_dir).unwrap();

        // Create company.yaml
        let company_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  code: "TEST-COMP"
  name: "Test Company"
"#;
        let mut company_file = File::create(company_dir.join("company.yaml")).unwrap();
        writeln!(company_file, "{}", company_content).unwrap();

        // Create project directory structure
        let project_dir = company_dir.join("projects").join("test-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create project.yaml
        let project_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "test-proj"
  name: "Test Project"
"#;
        let mut project_file = File::create(project_dir.join("project.yaml")).unwrap();
        writeln!(project_file, "{}", project_content).unwrap();

        // Persist the temporary directory for inspection after the test
        let _ = temp_dir.keep();
        root
    }

    #[test]
    fn test_detect_root_context() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create config.yaml
        let mut config_file = File::create(root.join("config.yaml")).unwrap();
        writeln!(config_file, "apiVersion: tasktaskrevolution.io/v1alpha1\nkind: Config").unwrap();

        let context = ExecutionContext::detect(root).unwrap();
        assert_eq!(context, ExecutionContext::Root);
        assert_eq!(context.display_name(), "Root (Global)");
        assert_eq!(context.company_code(), None);
        assert_eq!(context.project_code(), None);
    }

    #[test]
    fn test_detect_company_context() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create company.yaml
        let company_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  code: "TECH"
  name: "TechCorp"
"#;
        let mut company_file = File::create(root.join("company.yaml")).unwrap();
        writeln!(company_file, "{}", company_content).unwrap();

        let context = ExecutionContext::detect(root).unwrap();
        assert_eq!(context, ExecutionContext::Company("TECH".to_string()));
        assert_eq!(context.display_name(), "Company: TECH");
        assert_eq!(context.company_code(), Some("TECH".to_string()));
        assert_eq!(context.project_code(), None);
    }

    #[test]
    fn test_detect_project_context() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create project.yaml
        let project_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-1"
  name: "Test Project"
"#;
        let mut project_file = File::create(root.join("project.yaml")).unwrap();
        writeln!(project_file, "{}", project_content).unwrap();

        // This should fail because we can't determine the company from just project.yaml
        let result = ExecutionContext::detect(root);
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionContextError::NoContextFound { path } => {
                assert!(!path.is_empty());
            }
            _ => panic!("Expected NoContextFound error"),
        }
    }

    #[test]
    fn test_detect_project_context_with_company_path() {
        let temp_root = setup_test_environment();
        let project_dir = temp_root
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("test-project");

        let context = ExecutionContext::detect(&project_dir).unwrap();
        assert_eq!(
            context,
            ExecutionContext::Project("test-company".to_string(), "test-proj".to_string())
        );
        assert_eq!(context.display_name(), "Project: test-proj in test-company");
        assert_eq!(context.company_code(), Some("test-company".to_string()));
        assert_eq!(context.project_code(), Some("test-proj".to_string()));
    }

    #[test]
    fn test_no_context_found() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create a random file
        let mut random_file = File::create(root.join("random.txt")).unwrap();
        writeln!(random_file, "random content").unwrap();

        let result = ExecutionContext::detect(root);
        assert!(result.is_err());
        match result.unwrap_err() {
            ExecutionContextError::NoContextFound { path } => {
                assert!(!path.is_empty());
            }
            _ => panic!("Expected NoContextFound error"),
        }
    }

    #[test]
    fn test_validate_command() {
        let root_context = ExecutionContext::Root;
        let company_context = ExecutionContext::Company("TECH".to_string());
        let project_context = ExecutionContext::Project("TECH".to_string(), "proj-1".to_string());

        // Root context can do everything
        assert!(root_context.validate_command("create", "company").is_ok());
        assert!(root_context.validate_command("create", "project").is_ok());
        assert!(root_context.validate_command("create", "task").is_ok());

        // Company context can manage projects and resources
        assert!(company_context.validate_command("create", "project").is_ok());
        assert!(company_context.validate_command("create", "resource").is_ok());
        assert!(company_context.validate_command("list", "projects").is_ok());

        // Project context can manage tasks
        assert!(project_context.validate_command("create", "task").is_ok());
        assert!(project_context.validate_command("list", "tasks").is_ok());

        // Invalid combinations
        assert!(company_context.validate_command("create", "task").is_err());
        assert!(project_context.validate_command("create", "project").is_err());
    }

    #[test]
    fn test_asset_path_prefixes() {
        assert_eq!(ExecutionContext::Root.asset_path_prefix(), "");
        assert_eq!(ExecutionContext::Company("TECH".to_string()).asset_path_prefix(), "../");
        assert_eq!(
            ExecutionContext::Project("TECH".to_string(), "proj-1".to_string()).asset_path_prefix(),
            "../../"
        );
    }
}
