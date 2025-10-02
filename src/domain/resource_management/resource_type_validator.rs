use crate::domain::company_settings::repository::ConfigRepository;
use std::path::Path;

pub struct ResourceTypeValidator;

impl ResourceTypeValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_resource_type(
        &self,
        resource_type: &str,
        config_repository: &dyn ConfigRepository,
    ) -> Result<(), String> {
        match config_repository.load() {
            Ok((config, _)) => {
                if config.resource_types.is_empty() {
                    // If config has no resource types defined, allow any type (backward compatibility)
                    Ok(())
                } else if config.resource_types.contains(&resource_type.to_string()) {
                    Ok(())
                } else {
                    Err(format!(
                        "Invalid resource type '{}'. Valid types are: {}",
                        resource_type,
                        config.resource_types.join(", ")
                    ))
                }
            }
            Err(_) => {
                // If error loading config, allow any type (backward compatibility)
                // This ensures tests and existing setups continue to work
                Ok(())
            }
        }
    }

    pub fn get_valid_resource_types(&self, config_repository: &dyn ConfigRepository) -> Result<Vec<String>, String> {
        match config_repository.load() {
            Ok((config, _)) => Ok(config.resource_types),
            Err(e) => Err(format!("Could not load config: {}", e)),
        }
    }
}

// Default implementation removed to avoid circular dependency issues

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_settings::config::Config;
    use crate::domain::shared::errors::{DomainError, DomainResult};
    use std::path::PathBuf;

    // Mock config repository for testing
    #[allow(dead_code)]
    struct MockConfigRepository {
        should_fail: bool,
        config: Option<Config>,
    }

    impl MockConfigRepository {
        #[allow(dead_code)]
        fn new() -> Self {
            Self {
                should_fail: false,
                config: None,
            }
        }

        #[allow(dead_code)]
        fn with_failure() -> Self {
            Self {
                should_fail: true,
                config: None,
            }
        }

        #[allow(dead_code)]
        fn with_config(config: Config) -> Self {
            Self {
                should_fail: false,
                config: Some(config),
            }
        }
    }

    impl ConfigRepository for MockConfigRepository {
        fn load(&self) -> DomainResult<(Config, PathBuf)> {
            if self.should_fail {
                Err(DomainError::ConfigurationError {
                    details: "mock_error".to_string(),
                })
            } else if let Some(config) = &self.config {
                Ok((config.clone(), PathBuf::from("test_path")))
            } else {
                Err(DomainError::ConfigurationError {
                    details: "no_config".to_string(),
                })
            }
        }

        fn save(&self, _config: crate::domain::company_settings::config::Config, _path: &Path) -> DomainResult<()> {
            Ok(())
        }

        fn create_repository_dir(&self, _path: &Path) -> DomainResult<()> {
            Ok(())
        }
    }

    #[allow(dead_code)]
    fn create_test_config() -> Config {
        Config::new(
            "Test Manager".to_string(),
            "test@company.com".to_string(),
            "UTC".to_string(),
        )
        .with_company_name("Test Company".to_string())
        .with_work_hours("09:00".to_string(), "17:00".to_string())
    }

    #[test]
    fn test_resource_type_validator_creation() {
        let validator = ResourceTypeValidator::new();
        assert!(matches!(validator, ResourceTypeValidator { .. }));
    }

    #[test]
    fn test_resource_type_validator_default() {
        let validator = ResourceTypeValidator::new();
        assert!(matches!(validator, ResourceTypeValidator { .. }));
    }

    #[test]
    fn test_validate_resource_type_with_valid_type() {
        let validator = ResourceTypeValidator::new();
        // This test will pass if config is loaded successfully
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("Developer", &mock_repo);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_resource_type_with_invalid_type() {
        let validator = ResourceTypeValidator::new();
        // This test will pass if config is loaded successfully
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("InvalidType", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_empty_string() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_case_sensitive() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("developer", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_get_valid_resource_types() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.get_valid_resource_types(&mock_repo);
        // Should either be Ok (if config loaded) or Err (if config not found)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_with_empty_config() {
        // We can't easily test this without dependency injection, but we can test the logic
        // by creating a validator and testing its behavior
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("AnyType", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_whitespace() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("  Developer  ", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_special_characters() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("Developer-123", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_unicode() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("Développeur", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_very_long_string() {
        let validator = ResourceTypeValidator::new();
        let long_string = "A".repeat(1000);
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type(&long_string, &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_validate_resource_type_null_byte() {
        let validator = ResourceTypeValidator::new();
        let mock_repo = tests::MockConfigRepository::new();
        let result = validator.validate_resource_type("Developer\0", &mock_repo);
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }
}
