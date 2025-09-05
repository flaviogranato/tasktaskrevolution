use crate::domain::project_management::ProjectTemplate;
use serde_yaml;
use std::fs;
use std::path::Path;

pub struct ListTemplatesUseCase;

impl ListTemplatesUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, templates_dir: &Path) -> Result<Vec<TemplateInfo>, String> {
        if !templates_dir.exists() {
            return Err(format!("Templates directory does not exist: {:?}", templates_dir));
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(templates_dir).map_err(|e| format!("Failed to read templates directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_template_info(&path) {
                    Ok(template_info) => templates.push(template_info),
                    Err(e) => eprintln!("Warning: Failed to load template {:?}: {}", path, e),
                }
            }
        }

        // Sort templates by name
        templates.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(templates)
    }

    fn load_template_info(&self, path: &Path) -> Result<TemplateInfo, String> {
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read template file: {}", e))?;

        let template: ProjectTemplate =
            serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse template YAML: {}", e))?;

        // Validate template
        template
            .validate()
            .map_err(|e| format!("Template validation failed: {}", e))?;

        Ok(TemplateInfo {
            name: template.metadata.name,
            description: template.metadata.description,
            version: template.metadata.version,
            tags: template.metadata.tags,
            category: template.metadata.category,
            file_path: path.to_path_buf(),
            resource_count: template.spec.resources.len(),
            task_count: template.spec.tasks.len(),
            phase_count: template.spec.phases.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub category: String,
    pub file_path: std::path::PathBuf,
    pub resource_count: usize,
    pub task_count: usize,
    pub phase_count: usize,
}

impl TemplateInfo {
    pub fn display_name(&self) -> String {
        format!("{} (v{})", self.name, self.version)
    }

    pub fn display_tags(&self) -> String {
        self.tags.join(", ")
    }

    pub fn display_summary(&self) -> String {
        format!(
            "{} - {} resources, {} tasks, {} phases",
            self.description, self.resource_count, self.task_count, self.phase_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_template_info_display() {
        let info = TemplateInfo {
            name: "Web App".to_string(),
            description: "A web application template".to_string(),
            version: "1.0.0".to_string(),
            tags: vec!["web".to_string(), "frontend".to_string()],
            category: "application".to_string(),
            file_path: PathBuf::from("test.yaml"),
            resource_count: 3,
            task_count: 5,
            phase_count: 2,
        };

        assert_eq!(info.display_name(), "Web App (v1.0.0)");
        assert_eq!(info.display_tags(), "web, frontend");
        assert_eq!(
            info.display_summary(),
            "A web application template - 3 resources, 5 tasks, 2 phases"
        );
    }
}
