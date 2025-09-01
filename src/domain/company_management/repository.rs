use async_trait::async_trait;

use crate::domain::company_management::Company;
use crate::domain::shared::errors::DomainError;

/// Repository trait for Company entity operations.
#[async_trait]
pub trait CompanyRepository: Send + Sync {
    /// Saves a company to the repository.
    async fn save(&self, company: Company) -> Result<(), DomainError>;

    /// Finds a company by its unique identifier.
    async fn find_by_id(&self, id: &str) -> Result<Option<Company>, DomainError>;

    /// Finds a company by its code.
    async fn find_by_code(&self, code: &str) -> Result<Option<Company>, DomainError>;

    /// Finds a company by its name.
    async fn find_by_name(&self, name: &str) -> Result<Option<Company>, DomainError>;

    /// Retrieves all companies from the repository.
    async fn find_all(&self) -> Result<Vec<Company>, DomainError>;

    /// Updates an existing company.
    async fn update(&self, company: Company) -> Result<(), DomainError>;

    /// Soft deletes a company by changing its status to Inactive.
    async fn delete(&self, id: &str) -> Result<(), DomainError>;

    /// Generates the next available company code.
    async fn get_next_code(&self) -> Result<String, DomainError>;

    /// Checks if a company code already exists.
    async fn code_exists(&self, code: &str) -> Result<bool, DomainError>;

    /// Checks if a company name already exists.
    async fn name_exists(&self, name: &str) -> Result<bool, DomainError>;
}
