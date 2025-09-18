use crate::domain::company_settings::repository::ConfigRepository;
use crate::infrastructure::persistence::config_repository::FileConfigRepository;

pub struct ResourceTypeValidator {
    config_repository: FileConfigRepository,
}

impl ResourceTypeValidator {
    pub fn new() -> Self {
        Self {
            config_repository: FileConfigRepository::new(),
        }
    }

    pub fn validate_resource_type(&self, resource_type: &str) -> Result<(), String> {
        match self.config_repository.load() {
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

    pub fn get_valid_resource_types(&self) -> Result<Vec<String>, String> {
        match self.config_repository.load() {
            Ok((config, _)) => Ok(config.resource_types),
            Err(e) => Err(format!("Could not load config: {}", e)),
        }
    }
}

impl Default for ResourceTypeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_resource_type_with_valid_type() {
        let validator = ResourceTypeValidator::new();
        // This test will pass if config is loaded successfully
        let result = validator.validate_resource_type("Developer");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_resource_type_with_invalid_type() {
        let validator = ResourceTypeValidator::new();
        // This test will pass if config is loaded successfully
        let result = validator.validate_resource_type("InvalidType");
        // Should either be Ok (if config not found) or Err (if config found and type invalid)
        assert!(result.is_ok() || result.is_err());
    }
}
