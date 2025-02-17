use crate::domain::task::Task;
use crate::domain::task::TaskRepository;

pub struct CreateTaskUseCase<T: TaskRepository> {
    task_repository: T,
}

impl<T: TaskRepository> CreateTaskUseCase<T> {
    pub fn new(task_repository: T) -> Self {
        Self { task_repository }
    }

    pub fn execute(&self, name: String, description: Option<String>) -> Result<Task, Box<dyn std::error::Error>> {
        let task = Task::new(name, description);
        self.task_repository.save(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{Task, TaskRepository};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::error::Error;

    // Mock TaskRepository
    #[derive(Debug, Clone)]
    struct MockTaskRepository {
        saved_task: Rc<RefCell<Option<Task>>>,
    }

    impl MockTaskRepository {
        fn new() -> Self {
            MockTaskRepository {
                saved_task: Rc::new(RefCell::new(None)),
            }
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: Task) -> Result<Task, Box<dyn Error>> {
            *self.saved_task.borrow_mut() = Some(task.clone());
            Ok(task)
        }
    }

    #[test]
    fn test_create_task_use_case() -> Result<(), Box<dyn Error>> {
        let mock_repo = MockTaskRepository::new();
        let use_case = CreateTaskUseCase::new(mock_repo.clone());

        let name = "Test Task".to_string();
        let description = Some("Test Description".to_string());

        let result = use_case.execute(name.clone(), description.clone())?;

        assert_eq!(result.name, name);
        assert_eq!(result.description, description);

        let saved_task = mock_repo.saved_task.borrow();
        assert!(saved_task.is_some());
        assert_eq!(saved_task.as_ref().unwrap().name, name);
        assert_eq!(saved_task.as_ref().unwrap().description, description);

        Ok(())
    }
} 