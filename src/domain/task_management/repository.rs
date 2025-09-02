#![allow(dead_code)]

use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::AnyTask;

pub trait TaskRepository {
    fn save(&self, task: AnyTask) -> Result<AnyTask, DomainError>;
    fn save_in_hierarchy(&self, task: AnyTask, company_code: &str, project_code: &str) -> Result<AnyTask, DomainError>;
    fn find_all(&self) -> Result<Vec<AnyTask>, DomainError>;
    fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError>;
    fn find_by_project(&self, project_code: &str) -> Result<Vec<AnyTask>, DomainError>;
    fn find_all_by_project(&self, company_code: &str, project_code: &str) -> Result<Vec<AnyTask>, DomainError>;
    fn get_next_code(&self, project_code: &str) -> Result<String, DomainError>;
}
