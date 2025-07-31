use crate::domain::project_management::{
    AnyProject,
    layoff_period::LayoffPeriod,
    project::Project,
    state::{Cancelled, Completed, InProgress, Planned},
    vacation_rules::VacationRules,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use uuid7::{Uuid, uuid7};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

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
    Completed,
    Cancelled,
}

impl From<AnyProject> for ProjectManifest {
    fn from(source: AnyProject) -> Self {
        let (id, code, name, description, start_date, end_date, vacation_rules, timezone, status_manifest) =
            match source {
                AnyProject::Planned(p) => (
                    p.id,
                    p.code,
                    p.name,
                    p.description,
                    p.start_date,
                    p.end_date,
                    p.vacation_rules,
                    p.timezone,
                    ProjectStatusManifest::Planned,
                ),
                AnyProject::InProgress(p) => (
                    p.id,
                    p.code,
                    p.name,
                    p.description,
                    p.start_date,
                    p.end_date,
                    p.vacation_rules,
                    p.timezone,
                    ProjectStatusManifest::InProgress,
                ),
                AnyProject::Completed(p) => (
                    p.id,
                    p.code,
                    p.name,
                    p.description,
                    p.start_date,
                    p.end_date,
                    p.vacation_rules,
                    p.timezone,
                    ProjectStatusManifest::Completed,
                ),
                AnyProject::Cancelled(p) => (
                    p.id,
                    p.code,
                    p.name,
                    p.description,
                    p.start_date,
                    p.end_date,
                    p.vacation_rules,
                    p.timezone,
                    ProjectStatusManifest::Cancelled,
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
            },
            spec: ProjectSpec {
                timezone,
                start_date,
                end_date,
                status: status_manifest,
                vacation_rules: vacation_rules.map(VacationRulesManifest::from),
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
        let start_date = manifest.spec.start_date;
        let end_date = manifest.spec.end_date;
        let vacation_rules = manifest.spec.vacation_rules.map(|vr| vr.to());
        let timezone = manifest.spec.timezone;

        match manifest.spec.status {
            ProjectStatusManifest::Planned => Ok(AnyProject::Planned(Project {
                id,
                code,
                name,
                description,
                start_date,
                end_date,
                vacation_rules,
                timezone,
                state: Planned,
            })),
            ProjectStatusManifest::InProgress => Ok(AnyProject::InProgress(Project {
                id,
                code,
                name,
                description,
                start_date,
                end_date,
                vacation_rules,
                timezone,
                state: InProgress,
            })),
            ProjectStatusManifest::Completed => Ok(AnyProject::Completed(Project {
                id,
                code,
                name,
                description,
                start_date,
                end_date,
                vacation_rules,
                timezone,
                state: Completed,
            })),
            ProjectStatusManifest::Cancelled => Ok(AnyProject::Cancelled(Project {
                id,
                code,
                name,
                description,
                start_date,
                end_date,
                vacation_rules,
                timezone,
                state: Cancelled,
            })),
        }
    }
}

impl From<VacationRules> for VacationRulesManifest {
    fn from(source: VacationRules) -> Self {
        VacationRulesManifest {
            max_concurrent_vacations: source.max_concurrent_vacations,
            allow_layoff_vacations: source.allow_layoff_vacations,
            require_layoff_vacation_period: source.require_layoff_vacation_period,
            layoff_periods: source
                .layoff_periods
                .map(|periods| periods.into_iter().map(LayoffPeriodManifest::from).collect()),
        }
    }
}

impl VacationRulesManifest {
    pub fn to(&self) -> VacationRules {
        VacationRules {
            max_concurrent_vacations: self.max_concurrent_vacations,
            allow_layoff_vacations: self.allow_layoff_vacations,
            require_layoff_vacation_period: self.require_layoff_vacation_period,
            layoff_periods: self
                .layoff_periods
                .as_ref()
                .map(|periods| periods.iter().map(|period| period.to()).collect()),
        }
    }
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
        // Create a Planned project
        let original_project = ProjectBuilder::new("Test Project".to_string())
            .code("proj-1".to_string())
            .build();
        let original_any = AnyProject::from(original_project.clone());

        // Convert to Manifest
        let manifest = ProjectManifest::from(original_any);
        assert_eq!(manifest.metadata.name, "Test Project");
        assert_eq!(manifest.spec.status, ProjectStatusManifest::Planned);

        // Convert back to AnyProject
        let converted_any = AnyProject::try_from(manifest).unwrap();
        assert!(matches!(converted_any, AnyProject::Planned(_)));

        if let AnyProject::Planned(converted) = converted_any {
            assert_eq!(original_project.name, converted.name);
            assert_eq!(original_project.id, converted.id);
        }
    }
}
