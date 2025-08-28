use crate::domain::{
    project_management::{project::Project, state::Planned, vacation_rules::VacationRules},
    task_management::any_task::AnyTask,
    shared::errors::{DomainError, DomainErrorKind},
};
use std::collections::HashMap;
use uuid7::{Uuid, uuid7};

// Type states for the builder
pub struct New;
pub struct WithName;
pub struct WithCode;
pub struct WithDates;
pub struct Ready;

/// Builder for the `Project` struct using the typestate pattern.
///
/// This builder provides a more ergonomic and type-safe way to construct a `Project` instance,
/// ensuring all required fields are provided before a project can be built.
#[derive(Debug)]
pub struct ProjectBuilder<State> {
    id: Uuid,
    code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    vacation_rules: Option<VacationRules>,
    timezone: Option<String>,
    tasks: HashMap<String, AnyTask>,
    _state: std::marker::PhantomData<State>,
}

impl ProjectBuilder<New> {
    /// Creates a new `ProjectBuilder` with a required name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid7(),
            name: Some(name.into()),
            code: None,
            description: None,
            start_date: None,
            end_date: None,
            vacation_rules: None,
            timezone: None,
            tasks: HashMap::new(),
            _state: std::marker::PhantomData,
        }
    }
}

impl ProjectBuilder<WithName> {
    /// Sets the code for the project.
    pub fn code(mut self, code: impl Into<String>) -> ProjectBuilder<WithCode> {
        self.code = Some(code.into());
        ProjectBuilder {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            _state: std::marker::PhantomData,
        }
    }
}

impl ProjectBuilder<WithCode> {
    /// Sets the description for the project.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the start date for the project.
    pub fn start_date(mut self, start_date: impl Into<String>) -> Self {
        self.start_date = Some(start_date.into());
        self
    }

    /// Sets the end date for the project.
    pub fn end_date(mut self, end_date: impl Into<String>) -> ProjectBuilder<WithDates> {
        self.end_date = Some(end_date.into());
        ProjectBuilder {
            id: self.id,
            code: self.code,
            name: self.name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            _state: std::marker::PhantomData,
        }
    }
}

impl ProjectBuilder<WithDates> {
    /// Sets the vacation rules for the project.
    pub fn vacation_rules(mut self, vacation_rules: VacationRules) -> Self {
        self.vacation_rules = Some(vacation_rules);
        self
    }

    /// Sets the timezone for the project.
    pub fn timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }

    /// Adds a task to the project.
    pub fn add_task(mut self, task: AnyTask) -> Self {
        self.tasks.insert(task.code().to_string(), task);
        self
    }

    /// Validates the project configuration and builds the `Project` instance.
    pub fn build(self) -> Result<Project<Planned>, DomainError> {
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

        // Validate dates if both are provided
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date)
            && start > end {
            return Err(DomainError::new(DomainErrorKind::ProjectInvalidState {
                current: "invalid_dates".to_string(),
                expected: "start_date < end_date".to_string(),
            }).with_context("Start date must be before end date"));
        }

        Ok(Project {
            id: self.id,
            code,
            name,
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: Planned,
        })
    }
}

// Convenience methods for backward compatibility
impl ProjectBuilder<New> {
    /// Legacy method for backward compatibility.
    /// Prefer using the typestate pattern: new() -> code() -> end_date() -> build()
    pub fn code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    /// Legacy method for backward compatibility.
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Legacy method for backward compatibility.
    pub fn start_date(mut self, start_date: String) -> Self {
        self.start_date = Some(start_date);
        self
    }

    /// Legacy method for backward compatibility.
    pub fn end_date(mut self, end_date: String) -> Self {
        self.end_date = Some(end_date);
        self
    }

    /// Legacy method for backward compatibility.
    pub fn vacation_rules(mut self, vacation_rules: VacationRules) -> Self {
        self.vacation_rules = Some(vacation_rules);
        self
    }

    /// Legacy method for backward compatibility.
    /// 
    /// # Panics
    ///
    /// Panics if the name is not set, which should not happen if `new()` is used.
    pub fn build(self) -> Project<Planned> {
        // For legacy compatibility, we'll create a simple project without validation
        Project {
            id: self.id,
            code: self.code.expect("Project code must be set"),
            name: self.name.expect("Project name must be set"),
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            tasks: self.tasks,
            state: Planned,
        }
    }
}
