use crate::domain::resource_management::resource::Resource;
use crate::domain::shared::errors::DomainError;
use chrono::{DateTime, Local};

pub trait ResourceRepository {
    fn save(&self, resource: Resource) -> Result<Resource, DomainError>;
    fn find_all(&self) -> Result<Vec<Resource>, DomainError>;
    fn save_time_off(
        &self,
        resource_name: String,
        hours: u32,
        date: String,
        description: Option<String>,
    ) -> Result<Resource, DomainError>;
    fn save_vacation(
        &self,
        resource_name: String,
        start_date: String,
        end_date: String,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> Result<Resource, DomainError>;
    fn check_if_layoff_period(
        &self,
        start_date: &DateTime<Local>,
        end_date: &DateTime<Local>,
    ) -> bool;
}
