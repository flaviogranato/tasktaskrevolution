use crate::domain::project_management::{AnyProject, layoff_period::LayoffPeriod};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use uuid7::{Uuid, uuid7};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

/// Parse date from ISO format (YYYY-MM-DD)
fn parse_date_opt(s: &Option<String>) -> Result<Option<chrono::NaiveDate>, String> {
    s.as_ref()
        .map(|v| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").map_err(|e| e.to_string()))
        .transpose()
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ProjectMetadata,
    pub spec: ProjectSpec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub company_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSpec {
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    pub status: ProjectStatusManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacation_rules: Option<VacationRulesManifest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VacationRulesManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_vacations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub carry_over_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_layoff_vacations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_layoff_vacation_period: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layoff_periods: Option<Vec<LayoffPeriodManifest>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoffPeriodManifest {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ProjectStatusManifest {
    Planned,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

impl From<&crate::domain::project_management::project::ProjectStatus> for ProjectStatusManifest {
    fn from(status: &crate::domain::project_management::project::ProjectStatus) -> Self {
        match status {
            crate::domain::project_management::project::ProjectStatus::Planned => ProjectStatusManifest::Planned,
            crate::domain::project_management::project::ProjectStatus::InProgress => ProjectStatusManifest::InProgress,
            crate::domain::project_management::project::ProjectStatus::OnHold => ProjectStatusManifest::OnHold,
            crate::domain::project_management::project::ProjectStatus::Completed => ProjectStatusManifest::Completed,
            crate::domain::project_management::project::ProjectStatus::Cancelled => ProjectStatusManifest::Cancelled,
        }
    }
}

impl From<AnyProject> for ProjectManifest {
    fn from(source: AnyProject) -> Self {
        let (
            id,
            code,
            name,
            description,
            company_code,
            start_date,
            end_date,
            vacation_rules,
            timezone,
            status_manifest,
        ) = match source {
            AnyProject::Project(p) => (
                p.id,
                p.code,
                p.name,
                p.description,
                p.company_code,
                p.start_date,
                p.end_date,
                p.settings.vacation_rules,
                p.settings.timezone,
                ProjectStatusManifest::from(&p.status),
            ),
        };

        ProjectManifest {
            api_version: API_VERSION.to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata {
                id: Some(id.to_string()),
                code: Some(code),
                name,
                description: description.unwrap_or_default(),
                company_code: Some(company_code.clone()),
                created_at: None,
                updated_at: None,
                created_by: None,
                labels: None,
                annotations: None,
                namespace: None,
            },
            spec: ProjectSpec {
                timezone,
                start_date: start_date.map(|d| d.format("%Y-%m-%d").to_string()),
                end_date: end_date.map(|d| d.format("%Y-%m-%d").to_string()),
                status: status_manifest,
                vacation_rules: vacation_rules.map(|vr| VacationRulesManifest::from(&vr)),
            },
        }
    }
}

impl TryFrom<ProjectManifest> for AnyProject {
    type Error = String;

    fn try_from(manifest: ProjectManifest) -> Result<Self, Self::Error> {
        let id = manifest
            .metadata
            .id
            .map(|id_str| Uuid::from_str(&id_str))
            .transpose()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(uuid7);

        let code = manifest.metadata.code.ok_or("Project code is missing in manifest")?;
        let name = manifest.metadata.name;
        let description = if manifest.metadata.description.is_empty() {
            None
        } else {
            Some(manifest.metadata.description)
        };
        let company_code = manifest.metadata.company_code.unwrap_or_else(|| "COMP-001".to_string());
        
        // Parse dates from ISO format (YYYY-MM-DD)
        let start_date = parse_date_opt(&manifest.spec.start_date)?;
        let end_date = parse_date_opt(&manifest.spec.end_date)?;
        
        let vacation_rules = manifest.spec.vacation_rules;
        let timezone = manifest.spec.timezone;
        
        // Convert status from manifest to domain
        let status = match manifest.spec.status {
            ProjectStatusManifest::Planned => crate::domain::project_management::project::ProjectStatus::Planned,
            ProjectStatusManifest::InProgress => crate::domain::project_management::project::ProjectStatus::InProgress,
            ProjectStatusManifest::OnHold => crate::domain::project_management::project::ProjectStatus::OnHold,
            ProjectStatusManifest::Completed => crate::domain::project_management::project::ProjectStatus::Completed,
            ProjectStatusManifest::Cancelled => crate::domain::project_management::project::ProjectStatus::Cancelled,
        };

        // Create project with all the data from manifest
        let mut project = crate::domain::project_management::project::Project::new(
            code,
            name,
            company_code,
            manifest.metadata.created_by.unwrap_or_else(|| "system".to_string()),
        )
        .map_err(|e| e.to_string())?;

        // Set all fields from manifest
        project.id = id.to_string();
        project.description = description;
        project.status = status;
        project.start_date = start_date;
        project.end_date = end_date;
        
        // Set timezone and vacation rules in settings
        if let Some(tz) = timezone {
            project.settings.timezone = Some(tz);
        }
        
        if let Some(vr) = vacation_rules {
            project.settings.vacation_rules = Some(crate::domain::project_management::project::VacationRules {
                allowed_days_per_year: vr.max_concurrent_vacations.unwrap_or(20),
                carry_over_days: vr.carry_over_days.unwrap_or(5),
            });
        }

        Ok(AnyProject::Project(project))
    }
}

impl From<&crate::domain::project_management::project::VacationRules> for VacationRulesManifest {
    fn from(source: &crate::domain::project_management::project::VacationRules) -> Self {
        VacationRulesManifest {
            max_concurrent_vacations: Some(source.allowed_days_per_year),
            carry_over_days: Some(source.carry_over_days),
            allow_layoff_vacations: Some(true),          // Default value
            require_layoff_vacation_period: Some(false), // Default value
            layoff_periods: None,                        // Not implemented in the new VacationRules
        }
    }
}

impl VacationRulesManifest {
    // This method is no longer needed as we're using the project::VacationRules directly
}

impl From<LayoffPeriod> for LayoffPeriodManifest {
    fn from(source: LayoffPeriod) -> Self {
        LayoffPeriodManifest {
            start_date: source.start_date,
            end_date: source.end_date,
        }
    }
}

impl LayoffPeriodManifest {
    pub fn to(&self) -> LayoffPeriod {
        LayoffPeriod {
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::builder::ProjectBuilder;

    #[test]
    fn test_bidirectional_conversion() {
        use chrono::NaiveDate;
        
        // Create a project with all fields
        let original_project = ProjectBuilder::new()
            .name("Test Project".to_string())
            .code("proj-1".to_string())
            .company_code("COMP-001".to_string())
            .created_by("system".to_string())
            .description(Some("Test description".to_string()))
            .start_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            .end_date(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .timezone("UTC".to_string())
            .vacation_rules(crate::domain::project_management::project::VacationRules {
                allowed_days_per_year: 25,
                carry_over_days: 10,
            })
            .build()
            .unwrap();
        let original_any = AnyProject::from(original_project.clone());

        // Convert to Manifest
        let manifest = ProjectManifest::from(original_any);
        assert_eq!(manifest.metadata.name, "Test Project");
        assert_eq!(manifest.metadata.description, "Test description");
        assert_eq!(manifest.spec.status, ProjectStatusManifest::Planned);
        assert_eq!(manifest.spec.start_date, Some("2024-01-01".to_string()));
        assert_eq!(manifest.spec.end_date, Some("2024-12-31".to_string()));
        assert_eq!(manifest.spec.timezone, Some("UTC".to_string()));

        // Convert back to AnyProject
        let converted_any = AnyProject::try_from(manifest).unwrap();
        assert!(matches!(converted_any, AnyProject::Project(_)));

        let AnyProject::Project(converted) = converted_any;
        assert_eq!(original_project.name, converted.name);
        assert_eq!(original_project.id, converted.id);
        assert_eq!(original_project.description, converted.description);
        assert_eq!(original_project.start_date, converted.start_date);
        assert_eq!(original_project.end_date, converted.end_date);
        assert_eq!(original_project.settings.timezone, converted.settings.timezone);
        assert_eq!(original_project.settings.vacation_rules, converted.settings.vacation_rules);
    }
    
    #[test]
    fn test_date_parsing() {
        // Test valid ISO date
        let valid_date = Some("2024-01-15".to_string());
        let parsed = parse_date_opt(&valid_date).unwrap();
        assert_eq!(parsed, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        
        // Test invalid date
        let invalid_date = Some("2024-13-45".to_string());
        let result = parse_date_opt(&invalid_date);
        assert!(result.is_err());
        
        // Test None
        let none_date = None;
        let parsed = parse_date_opt(&none_date).unwrap();
        assert_eq!(parsed, None);
    }

    #[test]
    fn test_yaml_parsing_success() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Project
            metadata:
                id: "01996dev-0000-0000-0000-000000proj"
                code: "PROJ-001"
                name: "Test Project"
                description: "A test project"
                companyCode: "COMP-001"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                status: "Planned"
                startDate: "2024-01-01"
                endDate: "2024-12-31"
                timezone: "UTC"
                vacationRules:
                    allowedDaysPerYear: 25
                    carryOverDays: 10
        "#;

        let manifest: ProjectManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Project");
        assert_eq!(manifest.metadata.code, Some("PROJ-001".to_string()));
        assert_eq!(manifest.metadata.name, "Test Project");
        assert_eq!(manifest.metadata.description, "A test project");
        assert_eq!(manifest.metadata.company_code, Some("COMP-001".to_string()));
        assert_eq!(manifest.spec.status, ProjectStatusManifest::Planned);
        assert_eq!(manifest.spec.start_date, Some("2024-01-01".to_string()));
        assert_eq!(manifest.spec.end_date, Some("2024-12-31".to_string()));
        assert_eq!(manifest.spec.timezone, Some("UTC".to_string()));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_syntax() {
        let yaml_str = "invalid: yaml: content: [";
        let result: Result<ProjectManifest, _> = serde_yaml::from_str(yaml_str);
        
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
            kind: Project
            metadata:
                id: "01996dev-0000-0000-0000-000000proj"
                # Missing required fields: code, name, companyCode, createdAt, updatedAt, createdBy
            spec:
                status: "Planned"
        "#;

        let result: Result<ProjectManifest, _> = serde_yaml::from_str(yaml_str);
        
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
            kind: Project
            metadata:
                id: "01996dev-0000-0000-0000-000000proj"
                code: "PROJ-001"
                name: "Test Project"
                description: "A test project"
                companyCode: "COMP-001"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                status: "invalid_status"  # Invalid enum value
        "#;

        let result: Result<ProjectManifest, _> = serde_yaml::from_str(yaml_str);
        
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
            kind: Project
            metadata:
                id: "01996dev-0000-0000-0000-000000proj"
                code: "PROJ-001"
                name: "Test Project"
                description: "A test project"
                companyCode: "COMP-001"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                status: "InvalidStatus"  # Invalid status
                startDate: "invalid-date"  # Invalid date format
        "#;

        let result: Result<ProjectManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_success_with_optional_fields() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Project
            metadata:
                id: "01996dev-0000-0000-0000-000000proj"
                code: "PROJ-001"
                name: "Test Project"
                description: "A comprehensive test project"
                companyCode: "COMP-001"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                status: "InProgress"
                startDate: "2024-01-01"
                endDate: "2024-12-31"
                timezone: "America/Sao_Paulo"
                vacationRules:
                    maxConcurrentVacations: 30
                    carryOverDays: 5
        "#;

        let manifest: ProjectManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.metadata.description, "A comprehensive test project");
        assert_eq!(manifest.spec.status, ProjectStatusManifest::InProgress);
        assert_eq!(manifest.spec.timezone, Some("America/Sao_Paulo".to_string()));
        assert_eq!(manifest.spec.vacation_rules.as_ref().unwrap().max_concurrent_vacations, Some(30));
        assert_eq!(manifest.spec.vacation_rules.as_ref().unwrap().carry_over_days, Some(5));
    }
}
