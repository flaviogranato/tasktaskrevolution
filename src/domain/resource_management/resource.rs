use super::state::{Assigned, Available, ResourceState};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeOffEntry {
    pub date: DateTime<Local>,
    pub hours: u32,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Period {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub approved: bool,
    pub period_type: PeriodType,
    pub is_time_off_compensation: bool,
    pub compensated_hours: Option<u32>,
    pub is_layoff: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PeriodType {
    Vacation,
    TimeOff,
    BirthdayBreak,
    DayOff,
    SickLeave,
    PersonalLeave,
    TimeOffCompensation,
}

impl fmt::Display for PeriodType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeriodType::Vacation => write!(f, "Vacation"),
            PeriodType::TimeOff => write!(f, "TimeOff"),
            PeriodType::BirthdayBreak => write!(f, "BirthdayBreak"),
            PeriodType::DayOff => write!(f, "DayOff"),
            PeriodType::SickLeave => write!(f, "SickLeave"),
            PeriodType::PersonalLeave => write!(f, "PersonalLeave"),
            PeriodType::TimeOffCompensation => write!(f, "TimeOffCompensation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectAssignment {
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub allocation_percentage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource<S: ResourceState> {
    pub id: Option<String>,
    pub name: String,
    pub email: Option<String>,
    pub resource_type: String,
    pub vacations: Option<Vec<Period>>,
    pub time_off_balance: u32,
    pub time_off_history: Option<Vec<TimeOffEntry>>,
    pub state: S,
}

impl Resource<Available> {
    pub fn new(
        id: Option<String>,
        name: String,
        email: Option<String>,
        resource_type: String,
        vacations: Option<Vec<Period>>,
        time_off_balance: u32,
    ) -> Self {
        Self {
            id,
            name,
            email,
            resource_type,
            vacations,
            time_off_balance,
            time_off_history: Some(Vec::new()),
            state: Available,
        }
    }

    #[allow(dead_code)]
    pub fn assign_to_project(self, assignment: ProjectAssignment) -> Resource<Assigned> {
        Resource {
            id: self.id,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            state: Assigned {
                project_assignments: vec![assignment],
            },
        }
    }
}

impl Resource<Assigned> {
    #[allow(dead_code)]
    pub fn assign_to_another_project(mut self, assignment: ProjectAssignment) -> Self {
        self.state.project_assignments.push(assignment);
        self
    }
}

impl<S: ResourceState> Display for Resource<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource {{ id: {:?}, name: {}, email: {:?}, resource_type: {}, vacations: {:?}, time_off_balance: {}, state: {:?} }}",
            self.id, self.name, self.email, self.resource_type, self.vacations, self.time_off_balance, self.state
        )
    }
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Period {{ start_date: {}, end_date: {}, approved: {}, period_type: {}, is_time_off_compensation: {}, compensated_hours: {:?}, is_layoff: {} }}",
            self.start_date,
            self.end_date,
            self.approved,
            self.period_type,
            self.is_time_off_compensation,
            self.compensated_hours,
            self.is_layoff
        )
    }
}

impl Display for ProjectAssignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProjectAssignment: {{ project_id: {}, start_date: {}, end_date: {} }}",
            self.project_id, self.start_date, self.end_date
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local, TimeZone};

    // Helper to create a DateTime<Local> for tests
    fn dt(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn test_period_type_display() {
        assert_eq!(PeriodType::Vacation.to_string(), "Vacation");
        assert_eq!(PeriodType::TimeOff.to_string(), "TimeOff");
        assert_eq!(PeriodType::BirthdayBreak.to_string(), "BirthdayBreak");
        assert_eq!(PeriodType::DayOff.to_string(), "DayOff");
        assert_eq!(PeriodType::SickLeave.to_string(), "SickLeave");
        assert_eq!(PeriodType::PersonalLeave.to_string(), "PersonalLeave");
        assert_eq!(PeriodType::TimeOffCompensation.to_string(), "TimeOffCompensation");
    }

    #[test]
    fn test_period_display() {
        let period = Period {
            start_date: dt(2025, 1, 1),
            end_date: dt(2025, 1, 10),
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        };
        let expected = format!(
            "Period {{ start_date: {}, end_date: {}, approved: true, period_type: Vacation, is_time_off_compensation: false, compensated_hours: None, is_layoff: false }}",
            dt(2025, 1, 1),
            dt(2025, 1, 10)
        );
        assert_eq!(period.to_string(), expected);
    }

    #[test]
    fn test_project_assignment_display() {
        let assignment = ProjectAssignment {
            project_id: "PROJ-R-US".to_string(),
            start_date: dt(2025, 2, 1),
            end_date: dt(2025, 8, 1),
            allocation_percentage: 100,
        };
        let expected = format!(
            "ProjectAssignment: {{ project_id: PROJ-R-US, start_date: {}, end_date: {} }}",
            dt(2025, 2, 1),
            dt(2025, 8, 1)
        );
        assert_eq!(assignment.to_string(), expected);
    }

    #[test]
    fn test_resource_display() {
        let resource = Resource {
            id: Some("res-007".to_string()),
            name: "James".to_string(),
            email: Some("james@test.com".to_string()),
            resource_type: "Developer".to_string(),
            vacations: None,
            time_off_balance: 40,
            time_off_history: None,
            state: Available,
        };
        let expected = "Resource { id: Some(\"res-007\"), name: James, email: Some(\"james@test.com\"), resource_type: Developer, vacations: None, time_off_balance: 40, state: Available }";
        assert_eq!(resource.to_string(), expected);
    }

    #[test]
    fn test_resource_state_transition_to_assigned() {
        let resource = Resource::new(
            Some("res-001".to_string()),
            "Tester".to_string(),
            None,
            "QA".to_string(),
            None,
            40,
        );

        let assignment = ProjectAssignment {
            project_id: "PROJ-1".to_string(),
            start_date: dt(2025, 1, 1),
            end_date: dt(2025, 6, 1),
            allocation_percentage: 100,
        };

        let assigned_resource = resource.assign_to_project(assignment.clone());

        assert_eq!(assigned_resource.state.project_assignments.len(), 1);
        assert_eq!(assigned_resource.state.project_assignments[0], assignment);

        let another_assignment = ProjectAssignment {
            project_id: "PROJ-2".to_string(),
            start_date: dt(2025, 7, 1),
            end_date: dt(2025, 12, 1),
            allocation_percentage: 50,
        };

        let multi_assigned_resource = assigned_resource.assign_to_another_project(another_assignment.clone());
        assert_eq!(multi_assigned_resource.state.project_assignments.len(), 2);
    }
}
