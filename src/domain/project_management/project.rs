use crate::domain::project_management::vacation_rules::VacationRules;
use serde::Serialize;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Serialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_status_display() {
        assert_eq!(ProjectStatus::Planned.to_string(), "Planned");
        assert_eq!(ProjectStatus::InProgress.to_string(), "InProgress");
        assert_eq!(ProjectStatus::Completed.to_string(), "Completed");
        assert_eq!(ProjectStatus::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_project_display() {
        let project_with_id = Project {
            id: Some("ID-123".to_string()),
            name: "Project A".to_string(),
            description: None,
            start_date: None,
            end_date: None,
            status: ProjectStatus::Planned,
            vacation_rules: None,
        };
        assert_eq!(
            project_with_id.to_string(),
            "Project { id: Some(\"ID-123\"), name: Project A, status: Planned }"
        );

        let project_without_id = Project {
            id: None,
            name: "Project B".to_string(),
            description: None,
            start_date: None,
            end_date: None,
            status: ProjectStatus::Completed,
            vacation_rules: None,
        };
        assert_eq!(
            project_without_id.to_string(),
            "Project { id: None, name: Project B, status: Completed }"
        );
    }
}
