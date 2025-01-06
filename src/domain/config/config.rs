use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let name = "John Doe".to_string();
        let email = "john.doe@example.com".to_string();

        let expected_config = ConfigManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Config".to_string(),
            metadata: ConfigMetadata {
                manager_name: name.clone(),
                manager_email: email.clone(),
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
        };

        let actual_config = ConfigManifest::basic(&name, &email);

        assert_eq!(expected_config, actual_config);
    }

    #[test]
    fn test_yaml_serialization() {
        let name = "Jane Doe".to_string();
        let email = "jane.doe@example.com".to_string();
        let config = ConfigManifest::basic(&name, &email);

        let yaml = serde_yml::to_string(&config).unwrap();

        // Você pode descomentar para ver o YAML gerado:
        println!("{}", yaml);

        // Verifique se alguns campos importantes estão presentes no YAML:
        assert!(yaml.contains("apiVersion: tasktaskrevolution.io/v1alpha1"));
        assert!(yaml.contains("kind: Config"));
        assert!(yaml.contains("managerName: Jane Doe"));
        assert!(yaml.contains("managerEmail: jane.doe@example.com"));
    }

    #[test]
    fn test_config_manifest_new() {
        let manifest = ConfigManifest::new();

        assert_eq!(
            manifest.api_version,
            "tasktaskrevolution.io/v1alpha1".to_string()
        );
        assert_eq!(manifest.kind, "Config".to_string());
        assert_eq!(manifest.metadata, ConfigMetadata::default());
        assert_eq!(manifest.spec, None);
    }

    #[test]
    fn test_config_manifest_deserialize_empty() {
        let yaml_str = ""; // YAML vazio

        let manifest: Result<ConfigManifest, _> = serde_yml::from_str(yaml_str);

        assert!(manifest.is_err()); // Deve dar erro ao desserializar YAML vazio.
    }

    #[test]
    fn test_config_manifest_deserialize_with_data() {
        let yaml_str = r#"
     apiVersion: tasktaskrevolution.io/v1alpha1
     kind: Config
     metadata:
       managerName: John Doe
       managerEmail: john.doe@example.com
     spec:
     "#;

        let manifest: ConfigManifest = serde_yml::from_str(yaml_str).unwrap();

        assert_eq!(
            manifest.api_version,
            "tasktaskrevolution.io/v1alpha1".to_string()
        );
        assert_eq!(manifest.kind, "Config".to_string());
        assert_eq!(manifest.metadata.manager_name, "John Doe".to_string());
        assert_eq!(
            manifest.metadata.manager_email,
            "john.doe@example.com".to_string()
        );
        assert_eq!(manifest.spec, None);
    }

    #[test]
    fn test_config_manifest_serialize() {
        let manifest = ConfigManifest {
            api_version: "tasktaskrevolution/v1alpha1".to_string(),
            kind: "TestKind".to_string(),
            metadata: ConfigMetadata {
                manager_name: "Test Name".to_string(),
                manager_email: "test@email.com".to_string(),
            },
            spec: None,
        };

        let yaml_str = serde_yml::to_string(&manifest).unwrap();
        let manifest_deserialized: ConfigManifest = serde_yml::from_str(&yaml_str).unwrap();
        assert_eq!(manifest, manifest_deserialized);
    }
}
