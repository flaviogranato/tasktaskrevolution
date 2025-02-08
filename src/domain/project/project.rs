use crate::domain::project::vacation_rules::VacationRules;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: ProjectStatus,
    pub vacation_rules: Option<VacationRules>,
}

impl Project {
    pub fn new(
        id: Option<String>,
        name: String,
        description: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
        status: ProjectStatus,
        vacation_rules: Option<VacationRules>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            start_date,
            end_date,
            status,
            vacation_rules,
        }
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project {{ id: {:?}, name: {}, status: {} }}",
            self.id, self.name, self.status
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProjectStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Planned => write!(f, "Planned"),
            ProjectStatus::InProgress => write!(f, "InProgress"),
            ProjectStatus::Completed => write!(f, "Completed"),
            ProjectStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}
