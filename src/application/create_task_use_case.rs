use chrono::NaiveDate;
use crate::domain::task::{Task, TaskRepository};
use crate::domain::shared_kernel::errors::DomainError;

pub trait CreateTaskUseCase {
    fn execute(
        &self,
        title: String,
        description: String,
        due_date: NaiveDate,
    ) -> Result<Task, DomainError>;
}

pub struct CreateTaskUseCaseImpl<R: TaskRepository> {
    task_repository: R,
}

impl<R: TaskRepository> CreateTaskUseCaseImpl<R> {
    pub fn new(task_repository: R) -> Self {
        Self { task_repository }
    }
}

impl<R: TaskRepository> CreateTaskUseCase for CreateTaskUseCaseImpl<R> {
    fn execute(
        &self,
        title: String,
        description: String,
        due_date: NaiveDate,
    ) -> Result<Task, DomainError> {
        let task = Task::new(title, description, due_date)?;
        self.task_repository.save(task).map_err(|e| DomainError::Generic(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
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

        fn find_by_title(&self, _title: &str) -> Result<Option<Task>, Box<dyn std::error::Error>> {
            Ok(self.saved_task.lock().unwrap().clone())
        }

        fn list(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
            let task = self.saved_task.lock().unwrap().clone();
            Ok(task.map(|t| vec![t]).unwrap_or_default())
        }

        fn delete(&self, _title: &str) -> Result<(), Box<dyn std::error::Error>> {
            *self.saved_task.lock().unwrap() = None;
            Ok(())
        }
    }

    #[test]
    fn test_create_task_use_case() {
        let repository = MockTaskRepository {
            saved_task: Mutex::new(None),
        };
        let use_case = CreateTaskUseCaseImpl::new(repository);

        let due_date = Utc::now().naive_utc().date();
        let task = use_case
            .execute(
                "Test Task".to_string(),
                "Description".to_string(),
                due_date,
            )
            .unwrap();

        assert_eq!(task.title(), "Test Task");
        assert_eq!(task.description(), "Description");
        assert_eq!(task.due_date(), due_date);
    }
}
