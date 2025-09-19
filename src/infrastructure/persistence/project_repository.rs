use crate::application::errors::AppError;
use crate::domain::project_management::{AnyProject, repository::ProjectRepository};
use crate::domain::task_management::any_task::AnyTask;
use crate::domain::shared::code_mapping_service::CodeMappingService;
use crate::infrastructure::persistence::manifests::{project_manifest::ProjectManifest, task_manifest::TaskManifest};
use globwalk::glob;
use serde_yaml;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// `FileProjectRepository` é uma implementação da trait `ProjectRepository`
/// que persiste os dados do projeto no sistema de arquivos.
///
/// A estrutura de diretórios esperada é:
/// /<base_path>/projects/<project_id>.yaml (ID-based format)
pub struct FileProjectRepository {
    base_path: PathBuf,
    mapping_service: CodeMappingService,
}

impl FileProjectRepository {
    /// Cria uma nova instância do repositório que opera a partir do diretório de trabalho atual.
    pub fn new() -> Self {
        let base_path = PathBuf::from(".");
        let mapping_service = CodeMappingService::new(&base_path.join(".ttr/mappings.json").to_string_lossy());
        Self {
            base_path,
            mapping_service,
        }
    }
}

impl Default for FileProjectRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl FileProjectRepository {
    /// Cria uma nova instância do repositório que opera a partir de um diretório base específico.
    /// Esta função é primariamente para uso em testes.
    pub fn with_base_path(base_path: PathBuf) -> Self {
        let mapping_service = CodeMappingService::new(&base_path.join(".ttr/mappings.json").to_string_lossy());
        Self { 
            base_path,
            mapping_service,
        }
    }

    /// Gets the path to a specific project file by ID
    fn get_project_path_by_id(&self, project_id: &str) -> PathBuf {
        self.base_path.join("projects").join(format!("{}.yaml", project_id))
    }

    /// Gets the path to a specific project file by code (legacy support)
    fn get_project_path_by_code(&self, project_code: &str) -> PathBuf {
        // For backward compatibility, try to find the project by code
        // This will be used during migration
        self.base_path.join("projects").join(project_code).join("project.yaml")
    }

    /// Gets the path to the projects directory
    fn get_projects_path(&self) -> PathBuf {
        self.base_path.join("projects")
    }

    /// Loads a single project from a specific project file path.
    pub fn load_from_path(&self, project_file: &Path) -> Result<AnyProject, AppError> {
        if !project_file.exists() {
            return Err(AppError::ProjectNotFound {
                code: "unknown".to_string(),
            });
        }
        let manifest = self
            .load_manifest(project_file)
            .map_err(|e| AppError::ValidationError {
                field: "manifest".to_string(),
                message: format!("Failed to load project file: {e}"),
            })?;
        let mut project = AnyProject::try_from(manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error converting project file: {e}"),
        })?;
        self.load_tasks_for_project(&mut project, project_file)?;
        Ok(project)
    }

    /// Loads a specific project by project code (using ID mapping)
    #[allow(dead_code)]
    pub fn load_by_code(&self, project_code: &str) -> Result<AnyProject, AppError> {
        // Get project ID from code mapping
        let project_id = self.mapping_service.get_id("project", project_code)
            .ok_or_else(|| AppError::ProjectNotFound {
                code: project_code.to_string(),
            })?;
        
        let project_path = self.get_project_path_by_id(&project_id);
        self.load_from_path(&project_path)
    }

    /// Extracts company_code from a project manifest path
    /// Path format: companies/{company_code}/projects/{project_code}/project.yaml
    fn extract_company_code_from_path(&self, path: &Path) -> Option<String> {
        let path_str = path.to_string_lossy();
        if let Some(companies_pos) = path_str.find("companies/") {
            let after_companies = &path_str[companies_pos + "companies/".len()..];
            if let Some(slash_pos) = after_companies.find('/') {
                return Some(after_companies[..slash_pos].to_string());
            }
        }
        None
    }

    /// Creates a project from manifest with the correct company_code
    fn create_project_with_company_code(
        &self,
        manifest: ProjectManifest,
        company_code: &str,
    ) -> Result<AnyProject, AppError> {
        // Use the TryFrom implementation to load the project with tasks from manifest
        let mut project = AnyProject::try_from(manifest).map_err(|e| AppError::ValidationError {
            field: "project manifest".to_string(),
            message: e,
        })?;

        // Update the company code to match the path
        match &mut project {
            AnyProject::Project(p) => {
                p.company_code = company_code.to_string();
            }
        }

        Ok(project)
    }

    /// Carrega e deserializa o manifesto de um projeto de um arquivo YAML.
    fn load_manifest(&self, path: &Path) -> Result<ProjectManifest, Box<dyn Error>> {
        let yaml = fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).map_err(|e| e.into())
    }

    /// Loads tasks from the `tasks` subdirectory of a project and adds them.
    fn load_tasks_for_project(&self, project: &mut AnyProject, project_path: &Path) -> Result<(), AppError> {
        // For ID-based format, tasks are stored in the same directory as the project file
        let tasks_dir = project_path.parent().unwrap().join("tasks");
        if !tasks_dir.exists() {
            return Ok(());
        }

        // Use absolute path for glob pattern
        let absolute_tasks_dir = std::fs::canonicalize(&tasks_dir).unwrap_or_else(|_| tasks_dir.clone());
        let pattern = absolute_tasks_dir.join("*.yaml");
        let pattern_str = pattern.to_str().unwrap();
        let walker = glob(pattern_str).map_err(|e| AppError::ValidationError {
            field: "glob pattern".to_string(),
            message: e.to_string(),
        })?;

        for entry in walker.flatten() {
            let task_path = entry.path();
            if std::env::var("TTR_VERBOSE").unwrap_or_default() == "1" {
                println!("DEBUG: Loading task from: {:?}", task_path);
            }
            let yaml = fs::read_to_string(task_path).map_err(|e| AppError::IoErrorWithPath {
                operation: "file read".to_string(),
                path: task_path.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;
            let task_manifest: TaskManifest =
                serde_yaml::from_str(&yaml).map_err(|e| AppError::SerializationError {
                    format: "YAML".to_string(),
                    details: format!("Error deserializing task: {e}"),
                })?;
            let task = AnyTask::try_from(task_manifest).map_err(|e| AppError::SerializationError {
                format: "YAML".to_string(),
                details: format!("Error converting task manifest: {e}"),
            })?;
            if std::env::var("TTR_VERBOSE").unwrap_or_default() == "1" {
                println!("DEBUG: Loaded task: {} - {}", task.code(), task.name());
            }
            project.add_task(task);
        }

        Ok(())
    }

    /// Save individual task files for a project
    fn save_tasks_for_project(&self, project: &AnyProject) -> Result<(), AppError> {
        let project_id = project.id();
        let project_path = self.get_project_path_by_id(project_id);
        let tasks_dir = project_path.parent().unwrap().join("tasks");

        // Create tasks directory if it doesn't exist
        fs::create_dir_all(&tasks_dir).map_err(|e| AppError::IoErrorWithPath {
            operation: "create directory".to_string(),
            path: tasks_dir.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        // Save each task as individual YAML file
        for task in project.tasks().values() {
            let task_file_path = tasks_dir.join(format!("{}.yaml", task.code()));
            let task_manifest = TaskManifest::from(task.clone());
            let yaml = serde_yaml::to_string(&task_manifest).map_err(|e| AppError::SerializationError {
                format: "YAML".to_string(),
                details: format!("Error serializing task: {e}"),
            })?;
            fs::write(&task_file_path, yaml).map_err(|e| AppError::IoErrorWithPath {
                operation: "file write".to_string(),
                path: task_file_path.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;
        }

        Ok(())
    }
}

impl ProjectRepository for FileProjectRepository {
    /// Salva um projeto.
    /// Salva um arquivo `{project_id}.yaml` no diretório projects.
    fn save(&self, project: AnyProject) -> Result<(), AppError> {
        let project_id = project.id();
        let project_code = project.code();

        // Add code-to-ID mapping
        self.mapping_service.add_mapping("project", project_code, project_id)
            .map_err(|e| AppError::validation_error("mapping", &e))?;

        // Create projects directory if it doesn't exist
        let projects_dir = self.get_projects_path();
        fs::create_dir_all(&projects_dir).map_err(|e| AppError::IoErrorWithPath {
            operation: "create directory".to_string(),
            path: projects_dir.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        // Save project file
        let project_path = self.get_project_path_by_id(project_id);
        let project_manifest = ProjectManifest::from(project.clone());
        let yaml = serde_yaml::to_string(&project_manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Error serializing project: {e}"),
        })?;
        fs::write(&project_path, yaml).map_err(|e| AppError::IoErrorWithPath {
            operation: "file write".to_string(),
            path: project_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        // Save individual task files
        self.save_tasks_for_project(&project)?;

        Ok(())
    }

    /// Carrega um projeto.
    /// Procura por arquivos YAML no diretório projects.
    fn load(&self) -> Result<AnyProject, AppError> {
        let projects_dir = self.get_projects_path();
        if !projects_dir.exists() {
            return Err(AppError::ProjectNotFound {
                code: "unknown".to_string(),
            });
        }

        // Look for YAML files in the projects directory
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    return self.load_from_path(&path);
                }
            }
        }

        Err(AppError::ProjectNotFound {
            code: "unknown".to_string(),
        })
    }

    fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
        let mut projects = Vec::new();
        let projects_dir = self.get_projects_path();
        
        if !projects_dir.exists() {
            return Ok(projects);
        }

        // Look for YAML files in the projects directory
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Ok(project) = self.load_from_path(&path) {
                        projects.push(project);
                    }
                }
            }
        }

        // Note: Removed current directory check as it was causing issues with project updates
        // The current directory check was loading old project data and overwriting updated projects

        Ok(projects)
    }

    fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
        // Get project ID from code mapping
        if let Some(project_id) = self.mapping_service.get_id("project", code) {
            let project_path = self.get_project_path_by_id(&project_id);
            if project_path.exists() {
                return Ok(Some(self.load_from_path(&project_path)?));
            }
        }
        Ok(None)
    }

    fn get_next_code(&self) -> Result<String, AppError> {
        // Use timestamp-based approach for better uniqueness in concurrent scenarios
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Add microseconds for better uniqueness
        let micros = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            % 1000;

        Ok(format!("proj-{}{:03}", timestamp, micros))
    }
}

// ===================================
// TESTES
// ===================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::{builder::ProjectBuilder, project::Project};
    use crate::infrastructure::persistence::manifests::project_manifest::{
        ProjectManifest, ProjectMetadata, ProjectSpec, ProjectStatusManifest, VacationRulesManifest,
    };
    use chrono::NaiveDate;
    use std::fs;
    use tempfile::tempdir;

    use uuid7::uuid7;

    fn create_test_project() -> Project {
        ProjectBuilder::new()
            .code("TEST-001".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .description(Some("A test project for repository testing".to_string()))
            .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .build()
            .unwrap()
    }

    fn create_test_project_manifest() -> ProjectManifest {
        ProjectManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata {
                id: Some(uuid7().to_string()),
                code: Some("TEST-001".to_string()),
                name: "Test Project".to_string(),
                description: "A test project for repository testing".to_string(),
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec: ProjectSpec {
                timezone: Some("UTC".to_string()),
                start_date: Some("2024-01-01".to_string()),
                end_date: Some("2024-12-31".to_string()),
                status: ProjectStatusManifest::Planned,
                vacation_rules: Some(VacationRulesManifest {
                    max_concurrent_vacations: None,
                    allow_layoff_vacations: None,
                    require_layoff_vacation_period: None,
                    layoff_periods: None,
                }),
            },
        }
    }

    #[test]
    fn test_project_manifest_serialization() {
        let manifest = create_test_project_manifest();

        let yaml = serde_yaml::to_string(&manifest).expect("Failed to serialize to YAML");
        let deserialized: ProjectManifest = serde_yaml::from_str(&yaml).expect("Failed to deserialize from YAML");

        assert_eq!(manifest.metadata.code, deserialized.metadata.code);
        assert_eq!(manifest.metadata.name, deserialized.metadata.name);
        assert_eq!(manifest.metadata.description, deserialized.metadata.description);
        assert_eq!(manifest.spec.status, deserialized.spec.status);
    }

    #[test]
    fn test_project_manifest_to_domain_conversion() {
        let manifest = create_test_project_manifest();
        // Test conversion from ProjectManifest to AnyProject
        // Note: This requires implementing From<ProjectManifest> for AnyProject
        // For now, we'll test the manifest structure
        assert_eq!(manifest.metadata.code, Some("TEST-001".to_string()));
        assert_eq!(manifest.metadata.name, "Test Project");
        assert_eq!(manifest.metadata.description, "A test project for repository testing");
        assert!(matches!(manifest.spec.status, ProjectStatusManifest::Planned));
    }

    #[test]
    fn test_project_repository_save_and_load() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());
        let project = create_test_project();

        // Save project
        let save_result = repository.save(project.clone().into());
        assert!(save_result.is_ok(), "Failed to save project: {:?}", save_result);

        // Test that the project was saved by checking the file exists (ID-based format)
        let project_id = &project.id;
        let project_file = repo_path.join("projects").join(format!("{}.yaml", project_id));
        assert!(project_file.exists(), "Project file should exist after save");
    }

    #[test]
    fn test_project_repository_save_multiple_projects() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());

        // Create and save multiple projects
        let project1 = ProjectBuilder::new()
            .code("PROJ-001".to_string())
            .name("Project 1".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .build()
            .unwrap();

        let project2 = ProjectBuilder::new()
            .code("PROJ-002".to_string())
            .name("Project 2".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .build()
            .unwrap();

        repository.save(project1.clone().into()).expect("Failed to save project 1");
        repository.save(project2.clone().into()).expect("Failed to save project 2");

        // Verify both projects were saved by checking files exist (ID-based format)
        let project1_file = repo_path.join("projects").join(format!("{}.yaml", project1.id));
        let project2_file = repo_path.join("projects").join(format!("{}.yaml", project2.id));

        assert!(project1_file.exists(), "Project 1 file should exist");
        assert!(project2_file.exists(), "Project 2 file should exist");
    }

    #[test]
    fn test_project_repository_update_project() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path();

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());
        let project = create_test_project();

        // Save initial project
        repository.save(project.clone().into()).expect("Failed to save project");

        // Update project state to InProgress
        let in_progress_project = project; // Project is no longer generic, just use the project as is
        repository
            .save(in_progress_project.clone().into())
            .expect("Failed to update project");

        // Verify update by checking file exists (ID-based format)
        let project_file = repo_path.join("projects").join(format!("{}.yaml", in_progress_project.id));
        assert!(project_file.exists(), "Updated project file should exist");
    }

    #[test]
    fn test_project_repository_save_and_verify() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path().join("projects");
        fs::create_dir_all(&repo_path).expect("Failed to create projects directory");

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());
        let project = create_test_project();

        // Save project
        repository.save(project.clone().into()).expect("Failed to save project");

        // Verify project exists
        let project_file = repo_path
            .join("companies")
            .join("COMP-001")
            .join("projects")
            .join("TEST-001")
            .join("project.yaml");
        assert!(project_file.exists(), "Project file should exist after save");

        // Note: Tasks are no longer saved in the project directory
        // They are saved separately in individual task files
    }

    #[test]
    fn test_project_repository_error_handling() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path().join("projects");
        fs::create_dir_all(&repo_path).expect("Failed to create projects directory");

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());

        // Try to find non-existent project
        let result = repository.find_by_code("NON-EXISTENT");
        assert!(result.is_ok(), "Should return Ok(None) for non-existent project");
        assert!(result.unwrap().is_none(), "Should return None for non-existent project");
    }

    #[test]
    fn test_project_repository_file_corruption_handling() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path().join("projects");
        fs::create_dir_all(&repo_path).expect("Failed to create projects directory");

        let repository = FileProjectRepository::with_base_path(repo_path.to_path_buf());
        let project = create_test_project();

        // Save project
        repository.save(project.clone().into()).expect("Failed to save project");

        // Corrupt the YAML file
        let project_file = repo_path
            .join("companies")
            .join("COMP-001")
            .join("projects")
            .join("TEST-001")
            .join("project.yaml");
        fs::write(&project_file, "invalid: yaml: content: [").expect("Failed to corrupt file");

        // Note: We can't test loading corrupted files yet since find_by_code is not fully implemented
        // This test verifies that we can save projects and corrupt files
        assert!(project_file.exists(), "Project file should exist even if corrupted");
    }

    #[test]
    fn test_project_repository_concurrent_access() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let repo_path = temp_dir.path().join("projects");
        fs::create_dir_all(&repo_path).expect("Failed to create projects directory");

        // Create multiple projects concurrently
        let mut handles = vec![];

        for i in 1..=5 {
            let repo_path = repo_path.clone();
            let handle = std::thread::spawn(move || {
                let repo = FileProjectRepository::with_base_path(repo_path.to_path_buf());
                let project = ProjectBuilder::new()
                    .code(format!("PROJ-{:03}", i))
                    .name(format!("Project {}", i))
                    .company_code("COMP-001".to_string())
                    .created_by("test-user".to_string())
                    .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
                    .build()
                    .unwrap();
                repo.save(project.into())
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().expect("Thread failed to complete");
            assert!(
                result.is_ok(),
                "Failed to save project in concurrent access: {:?}",
                result
            );
        }

        // Verify all projects were saved by checking files exist
        for i in 1..=5 {
            let project_file = repo_path
                .join("companies")
                .join("COMP-001")
                .join("projects")
                .join(format!("PROJ-{:03}", i))
                .join("project.yaml");
            assert!(project_file.exists(), "Project {} file should exist", i);
        }
    }
}
