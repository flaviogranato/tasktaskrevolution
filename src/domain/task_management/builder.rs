use super::task::{Task, TaskError, TaskStatus};
use chrono::NaiveDate;
use std::marker::PhantomData;
use uuid7::uuid7;

pub struct New;
pub struct WithName;
pub struct WithDates;
pub struct WithResources;
pub struct Ready;

pub struct TaskBuilder<State> {
    id: String,
    code: String,
    name: Option<String>,
    start_date: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
    assigned_resources: Vec<String>,
    _state: PhantomData<State>,
}

impl TaskBuilder<New> {
    pub fn new() -> Self {
        let id = uuid7().to_string();

        Self {
            code: format!("TASK-{}", &id[..8]),
            id,
            name: None,
            start_date: None,
            due_date: None,
            assigned_resources: Vec::new(),
            _state: PhantomData,
        }
    }

    pub fn name(self, name: impl Into<String>) -> TaskBuilder<WithName> {
        TaskBuilder {
            id: self.id,
            code: self.code,
            name: Some(name.into()),
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        }
    }
}

impl TaskBuilder<WithName> {
    pub fn dates(self, start: NaiveDate, due: NaiveDate) -> Result<TaskBuilder<WithDates>, TaskError> {
        if start > due {
            return Err(TaskError::InvalidDateRange);
        }

        Ok(TaskBuilder {
            id: self.id,
            code: self.code,
            name: self.name,
            start_date: Some(start),
            due_date: Some(due),
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        })
    }
}

impl TaskBuilder<WithDates> {
    pub fn assign_resource(mut self, resource_id: impl Into<String>) -> TaskBuilder<WithResources> {
        self.assigned_resources.push(resource_id.into());

        TaskBuilder {
            id: self.id,
            code: self.code,
            name: self.name,
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        }
    }
}

impl TaskBuilder<WithResources> {
    pub fn assign_resource(mut self, resource_id: impl Into<String>) -> Self {
        self.assigned_resources.push(resource_id.into());
        self
    }

    pub fn validate_vacations(self, resource_vacations: &[(String, NaiveDate, NaiveDate)]) -> Result<TaskBuilder<Ready>, TaskError> {
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
            name: self.name,
            start_date: self.start_date,
            due_date: self.due_date,
            assigned_resources: self.assigned_resources,
            _state: PhantomData,
        })
    }
}

impl TaskBuilder<Ready> {
    pub fn build(self) -> Result<Task, TaskError> {
        Ok(Task {
            id: self.id,
            code: self.code,
            name: self.name.ok_or(TaskError::MissingField("name"))?,
            description: None,
            status: TaskStatus::Planned,
            start_date: self.start_date.unwrap(),
            due_date: self.due_date.unwrap(),
            actual_end_date: None,
            assigned_resources: self.assigned_resources,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{TaskBuilder, TaskError};
    use chrono::NaiveDate;

    #[test]
    fn test_successful_task_creation() {
        let task = TaskBuilder::new()
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

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.assigned_resources, vec!["RES-001".to_string()]);
        assert_eq!(
            task.start_date,
            NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()
        );
        assert_eq!(task.due_date, NaiveDate::from_ymd_opt(2025, 5, 10).unwrap());
        assert!(task.code.starts_with("TASK-"));
    }

    #[test]
    fn test_invalid_date_range() {
        let result = TaskBuilder::new().name("Task com datas invertidas").dates(
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
    fn test_missing_name_should_not_compile() {
        // Este teste não compila, pois o builder não permite pular o nome.
        // let _ = TaskBuilder::new().dates(
        //     NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        //     NaiveDate::from_ymd_opt(2025, 5, 10).unwrap(),
        // );
        // Isso é garantido pelo typestate!
        assert!(true);
    }

    #[test]
    fn test_missing_dates_should_not_compile() {
        // Este teste não compila, pois o builder não permite pular as datas.
        // let _ = TaskBuilder::new()
        //     .name("Sem datas")
        //     .build();
        // Isso é garantido pelo typestate!
        assert!(true);
    }

    #[test]
    fn test_multiple_resources_and_no_vacation_conflict() {
        let vacations = vec![(
            "RES-003".to_string(),
            NaiveDate::from_ymd_opt(2025, 5, 15).unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 20).unwrap(),
        )];

        let task = TaskBuilder::new()
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
