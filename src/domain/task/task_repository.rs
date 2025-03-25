use super::Task;

pub trait TaskRepository {
    fn save(&self, task: Task) -> Result<Task, Box<dyn std::error::Error>>;
}
