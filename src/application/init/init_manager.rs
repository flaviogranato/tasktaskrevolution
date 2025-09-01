use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use crate::domain::company_settings::Config;
use crate::domain::company_settings::repository::ConfigRepository;
use std::boxed::Box;

/// Data structure for initializing a manager/consultant
#[derive(Debug, Clone)]
pub struct InitManagerData {
    pub name: String,
    pub email: String,
    pub company_name: String,
    pub timezone: String,
    pub work_hours_start: String,
    pub work_hours_end: String,
}

/// Use case for initializing a manager/consultant
pub struct InitManagerUseCase {
    repository: Box<dyn ConfigRepository>,
}

impl InitManagerUseCase {
    pub fn new(repository: Box<dyn ConfigRepository>) -> Self {
        Self { repository }
    }

    /// Execute the initialization of a manager/consultant
    pub fn execute(&self, data: InitManagerData) -> Result<Config, DomainError> {
        // Validate input data
        self.validate_input(&data)?;

        // Create company config
        let mut config = Config::new(
            data.name.clone(),
            data.email.clone(),
            data.timezone.clone(),
        );

        // Set company name
        config = config.with_company_name(data.company_name.clone());

        // Set work hours
        config = config.with_work_hours(data.work_hours_start.clone(), data.work_hours_end.clone());

        // TODO: Save to repository when ConfigManifest is implemented
        // For now, just return the config

        Ok(config)
    }

    /// Validate input data
    fn validate_input(&self, data: &InitManagerData) -> Result<(), DomainError> {
        if data.name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Manager name cannot be empty".to_string(),
            }));
        }

        if data.company_name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "company_name".to_string(),
                message: "Company name cannot be empty".to_string(),
            }));
        }

        if !self.is_valid_email(&data.email) {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            }));
        }

        // Validate work hours
        self.validate_work_hours(&data.work_hours_start, &data.work_hours_end)?;

        Ok(())
    }

    /// Validate work hours format
    fn validate_work_hours(&self, start: &str, end: &str) -> Result<(), DomainError> {
        // Basic validation - just check if they're not empty for now
        if start.trim().is_empty() || end.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "work_hours".to_string(),
                message: "Work hours cannot be empty".to_string(),
            }));
        }

        // TODO: Add more sophisticated time validation if needed
        Ok(())
    }

    /// Validate email format (basic validation)
    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation - contains @ and has valid format
        email.contains('@') && email.len() > 5 && !email.starts_with('@') && !email.ends_with('@')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_settings::repository::ConfigRepository;
    use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
    use std::cell::RefCell;
    use std::path::PathBuf;

    // Mock repository for testing
    struct MockConfigRepository {
        should_fail: bool,
        saved_config: RefCell<Option<Config>>,
    }

    impl MockConfigRepository {
        fn new() -> Self {
            Self {
                should_fail: false,
                saved_config: RefCell::new(None),
            }
        }

        fn with_failure() -> Self {
            Self {
                should_fail: true,
                saved_config: RefCell::new(None),
            }
        }
    }

    impl ConfigRepository for MockConfigRepository {
        fn save(&self, _config: ConfigManifest, _path: PathBuf) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::new(DomainErrorKind::PersistenceError {
                    operation: "save".to_string(),
                    details: "Database connection failed".to_string(),
                }));
            }
            Ok(())
        }

        fn create_repository_dir(&self, _path: PathBuf) -> Result<(), DomainError> {
            Ok(())
        }

        fn load(&self) -> Result<(Config, PathBuf), DomainError> {
            self.saved_config.borrow().clone().map(|c| {
                (c, PathBuf::from("/tmp"))
            }).ok_or(DomainError::new(DomainErrorKind::ConfigurationMissing {
                field: "config".to_string()
            }))
        }
    }

    #[test]
    fn test_validate_input_success() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        let data = InitManagerData {
            name: "João Silva".to_string(),
            email: "joao@example.com".to_string(),
            company_name: "TechConsulting".to_string(),
            timezone: "UTC".to_string(),
            work_hours_start: "08:00".to_string(),
            work_hours_end: "18:00".to_string(),
        };

        let result = use_case.validate_input(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_input_empty_name() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        let data = InitManagerData {
            name: "".to_string(),
            email: "joao@example.com".to_string(),
            company_name: "TechConsulting".to_string(),
            timezone: "UTC".to_string(),
            work_hours_start: "08:00".to_string(),
            work_hours_end: "18:00".to_string(),
        };

        let result = use_case.validate_input(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_input_invalid_email() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        let data = InitManagerData {
            name: "João Silva".to_string(),
            email: "invalid-email".to_string(),
            company_name: "TechConsulting".to_string(),
            timezone: "UTC".to_string(),
            work_hours_start: "08:00".to_string(),
            work_hours_end: "18:00".to_string(),
        };

        let result = use_case.validate_input(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_work_hours_success() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        
        let result = use_case.validate_work_hours("08:00", "18:00");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_work_hours_empty() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        
        let result = use_case.validate_work_hours("", "18:00");
        assert!(result.is_err());
        
        let result = use_case.validate_work_hours("08:00", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_email() {
        let use_case = InitManagerUseCase::new(Box::new(MockConfigRepository::new()));
        
        assert!(use_case.is_valid_email("test@example.com"));
        assert!(use_case.is_valid_email("user.name@domain.co.uk"));
        assert!(!use_case.is_valid_email("invalid-email"));
        assert!(!use_case.is_valid_email("@example.com"));
        assert!(!use_case.is_valid_email("test@"));
        assert!(!use_case.is_valid_email(""));
    }

    #[test]
    fn test_init_manager_success() {
        let mock_repo = MockConfigRepository::new();
        let use_case = InitManagerUseCase::new(Box::new(mock_repo));
        
        let init_data = InitManagerData {
            name: "João Silva".to_string(),
            email: "joao.silva@consultoria.com".to_string(),
            company_name: "TechConsulting Ltda".to_string(),
            timezone: "America/Sao_Paulo".to_string(),
            work_hours_start: "08:00".to_string(),
            work_hours_end: "18:00".to_string(),
        };

        let result = use_case.execute(init_data);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.manager_name, "João Silva");
        assert_eq!(config.manager_email, "joao.silva@consultoria.com");
        assert_eq!(config.company_name, Some("TechConsulting Ltda".to_string()));
        assert_eq!(config.default_timezone, "America/Sao_Paulo");
        assert_eq!(config.work_hours_start, Some("08:00".to_string()));
        assert_eq!(config.work_hours_end, Some("18:00".to_string()));
    }
}
