use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid7::Uuid;

use crate::domain::resource_management::{
    AnyResource,
    resource::{Period, PeriodType, ProjectAssignment, Resource, ResourceScope, TimeOffEntry, WipLimits},
    state::{Assigned, Available},
};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

fn default_status() -> String {
    "Available".to_string()
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<chrono::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<chrono::NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacations: Option<Vec<PeriodManifest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_assignments: Option<Vec<ProjectAssignmentManifest>>,
    pub time_off_balance: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_off_history: Option<Vec<TimeOffEntry>>,
    #[serde(default = "default_scope")]
    pub scope: ResourceScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
}

fn default_scope() -> ResourceScope {
    ResourceScope::Company
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub email: String,
    pub code: String,
    pub resource_type: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
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
        let (id, code, name, email, resource_type, status, spec) = match source {
            AnyResource::Available(r) => (
                r.id,
                r.code,
                r.name,
                r.email,
                r.resource_type,
                "Available".to_string(),
                ResourceSpec {
                    start_date: r.start_date,
                    end_date: r.end_date,
                    vacations: r.vacations.map(|v| v.into_iter().map(PeriodManifest::from).collect()),
                    project_assignments: None,
                    time_off_balance: r.time_off_balance,
                    time_off_history: r.time_off_history,
                    scope: r.scope,
                    project_id: r.project_id,
                },
            ),
            AnyResource::Assigned(r) => (
                r.id,
                r.code,
                r.name,
                r.email,
                r.resource_type,
                "Assigned".to_string(),
                ResourceSpec {
                    start_date: r.start_date,
                    end_date: r.end_date,
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
                    scope: r.scope,
                    project_id: r.project_id,
                },
            ),
            AnyResource::Inactive(r) => (
                r.id,
                r.code,
                r.name,
                r.email,
                r.resource_type,
                "Inactive".to_string(),
                ResourceSpec {
                    start_date: r.start_date,
                    end_date: r.end_date,
                    vacations: r.vacations.map(|v| v.into_iter().map(PeriodManifest::from).collect()),
                    project_assignments: None,
                    time_off_balance: r.time_off_balance,
                    time_off_history: r.time_off_history,
                    scope: r.scope,
                    project_id: r.project_id,
                },
            ),
        };

        Self {
            api_version: API_VERSION.to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                id: Some(id.to_string()),
                name,
                email: email.unwrap_or_default(),
                code,
                resource_type,
                status,
                description: None,
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec,
        }
    }
}

impl TryFrom<ResourceManifest> for AnyResource {
    type Error = String;

    fn try_from(manifest: ResourceManifest) -> Result<Self, Self::Error> {
        let id = manifest
            .metadata
            .id
            .and_then(|id_str| Uuid::from_str(&id_str).ok())
            .unwrap_or_else(uuid7::uuid7);

        let code = manifest.metadata.code.clone();
        let name = manifest.metadata.name.clone();
        let email = if manifest.metadata.email.is_empty() {
            None
        } else {
            Some(manifest.metadata.email.clone())
        };
        let resource_type = manifest.metadata.resource_type.clone();
        let start_date = manifest.spec.start_date;
        let end_date = manifest.spec.end_date;
        let vacations = manifest
            .spec
            .vacations
            .as_ref()
            .map(|v| v.iter().map(|p| p.to()).collect());
        let time_off_balance = manifest.spec.time_off_balance;
        let time_off_history = manifest.spec.time_off_history.clone();
        let status = manifest.metadata.status.as_str();

        match status {
            "Assigned" => {
                // Validate that Assigned status requires project_assignments
                if manifest.spec.project_assignments.is_none() || manifest.spec.project_assignments.as_ref().unwrap().is_empty() {
                    return Err("Assigned status requires project_assignments to be present and non-empty".to_string());
                }
                
                let project_assignments = manifest
                    .spec
                    .project_assignments
                    .unwrap()
                    .into_iter()
                    .map(|a| a.to())
                    .collect();
                Ok(AnyResource::Assigned(Resource {
                    id,
                    code,
                    name,
                    email,
                    resource_type,
                    scope: manifest.spec.scope,
                    project_id: manifest.spec.project_id,
                    start_date,
                    end_date,
                    vacations,
                    time_off_balance,
                    time_off_history,
                    wip_limits: Some(WipLimits::new(5, 3, 100)),
                    task_assignments: Some(Vec::new()),
                    state: Assigned { project_assignments },
                }))
            }
            "Inactive" => Ok(AnyResource::Inactive(Resource {
                id,
                code,
                name,
                email,
                resource_type,
                scope: manifest.spec.scope,
                project_id: manifest.spec.project_id,
                start_date,
                end_date,
                vacations,
                time_off_balance,
                time_off_history,
                wip_limits: Some(WipLimits::new(5, 3, 100)),
                task_assignments: Some(Vec::new()),
                state: crate::domain::resource_management::state::Inactive,
            })),
            "Available" => {
                // Validate that Available status should not have project_assignments
                if manifest.spec.project_assignments.is_some() && !manifest.spec.project_assignments.as_ref().unwrap().is_empty() {
                    return Err("Available status should not have project_assignments".to_string());
                }
                
                Ok(AnyResource::Available(Resource {
                    id,
                    code,
                    name,
                    email,
                    resource_type,
                    scope: manifest.spec.scope,
                    project_id: manifest.spec.project_id,
                    start_date,
                    end_date,
                    vacations,
                    time_off_balance,
                    time_off_history,
                    wip_limits: Some(WipLimits::new(5, 3, 100)),
                    task_assignments: Some(Vec::new()),
                    state: Available,
                }))
            }
            _ => {
                // Default to Available for unknown statuses
                Ok(AnyResource::Available(Resource {
                    id,
                    code,
                    name,
                    email,
                    resource_type,
                    scope: manifest.spec.scope,
                    project_id: manifest.spec.project_id,
                    start_date,
                    end_date,
                    vacations,
                    time_off_balance,
                    time_off_history,
                    wip_limits: Some(WipLimits::new(5, 3, 100)),
                    task_assignments: Some(Vec::new()),
                    state: Available,
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::state::Available;
    use uuid7::uuid7;

    #[test]
    fn test_bidirectional_conversion() {
        // Create an Available resource
        let original_resource = Resource::<Available>::new(
            "dev-1".to_string(),
            "John Doe".to_string(),
            Some("john@doe.com".to_string()),
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            40,
        );

        // Convert to Manifest
        let manifest = ResourceManifest::from(AnyResource::Available(original_resource.clone()));
        assert_eq!(manifest.metadata.name, "John Doe");
        assert_eq!(manifest.metadata.code, "dev-1");
        assert_eq!(manifest.spec.time_off_balance, 40);
        assert!(manifest.spec.project_assignments.is_none());

        // Convert back to AnyResource
        let converted_any = AnyResource::try_from(manifest).unwrap();

        // Assert it's an Available resource with correct data
        if let AnyResource::Available(converted) = converted_any {
            assert_eq!(original_resource.id, converted.id);
            assert_eq!(original_resource.code, converted.code);
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
            "qa-1".to_string(),
            "Jane Doe".to_string(),
            None,
            "QA".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
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

    #[test]
    fn test_inactive_conversion() {
        // This test ensures that a manifest can be converted back and forth,
        // even if the resource is conceptually "inactive". The state is determined
        // by assignments, so a resource without assignments becomes "Available".
        let id = uuid7();
        let manifest = ResourceManifest {
            api_version: API_VERSION.to_string(),
            kind: "Resource".to_string(),
            metadata: ResourceMetadata {
                id: Some(id.to_string()),
                name: "Inactive User".to_string(),
                email: "".to_string(),
                code: "former-1".to_string(),
                resource_type: "Former".to_string(),
                status: "Inactive".to_string(),
                description: None,
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec: ResourceSpec {
                time_off_balance: 0,
                ..Default::default()
            },
        };

        let converted_any = AnyResource::try_from(manifest).unwrap();
        // Resource with status "Inactive" should be converted to Inactive state
        assert!(matches!(converted_any, AnyResource::Inactive(_)));
        if let AnyResource::Inactive(r) = converted_any {
            assert_eq!(r.id, id);
            assert_eq!(r.code, "former-1");
        }
    }

    #[test]
    fn test_conversion_with_vacations() {
        let mut resource = Resource::<Available>::new(
            "manager-1".to_string(),
            "On Holiday".to_string(),
            None,
            "Manager".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            80,
        );

        let vacation = Period {
            start_date: Local::now(),
            end_date: Local::now(),
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };
        resource.vacations = Some(vec![vacation]);

        let manifest = ResourceManifest::from(AnyResource::Available(resource.clone()));
        assert!(manifest.spec.vacations.is_some());
        assert_eq!(manifest.spec.vacations.as_ref().unwrap().len(), 1);

        let converted_any = AnyResource::try_from(manifest).unwrap();
        if let AnyResource::Available(converted) = converted_any {
            assert_eq!(converted.vacations.unwrap().len(), 1);
        } else {
            panic!("Expected Available state");
        }
    }

    #[test]
    fn test_conversion_no_email() {
        let original_resource = Resource::<Available>::new(
            "contractor-1".to_string(),
            "No Email".to_string(),
            None, // No email
            "Contractor".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            0,
        );

        let manifest = ResourceManifest::from(AnyResource::Available(original_resource.clone()));
        assert_eq!(manifest.metadata.email, ""); // Converts to empty string

        let converted_any = AnyResource::try_from(manifest).unwrap();
        if let AnyResource::Available(converted) = converted_any {
            assert_eq!(converted.email, None); // Converts back to None
        } else {
            panic!("Expected Available state");
        }
    }

    #[test]
    fn test_yaml_parsing_success() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                resourceType: "Developer"
                status: "Available"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                scope: "Company"
                timeOffBalance: 25
        "#;

        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Resource");
        assert_eq!(manifest.metadata.code, "DEV-001");
        assert_eq!(manifest.metadata.name, "John Doe");
        assert_eq!(manifest.metadata.email, "john@example.com");
        assert_eq!(manifest.metadata.resource_type, "Developer");
        assert_eq!(manifest.spec.scope, ResourceScope::Company);
        assert_eq!(manifest.spec.time_off_balance, 25);
        assert_eq!(manifest.metadata.status, "Available");
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_syntax() {
        let yaml_str = "invalid: yaml: content: [";
        let result: Result<ResourceManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_missing_required_field() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                # Missing required fields: code, name, createdAt, updatedAt, createdBy
            spec:
                resourceType: "Developer"
                scope: "Company"
        "#;

        let result: Result<ResourceManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_field_type() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                resourceType: "Developer"
                scope: "invalid_scope"  # Invalid enum value
                timeOffBalance: 25
        "#;

        let result: Result<ResourceManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_date_format() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                createdAt: "invalid-date"  # Invalid date format
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                resourceType: "Developer"
                scope: "Company"
                timeOffBalance: 25
        "#;

        let result: Result<ResourceManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_status_consistency_validation() {
        // Test that Assigned status requires project_assignments
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                resourceType: "Developer"
                status: "Assigned"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                scope: "Company"
                timeOffBalance: 25
                # Missing project_assignments for Assigned status
        "#;

        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        let result = AnyResource::try_from(manifest);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Assigned status requires project_assignments"));
    }

    #[test]
    fn test_status_consistency_validation_available_with_assignments() {
        // Test that Available status should not have project_assignments
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                resourceType: "Developer"
                status: "Available"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                scope: "Company"
                timeOffBalance: 25
                projectAssignments:
                  - projectId: "PROJ-001"
                    startDate: "2024-01-01T00:00:00Z"
                    endDate: "2024-12-31T23:59:59Z"
                    allocationPercentage: 100
        "#;

        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        let result = AnyResource::try_from(manifest);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Available status should not have project_assignments"));
    }

    #[test]
    fn test_status_consistency_validation_assigned_with_empty_assignments() {
        // Test that Assigned status requires non-empty project_assignments
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                resourceType: "Developer"
                status: "Assigned"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                scope: "Company"
                timeOffBalance: 25
                projectAssignments: []  # Empty assignments for Assigned status
        "#;

        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        let result = AnyResource::try_from(manifest);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Assigned status requires project_assignments"));
    }

    #[test]
    fn test_yaml_parsing_success_with_optional_fields() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Resource
            metadata:
                id: "01996dev-0000-0000-0000-000000res"
                code: "DEV-001"
                name: "John Doe"
                email: "john@example.com"
                resourceType: "Senior Developer"
                status: "Assigned"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                scope: "Project"
                timeOffBalance: 30
                projectAssignments:
                    - projectId: "PROJ-001"
                      startDate: "2024-01-01T00:00:00+00:00"
                      endDate: "2024-12-31T23:59:59+00:00"
                      allocationPercentage: 100
                vacations:
                    - startDate: "2024-07-01T00:00:00+00:00"
                      endDate: "2024-07-15T23:59:59+00:00"
                      approved: true
                      periodType: "vacation"
                      isTimeOffCompensation: false
                      compensatedHours: null
                      isLayoff: false
        "#;

        let manifest: ResourceManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.metadata.resource_type, "Senior Developer");
        assert_eq!(manifest.spec.scope, ResourceScope::Project);
        assert_eq!(manifest.spec.time_off_balance, 30);
        assert_eq!(manifest.metadata.status, "Assigned");
        assert!(manifest.spec.project_assignments.is_some());
        assert!(manifest.spec.vacations.is_some());
        assert_eq!(manifest.spec.vacations.as_ref().unwrap().len(), 1);
    }
}
