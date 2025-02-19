use std::fs;
use std::path::PathBuf;
use crate::domain::task::Task;
use crate::domain::task::TaskRepository;
use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;

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
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        let manifest: TaskManifest = task.clone().into();
        let yaml = serde_yaml::to_string(&manifest)?;
        
        let task_path = self.get_task_path(&task.id.to_string());
        
        // Cria o diretório tasks se não existir
        if let Some(parent) = task_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(task_path, yaml)?;
        
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_file_task_repository_save() {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileTaskRepository::new(temp_dir.path().to_path_buf());
        
        let task = Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
        );
        
        let saved_task = repository.save(task.clone()).unwrap();
        assert_eq!(saved_task, task);
        
        let task_path = repository.get_task_path(&task.id.to_string());
        assert!(task_path.exists());
        
        let content = fs::read_to_string(task_path).unwrap();
        assert!(content.contains("apiVersion: v1"));
        assert!(content.contains(&task.name));
        assert!(content.contains(task.description.as_ref().unwrap()));
    }
} 