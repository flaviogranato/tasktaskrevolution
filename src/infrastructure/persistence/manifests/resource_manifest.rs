use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::domain::resource_management::{
    AnyResource, Resource, TimeOffEntry,
    resource::{Period, PeriodType, ProjectAssignment},
    state::{Assigned, Available, Inactive},
};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

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
    #[serde(default)]
    pub time_off_history: Option<Vec<TimeOffEntry>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub name: String,
    #[serde(default)]
    pub email: String,
    pub code: String,
    pub resource_type: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAssignmentManifest {
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub allocation_percentage: u8,
}

impl From<ProjectAssignment> for ProjectAssignmentManifest {
    fn from(source: ProjectAssignment) -> Self {
        Self {
            project_id: source.project_id,
            start_date: source.start_date,
            end_date: source.end_date,
            allocation_percentage: source.allocation_percentage,
        }
    }
}
impl ProjectAssignmentManifest {
    pub fn to(&self) -> ProjectAssignment {
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

impl From<Period> for PeriodManifest {
    fn from(source: Period) -> Self {
        Self {
            start_date: source.start_date,
            end_date: source.end_date,
            approved: source.approved,
            period_type: PeriodTypeManifest::from(source.period_type),
            is_time_off_compensation: source.is_time_off_compensation,
            compensated_hours: source.compensated_hours,
            is_layoff: source.is_layoff,
        }
    }
}
impl PeriodManifest {
    pub fn to(&self) -> Period {
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

impl From<PeriodType> for PeriodTypeManifest {
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
}
impl PeriodTypeManifest {
    pub fn to(&self) -> PeriodType {
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

impl From<AnyResource> for ResourceManifest {
    fn from(source: AnyResource) -> Self {
        let (name, email, code, resource_type, spec) = match source {
            AnyResource::Available(r) => (
                r.name,
                r.email,
                r.id,
                r.resource_type,
                ResourceSpec {
                    vacations: r.vacations.map(|v| v.into_iter().map(PeriodManifest::from).collect()),
                    project_assignments: None,
                    time_off_balance: r.time_off_balance,
                    time_off_history: r.time_off_history,
                },
            ),
            AnyResource::Assigned(r) => (
                r.name,
                r.email,
                r.id,
                r.resource_type,
                ResourceSpec {
                    vacations: r.vacations.map(|v| v.into_iter().map(PeriodManifest::from).collect()),
                    project_assignments: Some(
                        r.state
                            .project_assignments
                            .into_iter()
                            .map(ProjectAssignmentManifest::from)
                            .collect(),
                    ),
                    time_off_balance: r.time_off_balance,
                    time_off_history: r.time_off_history,
                },
            ),
            AnyResource::Inactive(r) => (
                r.name,
                r.email,
                r.id,
                r.resource_type,
                ResourceSpec {
                    vacations: r.vacations.map(|v| v.into_iter().map(PeriodManifest::from).collect()),
                    project_assignments: None,
                    time_off_balance: r.time_off_balance,
                    time_off_history: r.time_off_history,
                },
            ),
        };

        Self {
            api_version: API_VERSION.to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                name,
                email: email.unwrap_or_default(),
                code: code.unwrap_or_default(),
                resource_type,
            },
            spec,
        }
    }
}

impl TryFrom<ResourceManifest> for AnyResource {
    type Error = String;

    fn try_from(manifest: ResourceManifest) -> Result<Self, Self::Error> {
        let id = Some(manifest.metadata.code);
        let name = manifest.metadata.name;
        let email = if manifest.metadata.email.is_empty() {
            None
        } else {
            Some(manifest.metadata.email)
        };
        let resource_type = manifest.metadata.resource_type;
        let vacations = manifest.spec.vacations.map(|v| v.into_iter().map(|p| p.to()).collect());
        let time_off_balance = manifest.spec.time_off_balance;
        let time_off_history = manifest.spec.time_off_history;

        if let Some(assignments_manifest) = manifest.spec.project_assignments {
            if !assignments_manifest.is_empty() {
                let project_assignments = assignments_manifest.into_iter().map(|a| a.to()).collect();
                return Ok(AnyResource::Assigned(Resource {
                    id,
                    name,
                    email,
                    resource_type,
                    vacations,
                    time_off_balance,
                    time_off_history,
                    state: Assigned { project_assignments },
                }));
            }
        }

        // Default to Available
        Ok(AnyResource::Available(Resource {
            id,
            name,
            email,
            resource_type,
            vacations,
            time_off_balance,
            time_off_history,
            state: Available,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::state::Available;

    #[test]
    fn test_bidirectional_conversion() {
        // Create an Available resource
        let original_resource = Resource::<Available>::new(
            Some("res-1".to_string()),
            "John Doe".to_string(),
            Some("john@doe.com".to_string()),
            "Developer".to_string(),
            None,
            40,
        );

        // Convert to Manifest
        let manifest = ResourceManifest::from(AnyResource::Available(original_resource.clone()));
        assert_eq!(manifest.metadata.name, "John Doe");
        assert_eq!(manifest.spec.time_off_balance, 40);
        assert!(manifest.spec.project_assignments.is_none());

        // Convert back to AnyResource
        let converted_any = AnyResource::try_from(manifest).unwrap();

        // Assert it's an Available resource with correct data
        if let AnyResource::Available(converted) = converted_any {
            assert_eq!(original_resource.name, converted.name);
            assert_eq!(original_resource.email, converted.email);
            assert_eq!(original_resource.time_off_balance, converted.time_off_balance);
        } else {
            panic!("Expected resource to be in Available state");
        }
    }

    #[test]
    fn test_assigned_conversion() {
        let resource = Resource::<Available>::new(
            Some("res-2".to_string()),
            "Jane Doe".to_string(),
            None,
            "QA".to_string(),
            None,
            0,
        );
        let assignment = ProjectAssignment {
            project_id: "PROJ-1".to_string(),
            start_date: Local::now(),
            end_date: Local::now(),
            allocation_percentage: 100,
        };
        let assigned_resource = resource.assign_to_project(assignment);

        let manifest = ResourceManifest::from(AnyResource::Assigned(assigned_resource));
        assert!(manifest.spec.project_assignments.is_some());
        assert_eq!(manifest.spec.project_assignments.as_ref().unwrap().len(), 1);

        let converted_any = AnyResource::try_from(manifest).unwrap();
        assert!(matches!(converted_any, AnyResource::Assigned(_)));
    }
}
