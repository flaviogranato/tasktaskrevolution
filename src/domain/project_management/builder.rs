use crate::domain::project_management::{project::Project, state::Planned, vacation_rules::VacationRules};

/// Builder for the `Project` struct.
///
/// This builder provides a more ergonomic way to construct a `Project` instance,
/// especially when many fields are optional. It also helps to avoid the "too many arguments"
/// lint in the `Project::new` constructor.
#[derive(Default)]
pub struct ProjectBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    vacation_rules: Option<VacationRules>,
    timezone: Option<String>,
}

impl ProjectBuilder {
    /// Creates a new `ProjectBuilder` with a required name.
    pub fn new(name: String) -> Self {
        Self {
            name: Some(name),
            ..Default::default()
        }
    }

    /// Sets the ID for the project.
    #[allow(dead_code)]
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the description for the project.
    pub fn description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the start date for the project.
    #[allow(dead_code)]
    pub fn start_date(mut self, start_date: String) -> Self {
        self.start_date = Some(start_date);
        self
    }

    /// Sets the end date for the project.
    #[allow(dead_code)]
    pub fn end_date(mut self, end_date: String) -> Self {
        self.end_date = Some(end_date);
        self
    }

    /// Sets the vacation rules for the project.
    #[allow(dead_code)]
    pub fn vacation_rules(mut self, vacation_rules: VacationRules) -> Self {
        self.vacation_rules = Some(vacation_rules);
        self
    }

    /// Builds the `Project` instance.
    ///
    /// # Panics
    ///
    /// Panics if the name is not set, which should not happen if `new()` is used.
    pub fn build(self) -> Project<Planned> {
        Project {
            id: self.id,
            name: self.name.expect("Project name must be set"),
            description: self.description,
            start_date: self.start_date,
            end_date: self.end_date,
            vacation_rules: self.vacation_rules,
            timezone: self.timezone,
            state: Planned,
        }
    }
}
