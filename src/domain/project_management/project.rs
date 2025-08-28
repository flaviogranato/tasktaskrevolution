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

// Common methods for all Project states
impl<S: ProjectState> Project<S> {
    // Getters
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn start_date(&self) -> &str {
        self.start_date.as_deref().unwrap_or("")
    }

    pub fn end_date(&self) -> &str {
        self.end_date.as_deref().unwrap_or("")
    }

    // Validation methods
    pub fn is_code_valid(&self) -> bool {
        !self.code.trim().is_empty()
    }

    pub fn is_name_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn is_date_range_valid(&self) -> bool {
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            if let (Ok(start_date), Ok(end_date)) = (
                chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d"),
                chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d")
            ) {
                return start_date <= end_date;
            }
        }
        false
    }

    pub fn validate(&self) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        if !self.is_code_valid() {
            errors.push("Project code cannot be empty".to_string());
        }

        if !self.is_name_valid() {
            errors.push("Project name cannot be empty".to_string());
        }

        if !self.is_date_range_valid() {
            errors.push("Project end date must be after start date".to_string());
        }

        Ok(errors)
    }
}

// Transition trait for state changes
pub trait Transition {
    type NextState: ProjectState;
    fn transition(self) -> Project<Self::NextState>;
}

impl Transition for Project<Planned> {
    type NextState = InProgress;
    
    fn transition(self) -> Project<InProgress> {
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
}

impl Transition for Project<InProgress> {
    type NextState = Completed;
    
    fn transition(self) -> Project<Completed> {
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
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_project_creation_with_valid_data() {
        let project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert_eq!(project.code(), "PROJ-001");
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.description(), Some("A test project"));
        assert_eq!(project.start_date(), "2024-01-01");
        assert_eq!(project.end_date(), "2024-12-31");
    }

    #[test]
    fn test_project_state_transitions() {
        let project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        // Transition from Planned to InProgress
        let in_progress_project: Project<InProgress> = project.transition();
        assert!(matches!(in_progress_project.state, InProgress));

        // Transition from InProgress to Completed
        let in_progress_project = Project::<InProgress> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: InProgress,
        };

        let completed_project: Project<Completed> = in_progress_project.transition();
        assert!(matches!(completed_project.state, Completed));
    }

    #[test]
    fn test_project_validation_dates() {
        // Valid date range
        let valid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(valid_project.is_date_range_valid());

        // Invalid date range (end before start)
        let invalid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-002".to_string(),
            name: "Invalid Project".to_string(),
            description: None,
            start_date: Some("2024-12-31".to_string()),
            end_date: Some("2024-01-01".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(!invalid_project.is_date_range_valid());
    }

    #[test]
    fn test_project_code_validation() {
        // Valid code format
        let valid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(valid_project.is_code_valid());

        // Invalid code (empty)
        let invalid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(!invalid_project.is_code_valid());
    }

    #[test]
    fn test_project_name_validation() {
        // Valid name
        let valid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(valid_project.is_name_valid());

        // Invalid name (empty)
        let invalid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "".to_string(),
            description: None,
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        assert!(!invalid_project.is_name_valid());
    }

    #[test]
    fn test_project_comprehensive_validation() {
        let valid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "PROJ-001".to_string(),
            name: "Test Project".to_string(),
            description: Some("A comprehensive test project".to_string()),
            start_date: Some("2024-01-01".to_string()),
            end_date: Some("2024-12-31".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        let validation_result = valid_project.validate();
        assert!(validation_result.is_ok());
        assert_eq!(validation_result.unwrap().len(), 0); // No validation errors

        let invalid_project = Project::<Planned> {
            id: uuid7::uuid7(),
            code: "".to_string(),
            name: "".to_string(),
            description: None,
            start_date: Some("2024-12-31".to_string()),
            end_date: Some("2024-01-01".to_string()),
            vacation_rules: Some(VacationRules::default()),
            timezone: Some("UTC".to_string()),
            tasks: HashMap::new(),
            state: Planned,
        };

        let validation_result = invalid_project.validate();
        assert!(validation_result.is_ok());
        let errors = validation_result.unwrap();
        assert!(errors.len() > 0); // Should have validation errors
        assert!(errors.iter().any(|e| e.contains("code")));
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("date")));
    }
}
