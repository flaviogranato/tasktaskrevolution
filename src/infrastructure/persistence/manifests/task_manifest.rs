use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::task::Task;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifest {
    pub api_version: String,
    pub metadata: TaskMetadata,
    pub spec: TaskSpec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskSpec {
    pub name: String,
    pub description: Option<String>,
}

impl From<Task> for TaskManifest {
    fn from(task: Task) -> Self {
        Self {
            api_version: "v1".to_string(),
            metadata: TaskMetadata {
                id: task.id,
                created_at: None,
                updated_at: None,
            },
            spec: TaskSpec {
                name: task.name,
                description: task.description,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;

    #[test]
    fn test_task_manifest_serialization() {
        let task = Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
        );
        let manifest: TaskManifest = task.into();

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        
        // O YAML deve seguir o formato:
        // apiVersion: v1
        // metadata:
        //   id: <uuid>
        // spec:
        //   name: Test Task
        //   description: Test Description
        assert!(yaml.contains("apiVersion: v1"));
        assert!(yaml.contains("metadata:"));
        assert!(yaml.contains("spec:"));
        assert!(yaml.contains("name: Test Task"));
        assert!(yaml.contains("description: Test Description"));
        
        // Campos opcionais não devem aparecer se None
        assert!(!yaml.contains("created_at:"));
        assert!(!yaml.contains("updated_at:"));
    }

    #[test]
    fn test_task_manifest_deserialization() {
        let yaml = r#"
            apiVersion: v1
            metadata:
              id: 550e8400-e29b-41d4-a716-446655440000
              created_at: "2024-01-01"
              updated_at: "2024-01-02"
            spec:
              name: Test Task
              description: Test Description
        "#;

        let manifest: TaskManifest = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(manifest.api_version, "v1");
        assert_eq!(manifest.spec.name, "Test Task");
        assert_eq!(manifest.spec.description, Some("Test Description".to_string()));
        assert_eq!(manifest.metadata.created_at, Some("2024-01-01".to_string()));
        assert_eq!(manifest.metadata.updated_at, Some("2024-01-02".to_string()));
    }
} 