use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ProjectMetadata,
    pub spec: ProjectSpec,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectMetadata {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSpec {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: ProjectStatus,
    pub vacation_rules: VacationRules,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VacationRules {
    pub max_concurrent_vacations: Option<u32>,
    pub allow_layoff_vacations: bool,
    pub require_layoff_vacation_period: bool,
    pub layoff_periods: Option<Vec<LayoffPeriod>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LayoffPeriod {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
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
                vacation_rules: VacationRules {
                    max_concurrent_vacations: None,
                    allow_layoff_vacations: false,
                    require_layoff_vacation_period: false,
                    layoff_periods: None,
                },
            },
        }
    }
}
