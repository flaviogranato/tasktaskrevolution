use super::state::{Cancelled, Completed, InProgress, Planned, ProjectState};
use crate::domain::project_management::vacation_rules::VacationRules;
use serde::Serialize;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Project<S: ProjectState> {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub vacation_rules: Option<VacationRules>,
    pub timezone: Option<String>,
    pub state: S,
}

impl<S: ProjectState> Display for Project<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project {{ id: {:?}, name: {}, status: {:?} }}",
            self.id, self.name, self.state
        )
    }
}

impl Project<Planned> {
    #[allow(dead_code)]
    pub fn start(self) -> Project<InProgress> {
        Project {
            id: self.id,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            state: InProgress,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(self) -> Project<Cancelled> {
        Project {
            id: self.id,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            state: Cancelled,
        }
    }
}

impl Project<InProgress> {
    #[allow(dead_code)]
    pub fn complete(self) -> Project<Completed> {
        Project {
            id: self.id,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            state: Completed,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(self) -> Project<Cancelled> {
        Project {
            id: self.id,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            state: Cancelled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_display() {
        let project_with_id = Project {
            id: Some("ID-123".to_string()),
            name: "Project A".to_string(),
            description: None,
            start_date: None,
            end_date: None,
            vacation_rules: None,
            timezone: None,
            state: Planned,
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
            vacation_rules: None,
            timezone: None,
            state: Completed,
        };
        assert_eq!(
            project_without_id.to_string(),
            "Project { id: None, name: Project B, status: Completed }"
        );
    }
}
