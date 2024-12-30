use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ResourceMetadata,
    pub spec: ResourceSpec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub name: String,
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSpec {
    pub project_assignments: Vec<ProjectAssignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacations: Option<Vec<Period>>,
    pub time_off_balance: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssignment {
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Local>>,
    pub approved: bool,
    pub period_type: PeriodType,
    pub is_time_off_compensation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensated_hours: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PeriodType {
    BirthdayBreak,
    DayOff,
    Vacation,
    SickLeave,
    PersonalLeave,
    TimeOffCompensation,
    Other(String),
}

impl ResourceManifest {
    pub fn new() -> Self {
        ResourceManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata::default(),
            spec: ResourceSpec::default(),
        }
    }
    pub fn basic(name: String, resource_type: String, project: Option<String>) -> Self {
        ResourceManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                code: None,
                name,
                email: None,
                resource_type,
            },
            spec: ResourceSpec::default(),
        }
    }
}

#[cfg(test)]
mod test {

    use chrono::Local;

    use crate::entities::resource::{
        Period, PeriodType, ProjectAssignment, ResourceManifest, ResourceMetadata, ResourceSpec,
    };

    #[test]
    fn test_resource_manifest_new() {
        let manifest = ResourceManifest::new();

        assert_eq!(
            manifest.api_version,
            "tasktaskrevolution.io/v1alpha1".to_string()
        );
        assert_eq!(manifest.kind, "Resource".to_string());
        assert_eq!(manifest.metadata, ResourceMetadata::default());
        assert_eq!(manifest.spec, ResourceSpec::default());
    }

    #[test]
    fn test_resource_manifest_deserialize_empty() {
        let yaml_str = ""; // YAML vazio

        let manifest: Result<ResourceManifest, _> = serde_yml::from_str(yaml_str);

        assert!(manifest.is_err()); // Deve dar erro ao desserializar YAML vazio.
    }

    #[test]
    fn test_resource_manifest_deserialize_with_data() {
        let yaml_str = r#"
apiVersion: custom.io/v1
kind: CustomResource
metadata:
  code: ABC123
  name: John Doe
  resourceType: devops
  email: john.doe@example.com
spec:
  projectAssignments: []
  vacations: null
  timeOffBalance: 0
"#;

        let manifest: ResourceManifest = serde_yml::from_str(yaml_str).unwrap();

        assert_eq!(manifest.api_version, "custom.io/v1".to_string());
        assert_eq!(manifest.kind, "CustomResource".to_string());
        assert_eq!(manifest.metadata.code, Some("ABC123".to_string()));
        assert_eq!(manifest.metadata.resource_type, "devops".to_string());
        assert_eq!(manifest.metadata.name, "John Doe".to_string());
        assert_eq!(
            manifest.metadata.email,
            Some("john.doe@example.com".to_string())
        );
        assert_eq!(manifest.spec.project_assignments, vec![]);
        assert_eq!(manifest.spec.vacations, None);
        assert_eq!(manifest.spec.time_off_balance, 0);
    }

    #[test]
    fn test_resource_manifest_serialize() {
        let manifest = ResourceManifest {
            api_version: "test/v1".to_string(),
            kind: "TestKind".to_string(),
            metadata: ResourceMetadata {
                code: Some("TESTCODE".to_string()),
                name: "Test Name".to_string(),
                resource_type: "devops".to_string(),
                email: Some("test@email.com".to_string()),
            },
            spec: ResourceSpec {
                project_assignments: vec![ProjectAssignment {
                    project_id: "proj1".to_string(),
                    start_date: None,
                    end_date: None,
                }],
                vacations: None,
                time_off_balance: 10,
            },
        };

        let yaml_str = serde_yml::to_string(&manifest).unwrap();
        let manifest_deserialized: ResourceManifest = serde_yml::from_str(&yaml_str).unwrap();
        assert_eq!(manifest, manifest_deserialized);
    }

    #[test]
    fn test_period_with_all_fields() {
        let now = Local::now();
        let tomorrow = now + chrono::Duration::days(1);

        let period = Period {
            start_date: Some(now.clone()),
            end_date: Some(tomorrow.clone()),
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
        };

        assert_eq!(period.start_date.unwrap(), now);
        assert_eq!(period.end_date.unwrap(), tomorrow);
        assert!(period.approved);
        assert_eq!(period.period_type, PeriodType::Vacation);
        assert!(!period.is_time_off_compensation);
        assert!(period.compensated_hours.is_none());
    }

    #[test]
    fn test_period_with_some_fields() {
        let period = Period {
            start_date: None,
            end_date: Some(Local::now()),
            approved: false,
            period_type: PeriodType::Other("Testing".to_string()),
            is_time_off_compensation: true,
            compensated_hours: Some(8),
        };

        assert!(period.start_date.is_none());
        assert!(period.end_date.is_some());
        assert!(!period.approved);
        assert_eq!(period.period_type, PeriodType::Other("Testing".to_string()));
        assert!(period.is_time_off_compensation);
        assert_eq!(period.compensated_hours.unwrap(), 8);
    }
}
