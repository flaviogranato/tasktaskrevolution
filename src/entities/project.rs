use serde::{Deserialize, Serialize};

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
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    pub status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacation_rules: Option<VacationRules>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VacationRules {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_vacations: Option<u32>,
    pub allow_layoff_vacations: Option<bool>,
    pub require_layoff_vacation_period: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layoff_periods: Option<Vec<LayoffPeriod>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoffPeriod {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ProjectStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl ProjectManifest {
    pub fn new(name: String, start_date: Option<String>, end_date: Option<String>) -> Self {
        ProjectManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata { name },
            spec: ProjectSpec {
                start_date,
                end_date,
                status: ProjectStatus::Planned,
                vacation_rules: Some(VacationRules {
                    max_concurrent_vacations: None,
                    allow_layoff_vacations: None,
                    require_layoff_vacation_period: None,
                    layoff_periods: None,
                }),
            },
        }
    }
}
