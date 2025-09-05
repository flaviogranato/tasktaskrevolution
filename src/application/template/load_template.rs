use crate::domain::project_management::ProjectTemplate;
use serde_yaml;
use std::fs;
use std::path::Path;

pub struct LoadTemplateUseCase;

impl LoadTemplateUseCase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadTemplateUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl LoadTemplateUseCase {
    pub fn execute(&self, template_path: &Path) -> Result<ProjectTemplate, String> {
        if !template_path.exists() {
            return Err(format!("Template file does not exist: {:?}", template_path));
        }

        let content = fs::read_to_string(template_path).map_err(|e| format!("Failed to read template file: {}", e))?;

        let template: ProjectTemplate =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse template YAML: {}", e))?;

        // Validate template
        template
            .validate()
            .map_err(|e| format!("Template validation failed: {}", e))?;

        Ok(template)
    }

    pub fn load_by_name(&self, templates_dir: &Path, template_name: &str) -> Result<ProjectTemplate, String> {
        let template_path = templates_dir.join(format!("{}.yaml", template_name));
        self.execute(&template_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_load_template_not_found() {
        let use_case = LoadTemplateUseCase::new();
        let result = use_case.execute(&PathBuf::from("nonexistent.yaml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_load_template_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let template_path = temp_dir.path().join("invalid.yaml");
        fs::write(&template_path, "invalid: yaml: content: [").unwrap();

        let use_case = LoadTemplateUseCase::new();
        let result = use_case.execute(&template_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse template YAML"));
    }
}
