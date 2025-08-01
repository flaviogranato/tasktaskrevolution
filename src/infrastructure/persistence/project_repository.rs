use crate::{
    domain::{
        project_management::{AnyProject, repository::ProjectRepository},
        shared::errors::DomainError,
        task_management::any_task::AnyTask,
    },
    infrastructure::persistence::manifests::{project_manifest::ProjectManifest, task_manifest::TaskManifest},
};
use globwalk::glob;
use serde_yaml;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// `FileProjectRepository` é uma implementação da trait `ProjectRepository`
/// que persiste os dados do projeto no sistema de arquivos.
///
/// A estrutura de diretórios esperada é:
/// /<base_path>/<project_name>/project.yaml
pub struct FileProjectRepository {
    base_path: PathBuf,
}

impl FileProjectRepository {
    /// Cria uma nova instância do repositório que opera a partir do diretório de trabalho atual.
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    /// Cria uma nova instância do repositório que opera a partir de um diretório base específico.
    /// Esta função é primariamente para uso em testes.
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Carrega e deserializa o manifesto de um projeto de um arquivo YAML.
    fn load_manifest(&self, path: &Path) -> Result<ProjectManifest, Box<dyn Error>> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| e.into())
    }

    /// Loads tasks from the `tasks` subdirectory of a project and adds them.
    fn load_tasks_for_project(&self, project: &mut AnyProject, project_path: &Path) -> Result<(), DomainError> {
        let tasks_dir = project_path.parent().unwrap().join("tasks");
        if !tasks_dir.exists() {
            return Ok(());
        }

        let pattern = tasks_dir.join("*.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;

        for entry in walker.flatten() {
            let task_path = entry.path();
            let yaml =
                fs::read_to_string(task_path).map_err(|e| DomainError::Io(format!("Error reading task file: {e}")))?;
            let task_manifest: TaskManifest = serde_yaml::from_str(&yaml)
                .map_err(|e| DomainError::Serialization(format!("Error deserializing task: {e}")))?;
            let task = AnyTask::try_from(task_manifest).map_err(DomainError::Serialization)?;
            project.add_task(task);
        }

        Ok(())
    }
}

impl ProjectRepository for FileProjectRepository {
    /// Salva um projeto.
    /// Cria um diretório com o nome do projeto e salva um arquivo `project.yaml` dentro dele.
    fn save(&self, project: AnyProject) -> Result<(), DomainError> {
        let project_dir = self.base_path.join(project.name());

        // Save project manifest
        fs::create_dir_all(&project_dir)
            .map_err(|e| DomainError::Io(format!("Error creating project directory: {e}")))?;
        let manifest_path = project_dir.join("project.yaml");
        let project_manifest = ProjectManifest::from(project.clone());
        let yaml = serde_yaml::to_string(&project_manifest)
            .map_err(|e| DomainError::Serialization(format!("Error serializing project: {e}")))?;
        fs::write(&manifest_path, yaml).map_err(|e| DomainError::Io(format!("Error saving project file: {e}")))?;

        // Save tasks
        let tasks_dir = project_dir.join("tasks");
        fs::create_dir_all(&tasks_dir).map_err(|e| DomainError::Io(format!("Error creating tasks directory: {e}")))?;

        for task in project.tasks().values() {
            let task_manifest = TaskManifest::from(task.clone());
            let task_yaml = serde_yaml::to_string(&task_manifest)
                .map_err(|e| DomainError::Serialization(format!("Error serializing task: {e}")))?;
            let task_path = tasks_dir.join(format!("{}.yaml", task.code()));
            fs::write(task_path, task_yaml).map_err(|e| DomainError::Io(format!("Error saving task file: {e}")))?;
        }

        Ok(())
    }

    /// Carrega um projeto.
    /// `path` deve ser o caminho para o diretório do projeto.
    fn load(&self) -> Result<AnyProject, DomainError> {
        let pattern = self.base_path.join("**/project.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;

        if let Some(Ok(entry)) = walker.into_iter().next() {
            let manifest_path = entry.path();
            let manifest = self
                .load_manifest(manifest_path)
                .map_err(|e| DomainError::Generic(format!("Failed to load project manifest: {e}")))?;
            let mut project = AnyProject::try_from(manifest).map_err(DomainError::Serialization)?;
            self.load_tasks_for_project(&mut project, manifest_path)?;
            Ok(project)
        } else {
            Err(DomainError::NotFound(
                "No 'project.yaml' file found in subdirectories.".to_string(),
            ))
        }
    }

    fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
        let mut projects = Vec::new();
        let mut processed_paths = std::collections::HashSet::new();

        // Padrão para buscar em subdiretórios
        let pattern = self.base_path.join("**/project.yaml");
        if let Ok(walker) = glob(pattern.to_str().unwrap()) {
            for entry in walker.flatten() {
                let manifest_path = entry.path();
                if processed_paths.contains(manifest_path) {
                    continue;
                }
                if let Ok(manifest) = self.load_manifest(manifest_path) {
                    if let Ok(mut project) = AnyProject::try_from(manifest) {
                        if self.load_tasks_for_project(&mut project, manifest_path).is_ok() {
                            projects.push(project);
                            processed_paths.insert(manifest_path.to_path_buf());
                        }
                    }
                }
            }
        }

        // Verifica também o diretório atual
        let current_dir_manifest = self.base_path.join("project.yaml");
        if current_dir_manifest.exists() && !processed_paths.contains(&current_dir_manifest) {
            if let Ok(manifest) = self.load_manifest(&current_dir_manifest) {
                if let Ok(mut project) = AnyProject::try_from(manifest) {
                    if self.load_tasks_for_project(&mut project, &current_dir_manifest).is_ok() {
                        projects.push(project);
                    }
                }
            }
        }

        Ok(projects)
    }

    fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
        // This is not the most performant implementation, but it is correct.
        // It avoids duplicating the task loading logic.
        let projects = self.find_all()?;
        for project in projects {
            if project.code() == code {
                return Ok(Some(project));
            }
        }
        Ok(None)
    }

    fn get_next_code(&self) -> Result<String, DomainError> {
        let pattern = self.base_path.join("**/project.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;

        let mut max_code = 0;

        for entry in walker.flatten() {
            let manifest_path = entry.path();
            if let Ok(manifest) = self.load_manifest(manifest_path) {
                if let Some(code) = manifest.metadata.code {
                    if let Some(num_str) = code.strip_prefix("proj-") {
                        if let Ok(num) = num_str.parse::<u32>() {
                            if num > max_code {
                                max_code = num;
                            }
                        }
                    }
                }
            }
        }

        Ok(format!("proj-{}", max_code + 1))
    }
}

// ===================================
// TESTES
// ===================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::builder::ProjectBuilder,
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use tempfile::tempdir;
    use uuid7::uuid7;

    /// Creates a simple test project.
    fn create_test_project(name: &str, code: &str) -> AnyProject {
        let project = ProjectBuilder::new(name.to_string())
            .code(code.to_string())
            .description(Some(format!("Description for {name}")))
            .start_date("2025-01-01".to_string())
            .end_date("2025-12-31".to_string())
            .build();
        project.into()
    }

    fn create_test_task(code: &str, name: &str, project_code: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: project_code.to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: vec![],
        }
        .into()
    }

    #[test]
    fn test_save_and_load_project() {
        // 1. Setup
        let temp_dir = tempdir().expect("Could not create temporary directory");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());
        let mut original_project = create_test_project("MyTestProject", "proj-1");
        original_project.add_task(create_test_task("task-1", "My First Task", "proj-1"));
        let project_name = original_project.name().to_string();

        // 2. Save the project
        repo.save(original_project.clone()).unwrap();

        // 3. Check if file structure was created
        let project_dir_path = temp_dir.path().join(&project_name);
        assert!(project_dir_path.exists());
        assert!(project_dir_path.join("project.yaml").exists());
        assert!(project_dir_path.join("tasks").exists());
        assert!(project_dir_path.join("tasks/task-1.yaml").exists());

        // 4. Load the project back
        let loaded_project = repo.find_by_code("proj-1").unwrap().unwrap();

        // 5. Verify data consistency
        assert_eq!(original_project.name(), loaded_project.name());
        assert_eq!(loaded_project.tasks().len(), 1);
        assert_eq!(loaded_project.tasks()["task-1"].name(), "My First Task");
    }

    #[test]
    fn test_load_non_existent_project() {
        // 1. Setup
        let temp_dir = tempdir().expect("Could not create temporary directory");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());

        // 2. Try to load
        let result = repo.load();

        // 3. Verify
        assert!(result.is_err());
        if let Err(DomainError::NotFound(msg)) = result {
            assert!(msg.contains("No 'project.yaml' file found"));
        } else {
            panic!("Expected a DomainError::NotFound");
        }
    }

    #[test]
    fn test_get_next_code() {
        // 1. Setup
        let temp_dir = tempdir().expect("Could not create temporary directory");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());

        // 2. Test with no projects
        let next_code = repo.get_next_code().unwrap();
        assert_eq!(next_code, "proj-1");

        // 3. Save some projects
        repo.save(create_test_project("Project Alpha", "proj-1")).unwrap();
        repo.save(create_test_project("Project Beta", "proj-2")).unwrap();
        repo.save(create_test_project("Project Gamma", "proj-5")).unwrap(); // Test with a gap

        // 4. Test again
        let next_code_after_saves = repo.get_next_code().unwrap();
        assert_eq!(next_code_after_saves, "proj-6");
    }

    #[test]
    fn test_find_all_projects() {
        // 1. Setup
        let temp_dir = tempdir().expect("Could not create temporary directory");
        let repo = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());

        // 2. Test with no projects
        let projects = repo.find_all().unwrap();
        assert!(projects.is_empty());

        // 3. Save some projects in root and subdirectories
        repo.save(create_test_project("Project Root", "proj-1")).unwrap(); // Saved in ./Project Root/

        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let repo_sub = FileProjectRepository::with_base_path(sub_dir);
        repo_sub.save(create_test_project("Project Sub", "proj-2")).unwrap(); // Saved in ./subdir/Project Sub/

        // 4. Test find_all from root
        let all_projects = repo.find_all().unwrap();
        assert_eq!(all_projects.len(), 2);
        assert!(all_projects.iter().any(|p| p.name() == "Project Root"));
        assert!(all_projects.iter().any(|p| p.name() == "Project Sub"));

        // 5. Test find_all from inside a project directory (should find only itself)
        let project_dir_repo = FileProjectRepository::with_base_path(temp_dir.path().join("Project Root"));
        let single_project = project_dir_repo.find_all().unwrap();
        assert_eq!(single_project.len(), 1);
        assert_eq!(single_project[0].name(), "Project Root");
    }
}
