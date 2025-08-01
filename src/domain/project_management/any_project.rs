use super::{
    project::Project,
    state::{Cancelled, Completed, InProgress, Planned},
    vacation_rules::VacationRules,
};
use serde::Serialize;

/// An enum to represent a Project in any of its possible states.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum AnyProject {
    Planned(Project<Planned>),
    InProgress(Project<InProgress>),
    Completed(Project<Completed>),
    Cancelled(Project<Cancelled>),
}

impl AnyProject {
    pub fn name(&self) -> &str {
        match self {
            AnyProject::Planned(p) => &p.name,
            AnyProject::InProgress(p) => &p.name,
            AnyProject::Completed(p) => &p.name,
            AnyProject::Cancelled(p) => &p.name,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AnyProject::Planned(p) => &p.code,
            AnyProject::InProgress(p) => &p.code,
            AnyProject::Completed(p) => &p.code,
            AnyProject::Cancelled(p) => &p.code,
        }
    }

    pub fn timezone(&self) -> Option<&String> {
        match self {
            AnyProject::Planned(p) => p.timezone.as_ref(),
            AnyProject::InProgress(p) => p.timezone.as_ref(),
            AnyProject::Completed(p) => p.timezone.as_ref(),
            AnyProject::Cancelled(p) => p.timezone.as_ref(),
        }
    }

    pub fn vacation_rules(&self) -> Option<&VacationRules> {
        match self {
            AnyProject::Planned(p) => p.vacation_rules.as_ref(),
            AnyProject::InProgress(p) => p.vacation_rules.as_ref(),
            AnyProject::Completed(p) => p.vacation_rules.as_ref(),
            AnyProject::Cancelled(p) => p.vacation_rules.as_ref(),
        }
    }
}

impl From<Project<Planned>> for AnyProject {
    fn from(project: Project<Planned>) -> Self {
        AnyProject::Planned(project)
    }
}

impl From<Project<InProgress>> for AnyProject {
    fn from(project: Project<InProgress>) -> Self {
        AnyProject::InProgress(project)
    }
}

impl From<Project<Completed>> for AnyProject {
    fn from(project: Project<Completed>) -> Self {
        AnyProject::Completed(project)
    }
}

impl From<Project<Cancelled>> for AnyProject {
    fn from(project: Project<Cancelled>) -> Self {
        AnyProject::Cancelled(project)
    }
}
