use crate::domain::resource_management::AnyResource;
use crate::domain::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, Local};

/// Repository trait for Resource entity operations.
/// 
/// This trait defines the contract for resource persistence operations,
/// following the Repository pattern from Domain-Driven Design.
/// Implementations should be provided by the infrastructure layer.
pub trait ResourceRepository {
    /// Saves a resource to the repository.
    /// 
    /// # Arguments
    /// * `resource` - The resource to save
    /// 
    /// # Returns
    /// * `Ok(saved_resource)` - The saved resource with any generated fields
    /// * `Err(DomainError)` if an error occurred during save
    fn save(&self, resource: AnyResource) -> DomainResult<AnyResource>;

    /// Saves a resource in the hierarchical structure (company/project/resource).
    /// 
    /// # Arguments
    /// * `resource` - The resource to save
    /// * `company_code` - The company code for the hierarchy
    /// * `project_code` - Optional project code for the hierarchy
    /// 
    /// # Returns
    /// * `Ok(saved_resource)` - The saved resource with any generated fields
    /// * `Err(DomainError)` if an error occurred during save
    fn save_in_hierarchy(
        &self,
        resource: AnyResource,
        company_code: &str,
        project_code: Option<&str>,
    ) -> DomainResult<AnyResource>;

    /// Retrieves all resources from the repository.
    /// 
    /// # Returns
    /// * `Ok(resources)` - Vector of all resources
    /// * `Err(DomainError)` if an error occurred during retrieval
    fn find_all(&self) -> DomainResult<Vec<AnyResource>>;

    /// Finds all resources for a specific company.
    /// 
    /// # Arguments
    /// * `company_code` - The company code to search for
    /// 
    /// # Returns
    /// * `Ok(resources)` - Vector of resources for the company
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_company(&self, company_code: &str) -> DomainResult<Vec<AnyResource>>;

    /// Finds all resources with their context information (company and project codes).
    /// 
    /// This method is useful for CQRS read models and reporting scenarios.
    /// 
    /// # Returns
    /// * `Ok(contexts)` - Vector of tuples containing (resource, company_code, project_codes)
    /// * `Err(DomainError)` if an error occurred during retrieval
    fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>>;

    /// Finds a resource by its code.
    /// 
    /// # Arguments
    /// * `code` - The resource code to search for
    /// 
    /// # Returns
    /// * `Ok(Some(resource))` if the resource was found
    /// * `Ok(None)` if no resource was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>>;

    /// Saves a time off entry for a resource.
    /// 
    /// # Arguments
    /// * `resource_name` - The name of the resource
    /// * `hours` - Number of hours for time off
    /// * `date` - Date for the time off (YYYY-MM-DD format)
    /// * `description` - Optional description for the time off
    /// 
    /// # Returns
    /// * `Ok(updated_resource)` - The resource with updated time off balance
    /// * `Err(DomainError)` if an error occurred during save
    fn save_time_off(
        &self,
        resource_name: &str,
        hours: u32,
        date: &str,
        description: Option<String>,
    ) -> DomainResult<AnyResource>;

    /// Saves a vacation entry for a resource.
    /// 
    /// # Arguments
    /// * `resource_name` - The name of the resource
    /// * `start_date` - Start date of vacation (YYYY-MM-DD format)
    /// * `end_date` - End date of vacation (YYYY-MM-DD format)
    /// * `is_time_off_compensation` - Whether this is time off compensation
    /// * `compensated_hours` - Optional compensated hours
    /// 
    /// # Returns
    /// * `Ok(updated_resource)` - The resource with updated vacation records
    /// * `Err(DomainError)` if an error occurred during save
    fn save_vacation(
        &self,
        resource_name: &str,
        start_date: &str,
        end_date: &str,
        is_time_off_compensation: bool,
        compensated_hours: Option<u32>,
    ) -> DomainResult<AnyResource>;

    /// Checks if a given period falls within a layoff period.
    /// 
    /// # Arguments
    /// * `start_date` - Start date of the period to check
    /// * `end_date` - End date of the period to check
    /// 
    /// # Returns
    /// * `true` if the period is within a layoff period
    /// * `false` otherwise
    fn check_if_layoff_period(&self, start_date: &DateTime<Local>, end_date: &DateTime<Local>) -> bool;

    /// Generates the next available resource code for a specific type.
    /// 
    /// # Arguments
    /// * `resource_type` - The type of resource to generate a code for
    /// 
    /// # Returns
    /// * `Ok(code)` - The next available resource code
    /// * `Err(DomainError)` if an error occurred during code generation
    fn get_next_code(&self, resource_type: &str) -> DomainResult<String>;
}

/// Extension trait for repositories that support ID-based operations.
/// 
/// This trait extends the base ResourceRepository with ID-based lookup capabilities,
/// which is useful for CQRS patterns and advanced querying scenarios.
pub trait ResourceRepositoryWithId: ResourceRepository {
    /// Finds a resource by its unique identifier.
    /// 
    /// # Arguments
    /// * `id` - The resource ID to search for
    /// 
    /// # Returns
    /// * `Ok(Some(resource))` if the resource was found
    /// * `Ok(None)` if no resource was found
    /// * `Err(DomainError)` if an error occurred during search
    fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>>;
}
