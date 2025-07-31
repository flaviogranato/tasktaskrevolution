use crate::domain::{
    resource_management::repository::ResourceRepository,
    shared::errors::DomainError,
    task_management::{AnyTask, repository::TaskRepository},
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssignResourceError {
    #[error("Task with code '{0}' not found.")]
    TaskNotFound(String),
    #[error("One or more resources not found: {0:?}")]
    ResourcesNotFound(Vec<String>),
    #[error(transparent)]
    RepositoryError(#[from] DomainError),
}

pub struct AssignResourceToTaskUseCase<TR, RR>
where
    TR: TaskRepository,
    RR: ResourceRepository,
{
    task_repository: TR,
    resource_repository: RR,
}

impl<TR, RR> AssignResourceToTaskUseCase<TR, RR>
where
    TR: TaskRepository,
    RR: ResourceRepository,
{
    pub fn new(task_repository: TR, resource_repository: RR) -> Self {
        Self {
            task_repository,
            resource_repository,
        }
    }

    pub fn execute(&self, task_code: &str, resource_codes: &[String]) -> Result<AnyTask, AssignResourceError> {
        // 1. Validate that all resources exist.
        // We assume the resource code is stored in the `id` field.
        let all_resources = self.resource_repository.find_all()?;
        let existing_resource_codes: HashSet<String> = all_resources
            .iter()
            .filter_map(|r| r.id().map(|id| id.to_string()))
            .collect();

        let not_found_resources: Vec<String> = resource_codes
            .iter()
            .filter(|rc| !existing_resource_codes.contains(*rc))
            .cloned()
            .collect();

        if !not_found_resources.is_empty() {
            return Err(AssignResourceError::ResourcesNotFound(not_found_resources));
        }

        // 2. Load the task.
        let task = self
            .task_repository
            .find_by_code(task_code)?
            .ok_or_else(|| AssignResourceError::TaskNotFound(task_code.to_string()))?;

        // 3. Update the task's assigned resources, avoiding duplicates.
        let mut current_assignees: HashSet<String> = task.assigned_resources().iter().cloned().collect();
        for new_resource in resource_codes {
            current_assignees.insert(new_resource.clone());
        }
        let new_assignees: Vec<String> = current_assignees.into_iter().collect();

        // Match on the task to update its state immutably.
        let final_task = match task {
            AnyTask::Planned(mut t) => {
                t.assigned_resources = new_assignees;
                AnyTask::Planned(t)
            }
            AnyTask::InProgress(mut t) => {
                t.assigned_resources = new_assignees;
                AnyTask::InProgress(t)
            }
            AnyTask::Blocked(mut t) => {
                t.assigned_resources = new_assignees;
                AnyTask::Blocked(t)
            }
            AnyTask::Completed(mut t) => {
                t.assigned_resources = new_assignees;
                AnyTask::Completed(t)
            }
            AnyTask::Cancelled(mut t) => {
                t.assigned_resources = new_assignees;
                AnyTask::Cancelled(t)
            }
        };

        // 4. Save the updated task.
        self.task_repository.save(final_task.clone())?;

        Ok(final_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        resource_management::{AnyResource, resource::Resource, state::Available},
        task_management::{AnyTask, state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap, path::Path};

    // Mock Task Repository
    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl MockTaskRepository {
        fn new(initial_tasks: Vec<AnyTask>) -> Self {
            let tasks = initial_tasks.into_iter().map(|t| (t.code().to_string(), t)).collect();
            Self {
                tasks: RefCell::new(tasks),
            }
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: AnyTask) -> Result<(), DomainError> {
            self.tasks.borrow_mut().insert(task.code().to_string(), task);
            Ok(())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().get(code).cloned())
        }
        // -- Unimplemented methods --
        fn load(&self, _path: &Path) -> Result<AnyTask, DomainError> {
            unimplemented!()
        }
        fn find_by_id(&self, _id: &str) -> Result<Option<AnyTask>, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!()
        }
        fn delete(&self, _id: &str) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn find_by_assignee(&self, _assignee: &str) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!()
        }
        fn find_by_date_range(&self, _start: NaiveDate, _end: NaiveDate) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!()
        }
    }

    // Mock Resource Repository
    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.clone())
        }
        // -- Unimplemented methods --
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _name: String,
            _hours: u32,
            _date: String,
            _desc: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _name: String,
            _start: String,
            _end: String,
            _comp: bool,
            _hours: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(
            &self,
            _start: &chrono::DateTime<chrono::Local>,
            _end: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }
    }

    // Helper to create a test task
    fn create_test_task(code: &str, assignees: Vec<&str>) -> AnyTask {
        Task::<Planned> {
            id: format!("task-{}", code),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: assignees.into_iter().map(String::from).collect(),
        }
        .into()
    }

    // Helper to create a test resource
    fn create_test_resource(code: &str) -> AnyResource {
        Resource::<Available> {
            id: Some(code.to_string()),
            name: format!("Resource {}", code),
            email: None,
            resource_type: "Developer".to_string(),
            vacations: None,
            time_off_balance: 0,
            time_off_history: None,
            state: Available,
        }
        .into()
    }

    #[test]
    fn test_assign_new_resources_success() {
        let task_repo = MockTaskRepository::new(vec![create_test_task("TSK-1", vec!["res-1"])]);
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1"), create_test_resource("res-2")],
        };
        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        let result = use_case.execute("TSK-1", &["res-2".to_string()]);

        assert!(result.is_ok());
        let updated_task = result.unwrap();
        let mut assignees = updated_task.assigned_resources().to_vec();
        assignees.sort();
        assert_eq!(assignees, vec!["res-1", "res-2"]);
    }

    #[test]
    fn test_assign_existing_resource_is_idempotent() {
        let task_repo = MockTaskRepository::new(vec![create_test_task("TSK-1", vec!["res-1"])]);
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        let result = use_case.execute("TSK-1", &["res-1".to_string()]);

        assert!(result.is_ok());
        let updated_task = result.unwrap();
        assert_eq!(updated_task.assigned_resources(), &["res-1"]);
    }

    #[test]
    fn test_assign_fails_if_task_not_found() {
        let task_repo = MockTaskRepository::new(vec![]);
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        let result = use_case.execute("TSK-NONEXISTENT", &["res-1".to_string()]);

        assert!(matches!(result, Err(AssignResourceError::TaskNotFound(_))));
    }

    #[test]
    fn test_assign_fails_if_resource_not_found() {
        let task_repo = MockTaskRepository::new(vec![create_test_task("TSK-1", vec![])]);
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        let result = use_case.execute("TSK-1", &["res-NONEXISTENT".to_string()]);

        assert!(matches!(result, Err(AssignResourceError::ResourcesNotFound(_))));
        if let Err(AssignResourceError::ResourcesNotFound(codes)) = result {
            assert_eq!(codes, vec!["res-NONEXISTENT"]);
        }
    }
}
