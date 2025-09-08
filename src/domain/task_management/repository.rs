#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::domain::task_management::AnyTask;

pub trait TaskRepository {
    fn save(&self, task: AnyTask) -> Result<AnyTask, AppError>;
    fn save_in_hierarchy(&self, task: AnyTask, company_code: &str, project_code: &str) -> Result<AnyTask, AppError>;
    fn find_all(&self) -> Result<Vec<AnyTask>, AppError>;
    fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, AppError>;
    fn find_by_project(&self, project_code: &str) -> Result<Vec<AnyTask>, AppError>;
    fn find_all_by_project(&self, company_code: &str, project_code: &str) -> Result<Vec<AnyTask>, AppError>;
    fn get_next_code(&self, project_code: &str) -> Result<String, AppError>;
}
