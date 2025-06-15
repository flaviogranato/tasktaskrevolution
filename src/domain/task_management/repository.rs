use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::{Task, TaskStatus};
use std::path::Path;

pub trait TaskRepository {
    /// Salva uma task no repositório
    fn save(&self, task: Task) -> Result<(), DomainError>;

    /// Carrega uma task de um arquivo específico
    fn load(&self, path: &Path) -> Result<Task, DomainError>;

    /// Busca uma task pelo código
    fn find_by_code(&self, code: &str) -> Result<Option<Task>, DomainError>;

    /// Busca uma task pelo ID
    fn find_by_id(&self, id: &str) -> Result<Option<Task>, DomainError>;

    /// Retorna todas as tasks
    fn find_all(&self) -> Result<Vec<Task>, DomainError>;

    /// Deleta uma task pelo código
    fn delete(&self, code: &str) -> Result<(), DomainError>;

    /// Atualiza o status de uma task
    fn update_status(&self, code: &str, new_status: TaskStatus) -> Result<Task, DomainError>;

    /// Busca tasks por responsável
    fn find_by_assignee(&self, assignee: &str) -> Result<Vec<Task>, DomainError>;

    /// Busca tasks por status
    fn find_by_status(&self, status: &TaskStatus) -> Result<Vec<Task>, DomainError>;

    /// Busca tasks por intervalo de datas
    fn find_by_date_range(&self, start_date: chrono::NaiveDate, end_date: chrono::NaiveDate) -> Result<Vec<Task>, DomainError>;
}
