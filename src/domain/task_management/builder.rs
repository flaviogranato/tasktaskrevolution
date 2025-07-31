use super::state::Planned;
use super::task::{Task, TaskError};
use chrono::NaiveDate;
use std::marker::PhantomData;
use uuid7::uuid7;

// Type states for the builder
#[allow(dead_code)]
pub struct New;
#[allow(dead_code)]
pub struct WithProjectCode;
#[allow(dead_code)]
pub struct WithName;
#[allow(dead_code)]
pub struct WithDates;
#[allow(dead_code)]
pub struct WithResources;
#[allow(dead_code)]
pub struct Ready;

/// A builder for creating `Task` instances in a controlled way, ensuring all
/// required fields are provided before a task can be built.
/// It uses the typestate pattern to enforce the order of method calls at compile time.
#[allow(dead_code)]
pub struct TaskBuilder<State> {
    id: String,
    project_code: Option<String>,
    code: String,
    name: Option<String>,
    start_date: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
    assigned_resources: Vec<String>,
    _state: PhantomData<State>,
}

#[allow(dead_code)]
impl TaskBuilder<New> {
    /// Starts building a new task.
    pub fn new() -> Self {
        let id = uuid7().to_string();
        Self {
            code: format!("TASK-{}", &id[..8]),
            id,
            project_code: None,
            name: None,
            start_date: None,
            due_date: None,
            assigned_resources: Vec::new(),
            _state: PhantomData,
        }
    }

    /// Sets the project code for the task.
    pub fn project_code(self, project_code: impl Into<String>) -> TaskBuilder<WithProjectCode> {
        TaskBuilder {
            id: self.id,
            code: self.code,
            project_code: Some(project_code.into()),
            name: self.name,
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        }
    }
}

impl Default for TaskBuilder<New> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl TaskBuilder<WithProjectCode> {
    /// Sets the name for the task.
    pub fn name(self, name: impl Into<String>) -> TaskBuilder<WithName> {
        TaskBuilder {
            id: self.id,
            code: self.code,
            project_code: self.project_code,
            name: Some(name.into()),
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        }
    }
}

#[allow(dead_code)]
impl TaskBuilder<WithName> {
    /// Sets the start and due dates for the task, validating that the range is valid.
    pub fn dates(self, start: NaiveDate, due: NaiveDate) -> Result<TaskBuilder<WithDates>, TaskError> {
        if start > due {
            return Err(TaskError::InvalidDateRange);
        }

        Ok(TaskBuilder {
            id: self.id,
            code: self.code,
            project_code: self.project_code,
            name: self.name,
            start_date: Some(start),
            due_date: Some(due),
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        })
    }
}

#[allow(dead_code)]
impl TaskBuilder<WithDates> {
    /// Assigns a resource to the task. This moves the builder to the `WithResources` state.
    pub fn assign_resource(mut self, resource_id: impl Into<String>) -> TaskBuilder<WithResources> {
        self.assigned_resources.push(resource_id.into());
        TaskBuilder {
            id: self.id,
            code: self.code,
            project_code: self.project_code,
            name: self.name,
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        }
    }
}

#[allow(dead_code)]
impl TaskBuilder<WithResources> {
    /// Assigns another resource to the task.
    pub fn assign_resource(mut self, resource_id: impl Into<String>) -> Self {
        self.assigned_resources.push(resource_id.into());
        self
    }

    /// Validates that no assigned resources are on vacation during the task's date range.
    /// This moves the builder to the final `Ready` state.
    pub fn validate_vacations(
        self,
        resource_vacations: &[(String, NaiveDate, NaiveDate)],
    ) -> Result<TaskBuilder<Ready>, TaskError> {
        let start = self.start_date.unwrap();
        let due = self.due_date.unwrap();

        for res in &self.assigned_resources {
            for (vac_res, vac_start, vac_end) in resource_vacations {
                if res == vac_res && start <= *vac_end && due >= *vac_start {
                    return Err(TaskError::ResourceOnVacation(res.clone()));
                }
            }
        }

        Ok(TaskBuilder {
            id: self.id,
            code: self.code,
            project_code: self.project_code,
            name: self.name,
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        })
    }
}

#[allow(dead_code)]
impl TaskBuilder<Ready> {
    /// Builds the final `Task<Planned>` instance.
    pub fn build(self) -> Result<Task<Planned>, TaskError> {
        Ok(Task {
            id: self.id,
            project_code: self.project_code.ok_or(TaskError::MissingField("project_code"))?,
            code: self.code,
            name: self.name.ok_or(TaskError::MissingField("name"))?,
            description: None,
            state: Planned, // The task starts in the 'Planned' state.
            start_date: self.start_date.unwrap(),
            due_date: self.due_date.unwrap(),
            actual_end_date: None,
            assigned_resources: self.assigned_resources,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_successful_task_creation() {
        let task = TaskBuilder::new()
            .project_code("PROJ-TEST")
            .name("Test Task")
            .dates(
                NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
            )
            .unwrap()
            .assign_resource("RES-001")
            .validate_vacations(&[])
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(task.project_code, "PROJ-TEST");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.assigned_resources, vec!["RES-001".to_string()]);
        assert_eq!(task.start_date, NaiveDate::from_ymd_opt(2025, 5, 1).unwrap());
        assert_eq!(task.due_date, NaiveDate::from_ymd_opt(2025, 5, 10).unwrap());
        assert!(task.code.starts_with("TASK-"));
        // The state is `Planned` by type, no need for a runtime assertion.
    }

    #[test]
    fn test_invalid_date_range() {
        let result = TaskBuilder::new()
            .project_code("PROJ-TEST")
            .name("Task com datas invertidas")
            .dates(
                NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
                NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
            );

        assert!(matches!(result, Err(TaskError::InvalidDateRange)));
    }

    #[test]
    fn test_resource_on_vacation() {
        let vacations = vec![(
            "RES-002".to_string(),
            NaiveDate::from_ymd_opt(2025, 5, 5).unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 7).unwrap(),
        )];

        let result = TaskBuilder::new()
            .project_code("PROJ-TEST")
            .name("Task com conflito de férias")
            .dates(
                NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
            )
            .unwrap()
            .assign_resource("RES-002")
            .validate_vacations(&vacations);

        assert!(matches!(result, Err(TaskError::ResourceOnVacation(res)) if res == "RES-002"));
    }

    #[test]
    fn test_multiple_resources_and_no_vacation_conflict() {
        let vacations = vec![(
            "RES-003".to_string(),
            NaiveDate::from_ymd_opt(2025, 5, 15).unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 20).unwrap(),
        )];

        let task = TaskBuilder::new()
            .project_code("PROJ-TEST")
            .name("Task multi recursos")
            .dates(
                NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
            )
            .unwrap()
            .assign_resource("RES-003")
            .assign_resource("RES-004")
            .validate_vacations(&vacations)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(
            task.assigned_resources,
            vec!["RES-003".to_string(), "RES-004".to_string()]
        );
    }
}
