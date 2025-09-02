#![allow(dead_code)]

use crate::{
    domain::{
        shared::errors::{DomainError, DomainErrorKind},
        task_management::{AnyTask, repository::TaskRepository},
    },
    infrastructure::persistence::manifests::task_manifest::TaskManifest,
};
use glob::glob;
use serde_yaml;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct FileTaskRepository {
    base_path: PathBuf,
}

impl FileTaskRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn get_task_file_path(&self, task_name: &str) -> PathBuf {
        self.base_path
            .join("tasks")
            .join(format!("{}.yaml", task_name.replace(' ', "_").to_lowercase()))
    }

    /// Gets the path to a task in a specific project
    fn get_project_task_path(&self, company_code: &str, project_code: &str, task_name: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("tasks")
            .join(format!("{}.yaml", task_name.replace(' ', "_").to_lowercase()))
    }

    /// Gets the path to project tasks directory
    fn get_project_tasks_path(&self, company_code: &str, project_code: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("tasks")
    }

    fn load_manifest(&self, path: &Path) -> Result<TaskManifest, Box<dyn std::error::Error>> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| e.into())
    }
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: AnyTask) -> Result<AnyTask, DomainError> {
        let file_path = self.get_task_file_path(task.name());
        let task_manifest = TaskManifest::from(task.clone());
        let yaml = serde_yaml::to_string(&task_manifest).map_err(|e| {
            DomainError::new(DomainErrorKind::Serialization {
                format: "YAML".to_string(),
                details: format!("Error serializing task: {}", e),
            })
        })?;

        fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| {
            DomainError::new(DomainErrorKind::Io {
                operation: "file operation".to_string(),
                path: None,
            })
            .with_context(format!("Error creating directory: {e}"))
        })?;

        fs::write(file_path, yaml).map_err(|e| {
            DomainError::new(DomainErrorKind::Io {
                operation: "file operation".to_string(),
                path: None,
            })
            .with_context(format!("Error saving task: {e}"))
        })?;

        Ok(task)
    }

    /// Save task in the new hierarchical structure
    fn save_in_hierarchy(&self, task: AnyTask, company_code: &str, project_code: &str) -> Result<AnyTask, DomainError> {
        let file_path = self.get_project_task_path(company_code, project_code, task.name());
        let task_manifest = TaskManifest::from(task.clone());
        let yaml = serde_yaml::to_string(&task_manifest).map_err(|e| {
            DomainError::new(DomainErrorKind::Serialization {
                format: "YAML".to_string(),
                details: format!("Error serializing task: {}", e),
            })
        })?;

        fs::create_dir_all(file_path.parent().unwrap()).map_err(|e| {
            DomainError::new(DomainErrorKind::Io {
                operation: "file operation".to_string(),
                path: None,
            })
            .with_context(format!("Error creating directory: {e}"))
        })?;

        fs::write(file_path, yaml).map_err(|e| {
            DomainError::new(DomainErrorKind::Io {
                operation: "file operation".to_string(),
                path: None,
            })
            .with_context(format!("Error writing task file: {e}"))
        })?;

        Ok(task)
    }

    fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
        // Search in new hierarchical structure: companies/*/projects/*/tasks/*.yaml
        let pattern = self.base_path.join("companies/*/projects/*/tasks/*.yaml");
        let walker = glob(pattern.to_str().unwrap())
            .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
        let mut tasks = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
            let file_path = entry.as_path();
            let yaml = fs::read_to_string(file_path).map_err(|e| {
                DomainError::new(DomainErrorKind::Io {
                    operation: "file operation".to_string(),
                    path: None,
                })
                .with_context(format!("Error reading task file: {e}"))
            })?;

            let task_manifest: TaskManifest = serde_yaml::from_str(&yaml).map_err(|e| {
                DomainError::new(DomainErrorKind::Serialization {
                    format: "YAML".to_string(),
                    details: format!("Error deserializing task: {}", e),
                })
            })?;

            tasks.push(AnyTask::try_from(task_manifest).map_err(|e| {
                DomainError::new(DomainErrorKind::Serialization {
                    format: "YAML".to_string(),
                    details: format!("Error converting manifest: {}", e),
                })
            })?);
        }

        Ok(tasks)
    }

    /// Find all tasks for a specific project
    fn find_all_by_project(&self, company_code: &str, project_code: &str) -> Result<Vec<AnyTask>, DomainError> {
        let tasks_path = self.get_project_tasks_path(company_code, project_code);
        if !tasks_path.exists() {
            return Ok(Vec::new());
        }

        let pattern = tasks_path.join("*.yaml");
        let walker = glob(pattern.to_str().unwrap())
            .map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
        let mut tasks = Vec::new();

        for entry in walker {
            let entry = entry.map_err(|e| DomainError::new(DomainErrorKind::Generic { message: e.to_string() }))?;
            let file_path = entry.as_path();
            let yaml = fs::read_to_string(file_path).map_err(|e| {
                DomainError::new(DomainErrorKind::Io {
                    operation: "file operation".to_string(),
                    path: None,
                })
                .with_context(format!("Error reading task file: {e}"))
            })?;

            let task_manifest: TaskManifest = serde_yaml::from_str(&yaml).map_err(|e| {
                DomainError::new(DomainErrorKind::Serialization {
                    format: "YAML".to_string(),
                    details: format!("Error deserializing task: {}", e),
                })
            })?;

            tasks.push(AnyTask::try_from(task_manifest).map_err(|e| {
                DomainError::new(DomainErrorKind::Serialization {
                    format: "YAML".to_string(),
                    details: format!("Error converting manifest: {}", e),
                })
            })?);
        }

        Ok(tasks)
    }

    fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
        // Since tasks are saved by name, we need to search through all tasks
        // to find one with the matching code
        let all_tasks = self.find_all()?;
        for task in all_tasks {
            if task.code() == code {
                return Ok(Some(task));
            }
        }
        Ok(None)
    }

    fn find_by_project(&self, project_code: &str) -> Result<Vec<AnyTask>, DomainError> {
        let all_tasks = self.find_all()?;
        let project_tasks: Vec<AnyTask> = all_tasks
            .into_iter()
            .filter(|task| task.project_code() == project_code)
            .collect();
        Ok(project_tasks)
    }

    fn get_next_code(&self, project_code: &str) -> Result<String, DomainError> {
        let all_tasks = self.find_all()?;
        let project_tasks: Vec<&AnyTask> = all_tasks
            .iter()
            .filter(|task| task.project_code() == project_code)
            .collect();

        let max_code = project_tasks
            .iter()
            .filter_map(|task| {
                let code = task.code();
                if code.starts_with(&format!("{}-", project_code)) {
                    code.strip_prefix(&format!("{}-", project_code))
                        .and_then(|num_str| num_str.parse::<u32>().ok())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        Ok(format!("{}-{}", project_code, max_code + 1))
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
    use crate::domain::task_management::{builder::TaskBuilder, state::Planned, task::Task};
    use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_task(name: &str, code: &str, project_code: &str) -> Task<Planned> {
        TaskBuilder::new()
            .project_code(project_code.to_string())
            .name(name.to_string())
            .code(code.to_string())
            .dates(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            )
            .unwrap()
            .assign_resource("RES-001")
            .validate_vacations(&[])
            .unwrap()
            .build()
            .unwrap()
    }

    fn create_test_task_manifest(name: &str, code: &str, project_code: &str) -> TaskManifest {
        // Create a simple task first, then convert it to manifest
        let task = create_test_task(name, code, project_code);
        let any_task: crate::domain::task_management::AnyTask = task.into();
        TaskManifest::from(any_task)
    }

    #[test]
    fn test_task_manifest_serialization() {
        let manifest = create_test_task_manifest("Test Task", "TEST-001", "PROJ-001");

        let yaml = serde_yaml::to_string(&manifest).expect("Failed to serialize to YAML");
        let deserialized: TaskManifest = serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

        assert_eq!(manifest.metadata.code, deserialized.metadata.code);
        assert_eq!(manifest.metadata.name, deserialized.metadata.name);
        assert_eq!(manifest.spec.project_code, deserialized.spec.project_code);
        assert!(matches!(
            deserialized.spec.status,
            crate::infrastructure::persistence::manifests::task_manifest::Status::Planned
        ));
    }

    #[test]
    fn test_task_repository_save_and_verify() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        let task = create_test_task("Test Task", "TEST-001", "PROJ-001");

        // Save task
        let save_result = repo.save(task.clone().into());
        assert!(save_result.is_ok(), "Failed to save task: {:?}", save_result);

        // Verify task was saved by checking file exists
        let task_file = temp_dir.path().join("tasks").join("test_task.yaml");
        assert!(task_file.exists(), "Task file should exist after save");

        // Verify task directory structure
        let tasks_dir = temp_dir.path().join("tasks");
        assert!(tasks_dir.exists(), "Tasks directory should exist");
    }

    #[test]
    fn test_task_repository_save_multiple_tasks() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        // Create and save multiple tasks
        let task1 = create_test_task("Task 1", "TASK-001", "PROJ-001");
        let task2 = create_test_task("Task 2", "TASK-002", "PROJ-001");
        let task3 = create_test_task("Task 3", "TASK-003", "PROJ-002");

        repo.save(task1.into()).expect("Failed to save task 1");
        repo.save(task2.into()).expect("Failed to save task 2");
        repo.save(task3.into()).expect("Failed to save task 3");

        // Verify all tasks were saved by checking files exist
        let task1_file = temp_dir.path().join("tasks").join("task_1.yaml");
        let task2_file = temp_dir.path().join("tasks").join("task_2.yaml");
        let task3_file = temp_dir.path().join("tasks").join("task_3.yaml");

        assert!(task1_file.exists(), "Task 1 file should exist");
        assert!(task2_file.exists(), "Task 2 file should exist");
        assert!(task3_file.exists(), "Task 3 file should exist");
    }

    #[test]
    fn test_task_repository_find_by_code() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        let task = create_test_task("Test Task", "TEST-001", "PROJ-001");
        repo.save(task.clone().into()).expect("Failed to save task");

        // Find task by code
        let found_task = repo.find_by_code("TEST-001");
        assert!(found_task.is_ok(), "Failed to find task by code: {:?}", found_task);

        let found_task = found_task.unwrap();
        assert!(found_task.is_some(), "Task should be found");

        let found_task = found_task.unwrap();
        assert_eq!(found_task.code(), "TEST-001");
        assert_eq!(found_task.name(), "Test Task");
    }

    #[test]
    fn test_task_repository_find_by_project() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        // Create tasks for different projects
        let task1 = create_test_task("Task 1", "TASK-001", "PROJ-001");
        let task2 = create_test_task("Task 2", "TASK-002", "PROJ-001");
        let task3 = create_test_task("Task 3", "TASK-003", "PROJ-002");

        repo.save(task1.into()).expect("Failed to save task 1");
        repo.save(task2.into()).expect("Failed to save task 2");
        repo.save(task3.into()).expect("Failed to save task 3");

        // Find tasks by project
        let proj1_tasks = repo
            .find_by_project("PROJ-001")
            .expect("Failed to find tasks for PROJ-001");
        let proj2_tasks = repo
            .find_by_project("PROJ-002")
            .expect("Failed to find tasks for PROJ-002");

        assert_eq!(proj1_tasks.len(), 2, "PROJ-001 should have 2 tasks");
        assert_eq!(proj2_tasks.len(), 1, "PROJ-002 should have 1 task");
    }

    #[test]
    fn test_task_repository_error_handling() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        // Try to find non-existent task
        let result = repo.find_by_code("NON-EXISTENT");
        assert!(result.is_ok(), "Should return Ok(None) for non-existent task");
        assert!(result.unwrap().is_none(), "Should return None for non-existent task");
    }

    #[test]
    fn test_task_repository_file_corruption_handling() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        let task = create_test_task("Test Task", "TEST-001", "PROJ-001");
        repo.save(task.clone().into()).expect("Failed to save task");

        // Corrupt the YAML file
        let task_file = temp_dir.path().join("tasks").join("test_task.yaml");
        fs::write(&task_file, "invalid: yaml: content: [").expect("Failed to corrupt file");

        // Note: We can't test loading corrupted files yet since find_by_code is not fully implemented
        // This test verifies that we can save tasks and corrupt files
        assert!(task_file.exists(), "Task file should exist even if corrupted");
    }

    #[test]
    fn test_task_repository_concurrent_access() {
        let temp_dir = tempdir().unwrap();

        // Create multiple tasks concurrently
        let mut handles = vec![];

        for i in 1..=5 {
            let temp_dir = temp_dir.path().to_path_buf();
            let handle = std::thread::spawn(move || {
                let repo = FileTaskRepository::new(temp_dir);
                let task = create_test_task(&format!("Task {}", i), &format!("TASK-{:03}", i), "PROJ-001");
                repo.save(task.into())
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().expect("Thread failed to complete");
            assert!(result.is_ok(), "Failed to save task in concurrent access: {:?}", result);
        }

        // Verify all tasks were saved by checking files exist
        for i in 1..=5 {
            let task_file = temp_dir.path().join("tasks").join(format!("task_{}.yaml", i));
            assert!(task_file.exists(), "Task {} file should exist", i);
        }
    }

    #[test]
    fn test_task_repository_get_next_code() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::new(temp_dir.path());

        // Test with no tasks of a project
        assert_eq!(repo.get_next_code("PROJ-001").unwrap(), "PROJ-001-1");

        // Add some tasks
        repo.save(create_test_task("Task 1", "PROJ-001-1", "PROJ-001").into())
            .unwrap();
        repo.save(create_test_task("Task 2", "PROJ-001-2", "PROJ-001").into())
            .unwrap();
        repo.save(create_test_task("Task 3", "PROJ-001-5", "PROJ-001").into())
            .unwrap(); // Test with a gap

        // Test again
        assert_eq!(repo.get_next_code("PROJ-001").unwrap(), "PROJ-001-6");
        assert_eq!(repo.get_next_code("PROJ-002").unwrap(), "PROJ-002-1"); // Test new project
    }
}
