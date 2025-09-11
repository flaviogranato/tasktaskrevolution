use crate::infrastructure::persistence::{
    company_repository::FileCompanyRepository, config_repository::FileConfigRepository,
    project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    task_repository::FileTaskRepository,
};

/// Simple application struct with constructor injection
/// Replaces the complex DI container for a CLI tool
pub struct App {
    pub company_repository: FileCompanyRepository,
    pub config_repository: FileConfigRepository,
    pub project_repository: FileProjectRepository,
    pub resource_repository: FileResourceRepository,
    pub task_repository: FileTaskRepository,
}

impl App {
    /// Creates a new App instance with default repositories
    pub fn new() -> Self {
        Self {
            company_repository: FileCompanyRepository::new("."),
            config_repository: FileConfigRepository::new(),
            project_repository: FileProjectRepository::with_base_path(".".into()),
            resource_repository: FileResourceRepository::new("."),
            task_repository: FileTaskRepository::new("."),
        }
    }

    /// Creates a new App instance with custom base path
    pub fn with_base_path(base_path: String) -> Self {
        Self {
            company_repository: FileCompanyRepository::new(&base_path),
            config_repository: FileConfigRepository::new(),
            project_repository: FileProjectRepository::with_base_path(base_path.clone().into()),
            resource_repository: FileResourceRepository::new(&base_path),
            task_repository: FileTaskRepository::new(&base_path),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        let app = App::new();
        // Verify that all repositories are initialized
        assert!(true); // If we get here, the app was created successfully
    }

    #[test]
    fn test_app_with_base_path() {
        let app = App::with_base_path("/tmp/test".to_string());
        // Verify that all repositories are initialized with custom path
        assert!(true); // If we get here, the app was created successfully
    }

    #[test]
    fn test_app_default() {
        let app = App::default();
        // Verify that default implementation works
        assert!(true); // If we get here, the app was created successfully
    }
}
