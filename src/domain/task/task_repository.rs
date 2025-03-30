use crate::domain::task::Task;

pub trait TaskRepository {
    fn save(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>>;
    fn find_by_title(&self, title: &str) -> Result<Option<Task>, Box<dyn std::error::Error>>;
    fn list(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>>;
    fn delete(&self, title: &str) -> Result<(), Box<dyn std::error::Error>>;
}
