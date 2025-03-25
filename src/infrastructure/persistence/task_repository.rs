use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::task::Task;
use crate::domain::task::TaskRepository;
use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;
use std::fs;
use std::path::PathBuf;

pub struct FileTaskRepository {
    base_path: PathBuf,
}

impl FileTaskRepository {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn get_task_path(&self, id: &str) -> PathBuf {
        self.base_path.join("tasks").join(format!("{}.yaml", id))
    }

    fn generate_next_task_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        let tasks_dir = self.base_path.join("tasks");
        if !tasks_dir.exists() {
            fs::create_dir_all(&tasks_dir)?;
            return Ok("task-1".to_string());
        }

        let entries = fs::read_dir(&tasks_dir)?;
        let mut max_id = 0;

        for entry in entries {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.starts_with("task-") && file_name.ends_with(".yaml") {
                if let Some(id_str) = file_name
                    .strip_prefix("task-")
                    .and_then(|s| s.strip_suffix(".yaml"))
                {
                    if let Ok(id) = id_str.parse::<u32>() {
                        max_id = max_id.max(id);
                    }
                }
            }
        }

        Ok(format!("task-{}", max_id + 1))
    }

    pub fn load_tasks(&self) -> Result<Vec<Task>, std::io::Error> {
        let tasks_dir = self.base_path.join("tasks");
        if !tasks_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for entry in std::fs::read_dir(tasks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "yaml") {
                let contents = std::fs::read_to_string(&path)?;
                let manifest: TaskManifest = serde_yaml::from_str(&contents)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                tasks.push(manifest.to());
            }
        }

        Ok(tasks)
    }
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, mut task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        task.id = self.generate_next_task_id()?;
        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let yaml = serde_yaml::to_string(&manifest)?;

        let task_path = self.get_task_path(&task.id);

        fs::write(task_path, yaml)?;

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use tempfile::TempDir;

    fn format_datetime(dt: DateTime<Utc>) -> String {
        format!("{}", dt.format("%Y-%m-%dT%H:%M"))
    }

    #[test]
    fn test_file_task_repository_save() {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileTaskRepository::new(temp_dir.path().to_path_buf());

        let task = Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
            Some(Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap()),
        );

        let saved_task = repository.save(task).unwrap();
        assert_eq!(saved_task.id, "task-1");

        let task_path = repository.get_task_path(&saved_task.id);
        assert!(task_path.exists());

        let content = fs::read_to_string(task_path).unwrap();
        let expected_yaml = format!(
            r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: {}
  createdAt: {}
  dueDate: 2024-03-15
spec:
  name: Test Task
  description: Test Description"#,
            saved_task.id,
            format_datetime(saved_task.created_at)
        );

        assert_eq!(content.trim(), expected_yaml.trim());
    }
}
