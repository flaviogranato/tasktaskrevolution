use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::domain::{
    resource_management::resource::{Period, PeriodType, ProjectAssignment, Resource},
    shared::convertable::Convertable,
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
pub struct ResourceSpec {
    pub vacations: Option<Vec<PeriodManifest>>,
    pub project_assignments: Option<Vec<ProjectAssignmentManifest>>,
    pub time_off_balance: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub name: String,
    pub email: Option<String>,
    pub code: String,
    pub resource_type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssignmentManifest {
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub allocation_percentage: u8,
}

impl Convertable<ProjectAssignment> for ProjectAssignmentManifest {
    fn from(source: ProjectAssignment) -> Self {
        Self {
            project_id: source.project_id,
            start_date: source.start_date,
            end_date: source.end_date,
            allocation_percentage: source.allocation_percentage,
        }
    }
    fn to(&self) -> ProjectAssignment {
        ProjectAssignment {
            project_id: self.project_id.clone(),
            start_date: self.start_date,
            end_date: self.end_date,
            allocation_percentage: self.allocation_percentage,
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
    pub is_layoff: bool,
}

impl Convertable<Period> for PeriodManifest {
    fn from(source: Period) -> Self {
        Self {
            start_date: source.start_date,
            end_date: source.end_date,
            approved: source.approved,
            period_type: <PeriodTypeManifest as Convertable<PeriodType>>::from(source.period_type),
            is_time_off_compensation: source.is_time_off_compensation,
            compensated_hours: source.compensated_hours,
            is_layoff: source.is_layoff,
        }
    }
    fn to(&self) -> Period {
        Period {
            start_date: self.start_date,
            end_date: self.end_date,
            approved: self.approved,
            period_type: self.period_type.to(),
            is_time_off_compensation: self.is_time_off_compensation,
            compensated_hours: self.compensated_hours,
            is_layoff: self.is_layoff,
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
    TimeOff,
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
            PeriodType::TimeOff => PeriodTypeManifest::TimeOff,
        }
    }
    fn to(&self) -> PeriodType {
        match self {
            PeriodTypeManifest::BirthdayBreak => PeriodType::BirthdayBreak,
            PeriodTypeManifest::DayOff => PeriodType::DayOff,
            PeriodTypeManifest::Vacation => PeriodType::Vacation,
            PeriodTypeManifest::SickLeave => PeriodType::SickLeave,
            PeriodTypeManifest::PersonalLeave => PeriodType::PersonalLeave,
            PeriodTypeManifest::TimeOffCompensation => PeriodType::TimeOffCompensation,
            PeriodTypeManifest::TimeOff => PeriodType::TimeOff,
        }
    }
}

impl Convertable<Resource> for ResourceManifest {
    fn from(source: Resource) -> Self {
        Self {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                name: source.name.clone(),
                email: source.email.clone(),
                code: source.id.unwrap_or_default(),
                resource_type: source.resource_type,
            },
            spec: ResourceSpec {
                vacations: source.vacations.map(|v| {
                    v.into_iter()
                        .map(<PeriodManifest as Convertable<Period>>::from)
                        .collect()
                }),
                project_assignments: source.project_assignments.map(|pa| {
                    pa.into_iter()
                        .map(<ProjectAssignmentManifest as Convertable<ProjectAssignment>>::from)
                        .collect()
                }),
                time_off_balance: source.time_off_balance,
            },
        }
    }

    fn to(&self) -> Resource {
        Resource::new(
            Some(self.metadata.code.clone()),
            self.metadata.name.clone(),
            self.metadata.email.clone(),
            self.metadata.resource_type.clone(),
            self.spec
                .vacations
                .as_ref()
                .map(|v| v.iter().map(|p| p.to()).collect()),
            self.spec
                .project_assignments
                .as_ref()
                .map(|pa| pa.iter().map(|a| a.to()).collect()),
            self.spec.time_off_balance,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_invalid_yaml() {
        let yaml_str = "invalid: - yaml: content";
        let manifest: Result<ResourceManifest, _> = serde_yaml::from_str(yaml_str);
        assert!(manifest.is_err());
    }

    #[test]
    fn test_deserialize_valid_resource() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                name: "John Doe"
                email: "john@doe.com"
                code: "john-doe"
                resourceType: "Developer"
            spec:
                vacations: null
                projectAssignments: null
                timeOffBalance: 0
        "#;
        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(manifest.metadata.name, "John Doe");
        assert_eq!(manifest.metadata.code, "john-doe");
        assert_eq!(manifest.metadata.resource_type, "Developer");
    }
}
