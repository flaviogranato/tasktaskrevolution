use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::domain::{
    resource::resource::{Period, PeriodType, ProjectAssignment, Resource},
    shared_kernel::convertable::Convertable,
};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_assignments: Option<Vec<ProjectAssignmentManifest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacations: Option<Vec<PeriodManifest>>,
    pub time_off_balance: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssignmentManifest {
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
}

impl Convertable<ProjectAssignment> for ProjectAssignmentManifest {
    fn from(source: ProjectAssignment) -> Self {
        ProjectAssignmentManifest {
            project_id: source.project_id,
            start_date: source.start_date,
            end_date: source.end_date,
        }
    }
    fn to(self) -> ProjectAssignment {
        ProjectAssignment {
            project_id: self.project_id,
            start_date: self.start_date,
            end_date: self.end_date,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PeriodManifest {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub approved: bool,
    pub period_type: PeriodTypeManifest,
    pub is_time_off_compensation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensated_hours: Option<u32>,
}

impl Convertable<Period> for PeriodManifest {
    fn from(source: Period) -> Self {
        PeriodManifest {
            start_date: source.start_date,
            end_date: source.end_date,
            approved: source.approved,
            period_type: <PeriodTypeManifest as Convertable<PeriodType>>::from(source.period_type),
            is_time_off_compensation: source.is_time_off_compensation,
            compensated_hours: source.compensated_hours,
        }
    }
    fn to(self) -> Period {
        Period {
            start_date: self.start_date,
            end_date: self.end_date,
            approved: self.approved,
            period_type: self.period_type.to(),
            is_time_off_compensation: self.is_time_off_compensation,
            compensated_hours: self.compensated_hours,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PeriodTypeManifest {
    BirthdayBreak,
    DayOff,
    Vacation,
    SickLeave,
    PersonalLeave,
    TimeOffCompensation,
}

impl Convertable<PeriodType> for PeriodTypeManifest {
    fn from(source: PeriodType) -> Self {
        match source {
            PeriodType::BirthdayBreak => PeriodTypeManifest::BirthdayBreak,
            PeriodType::DayOff => PeriodTypeManifest::DayOff,
            PeriodType::Vacation => PeriodTypeManifest::Vacation,
            PeriodType::SickLeave => PeriodTypeManifest::SickLeave,
            PeriodType::PersonalLeave => PeriodTypeManifest::PersonalLeave,
            PeriodType::TimeOffCompensation => PeriodTypeManifest::TimeOffCompensation,
        }
    }
    fn to(self) -> PeriodType {
        match self {
            PeriodTypeManifest::BirthdayBreak => PeriodType::BirthdayBreak,
            PeriodTypeManifest::DayOff => PeriodType::DayOff,
            PeriodTypeManifest::Vacation => PeriodType::Vacation,
            PeriodTypeManifest::SickLeave => PeriodType::SickLeave,
            PeriodTypeManifest::PersonalLeave => PeriodType::PersonalLeave,
            PeriodTypeManifest::TimeOffCompensation => PeriodType::TimeOffCompensation,
        }
    }
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
    pub fn basic(name: String, resource_type: String) -> Self {
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

impl Convertable<Resource> for ResourceManifest {
    fn from(source: Resource) -> Self {
        ResourceManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                code: source.id,
                name: source.name,
                email: source.email,
                resource_type: source.resource_type,
            },
            spec: ResourceSpec {
                project_assignments: source.project_assignments.map(|project_assignments| {
                    project_assignments
                        .into_iter()
                        .map(<ProjectAssignmentManifest as Convertable<ProjectAssignment>>::from)
                        .collect()
                }),
                vacations: source.vacations.map(|periods| {
                    periods
                        .into_iter()
                        .map(<PeriodManifest as Convertable<Period>>::from)
                        .collect()
                }),
                time_off_balance: source.time_off_balance,
            },
        }
    }
    fn to(self) -> Resource {
        Resource {
            id: self.metadata.code,
            name: self.metadata.name,
            email: self.metadata.email,
            resource_type: self.metadata.resource_type,
            time_off_balance: self.spec.time_off_balance,
            vacations: self
                .spec
                .vacations
                .map(|periods| periods.into_iter().map(|period| period.to()).collect()),
            project_assignments: self
                .spec
                .project_assignments
                .map(|assignments| assignments.into_iter().map(|assign| assign.to()).collect()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Local;

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
        assert_eq!(manifest.spec.project_assignments, Some(vec![]));
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
                project_assignments: Some(vec![ProjectAssignmentManifest {
                    project_id: "proj1".to_string(),
                    start_date: Local::now(),
                    end_date: Local::now(),
                }]),
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

        let period = PeriodManifest {
            start_date: now.clone(),
            end_date: tomorrow.clone(),
            approved: true,
            period_type: PeriodTypeManifest::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
        };

        assert_eq!(period.start_date, now);
        assert_eq!(period.end_date, tomorrow);
        assert!(period.approved);
        assert_eq!(period.period_type, PeriodTypeManifest::Vacation);
        assert!(!period.is_time_off_compensation);
        assert!(period.compensated_hours.is_none());
    }

    #[test]
    fn test_period_with_some_fields() {
        let period = PeriodManifest {
            start_date: Local::now(),
            end_date: Local::now(),
            approved: false,
            period_type: PeriodTypeManifest::Vacation,
            is_time_off_compensation: true,
            compensated_hours: Some(8),
        };

        assert!(!period.approved);
        assert_eq!(period.period_type, PeriodTypeManifest::Vacation);
        assert!(period.is_time_off_compensation);
        assert_eq!(period.compensated_hours.unwrap(), 8);
    }
}
