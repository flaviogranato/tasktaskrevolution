use crate::domain::company_settings::Config;
use crate::domain::shared::convertable::Convertable;
use crate::infrastructure::persistence::manifests::project_manifest::VacationRulesManifest;
use chrono::Utc;
use serde::{Deserialize, Serialize};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ConfigMetadata,
    pub spec: ConfigSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMetadata {
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSpec {
    pub manager_name: String,
    pub manager_email: String,
    #[serde(default = "default_timezone")]
    pub default_timezone: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacation_rules: Option<VacationRulesManifest>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

impl ConfigManifest {
    pub fn new() -> Self {
        ConfigManifest {
            api_version: API_VERSION.to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata { created_at: Utc::now() },
            spec: ConfigSpec {
                manager_name: "Default Manager".to_string(),
                manager_email: "email@example.com".to_string(),
                default_timezone: "UTC".to_string(),
                currency: None,
                work_hours_per_day: None,
                work_days_per_week: None,
                date_format: None,
                default_task_duration: None,
                locale: None,
                vacation_rules: None,
            },
        }
    }
}

impl Default for ConfigManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl Convertable<Config> for ConfigManifest {
    fn from(source: Config) -> Self {
        ConfigManifest {
            api_version: API_VERSION.to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata { created_at: Utc::now() },
            spec: ConfigSpec {
                manager_name: source.manager_name,
                manager_email: source.manager_email,
                default_timezone: "UTC".to_string(),
                currency: None,
                work_hours_per_day: None,
                work_days_per_week: None,
                date_format: None,
                default_task_duration: None,
                locale: None,
                vacation_rules: None,
            },
        }
    }

    fn to(&self) -> Config {
        Config {
            manager_name: self.spec.manager_name.clone(),
            manager_email: self.spec.manager_email.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                createdAt: "2024-01-01T00:00:00Z"
            spec:
                managerName: "John Doe"
                managerEmail: "john@doe.com"
                defaultTimezone: "UTC"
        "#;
        let manifest: ConfigManifest = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(manifest.spec.manager_name, "John Doe");
    }
}
