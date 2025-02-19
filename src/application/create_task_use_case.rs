use crate::domain::task::Task;
use crate::domain::task::TaskRepository;

pub struct CreateTaskUseCase<R: TaskRepository> {
    task_repository: R,
}

impl<R: TaskRepository> CreateTaskUseCase<R> {
    pub fn new(task_repository: R) -> Self {
        Self { task_repository }
    }

    pub fn execute(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let task = Task::new(name, description);
        self.task_repository.save(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockTaskRepository {
        saved_task: Mutex<Option<Task>>,
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>> {
            let cloned_task = task.clone();
            *self.saved_task.lock().unwrap() = Some(task);
            Ok(cloned_task)
        }
    }

    #[test]
    fn test_create_task_use_case() {
        let repository = MockTaskRepository {
            saved_task: Mutex::new(None),
        };
        let use_case = CreateTaskUseCase::new(repository);

        let task = use_case
            .execute("Test Task".to_string(), Some("Description".to_string()))
            .unwrap();

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, Some("Description".to_string()));
    }
} 