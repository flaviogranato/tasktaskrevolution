use serde::{Deserialize, Serialize};

use crate::domain::config::config::Config;
use crate::domain::shared_kernel::convertable::Convertable;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ConfigMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec: Option<ConfigSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMetadata {
    pub manager_name: String,
    pub manager_email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_hours_per_day: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_days_per_week: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_task_duration: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

impl ConfigManifest {
    pub fn new() -> Self {
        ConfigManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata::default(),
            spec: None,
        }
    }
    pub fn basic(name: &String, email: &String) -> Self {
        ConfigManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata {
                manager_name: name.to_string(),
                manager_email: email.to_string(),
            },
            spec: Some(ConfigSpec {
                currency: Some("BRL".to_string()),
                work_hours_per_day: Some(8),
                work_days_per_week: Some(vec![
                    "segunda-feira".to_string(),
                    "terça-feira".to_string(),
                    "quarta-feira".to_string(),
                    "quinta-feira".to_string(),
                    "sexta-feira".to_string(),
                ]),
                date_format: Some("yyyy-mm-dd".to_string()),
                default_task_duration: Some(8),
                locale: Some("pt_BR".to_string()),
            }),
        }
    }
}

impl Convertable<Config> for ConfigManifest {
    fn from(source: Config) -> Self {
        ConfigManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata {
                manager_name: source.manager_name,
                manager_email: source.manager_email,
            },
            spec: None,
        }
    }

    fn to(self) -> Config {
        Config {
            manager_name: self.metadata.manager_name,
            manager_email: self.metadata.manager_email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_config_serialization() {
        let config = ConfigManifest::basic(&"Test".to_string(), &"test@test.com".to_string());
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("managerName: Test"));
    }

    #[test]
    fn test_deserialize_invalid_yaml() {
        let yaml_str = "invalid: - yaml: content";
        let manifest: Result<ConfigManifest, _> = serde_yaml::from_str(yaml_str);
        assert!(manifest.is_err());
    }

    #[test]
    fn test_deserialize_valid_config() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Config
            metadata:
                managerName: "John Doe"
                managerEmail: "john@doe.com"
        "#;
        let manifest: ConfigManifest = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(manifest.metadata.manager_name, "John Doe");
    }

    #[test]
    fn test_serialize_deserialize() {
        let manifest = ConfigManifest::basic(
            &"John Doe".to_string(),
            &"john@doe.com".to_string(),
        );
        let yaml_str = serde_yaml::to_string(&manifest).unwrap();
        let manifest_deserialized: ConfigManifest = serde_yaml::from_str(&yaml_str).unwrap();
        assert_eq!(manifest.metadata.manager_name, manifest_deserialized.metadata.manager_name);
    }
}
