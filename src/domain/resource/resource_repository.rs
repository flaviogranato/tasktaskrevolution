use crate::domain::shared_kernel::errors::DomainError;

use super::resource::Resource;

pub trait ResourceRepository {
    fn save(&self, resource: Resource) -> Result<Resource, DomainError>;
    fn find_all(&self) -> Result<Vec<Resource>, DomainError>;
    fn save_time_off(&self, resource_name: String, hours: u32, date: String, description: Option<String>) -> Result<Resource, DomainError>;
    fn save_vacation(&self, resource_name: String, start_date: String, end_date: String, is_time_off_compensation: bool, compensated_hours: Option<u32>) -> Result<Resource, DomainError>;
}
