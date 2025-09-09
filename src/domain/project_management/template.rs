use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub metadata: TemplateMetadata,
    pub spec: TemplateSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub tags: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSpec {
    pub project: TemplateProject,
    pub resources: Vec<TemplateResource>,
    pub tasks: Vec<TemplateTask>,
    pub phases: Vec<TemplatePhase>,
    pub variables: HashMap<String, TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateProject {
    pub name: String,
    pub description: String,
    pub start_date: String,
    pub end_date: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateResource {
    pub name: String,
    pub r#type: String,
    pub skills: Vec<String>,
    pub capacity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTask {
    pub name: String,
    pub description: String,
    pub priority: String,
    pub category: String,
    pub estimated_hours: u32,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePhase {
    pub name: String,
    pub duration: u32, // weeks
    pub tasks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub r#type: String,
    pub required: bool,
    pub description: String,
    pub example: String,
    pub default: Option<String>,
}

impl ProjectTemplate {
    pub fn new(
        name: String,
        description: String,
        version: String,
        tags: Vec<String>,
        category: String,
        spec: TemplateSpec,
    ) -> Self {
        Self {
            metadata: TemplateMetadata {
                name,
                description,
                version,
                tags,
                category,
            },
            spec,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate required fields
        if self.metadata.name.is_empty() {
            return Err("Template name cannot be empty".to_string());
        }

        if self.metadata.version.is_empty() {
            return Err("Template version cannot be empty".to_string());
        }

        // Validate project template
        if self.spec.project.name.is_empty() {
            return Err("Project name template cannot be empty".to_string());
        }

        // Validate resources
        for (i, resource) in self.spec.resources.iter().enumerate() {
            if resource.name.is_empty() {
                return Err(format!("Resource {} name cannot be empty", i));
            }
            if resource.r#type.is_empty() {
                return Err(format!("Resource {} type cannot be empty", i));
            }
        }

        // Validate tasks
        for (i, task) in self.spec.tasks.iter().enumerate() {
            if task.name.is_empty() {
                return Err(format!("Task {} name cannot be empty", i));
            }
            if task.estimated_hours == 0 {
                return Err(format!("Task {} estimated hours must be greater than 0", i));
            }
        }

        // Validate phases
        for (i, phase) in self.spec.phases.iter().enumerate() {
            if phase.name.is_empty() {
                return Err(format!("Phase {} name cannot be empty", i));
            }
            if phase.duration == 0 {
                return Err(format!("Phase {} duration must be greater than 0", i));
            }
        }

        // Validate variables
        for (name, variable) in &self.spec.variables {
            if name.is_empty() {
                return Err("Variable name cannot be empty".to_string());
            }
            if variable.description.is_empty() {
                return Err(format!("Variable {} description cannot be empty", name));
            }
        }

        Ok(())
    }

    pub fn render(&self, variables: &HashMap<String, String>) -> Result<RenderedTemplate, String> {
        let mut rendered = RenderedTemplate {
            project: self.render_project(variables)?,
            resources: Vec::new(),
            tasks: Vec::new(),
            phases: self.spec.phases.clone(),
        };

        // Render resources
        for resource in &self.spec.resources {
            rendered.resources.push(self.render_resource(resource, variables)?);
        }

        // Render tasks
        for task in &self.spec.tasks {
            rendered.tasks.push(self.render_task(task, variables)?);
        }

        Ok(rendered)
    }

    fn render_project(&self, variables: &HashMap<String, String>) -> Result<RenderedProject, String> {
        Ok(RenderedProject {
            name: self.render_string(&self.spec.project.name, variables)?,
            description: self.render_string(&self.spec.project.description, variables)?,
            start_date: self.render_string(&self.spec.project.start_date, variables)?,
            end_date: self.render_string(&self.spec.project.end_date, variables)?,
            timezone: self.render_string(&self.spec.project.timezone, variables)?,
        })
    }

    fn render_resource(
        &self,
        resource: &TemplateResource,
        variables: &HashMap<String, String>,
    ) -> Result<RenderedResource, String> {
        Ok(RenderedResource {
            name: self.render_string(&resource.name, variables)?,
            r#type: self.render_string(&resource.r#type, variables)?,
            skills: resource.skills.clone(),
            capacity: resource.capacity,
        })
    }

    fn render_task(&self, task: &TemplateTask, variables: &HashMap<String, String>) -> Result<RenderedTask, String> {
        Ok(RenderedTask {
            name: self.render_string(&task.name, variables)?,
            description: self.render_string(&task.description, variables)?,
            priority: task.priority.clone(),
            category: task.category.clone(),
            estimated_hours: task.estimated_hours,
            dependencies: task.dependencies.clone(),
        })
    }

    fn render_string(&self, template: &str, variables: &HashMap<String, String>) -> Result<String, String> {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
            // Find the first unresolved placeholder
            let start = result.find("{{").unwrap();
            let end = result[start..].find("}}").unwrap() + start + 2;
            let unresolved = &result[start..end];
            return Err(format!("Unresolved placeholders in template: {}", unresolved));
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct RenderedTemplate {
    pub project: RenderedProject,
    pub resources: Vec<RenderedResource>,
    pub tasks: Vec<RenderedTask>,
    pub phases: Vec<TemplatePhase>,
}

#[derive(Debug, Clone)]
pub struct RenderedProject {
    pub name: String,
    pub description: String,
    pub start_date: String,
    pub end_date: String,
    pub timezone: String,
}

#[derive(Debug, Clone)]
pub struct RenderedResource {
    pub name: String,
    pub r#type: String,
    pub skills: Vec<String>,
    pub capacity: u8,
}

#[derive(Debug, Clone)]
pub struct RenderedTask {
    pub name: String,
    pub description: String,
    pub priority: String,
    pub category: String,
    pub estimated_hours: u32,
    pub dependencies: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_template_validation() {
        let template = ProjectTemplate::new(
            "Test Template".to_string(),
            "Test Description".to_string(),
            "1.0.0".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
            TemplateSpec {
                project: TemplateProject {
                    name: "{{project_name}}".to_string(),
                    description: "{{description}}".to_string(),
                    start_date: "{{start_date}}".to_string(),
                    end_date: "{{end_date}}".to_string(),
                    timezone: "UTC".to_string(),
                },
                resources: vec![],
                tasks: vec![],
                phases: vec![],
                variables: HashMap::new(),
            },
        );

        assert!(template.validate().is_ok());
    }

    #[test]
    fn test_template_rendering() {
        let template = ProjectTemplate::new(
            "Test Template".to_string(),
            "Test Description".to_string(),
            "1.0.0".to_string(),
            vec!["test".to_string()],
            "test".to_string(),
            TemplateSpec {
                project: TemplateProject {
                    name: "{{project_name}}".to_string(),
                    description: "{{description}}".to_string(),
                    start_date: "{{start_date}}".to_string(),
                    end_date: "{{end_date}}".to_string(),
                    timezone: "UTC".to_string(),
                },
                resources: vec![],
                tasks: vec![],
                phases: vec![],
                variables: HashMap::new(),
            },
        );

        let mut variables = HashMap::new();
        variables.insert("project_name".to_string(), "My Project".to_string());
        variables.insert("description".to_string(), "My Description".to_string());
        variables.insert("start_date".to_string(), "2024-01-01".to_string());
        variables.insert("end_date".to_string(), "2024-12-31".to_string());

        let rendered = template.render(&variables).unwrap();
        assert_eq!(rendered.project.name, "My Project");
        assert_eq!(rendered.project.description, "My Description");
    }
}
