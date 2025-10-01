#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::domain::company_settings::{config::Config, repository::ConfigRepository};
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::shared::convertable::Convertible;
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct InitializeRepositoryUseCase<R: ConfigRepository> {
    repository: R,
}

impl<R: ConfigRepository> InitializeRepositoryUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    pub fn execute(&self, path: PathBuf, manager_name: String, manager_email: String) -> Result<(), AppError> {
        let config = Config::new(manager_name.clone(), manager_email.clone(), "UTC".to_string());
        self.repository.create_repository_dir(&path)?;
        self.repository.save(config, &path)?;
        println!("Configuração iniciada em: {}", path.display());

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::errors::AppError;
    use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
    use std::cell::RefCell;

    struct MockConfigRepository {
        should_fail: bool,
        saved_config: RefCell<Option<ConfigManifest>>,
        created_path: RefCell<Option<PathBuf>>,
    }

    impl MockConfigRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
                created_path: RefCell::new(None),
            }
        }
    }

    impl ConfigRepository for MockConfigRepository {
        fn save(&self, config: Config, path: &Path) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Erro mockado ao salvar".to_string(),
                });
            }
            *self.saved_config.borrow_mut() = Some(config.clone());
            *self.created_path.borrow_mut() = Some(path.clone());

            Ok(())
        }

        fn create_repository_dir(&self, path: &Path) -> DomainResult<()> {
            *self.created_path.borrow_mut() = Some(path.clone());
            Ok(())
        }

        fn load(&self) -> DomainResult<(Config, PathBuf)> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Erro mockado ao carregar".to_string(),
                });
            }
            let config = Config::new("mock".to_string(), "mock@email.com".to_string(), "UTC".to_string());
            let path = PathBuf::from("/mock/path");
            Ok((config, path))
        }
    }
    #[test]
    fn test_create_config_success() {
        let mock_repo = MockConfigRepository::new(false);
        let use_case = InitializeRepositoryUseCase::new(mock_repo);
        let manager_name = "John".to_string();
        let manager_email = "john@nothing.com".to_string();
        let repo_path = PathBuf::new();

        let result = use_case.execute(repo_path, manager_name, manager_email);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_config_failure() {
        let mock_repo = MockConfigRepository::new(true);
        let use_case = InitializeRepositoryUseCase::new(mock_repo);
        let manager_name = "John".to_string();
        let manager_email = "john@nothing.com".to_string();
        let repo_path = PathBuf::new();

        let result = use_case.execute(repo_path, manager_name, manager_email);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockConfigRepository::new(false);
        let use_case = InitializeRepositoryUseCase::new(mock_repo);
        let manager_name = "John".to_string();
        let manager_email = "john@nothing.com".to_string();
        let repo_path = PathBuf::new();
        let _ = use_case.execute(repo_path, manager_name.clone(), manager_email.clone());

        let saved_config = use_case.repository.saved_config.borrow();
        assert!(saved_config.is_some());
        assert_eq!(saved_config.as_ref().unwrap().spec.manager_name, manager_name);
        assert_eq!(saved_config.as_ref().unwrap().spec.manager_email, manager_email);
    }
}
