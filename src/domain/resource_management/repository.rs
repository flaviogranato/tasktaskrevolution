use crate::domain::resource_management::AnyResource;
use crate::domain::shared::errors::DomainError;
use chrono::{DateTime, Local};

pub trait ResourceRepository {
    fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError>;
    fn save_in_hierarchy(&self, resource: AnyResource, company_code: &str, project_code: Option<&str>) -> Result<AnyResource, DomainError>;
    fn find_all(&self) -> Result<Vec<AnyResource>, DomainError>;
    fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, DomainError>;
    fn save_time_off(
        &self,
        resource_name: &str,
        hours: u32,
        date: &str,
        description: Option<String>,
    ) -> Result<AnyResource, DomainError>;
    fn save_vacation(
        &self,
        resource_name: &str,
        start_date: &str,
        end_date: &str,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<AnyResource, DomainError>;
    fn check_if_layoff_period(&self, start_date: &DateTime<Local>, end_date: &DateTime<Local>) -> bool;
    fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError>;
}
