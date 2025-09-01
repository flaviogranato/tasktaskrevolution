use super::project::{Project, ProjectSettings, VacationRules, WorkHours};
use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use crate::domain::shared::errors::DomainErrorKind::ProjectInvalidState;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;
use uuid7::Uuid;

/// Builder for creating `Project` instances with a fluent interface.
/// This builder ensures that all required fields are set before building.
#[derive(Debug, Clone)]
pub struct ProjectBuilder {
    id: String,
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    company_code: Option<String>,
    created_by: Option<String>,
    vacation_rules: Option<VacationRules>,
    timezone: Option<String>,
    work_hours: Option<WorkHours>,
    tasks: HashMap<String, crate::domain::task_management::any_task::AnyTask>,
}

impl ProjectBuilder {
    /// Creates a new `ProjectBuilder` instance.
    pub fn new() -> Self {
        Self {
            id: uuid7::uuid7().to_string(),
            code: None,
            name: None,
            description: None,
            start_date: None,
            end_date: None,
            company_code: None,
            created_by: None,
            vacation_rules: None,
            timezone: None,
            work_hours: None,
            tasks: HashMap::new(),
        }
    }

    /// Sets the project code.
    pub fn code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    /// Sets the project name.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the project description.
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the project start date.
    pub fn start_date(mut self, start_date: NaiveDate) -> Self {
        self.start_date = Some(start_date);
        self
    }

    /// Sets the project end date.
    pub fn end_date(mut self, end_date: NaiveDate) -> Self {
        self.end_date = Some(end_date);
        self
    }

    /// Sets the company code.
    pub fn company_code(mut self, company_code: String) -> Self {
        self.company_code = Some(company_code);
        self
    }

    /// Sets the creator of the project.
    pub fn created_by(mut self, created_by: String) -> Self {
        self.created_by = Some(created_by);
        self
    }

    /// Sets the vacation rules.
    pub fn vacation_rules(mut self, vacation_rules: VacationRules) -> Self {
        self.vacation_rules = Some(vacation_rules);
        self
    }

    /// Sets the timezone.
    pub fn timezone(mut self, timezone: String) -> Self {
        self.timezone = Some(timezone);
        self
    }

    /// Sets the work hours.
    pub fn work_hours(mut self, work_hours: WorkHours) -> Self {
        self.work_hours = Some(work_hours);
        self
    }

    /// Adds a task to the project.
    pub fn add_task(mut self, task: crate::domain::task_management::any_task::AnyTask) -> Self {
        self.tasks.insert(task.code().to_string(), task);
        self
    }

    /// Validates the project configuration and builds the `Project` instance.
    pub fn build(self) -> Result<Project, DomainError> {
        // Validate required fields
        let code = self.code.ok_or_else(|| {
            DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: "Project code is required".to_string(),
            })
        })?;

        let name = self.name.ok_or_else(|| {
            DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Project name is required".to_string(),
            })
        })?;

        let company_code = self.company_code.ok_or_else(|| {
            DomainError::new(DomainErrorKind::ValidationError {
                field: "company_code".to_string(),
                message: "Company code is required".to_string(),
            })
        })?;

        let created_by = self.created_by.ok_or_else(|| {
            DomainError::new(DomainErrorKind::ValidationError {
                field: "created_by".to_string(),
                message: "Creator is required".to_string(),
            })
        })?;

        // Validate dates if both are provided
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            if start > end {
                return Err(DomainError::new(ProjectInvalidState {
                    current: "invalid_dates".to_string(),
                    expected: "start_date < end_date".to_string(),
                })
                .with_context("Start date must be before end date"));
            }
        }

        let now = Utc::now();
        
        let settings = ProjectSettings {
            timezone: self.timezone,
            vacation_rules: self.vacation_rules,
            work_hours: self.work_hours,
        };

        Ok(Project {
            id: self.id,
            code,
            name,
            description: self.description,
            status: super::project::ProjectStatus::Planned,
            priority: super::project::ProjectPriority::Medium,
            start_date: self.start_date,
            end_date: self.end_date,
            actual_start_date: None,
            actual_end_date: None,
            company_code,
            manager_id: None,
            created_at: now,
            updated_at: now,
            created_by,
            tasks: self.tasks,
            resources: HashMap::new(),
            settings,
            metadata: HashMap::new(),
        })
    }
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_project_builder_with_required_fields() {
        let project = ProjectBuilder::new()
            .code("PROJ-001".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("user@example.com".to_string())
            .build()
            .unwrap();

        assert_eq!(project.code(), "PROJ-001");
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.company_code(), "COMP-001");
        assert_eq!(project.created_by(), "user@example.com");
    }

    #[test]
    fn test_project_builder_with_optional_fields() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let project = ProjectBuilder::new()
            .code("PROJ-001".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("user@example.com".to_string())
            .description(Some("A test project".to_string()))
            .start_date(start_date)
            .end_date(end_date)
            .build()
            .unwrap();

        assert_eq!(project.description, Some("A test project".to_string()));
        assert_eq!(project.start_date, Some(start_date));
        assert_eq!(project.end_date, Some(end_date));
    }

    #[test]
    fn test_project_builder_validation_missing_code() {
        let result = ProjectBuilder::new()
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("user@example.com".to_string())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_project_builder_validation_missing_name() {
        let result = ProjectBuilder::new()
            .code("PROJ-001".to_string())
            .company_code("COMP-001".to_string())
            .created_by("user@example.com".to_string())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_project_builder_validation_invalid_dates() {
        let start_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let result = ProjectBuilder::new()
            .code("PROJ-001".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("user@example.com".to_string())
            .start_date(start_date)
            .end_date(end_date)
            .build();

        assert!(result.is_err());
    }
}
