use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::{
    AnyTask, Task,
    repository::TaskRepository,
    state::{Completed, InProgress},
};
use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;
use chrono::NaiveDate;
use glob::glob;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A file-based implementation of the `TaskRepository`.
///
/// This repository stores tasks as individual YAML files within a specified base directory.
pub struct FileTaskRepository {
    base_path: PathBuf,
}

impl FileTaskRepository {
    /// Creates a new `FileTaskRepository` with a specified base path.
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        FileTaskRepository {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// A private helper to get the full path for a task's YAML file.
    fn get_task_file_path(&self, code: &str) -> PathBuf {
        self.get_tasks_directory().join(format!("{code}.yaml"))
    }

    /// A private helper to get the directory where tasks are stored.
    fn get_tasks_directory(&self) -> PathBuf {
        self.base_path.join("tasks")
    }

    /// Loads a `TaskManifest` from a given file path.
    fn load_manifest(&self, path: &Path) -> Result<TaskManifest, DomainError> {
        let file_content =
            fs::read_to_string(path).map_err(|e| DomainError::Io(format!("Failed to read task file: {e}")))?;
        serde_yaml::from_str(&file_content)
            .map_err(|e| DomainError::Serialization(format!("Failed to parse task YAML: {e}")))
    }

    /// Saves a `TaskManifest` to its corresponding file.
    fn save_manifest(&self, manifest: &TaskManifest) -> Result<(), DomainError> {
        let tasks_dir = self.get_tasks_directory();
        fs::create_dir_all(&tasks_dir)
            .map_err(|e| DomainError::Io(format!("Failed to create tasks directory: {e}")))?;

        let file_path = self.get_task_file_path(&manifest.metadata.code);
        let mut file =
            fs::File::create(file_path).map_err(|e| DomainError::Io(format!("Failed to create task file: {e}")))?;

        let yaml_string = serde_yaml::to_string(manifest)
            .map_err(|e| DomainError::Serialization(format!("Failed to serialize task: {e}")))?;

        file.write_all(yaml_string.as_bytes())
            .map_err(|e| DomainError::Io(format!("Failed to write to task file: {e}")))?;

        Ok(())
    }

    // --- Public, non-trait methods for specific operations ---

    pub fn update_progress(&self, code: &str, progress: u8) -> Result<AnyTask, DomainError> {
        let task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::NotFound(format!("Task with code '{code}' not found")))?;

        match task {
            AnyTask::InProgress(t) => {
                let updated_task = t.update_progress(progress);
                let any_task = AnyTask::from(updated_task);
                self.save(any_task.clone())?;
                Ok(any_task)
            }
            _ => Err(DomainError::InvalidState(
                "Task is not in progress, cannot update progress".to_string(),
            )),
        }
    }

    pub fn complete_task(&self, code: &str) -> Result<Task<Completed>, DomainError> {
        let task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::NotFound(format!("Task with code '{code}' not found")))?;

        match task {
            AnyTask::InProgress(t) => {
                let completed_task = t.complete();
                self.save(AnyTask::from(completed_task.clone()))?;
                Ok(completed_task)
            }
            _ => Err(DomainError::InvalidState(
                "Task can only be completed from the InProgress state".to_string(),
            )),
        }
    }
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: AnyTask) -> Result<(), DomainError> {
        let task_manifest = TaskManifest::from(task);
        self.save_manifest(&task_manifest)
    }

    fn load(&self, path: &Path) -> Result<AnyTask, DomainError> {
        let manifest = self.load_manifest(path)?;
        AnyTask::try_from(manifest).map_err(|e| DomainError::Serialization(format!("Conversion failed: {e}")))
    }

    fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
        let file_path = self.get_task_file_path(code);
        if !file_path.exists() {
            return Ok(None);
        }
        let manifest = self.load_manifest(&file_path)?;
        let task =
            AnyTask::try_from(manifest).map_err(|e| DomainError::Serialization(format!("Conversion failed: {e}")))?;
        Ok(Some(task))
    }

    fn find_by_id(&self, id: &str) -> Result<Option<AnyTask>, DomainError> {
        let tasks = self.find_all()?;
        Ok(tasks.into_iter().find(|task| task.id() == id))
    }

    fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
        let tasks_dir = self.get_tasks_directory();
        if !tasks_dir.exists() {
            return Ok(Vec::new());
        }

        let pattern = tasks_dir.join("*.yaml");
        let walker = glob(pattern.to_str().unwrap())
            .map_err(|e| DomainError::Generic(format!("Failed to read glob pattern: {e}")))?;

        let mut tasks = Vec::new();
        for entry in walker.flatten() {
            if let Ok(task) = self.load(entry.as_path()) {
                tasks.push(task);
            } else {
                eprintln!("Warning: Could not load task from {:?}", entry.as_path());
            }
        }
        Ok(tasks)
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let task = self
            .find_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(format!("Task with id '{id}' not found for deletion")))?;
        let file_path = self.get_task_file_path(task.code());
        fs::remove_file(file_path).map_err(|e| DomainError::Io(format!("Failed to delete task file: {e}")))
    }

    fn find_by_assignee(&self, assignee: &str) -> Result<Vec<AnyTask>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| task.assigned_resources().contains(&assignee.to_string()))
            .collect())
    }

    fn find_by_date_range(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<AnyTask>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| {
                let task_start = task.start_date();
                let task_due = task.due_date();
                // Simple overlap check
                task_start <= end_date && task_due >= start_date
            })
            .collect())
    }
}

impl Default for FileTaskRepository {
    fn default() -> Self {
        Self::new(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_management::state::{InProgress, Planned};
    use tempfile::tempdir;

    fn create_planned_task(code: &str) -> Task<Planned> {
        Task {
            id: format!("id-{code}"),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: vec!["res-1".to_string()],
        }
    }

    #[test]
    fn test_save_and_find_by_code() {
        let dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(dir.path());
        let task = create_planned_task("TSK-001");

        // Save and retrieve
        repo.save(AnyTask::from(task.clone())).unwrap();
        let found_task_opt = repo.find_by_code("TSK-001").unwrap();

        assert!(found_task_opt.is_some());
        let found_any_task = found_task_opt.unwrap();

        // Assert fields
        assert_eq!(found_any_task.id(), task.id);
        assert_eq!(found_any_task.code(), task.code);
        assert!(matches!(found_any_task, AnyTask::Planned(_)));
    }

    #[test]
    fn test_find_all() {
        let dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(dir.path());

        let task1 = create_planned_task("TSK-001");
        let task2 = create_planned_task("TSK-002");

        repo.save(task1.into()).unwrap();
        repo.save(task2.into()).unwrap();

        let all_tasks = repo.find_all().unwrap();
        assert_eq!(all_tasks.len(), 2);
    }

    #[test]
    fn test_update_progress_success() {
        let dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(dir.path());

        // Create a planned task, start it, and save it
        let task: Task<Planned> = create_planned_task("TSK-PROGRESS");
        let in_progress_task: Task<InProgress> = task.start();
        repo.save(in_progress_task.into()).unwrap();

        // Update progress
        let updated_task_any = repo.update_progress("TSK-PROGRESS", 50).unwrap();

        // Verify
        match updated_task_any {
            AnyTask::InProgress(t) => assert_eq!(t.state.progress, 50),
            _ => panic!("Task should be in InProgress state"),
        }

        // Verify it was saved correctly
        let reloaded_task_any = repo.find_by_code("TSK-PROGRESS").unwrap().unwrap();
        match reloaded_task_any {
            AnyTask::InProgress(t) => assert_eq!(t.state.progress, 50),
            _ => panic!("Reloaded task should be in InProgress state"),
        }
    }

    #[test]
    fn test_update_progress_on_planned_task_fails() {
        let dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(dir.path());

        let task = create_planned_task("TSK-FAIL");
        repo.save(task.into()).unwrap();

        let result = repo.update_progress("TSK-FAIL", 50);
        assert!(result.is_err());
        matches!(result.unwrap_err(), DomainError::InvalidState(_));
    }
}
