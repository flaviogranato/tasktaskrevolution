use crate::domain::task::Task;
use crate::domain::shared_kernel::errors::DomainError;

/// Serviço responsável por gerenciar operações relacionadas a tarefas
pub struct TaskService;

impl TaskService {
    /// Cria uma nova instância do TaskService
    pub fn new() -> Self {
        Self
    }

    /// Cria uma nova tarefa
    pub fn create_task(&self, title: String, description: String, due_date: chrono::NaiveDate) -> Result<Task, DomainError> {
        Task::new(title, description, due_date)
    }

    /// Atualiza uma tarefa existente
    pub fn update_task(&self, task: &mut Task, title: Option<String>, description: Option<String>, due_date: Option<chrono::NaiveDate>) -> Result<(), DomainError> {
        if let Some(title) = title {
            task.set_title(title)?;
        }
        if let Some(description) = description {
            task.set_description(description)?;
        }
        if let Some(due_date) = due_date {
            task.set_due_date(due_date)?;
        }
        Ok(())
    }

    /// Marca uma tarefa como concluída
    pub fn complete_task(&self, task: &mut Task) -> Result<(), DomainError> {
        task.complete()
    }
} 