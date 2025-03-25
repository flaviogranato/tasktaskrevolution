use crate::domain::task::Task;
use crate::domain::task::TaskRepository;
use chrono::{DateTime, Utc};

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
        due_date: Option<DateTime<Utc>>,
    ) -> Result<Task, Box<dyn std::error::Error>> {
        let task = Task::new(name, description, due_date);
        self.task_repository.save(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
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

        let due_date = Some(Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap());
        let task = use_case
            .execute(
                "Test Task".to_string(),
                Some("Description".to_string()),
                due_date,
            )
            .unwrap();

        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, Some("Description".to_string()));
        assert_eq!(task.due_date, due_date);
    }
}
