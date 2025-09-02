#![allow(dead_code)]

use crate::domain::company_settings::{
    config::{Config, WorkDay},
    repository::ConfigRepository,
};
use crate::domain::shared::errors::{DomainError, DomainErrorKind};

#[allow(dead_code)]
pub struct UpdateCompanyConfigUseCase<R>
where
    R: ConfigRepository,
{
    repository: R,
}

impl<R> UpdateCompanyConfigUseCase<R>
where
    R: ConfigRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// Updates company configuration with conflict resolution
    pub fn execute(&self, updates: CompanyConfigUpdates) -> Result<Config, DomainError> {
        // Load existing configuration
        let (mut config, _) = self.repository.load()?;

        // Validate updates before applying
        if let Some(manager_name) = &updates.manager_name
            && manager_name.trim().is_empty()
        {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "manager_name".to_string(),
                message: "Manager name cannot be empty".to_string(),
            }));
        }

        if let Some(manager_email) = &updates.manager_email
            && (manager_email.trim().is_empty() || !manager_email.contains('@') || !manager_email.contains('.'))
        {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "manager_email".to_string(),
                message: "Invalid email format".to_string(),
            }));
        }

        // Apply updates
        if let Some(company_name) = updates.company_name {
            config.update_company_name(company_name);
        }

        if let Some(manager_name) = updates.manager_name
            && let Some(manager_email) = &updates.manager_email
        {
            config.update_manager(manager_name, manager_email.clone());
        }

        if let Some(timezone) = updates.default_timezone {
            config.update_timezone(timezone);
        }

        if let Some(start) = updates.work_hours_start
            && let Some(end) = updates.work_hours_end
        {
            config.update_work_hours(start, end);
        }

        if let Some(work_days) = updates.work_days {
            // Convert string work days to WorkDay enum
            let work_days_enum: Vec<WorkDay> = work_days.iter().filter_map(|day| WorkDay::from_str(day)).collect();

            if !work_days_enum.is_empty() {
                config.update_work_days(work_days_enum);
            }
        }

        Ok(config)
    }

    /// Updates configuration from YAML string (for manual edits)
    pub fn update_from_yaml(&self, yaml_content: &str) -> Result<Config, DomainError> {
        // Parse YAML content
        let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
            DomainError::new(DomainErrorKind::Serialization {
                format: "YAML".to_string(),
                details: e.to_string(),
            })
        })?;

        // Create new config from YAML
        let manager_name = yaml_data["manager_name"]
            .as_str()
            .ok_or_else(|| {
                DomainError::new(DomainErrorKind::ConfigurationMissing {
                    field: "manager_name".to_string(),
                })
            })?
            .to_string();

        let manager_email = yaml_data["manager_email"]
            .as_str()
            .ok_or_else(|| {
                DomainError::new(DomainErrorKind::ConfigurationMissing {
                    field: "manager_email".to_string(),
                })
            })?
            .to_string();

        let default_timezone = yaml_data["default_timezone"]
            .as_str()
            .ok_or_else(|| {
                DomainError::new(DomainErrorKind::ConfigurationMissing {
                    field: "default_timezone".to_string(),
                })
            })?
            .to_string();

        let mut config = Config::new(manager_name, manager_email, default_timezone);

        // Set optional fields
        if let Some(company_name) = yaml_data["company_name"].as_str() {
            config.update_company_name(company_name.to_string());
        }

        if let Some(start) = yaml_data["work_hours_start"].as_str()
            && let Some(end) = yaml_data["work_hours_end"].as_str()
        {
            config.update_work_hours(start.to_string(), end.to_string());
        }

        if let Some(work_days) = yaml_data["work_days"].as_sequence() {
            let work_days_strings: Vec<String> = work_days
                .iter()
                .filter_map(|day| day.as_str().map(|s| s.to_string()))
                .collect();

            if !work_days_strings.is_empty() {
                let work_days_enum: Vec<WorkDay> = work_days_strings
                    .iter()
                    .filter_map(|day| WorkDay::from_str(day))
                    .collect();

                if !work_days_enum.is_empty() {
                    config.update_work_days(work_days_enum);
                }
            }
        }

        Ok(config)
    }

    /// Merges CLI updates with existing YAML configuration
    pub fn merge_updates(&self, cli_updates: CompanyConfigUpdates) -> Result<Config, DomainError> {
        // Load existing configuration
        let (mut config, _) = self.repository.load()?;

        // Apply CLI updates (preserving existing YAML values)
        if let Some(timezone) = cli_updates.default_timezone {
            config.update_timezone(timezone);
        }

        if let Some(start) = cli_updates.work_hours_start
            && let Some(end) = cli_updates.work_hours_end
        {
            config.update_work_hours(start, end);
        }

        if let Some(company_name) = cli_updates.company_name {
            config.update_company_name(company_name);
        }

        if let Some(manager_name) = cli_updates.manager_name
            && let Some(manager_email) = cli_updates.manager_email
        {
            config.update_manager(manager_name, manager_email);
        }

        Ok(config)
    }

    /// Validates YAML content before applying
    pub fn validate_yaml(&self, yaml_content: &str) -> Result<(), DomainError> {
        // Try to parse YAML first
        let yaml_data: serde_yaml::Value = serde_yaml::from_str(yaml_content).map_err(|e| {
            DomainError::new(DomainErrorKind::Serialization {
                format: "YAML".to_string(),
                details: e.to_string(),
            })
        })?;

        // Validate required fields
        let manager_name = yaml_data["manager_name"].as_str();
        let manager_email = yaml_data["manager_email"].as_str();
        let default_timezone = yaml_data["default_timezone"].as_str();

        if manager_name.is_none() || manager_name.unwrap().trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "manager_name".to_string(),
                message: "Manager name is required and cannot be empty".to_string(),
            }));
        }

        if manager_email.is_none() || manager_email.unwrap().trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "manager_email".to_string(),
                message: "Manager email is required and cannot be empty".to_string(),
            }));
        }

        if default_timezone.is_none() || default_timezone.unwrap().trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "default_timezone".to_string(),
                message: "Default timezone is required and cannot be empty".to_string(),
            }));
        }

        // Validate email format (basic validation)
        if let Some(email) = manager_email
            && (!email.contains('@') || !email.contains('.'))
        {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "manager_email".to_string(),
                message: "Invalid email format".to_string(),
            }));
        }

        // Validate timezone (basic validation)
        if let Some(timezone) = default_timezone {
            let valid_timezones = [
                "UTC",
                "GMT",
                "EST",
                "PST",
                "CST",
                "MST",
                "America/New_York",
                "America/Los_Angeles",
                "America/Chicago",
                "Europe/London",
                "Europe/Paris",
                "Europe/Berlin",
                "Asia/Tokyo",
                "Asia/Shanghai",
                "Asia/Dubai",
                "America/Sao_Paulo",
                "America/Argentina/Buenos_Aires",
            ];

            if !valid_timezones.contains(&timezone) {
                return Err(DomainError::new(DomainErrorKind::ValidationError {
                    field: "default_timezone".to_string(),
                    message: "Invalid timezone format".to_string(),
                }));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CompanyConfigUpdates {
    pub company_name: Option<String>,
    pub manager_name: Option<String>,
    pub manager_email: Option<String>,
    pub default_timezone: Option<String>,
    pub work_hours_start: Option<String>,
    pub work_hours_end: Option<String>,
    pub work_days: Option<Vec<String>>,
    pub source: UpdateSource,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum UpdateSource {
    Cli,
    YamlEdit,
    Merge,
}

impl CompanyConfigUpdates {
    pub fn new() -> Self {
        Self {
            company_name: None,
            manager_name: None,
            manager_email: None,
            default_timezone: None,
            work_hours_start: None,
            work_hours_end: None,
            work_days: None,
            source: UpdateSource::Cli,
        }
    }

    pub fn from_yaml() -> Self {
        Self {
            company_name: None,
            manager_name: None,
            manager_email: None,
            default_timezone: None,
            work_hours_start: None,
            work_hours_end: None,
            work_days: None,
            source: UpdateSource::YamlEdit,
        }
    }

    pub fn with_company_name(mut self, name: String) -> Self {
        self.company_name = Some(name);
        self
    }

    pub fn with_manager(mut self, name: String, email: String) -> Self {
        self.manager_name = Some(name);
        self.manager_email = Some(email);
        self
    }

    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.default_timezone = Some(timezone);
        self
    }

    pub fn with_work_hours(mut self, start: String, end: String) -> Self {
        self.work_hours_start = Some(start);
        self.work_hours_end = Some(end);
        self
    }

    pub fn with_work_days(mut self, days: Vec<String>) -> Self {
        self.work_days = Some(days);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_settings::config::Config;
    use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
    use std::cell::RefCell;
    use std::path::PathBuf;

    // Mock repository for testing
    struct MockConfigRepository {
        config: RefCell<Option<Config>>,
    }

    impl ConfigRepository for MockConfigRepository {
        fn save(
            &self,
            _config: ConfigManifest,
            _path: PathBuf,
        ) -> Result<(), crate::domain::shared::errors::DomainError> {
            Ok(())
        }

        fn create_repository_dir(&self, _path: PathBuf) -> Result<(), crate::domain::shared::errors::DomainError> {
            Ok(())
        }

        fn load(&self) -> Result<(Config, PathBuf), crate::domain::shared::errors::DomainError> {
            self.config.borrow().clone().map(|c| (c, PathBuf::from("/tmp"))).ok_or(
                crate::domain::shared::errors::DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::ConfigurationMissing {
                        field: "config".to_string(),
                    },
                ),
            )
        }
    }

    #[test]
    fn test_update_company_config_success() {
        // Arrange
        let initial_config = Config::new(
            "John Doe".to_string(),
            "john@company.com".to_string(),
            "UTC".to_string(),
        )
        .with_company_name("Test Company".to_string());

        let mock_repo = MockConfigRepository {
            config: RefCell::new(Some(initial_config.clone())),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);

        let updates = CompanyConfigUpdates::new()
            .with_company_name("Updated Company".to_string())
            .with_timezone("America/New_York".to_string());

        // Act
        let result = use_case.execute(updates);

        // Assert
        assert!(result.is_ok(), "Expected successful update");
        let updated_config = result.unwrap();
        assert_eq!(updated_config.company_name, Some("Updated Company".to_string()));
        assert_eq!(updated_config.default_timezone, "America/New_York");
    }

    #[test]
    fn test_update_company_config_not_found() {
        // Arrange
        let mock_repo = MockConfigRepository {
            config: RefCell::new(None),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let updates = CompanyConfigUpdates::new().with_company_name("New Company".to_string());

        // Act
        let result = use_case.execute(updates);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error.kind(), DomainErrorKind::ConfigurationMissing { .. }));
    }

    #[test]
    fn test_update_company_config_invalid_data() {
        // Arrange
        let initial_config = Config::new(
            "John Doe".to_string(),
            "john@company.com".to_string(),
            "UTC".to_string(),
        );

        let mock_repo = MockConfigRepository {
            config: RefCell::new(Some(initial_config)),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let updates = CompanyConfigUpdates::new().with_manager("".to_string(), "invalid-email".to_string()); // Invalid data

        // Act
        let result = use_case.execute(updates);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error.kind(), DomainErrorKind::ValidationError { .. }));
    }

    #[test]
    fn test_update_from_yaml_success() {
        // Arrange
        let mock_repo = MockConfigRepository {
            config: RefCell::new(None),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let yaml_content = r#"
company_name: "YAML Company"
manager_name: "YAML Manager"
manager_email: "yaml@company.com"
default_timezone: "Europe/London"
work_hours_start: "08:00"
work_hours_end: "17:00"
work_days: ["monday", "tuesday", "wednesday"]
        "#;

        // Act
        let result = use_case.update_from_yaml(yaml_content);

        // Assert
        assert!(result.is_ok(), "Expected successful YAML update");
        let config = result.unwrap();
        assert_eq!(config.company_name, Some("YAML Company".to_string()));
        assert_eq!(config.manager_name, "YAML Manager");
        assert_eq!(config.default_timezone, "Europe/London");
    }

    #[test]
    fn test_update_from_yaml_invalid_format() {
        // Arrange
        let mock_repo = MockConfigRepository {
            config: RefCell::new(None),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let invalid_yaml = "invalid: yaml: content: [";

        // Act
        let result = use_case.update_from_yaml(invalid_yaml);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error.kind(), DomainErrorKind::Serialization { .. }));
    }

    #[test]
    fn test_merge_updates_cli_with_yaml() {
        // Arrange
        let initial_config = Config::new(
            "John Doe".to_string(),
            "john@company.com".to_string(),
            "UTC".to_string(),
        )
        .with_company_name("Initial Company".to_string());

        let mock_repo = MockConfigRepository {
            config: RefCell::new(Some(initial_config)),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let cli_updates = CompanyConfigUpdates::new()
            .with_timezone("America/Sao_Paulo".to_string())
            .with_work_hours("09:00".to_string(), "18:00".to_string());

        // Act
        let result = use_case.merge_updates(cli_updates);

        // Assert
        assert!(result.is_ok(), "Expected successful merge");
        let merged_config = result.unwrap();
        assert_eq!(merged_config.company_name, Some("Initial Company".to_string())); // Preserved from YAML
        assert_eq!(merged_config.default_timezone, "America/Sao_Paulo"); // Updated from CLI
        assert_eq!(merged_config.work_hours_start, Some("09:00".to_string())); // Added from CLI
    }

    #[test]
    fn test_validate_yaml_success() {
        // Arrange
        let mock_repo = MockConfigRepository {
            config: RefCell::new(None),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let valid_yaml = r#"
company_name: "Valid Company"
manager_name: "Valid Manager"
manager_email: "valid@company.com"
default_timezone: "UTC"
        "#;

        // Act
        let result = use_case.validate_yaml(valid_yaml);

        // Assert
        assert!(result.is_ok(), "Expected successful validation");
    }

    #[test]
    fn test_validate_yaml_invalid_data() {
        // Arrange
        let mock_repo = MockConfigRepository {
            config: RefCell::new(None),
        };

        let use_case = UpdateCompanyConfigUseCase::new(mock_repo);
        let invalid_yaml = r#"
company_name: ""
manager_name: "Manager"
manager_email: "not-an-email"
default_timezone: "Invalid/Timezone"
        "#;

        // Act
        let result = use_case.validate_yaml(invalid_yaml);

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error.kind(), DomainErrorKind::ValidationError { .. }));
    }
}
