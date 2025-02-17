use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::error::Error;
use std::fmt;

use serde_yaml;

use crate::domain::task::{Task, TaskRepository};

pub struct FileTaskRepository {
    base_path: PathBuf,
}

// Custom error type
#[derive(Debug)]
pub enum TaskRepositoryError {
    FileOpenError(String),
    FileWriteError(String),
    YamlError(String),
    DirectoryCreationError(String),
}

impl fmt::Display for TaskRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskRepositoryError::FileOpenError(msg) => write!(f, "Erro ao abrir o arquivo: {}", msg),
            TaskRepositoryError::FileWriteError(msg) => write!(f, "Erro ao escrever no arquivo: {}", msg),
            TaskRepositoryError::YamlError(msg) => write!(f, "Erro ao serializar/desserializar YAML: {}", msg),
            TaskRepositoryError::DirectoryCreationError(msg) => write!(f, "Erro ao criar diretório: {}", msg),
        }
    }
}

impl Error for TaskRepositoryError {}

impl FileTaskRepository {
    pub fn new() -> Self {
        let mut base_path = PathBuf::from(std::env::current_dir().unwrap());
        base_path.push("tasks");
        Self { base_path }
    }
}

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: Task) -> Result<Task, Box<dyn Error>> {
        // Criar diretório tasks se não existir
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .map_err(|e| TaskRepositoryError::DirectoryCreationError(e.to_string()))?;
        }

        // Criar arquivo YAML para a task
        let file_name = format!("{}.yaml", task.id);
        let file_path = self.base_path.join(file_name);

        let yaml = serde_yaml::to_string(&task)
            .map_err(|e| TaskRepositoryError::YamlError(e.to_string()))?;

        let mut file = File::create(&file_path)
            .map_err(|e| TaskRepositoryError::FileOpenError(e.to_string()))?;

        file.write_all(yaml.as_bytes())
            .map_err(|e| TaskRepositoryError::FileWriteError(e.to_string()))?;

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_task_repository_save() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempdir()?;
        let repo = FileTaskRepository {
            base_path: temp_dir.path().to_path_buf(),
        };

        let task = Task::new("Test Task".to_string(), Some("Test Description".to_string()));
        let saved_task = repo.save(task.clone())?;

        let file_path = repo.base_path.join(format!("{}.yaml", saved_task.id));
        assert!(file_path.exists());

        let content = fs::read_to_string(file_path)?;
        let loaded_task: Task = serde_yaml::from_str(&content)?;

        assert_eq!(loaded_task.id, task.id);
        assert_eq!(loaded_task.name, task.name);
        assert_eq!(loaded_task.description, task.description);

        temp_dir.close()?;
        Ok(())
    }
} 