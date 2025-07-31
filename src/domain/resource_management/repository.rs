use crate::domain::resource_management::AnyResource;
use crate::domain::shared::errors::DomainError;
use chrono::{DateTime, Local};

pub trait ResourceRepository {
    fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError>;
    fn find_all(&self) -> Result<Vec<AnyResource>, DomainError>;
    fn save_time_off(
        &self,
        resource_name: String,
        hours: u32,
        date: String,
        description: Option<String>,
    ) -> Result<AnyResource, DomainError>;
    fn save_vacation(
        &self,
        resource_name: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<AnyResource, DomainError>;
    fn check_if_layoff_period(&self, start_date: &DateTime<Local>, end_date: &DateTime<Local>) -> bool;
    fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError>;
}
