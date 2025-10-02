#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::task_management::AnyTask;

pub trait TaskRepository {
    fn save(&self, task: AnyTask) -> DomainResult<AnyTask>;
    fn save_in_hierarchy(&self, task: AnyTask, company_code: &str, project_code: &str) -> DomainResult<AnyTask>;
    fn find_all(&self) -> DomainResult<Vec<AnyTask>>;
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyTask>>;
    fn find_by_project(&self, project_code: &str) -> DomainResult<Vec<AnyTask>>;
    fn find_all_by_project(&self, company_code: &str, project_code: &str) -> DomainResult<Vec<AnyTask>>;
    fn get_next_code(&self, project_code: &str) -> DomainResult<String>;
}
