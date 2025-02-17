use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl Task {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_task_with_description() {
        let name = "Test Task".to_string();
        let description = Some("Test Description".to_string());
        let task = Task::new(name.clone(), description.clone());

        assert_eq!(task.name, name);
        assert_eq!(task.description, description);
        assert!(task.id.to_string().len() > 0);
    }

    #[test]
    fn test_new_task_without_description() {
        let name = "Test Task".to_string();
        let task = Task::new(name.clone(), None);

        assert_eq!(task.name, name);
        assert_eq!(task.description, None);
        assert!(task.id.to_string().len() > 0);
    }

    #[test]
    fn test_task_clone() {
        let original = Task::new("Original Task".to_string(), Some("Description".to_string()));
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.description, cloned.description);
    }

    #[test]
    fn test_task_debug_output() {
        let task = Task::new(
            "Debug Test".to_string(),
            Some("Test debug output".to_string()),
        );
        
        let debug_output = format!("{:?}", task);
        assert!(debug_output.contains("Debug Test"));
        assert!(debug_output.contains("Test debug output"));
        assert!(debug_output.contains(&task.id.to_string()));
    }
} 