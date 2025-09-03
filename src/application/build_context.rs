use serde::Deserialize;
use serde_yaml;
use std::path::Path;

/// Represents the different contexts where the build command can be executed
#[derive(Debug, Clone, PartialEq)]
pub enum BuildContext {
    /// Root context: config.yaml is in the current directory
    Root,
    /// Company context: company.yaml is in the current directory
    Company(String), // company code
    /// Project context: project.yaml is in the current directory
    Project(String), // project code
}

/// Error types for build context detection
#[derive(Debug, Clone, PartialEq)]
pub enum BuildContextError {
    NoContextFound { path: String },
    InvalidYaml { file: String, error: String },
    IoError { error: String },
}

impl std::fmt::Display for BuildContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildContextError::NoContextFound { path } => {
                write!(f, "No valid build context found in directory: {}", path)
            }
            BuildContextError::InvalidYaml { file, error } => {
                write!(f, "Invalid YAML file: {} - {}", file, error)
            }
            BuildContextError::IoError { error } => {
                write!(f, "IO error: {}", error)
            }
        }
    }
}

impl std::error::Error for BuildContextError {}

/// Metadata structure for company.yaml files
#[derive(Debug, Deserialize)]
struct CompanyMetadata {
    code: String,
}

/// Metadata structure for project.yaml files
#[derive(Debug, Deserialize)]
struct ProjectMetadata {
    code: String,
}

/// Manifest structure for company.yaml files
#[derive(Debug, Deserialize)]
struct CompanyManifest {
    metadata: CompanyMetadata,
}

/// Manifest structure for project.yaml files
#[derive(Debug, Deserialize)]
struct ProjectManifest {
    metadata: ProjectMetadata,
}

impl BuildContext {
    /// Detects the build context by analyzing the current directory structure
    pub fn detect(path: &Path) -> Result<Self, BuildContextError> {
        let path_str = path.to_string_lossy().to_string();

        // Check for config.yaml (root context)
        if path.join("config.yaml").exists() {
            return Ok(BuildContext::Root);
        }

        // Check for company.yaml (company context)
        if let Some(company_code) = Self::find_company_yaml(path)? {
            return Ok(BuildContext::Company(company_code));
        }

        // Check for project.yaml (project context)
        if let Some(project_code) = Self::find_project_yaml(path)? {
            return Ok(BuildContext::Project(project_code));
        }

        Err(BuildContextError::NoContextFound { path: path_str })
    }

    /// Finds and parses company.yaml file in the given path
    fn find_company_yaml(path: &Path) -> Result<Option<String>, BuildContextError> {
        let company_yaml_path = path.join("company.yaml");

        if !company_yaml_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&company_yaml_path).map_err(|e| BuildContextError::IoError {
            error: format!("Failed to read company.yaml: {}", e),
        })?;

        let manifest: CompanyManifest = serde_yaml::from_str(&content).map_err(|e| BuildContextError::InvalidYaml {
            file: "company.yaml".to_string(),
            error: e.to_string(),
        })?;

        Ok(Some(manifest.metadata.code))
    }

    /// Finds and parses project.yaml file in the given path
    fn find_project_yaml(path: &Path) -> Result<Option<String>, BuildContextError> {
        let project_yaml_path = path.join("project.yaml");

        if !project_yaml_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&project_yaml_path).map_err(|e| BuildContextError::IoError {
            error: format!("Failed to read project.yaml: {}", e),
        })?;

        let manifest: ProjectManifest = serde_yaml::from_str(&content).map_err(|e| BuildContextError::InvalidYaml {
            file: "project.yaml".to_string(),
            error: e.to_string(),
        })?;

        Ok(Some(manifest.metadata.code))
    }

    /// Returns the display name for the context
    pub fn display_name(&self) -> String {
        match self {
            BuildContext::Root => "Global Dashboard".to_string(),
            BuildContext::Company(code) => format!("Company: {}", code),
            BuildContext::Project(code) => format!("Project: {}", code),
        }
    }

    /// Returns the relative path prefix for assets based on context
    #[allow(dead_code)]
    pub fn asset_path_prefix(&self) -> String {
        match self {
            BuildContext::Root => "".to_string(),
            BuildContext::Company(_) => "../".to_string(),
            BuildContext::Project(_) => "../../".to_string(),
        }
    }

    /// Returns the output directory structure based on context
    #[allow(dead_code)]
    pub fn output_structure(&self) -> OutputStructure {
        match self {
            BuildContext::Root => OutputStructure {
                index_path: "index.html".to_string(),
                projects_base: "companies".to_string(),
                assets_base: "assets".to_string(),
            },
            BuildContext::Company(_) => OutputStructure {
                index_path: "index.html".to_string(),
                projects_base: "projects".to_string(),
                assets_base: "assets".to_string(),
            },
            BuildContext::Project(_) => OutputStructure {
                index_path: "index.html".to_string(),
                projects_base: "".to_string(), // Not applicable for project context
                assets_base: "assets".to_string(),
            },
        }
    }
}

/// Defines the output directory structure for different contexts
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OutputStructure {
    pub index_path: String,
    pub projects_base: String,
    pub assets_base: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_detect_root_context() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create config.yaml
        let mut config_file = File::create(root.join("config.yaml")).unwrap();
        writeln!(config_file, "apiVersion: tasktaskrevolution.io/v1alpha1\nkind: Config").unwrap();

        let context = BuildContext::detect(root).unwrap();
        assert_eq!(context, BuildContext::Root);
    }

    #[test]
    fn test_detect_company_context() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create company.yaml
        let company_content = r#"
apiVersion: company.tasktaskrevolution.io/v1
kind: Company
metadata:
  code: "TECH"
  name: "TechCorp"
"#;
        let mut company_file = File::create(root.join("company.yaml")).unwrap();
        writeln!(company_file, "{}", company_content).unwrap();

        let context = BuildContext::detect(root).unwrap();
        assert_eq!(context, BuildContext::Company("TECH".to_string()));
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

        let context = BuildContext::detect(root).unwrap();
        assert_eq!(context, BuildContext::Project("proj-1".to_string()));
    }

    #[test]
    fn test_no_context_found() {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path();

        // Create a random file
        let mut random_file = File::create(root.join("random.txt")).unwrap();
        writeln!(random_file, "random content").unwrap();

        let result = BuildContext::detect(root);
        assert!(result.is_err());
        match result.unwrap_err() {
            BuildContextError::NoContextFound { path } => {
                // The path should contain the temp directory path
                assert!(!path.is_empty());
            }
            _ => panic!("Expected NoContextFound error"),
        }
    }

    #[test]
    fn test_display_names() {
        assert_eq!(BuildContext::Root.display_name(), "Global Dashboard");
        assert_eq!(
            BuildContext::Company("TECH".to_string()).display_name(),
            "Company: TECH"
        );
        assert_eq!(
            BuildContext::Project("proj-1".to_string()).display_name(),
            "Project: proj-1"
        );
    }

    #[test]
    fn test_asset_path_prefixes() {
        assert_eq!(BuildContext::Root.asset_path_prefix(), "");
        assert_eq!(BuildContext::Company("TECH".to_string()).asset_path_prefix(), "../");
        assert_eq!(
            BuildContext::Project("proj-1".to_string()).asset_path_prefix(),
            "../../"
        );
    }
}
