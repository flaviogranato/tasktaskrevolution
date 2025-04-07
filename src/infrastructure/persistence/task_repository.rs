use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::task::Task;
use crate::domain::task::TaskRepository;
use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;
use std::fs;
use std::path::Path;
use std::io;
use serde_yaml;

pub struct FileTaskRepository {
    base_path: std::path::PathBuf,
}

impl FileTaskRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn get_task_path(&self, title: &str) -> std::path::PathBuf {
        let sanitized_title = title
            .to_lowercase()
            .replace(' ', "-")
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
        
        self.base_path
            .join("tasks")
            .join(format!("{}.yaml", sanitized_title))
    }

    fn ensure_task_directory(&self) -> io::Result<()> {
        let task_dir = self.base_path.join("tasks");
        if !task_dir.exists() {
            fs::create_dir_all(&task_dir)?;
        }
        Ok(())
    }
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
        self.ensure_task_directory()?;

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let task_path = self.get_task_path(task.title());
        
        let yaml = serde_yaml::to_string(&manifest)?;
        fs::write(&task_path, yaml)?;

        Ok(task)
    }

    fn find_by_title(&self, title: &str) -> Result<Option<Task>, Box<dyn std::error::Error>> {
        let task_path = self.get_task_path(title);
        
        if !task_path.exists() {
            return Ok(None);
        }

        let yaml = fs::read_to_string(&task_path)?;
        let manifest: TaskManifest = serde_yaml::from_str(&yaml)?;
        
        Ok(Some(manifest.to()))
    }

    fn list(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let task_dir = self.base_path.join("tasks");
        if !task_dir.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(task_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let yaml = fs::read_to_string(entry.path())?;
                let manifest: TaskManifest = serde_yaml::from_str(&yaml)?;
                tasks.push(manifest.to());
            }
        }

        Ok(tasks)
    }

    fn delete(&self, title: &str) -> Result<(), Box<dyn std::error::Error>> {
        let task_path = self.get_task_path(title);
        if task_path.exists() {
            fs::remove_file(task_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_find_task() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let repository = FileTaskRepository::new(temp_dir.path());

        let task = Task::new(
            "Test Task".to_string(),
            "Description".to_string(),
            Utc::now().naive_utc().date(),
        )?;

        repository.save(task.clone())?;

        let found = repository.find_by_title("Test Task")?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().title(), task.title());

        Ok(())
    }

    #[test]
    fn test_list_tasks() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let repository = FileTaskRepository::new(temp_dir.path());

        let task1 = Task::new(
            "Task 1".to_string(),
            "Description 1".to_string(),
            Utc::now().naive_utc().date(),
        )?;

        let task2 = Task::new(
            "Task 2".to_string(),
            "Description 2".to_string(),
            Utc::now().naive_utc().date(),
        )?;

        repository.save(task1.clone())?;
        repository.save(task2.clone())?;

        let tasks = repository.list()?;
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.title() == task1.title()));
        assert!(tasks.iter().any(|t| t.title() == task2.title()));

        Ok(())
    }

    #[test]
    fn test_delete_task() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let repository = FileTaskRepository::new(temp_dir.path());

        let task = Task::new(
            "Test Task".to_string(),
            "Description".to_string(),
            Utc::now().naive_utc().date(),
        )?;

        repository.save(task.clone())?;
        repository.delete("Test Task")?;

        let found = repository.find_by_title("Test Task")?;
        assert!(found.is_none());

        Ok(())
    }
}
