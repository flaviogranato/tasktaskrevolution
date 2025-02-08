use chrono::{DateTime, Local};
use std::fmt::Display;

#[derive(Clone)]
pub struct Resource {
    pub id: Option<String>,
    pub name: String,
    pub email: Option<String>,
    pub resource_type: String,
    pub vacations: Option<Vec<Period>>,
    pub project_assignments: Option<Vec<ProjectAssignment>>,
    pub time_off_balance: u32,
}

impl Resource {
    pub fn new(
        id: Option<String>,
        name: String,
        email: Option<String>,
        resource_type: String,
        vacations: Option<Vec<Period>>,
        project_assignments: Option<Vec<ProjectAssignment>>,
        time_off_balance: u32,
    ) -> Self {
        Self {
            id,
            name,
            email,
            resource_type,
            vacations,
            project_assignments,
            time_off_balance,
        }
    }
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource {{ id: {:?}, name: {}, email: {:?}, resource_type: {}, vacations: {:?}, project_assignments: {:?}, time_off_balance: {} }}",
            self.id, self.name, self.email, self.resource_type, self.vacations, self.project_assignments, self.time_off_balance
        )
    }
}

#[derive(Debug, Clone)]
pub struct Period {
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub approved: bool,
    pub period_type: PeriodType,
    pub is_time_off_compensation: bool,
    pub compensated_hours: Option<u32>,
}

impl Display for Period {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Period {{ start_date: {}, end_date: {}, approved: {}, period_type: {}, is_time_off_compensation: {}, compensated_hours: {:?} }}",
        self.start_date, self.end_date, self.approved, self.period_type, self.is_time_off_compensation, self.compensated_hours)
    }
}

#[derive(Debug, Clone)]
pub enum PeriodType {
    BirthdayBreak,
    DayOff,
    Vacation,
    SickLeave,
    PersonalLeave,
    TimeOffCompensation,
}

impl Display for PeriodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeriodType::BirthdayBreak => write!(f, "BirthdayBreak"),
            PeriodType::DayOff => write!(f, "DayOff"),
            PeriodType::Vacation => write!(f, "Vacation"),
            PeriodType::SickLeave => write!(f, "SickLeave"),
            PeriodType::PersonalLeave => write!(f, "PersonalLeave"),
            PeriodType::TimeOffCompensation => write!(f, "TimeOffCompensation"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectAssignment {
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
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
