#![allow(dead_code)]

use crate::domain::company_management::Company;
use crate::application::errors::AppError;

/// Repository trait for Company entity operations.
pub trait CompanyRepository: Send + Sync {
    /// Saves a company to the repository.
    fn save(&self, company: Company) -> Result<(), AppError>;

    /// Finds a company by its unique identifier.
    fn find_by_id(&self, id: &str) -> Result<Option<Company>, AppError>;

    /// Finds a company by its code.
    fn find_by_code(&self, code: &str) -> Result<Option<Company>, AppError>;

    /// Finds a company by its name.
    fn find_by_name(&self, name: &str) -> Result<Option<Company>, AppError>;

    /// Retrieves all companies from the repository.
    fn find_all(&self) -> Result<Vec<Company>, AppError>;

    /// Updates an existing company.
    fn update(&self, company: Company) -> Result<(), AppError>;

    /// Soft deletes a company by changing its status to Inactive.
    fn delete(&self, id: &str) -> Result<(), AppError>;

    /// Generates the next available company code.
    fn get_next_code(&self) -> Result<String, AppError>;

    /// Checks if a company code already exists.
    fn code_exists(&self, code: &str) -> Result<bool, AppError>;

    /// Checks if a name already exists.
    fn name_exists(&self, name: &str) -> Result<bool, AppError>;
}
