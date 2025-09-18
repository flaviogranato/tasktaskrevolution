#![allow(dead_code)]

use super::state::{Assigned, Available, Inactive, ResourceState};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use uuid7::{Uuid, uuid7};

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
pub struct WipLimits {
    pub max_concurrent_tasks: u32,
    pub max_concurrent_projects: u32,
    pub max_allocation_percentage: u8,
    pub enabled: bool,
}

impl WipLimits {
    pub fn new(max_concurrent_tasks: u32, max_concurrent_projects: u32, max_allocation_percentage: u8) -> Self {
        Self {
            max_concurrent_tasks,
            max_concurrent_projects,
            max_allocation_percentage,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            max_concurrent_tasks: u32::MAX,
            max_concurrent_projects: u32::MAX,
            max_allocation_percentage: 100,
            enabled: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.max_concurrent_tasks > 0
            && self.max_concurrent_projects > 0
            && self.max_allocation_percentage > 0
            && self.max_allocation_percentage <= 100
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskAssignment {
    pub task_id: String,
    pub project_id: String,
    pub start_date: DateTime<Local>,
    pub end_date: DateTime<Local>,
    pub allocation_percentage: u8,
    pub status: TaskAssignmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskAssignmentStatus {
    Active,
    Blocked,
    Completed,
    Cancelled,
}

impl fmt::Display for TaskAssignmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskAssignmentStatus::Active => write!(f, "Active"),
            TaskAssignmentStatus::Blocked => write!(f, "Blocked"),
            TaskAssignmentStatus::Completed => write!(f, "Completed"),
            TaskAssignmentStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WipStatus {
    WithinLimits,
    NearLimit,
    Exceeded,
    Disabled,
}

impl fmt::Display for WipStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WipStatus::WithinLimits => write!(f, "Within Limits"),
            WipStatus::NearLimit => write!(f, "Near Limit"),
            WipStatus::Exceeded => write!(f, "Exceeded"),
            WipStatus::Disabled => write!(f, "Disabled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource<S: ResourceState> {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub email: Option<String>,
    pub resource_type: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub vacations: Option<Vec<Period>>,
    pub time_off_balance: u32,
    pub time_off_history: Option<Vec<TimeOffEntry>>,
    pub wip_limits: Option<WipLimits>,
    pub task_assignments: Option<Vec<TaskAssignment>>,
    pub state: S,
}

impl Resource<Available> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        code: String,
        name: String,
        email: Option<String>,
        resource_type: String,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        vacations: Option<Vec<Period>>,
        time_off_balance: u32,
    ) -> Self {
        Self {
            id: uuid7(),
            code,
            name,
            email,
            resource_type,
            start_date,
            end_date,
            vacations,
            time_off_balance,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)), // Default WIP limits
            task_assignments: Some(Vec::new()),
            state: Available,
        }
    }

    #[allow(dead_code)]
    pub fn assign_to_project(self, assignment: ProjectAssignment) -> Resource<Assigned> {
        Resource {
            id: self.id,
            code: self.code,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            start_date: self.start_date,
            end_date: self.end_date,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            wip_limits: self.wip_limits,
            task_assignments: self.task_assignments,
            state: Assigned {
                project_assignments: vec![assignment],
            },
        }
    }

    #[allow(dead_code)]
    pub fn deactivate(self) -> Resource<Inactive> {
        Resource {
            id: self.id,
            code: self.code,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            start_date: self.start_date,
            end_date: self.end_date,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            wip_limits: self.wip_limits,
            task_assignments: self.task_assignments,
            state: Inactive,
        }
    }
}

impl Resource<Assigned> {
    #[allow(dead_code)]
    pub fn assign_to_another_project(mut self, assignment: ProjectAssignment) -> Self {
        self.state.project_assignments.push(assignment);
        self
    }

    #[allow(dead_code)]
    pub fn deactivate(self) -> Resource<Inactive> {
        Resource {
            id: self.id,
            code: self.code,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            start_date: self.start_date,
            end_date: self.end_date,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            wip_limits: self.wip_limits,
            task_assignments: self.task_assignments,
            state: Inactive,
        }
    }
}

impl<S: ResourceState> Display for Resource<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource {{ id: {:?}, code: {}, name: {}, email: {:?}, resource_type: {}, vacations: {:?}, time_off_balance: {}, state: {:?} }}",
            self.id,
            self.code,
            self.name,
            self.email,
            self.resource_type,
            self.vacations,
            self.time_off_balance,
            self.state
        )
    }
}

// Common methods for all Resource states
impl<S: ResourceState> Resource<S> {
    // Getters
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    // --- Zero-copy accessors ---

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    pub fn vacations(&self) -> Option<&[Period]> {
        self.vacations.as_deref()
    }

    pub fn time_off_history(&self) -> Option<&[TimeOffEntry]> {
        self.time_off_history.as_deref()
    }

    pub fn time_off_balance(&self) -> u32 {
        self.time_off_balance
    }

    pub fn vacations_iter(&self) -> Option<impl Iterator<Item = &Period>> {
        self.vacations().map(|v| v.iter())
    }

    pub fn time_off_history_iter(&self) -> Option<impl Iterator<Item = &TimeOffEntry>> {
        self.time_off_history().map(|h| h.iter())
    }

    // Validation methods
    pub fn is_code_valid(&self) -> bool {
        !self.code.trim().is_empty()
    }

    pub fn is_name_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn is_email_valid(&self) -> bool {
        if let Some(email) = &self.email {
            // Simple email validation - check for @ and basic format
            email.contains('@') && email.contains('.') && email.len() > 5
        } else {
            true // No email is valid
        }
    }

    pub fn validate(&self) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        if !self.is_code_valid() {
            errors.push("Resource code cannot be empty".to_string());
        }

        if !self.is_name_valid() {
            errors.push("Resource name cannot be empty".to_string());
        }

        if !self.is_email_valid() {
            errors.push("Resource email format is invalid".to_string());
        }

        if let Some(ref wip_limits) = self.wip_limits
            && !wip_limits.is_valid() {
            errors.push("WIP limits configuration is invalid".to_string());
        }

        Ok(errors)
    }

    // WIP Limits management
    pub fn set_wip_limits(&mut self, limits: WipLimits) -> Result<(), String> {
        if !limits.is_valid() {
            return Err("Invalid WIP limits configuration".to_string());
        }
        self.wip_limits = Some(limits);
        Ok(())
    }

    pub fn get_wip_limits(&self) -> Option<&WipLimits> {
        self.wip_limits.as_ref()
    }

    pub fn disable_wip_limits(&mut self) {
        self.wip_limits = Some(WipLimits::disabled());
    }

    // Task assignment management
    pub fn assign_to_task(&mut self, task_assignment: TaskAssignment) -> Result<(), String> {
        if let Some(ref limits) = self.wip_limits
            && limits.enabled {
                // Check if resource can be assigned to more tasks
                let current_active_tasks = self.get_active_task_count();
                if current_active_tasks >= limits.max_concurrent_tasks {
                    return Err(format!(
                        "Resource has reached maximum concurrent tasks limit ({}). Current active tasks: {}",
                        limits.max_concurrent_tasks, current_active_tasks
                    ));
                }

                // Check allocation percentage
                let current_allocation = self.get_current_allocation_percentage();
                if current_allocation + task_assignment.allocation_percentage as u32 > limits.max_allocation_percentage as u32 {
                    return Err(format!(
                        "Assignment would exceed maximum allocation percentage ({}). Current allocation: {}%, New assignment: {}%",
                        limits.max_allocation_percentage, current_allocation, task_assignment.allocation_percentage
                    ));
                }
            }

        if let Some(ref mut assignments) = self.task_assignments {
            assignments.push(task_assignment);
        } else {
            self.task_assignments = Some(vec![task_assignment]);
        }

        Ok(())
    }

    pub fn remove_task_assignment(&mut self, task_id: &str) -> bool {
        if let Some(ref mut assignments) = self.task_assignments
            && let Some(pos) = assignments.iter().position(|a| a.task_id == task_id) {
            assignments.remove(pos);
            return true;
        }
        false
    }

    pub fn get_active_task_count(&self) -> u32 {
        if let Some(ref assignments) = self.task_assignments {
            assignments.iter()
                .filter(|a| a.status == TaskAssignmentStatus::Active)
                .count() as u32
        } else {
            0
        }
    }

    pub fn get_current_allocation_percentage(&self) -> u32 {
        if let Some(ref assignments) = self.task_assignments {
            assignments.iter()
                .filter(|a| a.status == TaskAssignmentStatus::Active)
                .map(|a| a.allocation_percentage as u32)
                .sum()
        } else {
            0
        }
    }

    pub fn get_task_assignments(&self) -> Option<&[TaskAssignment]> {
        self.task_assignments.as_deref()
    }

    pub fn is_wip_limits_exceeded(&self) -> bool {
        if let Some(ref limits) = self.wip_limits {
            if !limits.enabled {
                return false;
            }

            let active_tasks = self.get_active_task_count();
            let current_allocation = self.get_current_allocation_percentage();

            active_tasks > limits.max_concurrent_tasks || 
            current_allocation > limits.max_allocation_percentage as u32
        } else {
            false
        }
    }

    pub fn get_wip_status(&self) -> WipStatus {
        if let Some(ref limits) = self.wip_limits {
            if !limits.enabled {
                return WipStatus::Disabled;
            }

            let active_tasks = self.get_active_task_count();
            let current_allocation = self.get_current_allocation_percentage();

            if active_tasks >= limits.max_concurrent_tasks || 
               current_allocation >= limits.max_allocation_percentage as u32 {
                WipStatus::Exceeded
            } else if active_tasks >= limits.max_concurrent_tasks * 3 / 4 || 
                      current_allocation >= (limits.max_allocation_percentage as u32 * 3 / 4) {
                WipStatus::NearLimit
            } else {
                WipStatus::WithinLimits
            }
        } else {
            WipStatus::Disabled
        }
    }
}

// Transition trait for state changes
pub trait Transition {
    type NextState: ResourceState;
    fn transition(self) -> Resource<Self::NextState>;
}

impl Transition for Resource<Available> {
    type NextState = Inactive;

    fn transition(self) -> Resource<Inactive> {
        Resource {
            id: self.id,
            code: self.code,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            start_date: self.start_date,
            end_date: self.end_date,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            wip_limits: self.wip_limits,
            task_assignments: self.task_assignments,
            state: Inactive,
        }
    }
}

impl Transition for Resource<Inactive> {
    type NextState = Available;

    fn transition(self) -> Resource<Available> {
        Resource {
            id: self.id,
            code: self.code,
            name: self.name,
            email: self.email,
            resource_type: self.resource_type,
            start_date: self.start_date,
            end_date: self.end_date,
            vacations: self.vacations,
            time_off_balance: self.time_off_balance,
            time_off_history: self.time_off_history,
            wip_limits: self.wip_limits,
            task_assignments: self.task_assignments,
            state: Available,
        }
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
    use uuid7::uuid7;

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
        let id = uuid7();
        let resource = Resource {
            id,
            code: "dev-7".to_string(),
            name: "James".to_string(),
            email: Some("james@test.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: None,
            time_off_balance: 40,
            time_off_history: None,
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };
        let expected = format!(
            "Resource {{ id: {id:?}, code: dev-7, name: James, email: Some(\"james@test.com\"), resource_type: Developer, vacations: None, time_off_balance: 40, state: Available }}"
        );
        assert_eq!(resource.to_string(), expected);
    }

    #[test]
    fn test_resource_state_transition_to_assigned() {
        let resource = Resource::new(
            "qa-1".to_string(),
            "Tester".to_string(),
            None,
            "QA".to_string(),
            None,
            None,
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

    #[test]
    fn test_resource_creation_with_valid_data() {
        let resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert_eq!(resource.code(), "RES-001");
        assert_eq!(resource.name(), "John Doe");
        assert_eq!(resource.email(), Some("john.doe@example.com"));
        assert!(resource.is_code_valid());
        assert!(resource.is_name_valid());
        assert!(resource.is_email_valid());
    }

    #[test]
    fn test_resource_code_validation() {
        // Valid code
        let valid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "Test Resource".to_string(),
            email: Some("test@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(valid_resource.is_code_valid());

        // Invalid code (empty)
        let invalid_resource = Resource::<Available> {
            id: uuid7(),
            code: "".to_string(),
            name: "Test Resource".to_string(),
            email: Some("test@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(!invalid_resource.is_code_valid());
    }

    #[test]
    fn test_resource_name_validation() {
        // Valid name
        let valid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(valid_resource.is_name_valid());

        // Invalid name (empty)
        let invalid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "".to_string(),
            email: Some("john@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(!invalid_resource.is_name_valid());
    }

    #[test]
    fn test_resource_email_validation() {
        // Valid email
        let valid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(valid_resource.is_email_valid());

        // Invalid email format
        let invalid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("invalid-email".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(!invalid_resource.is_email_valid());

        // No email (should be valid)
        let no_email_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: None,
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        assert!(no_email_resource.is_email_valid());
    }

    #[test]
    fn test_resource_comprehensive_validation() {
        let valid_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("john.doe@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        let validation_result = valid_resource.validate();
        assert!(validation_result.is_ok());
        assert_eq!(validation_result.unwrap().len(), 0); // No validation errors

        let invalid_resource = Resource::<Available> {
            id: uuid7(),
            code: "".to_string(),
            name: "".to_string(),
            email: Some("invalid-email".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        let validation_result = invalid_resource.validate();
        assert!(validation_result.is_ok());
        let errors = validation_result.unwrap();
        assert!(!errors.is_empty()); // Should have validation errors
        assert!(errors.iter().any(|e| e.contains("code")));
        assert!(errors.iter().any(|e| e.contains("name")));
        assert!(errors.iter().any(|e| e.contains("email")));
    }

    #[test]
    fn test_resource_state_transitions() {
        let available_resource = Resource::<Available> {
            id: uuid7(),
            code: "RES-001".to_string(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            resource_type: "Developer".to_string(),
            start_date: None,
            end_date: None,
            vacations: Some(Vec::new()),
            time_off_balance: 160,
            time_off_history: Some(Vec::new()),
            wip_limits: Some(WipLimits::new(5, 3, 100)),
            task_assignments: Some(Vec::new()),
            state: Available,
        };

        // Transition from Available to Inactive
        let inactive_resource: Resource<Inactive> = available_resource.deactivate();
        assert!(matches!(inactive_resource.state, Inactive));

        // Note: We don't have a direct transition from Inactive to Available
        // This would need to be implemented if needed
    }

    #[test]
    fn test_wip_limits_creation() {
        let limits = WipLimits::new(5, 3, 100);
        assert_eq!(limits.max_concurrent_tasks, 5);
        assert_eq!(limits.max_concurrent_projects, 3);
        assert_eq!(limits.max_allocation_percentage, 100);
        assert!(limits.enabled);
        assert!(limits.is_valid());
    }

    #[test]
    fn test_wip_limits_disabled() {
        let limits = WipLimits::disabled();
        assert_eq!(limits.max_concurrent_tasks, u32::MAX);
        assert_eq!(limits.max_concurrent_projects, u32::MAX);
        assert_eq!(limits.max_allocation_percentage, 100);
        assert!(!limits.enabled);
    }

    #[test]
    fn test_wip_limits_validation() {
        let valid_limits = WipLimits::new(5, 3, 100);
        assert!(valid_limits.is_valid());

        let invalid_limits = WipLimits {
            max_concurrent_tasks: 0,
            max_concurrent_projects: 3,
            max_allocation_percentage: 100,
            enabled: true,
        };
        assert!(!invalid_limits.is_valid());
    }

    #[test]
    fn test_task_assignment_status_display() {
        assert_eq!(TaskAssignmentStatus::Active.to_string(), "Active");
        assert_eq!(TaskAssignmentStatus::Blocked.to_string(), "Blocked");
        assert_eq!(TaskAssignmentStatus::Completed.to_string(), "Completed");
        assert_eq!(TaskAssignmentStatus::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_wip_status_display() {
        assert_eq!(WipStatus::WithinLimits.to_string(), "Within Limits");
        assert_eq!(WipStatus::NearLimit.to_string(), "Near Limit");
        assert_eq!(WipStatus::Exceeded.to_string(), "Exceeded");
        assert_eq!(WipStatus::Disabled.to_string(), "Disabled");
    }

    #[test]
    fn test_resource_wip_limits_management() {
        let mut resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            Some("john@example.com".to_string()),
            "Developer".to_string(),
            None,
            None,
            None,
            160,
        );

        // Test setting WIP limits
        let limits = WipLimits::new(3, 2, 80);
        assert!(resource.set_wip_limits(limits.clone()).is_ok());
        assert_eq!(resource.get_wip_limits(), Some(&limits));

        // Test disabling WIP limits
        resource.disable_wip_limits();
        assert!(!resource.get_wip_limits().unwrap().enabled);
    }

    #[test]
    fn test_resource_task_assignment() {
        let mut resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            Some("john@example.com".to_string()),
            "Developer".to_string(),
            None,
            None,
            None,
            160,
        );

        // Set WIP limits
        let limits = WipLimits::new(2, 1, 100);
        resource.set_wip_limits(limits).unwrap();

        // Test successful task assignment
        let task_assignment = TaskAssignment {
            task_id: "TASK-001".to_string(),
            project_id: "PROJ-001".to_string(),
            start_date: dt(2025, 1, 1),
            end_date: dt(2025, 1, 31),
            allocation_percentage: 50,
            status: TaskAssignmentStatus::Active,
        };

        assert!(resource.assign_to_task(task_assignment).is_ok());
        assert_eq!(resource.get_active_task_count(), 1);
        assert_eq!(resource.get_current_allocation_percentage(), 50);

        // Test WIP limit exceeded
        let task_assignment2 = TaskAssignment {
            task_id: "TASK-002".to_string(),
            project_id: "PROJ-001".to_string(),
            start_date: dt(2025, 1, 1),
            end_date: dt(2025, 1, 31),
            allocation_percentage: 60,
            status: TaskAssignmentStatus::Active,
        };

        assert!(resource.assign_to_task(task_assignment2).is_err());

        // Test removing task assignment
        assert!(resource.remove_task_assignment("TASK-001"));
        assert_eq!(resource.get_active_task_count(), 0);
        assert_eq!(resource.get_current_allocation_percentage(), 0);
    }

    #[test]
    fn test_resource_wip_status() {
        let mut resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            Some("john@example.com".to_string()),
            "Developer".to_string(),
            None,
            None,
            None,
            160,
        );

        // Test disabled WIP limits
        resource.disable_wip_limits();
        assert_eq!(resource.get_wip_status(), WipStatus::Disabled);

        // Test within limits
        let limits = WipLimits::new(5, 3, 100);
        resource.set_wip_limits(limits).unwrap();
        assert_eq!(resource.get_wip_status(), WipStatus::WithinLimits);

        // Test near limit
        let limits = WipLimits::new(4, 3, 100);
        resource.set_wip_limits(limits).unwrap();
        
        // Add 3 tasks (75% of limit)
        for i in 0..3 {
            let task_assignment = TaskAssignment {
                task_id: format!("TASK-{:03}", i),
                project_id: "PROJ-001".to_string(),
                start_date: dt(2025, 1, 1),
                end_date: dt(2025, 1, 31),
                allocation_percentage: 25,
                status: TaskAssignmentStatus::Active,
            };
            resource.assign_to_task(task_assignment).unwrap();
        }
        assert_eq!(resource.get_wip_status(), WipStatus::NearLimit);
    }
}
