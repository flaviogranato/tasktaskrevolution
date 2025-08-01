use super::super::task_management::any_task::AnyTask;
use super::state::{Cancelled, Completed, InProgress, Planned, ProjectState};
use crate::domain::project_management::vacation_rules::VacationRules;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use uuid7::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Project<S: ProjectState> {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub vacation_rules: Option<VacationRules>,
    pub timezone: Option<String>,
    pub tasks: HashMap<String, AnyTask>,
    pub state: S,
}

impl<S: ProjectState> Display for Project<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Project {{ id: {:?}, code: {}, name: {}, status: {:?} }}",
            self.id, self.code, self.name, self.state
        )
    }
}

impl Project<Planned> {
    #[allow(dead_code)]
    pub fn start(self) -> Project<InProgress> {
        Project {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: InProgress,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(self) -> Project<Cancelled> {
        Project {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: Cancelled,
        }
    }
}

impl Project<InProgress> {
    #[allow(dead_code)]
    pub fn complete(self) -> Project<Completed> {
        Project {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: Completed,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(self) -> Project<Cancelled> {
        Project {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: Cancelled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid7::uuid7;

    #[test]
    fn test_project_display() {
        let id = uuid7();
        let project_with_id = Project {
            id,
            code: "proj-a".to_string(),
            name: "Project A".to_string(),
            description: None,
            start_date: None,
            end_date: None,
            vacation_rules: None,
            timezone: None,
            tasks: HashMap::new(),
            state: Planned,
        };
        assert_eq!(
            project_with_id.to_string(),
            format!("Project {{ id: {id:?}, code: proj-a, name: Project A, status: Planned }}")
        );
    }
}
