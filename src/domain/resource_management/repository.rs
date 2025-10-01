use crate::domain::resource_management::AnyResource;
use crate::domain::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, Local};

pub trait ResourceRepository {
    fn save(&self, resource: AnyResource) -> DomainResult<AnyResource>;
    fn save_in_hierarchy(
        &self,
        resource: AnyResource,
        company_code: &str,
        project_code: Option<&str>,
    ) -> DomainResult<AnyResource>;
    fn find_all(&self) -> DomainResult<Vec<AnyResource>>;
    fn find_by_company(&self, company_code: &str) -> DomainResult<Vec<AnyResource>>;

    /// Find all resources with their context information (company and project codes)
    fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>>;
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>>;
    fn save_time_off(
        &self,
        resource_name: &str,
        hours: u32,
        date: &str,
        description: Option<String>,
    ) -> DomainResult<AnyResource>;
    fn save_vacation(
        &self,
        resource_name: &str,
        start_date: &str,
        end_date: &str,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> DomainResult<AnyResource>;
    fn check_if_layoff_period(&self, start_date: &DateTime<Local>, end_date: &DateTime<Local>) -> bool;
    fn get_next_code(&self, resource_type: &str) -> DomainResult<String>;
}

/// Extension trait for repositories that support ID-based operations
pub trait ResourceRepositoryWithId: ResourceRepository {
    fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>>;
}
