#![allow(dead_code)]

use crate::domain::company_management::Company;
use crate::domain::shared::errors::{DomainError, DomainResult};

/// Repository trait for Company entity operations.
pub trait CompanyRepository: Send + Sync {
    /// Saves a company to the repository.
    fn save(&self, company: Company) -> DomainResult<()>;

    /// Finds a company by its unique identifier.
    fn find_by_id(&self, id: &str) -> DomainResult<Option<Company>>;

    /// Finds a company by its code.
    fn find_by_code(&self, code: &str) -> DomainResult<Option<Company>>;

    /// Finds a company by its name.
    fn find_by_name(&self, name: &str) -> DomainResult<Option<Company>>;

    /// Retrieves all companies from the repository.
    fn find_all(&self) -> DomainResult<Vec<Company>>;

    /// Updates an existing company.
    fn update(&self, company: Company) -> DomainResult<()>;

    /// Soft deletes a company by changing its status to Inactive.
    fn delete(&self, id: &str) -> DomainResult<()>;

    /// Generates the next available company code.
    fn get_next_code(&self) -> DomainResult<String>;

    /// Checks if a company code already exists.
    fn code_exists(&self, code: &str) -> DomainResult<bool>;

    /// Checks if a name already exists.
    fn name_exists(&self, name: &str) -> DomainResult<bool>;
}
