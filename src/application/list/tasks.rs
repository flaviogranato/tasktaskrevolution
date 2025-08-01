use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::{any_task::AnyTask, repository::TaskRepository};

pub struct ListTasksUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> ListTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<AnyTask>, DomainError> {
        self.repository.find_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        shared::errors::DomainError,
        task_management::{any_task::AnyTask, state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap, path::Path};
    use uuid7::uuid7;

    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl TaskRepository for MockTaskRepository {
        fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }
        fn save(&self, _task: AnyTask) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<AnyTask>, DomainError> {
            unimplemented!()
        }
        fn load(&self, _path: &Path) -> Result<AnyTask, DomainError> {
            unimplemented!()
        }
        fn find_by_id(&self, _id: &str) -> Result<Option<AnyTask>, DomainError> {
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
        fn get_next_code(&self) -> Result<String, DomainError> {
            unimplemented!()
        }
    }

    fn create_test_task(code: &str, name: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: vec![],
        }
        .into()
    }

    #[test]
    fn test_list_tasks_success() {
        let tasks = vec![
            create_test_task("TSK-1", "First task"),
            create_test_task("TSK-2", "Second task"),
        ];
        let mut task_map = HashMap::new();
        for task in tasks {
            task_map.insert(task.code().to_string(), task);
        }

        let mock_repo = MockTaskRepository {
            tasks: RefCell::new(task_map),
        };
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "First task"));
        assert!(result.iter().any(|t| t.code() == "TSK-2"));
    }

    #[test]
    fn test_list_tasks_empty() {
        let mock_repo = MockTaskRepository {
            tasks: RefCell::new(HashMap::new()),
        };
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert!(result.is_empty());
    }
}
