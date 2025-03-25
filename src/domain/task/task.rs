use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(name: String, description: Option<String>, due_date: Option<DateTime<Utc>>) -> Self {
        Self {
            id: String::new(),
            name,
            description,
            created_at: Utc::now(),
            due_date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_new_task_with_description() {
        let name = "Test Task".to_string();
        let description = Some("Test Description".to_string());
        let due_date = Some(Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap());
        let task = Task::new(name.clone(), description.clone(), due_date);

        assert_eq!(task.name, name);
        assert_eq!(task.description, description);
        assert_eq!(task.due_date, due_date);
        assert_eq!(task.id, "");
    }

    #[test]
    fn test_new_task_without_description() {
        let name = "Test Task".to_string();
        let task = Task::new(name.clone(), None, None);

        assert_eq!(task.name, name);
        assert_eq!(task.description, None);
        assert_eq!(task.id, ""); // ID começa vazio
    }

    #[test]
    fn test_task_clone() {
        let mut original = Task::new(
            "Original Task".to_string(),
            Some("Description".to_string()),
            None,
        );
        original.id = "task-1".to_string(); // Simulando um ID gerado
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.name, cloned.name);
        assert_eq!(original.description, cloned.description);
    }

    #[test]
    fn test_task_debug_output() {
        let mut task = Task::new(
            "Debug Test".to_string(),
            Some("Test debug output".to_string()),
            None,
        );
        task.id = "task-1".to_string(); // Simulando um ID gerado

        let debug_output = format!("{:?}", task);
        assert!(debug_output.contains("Debug Test"));
        assert!(debug_output.contains("Test debug output"));
        assert!(debug_output.contains(&task.id));
    }
}
