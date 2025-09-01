use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid7;

use crate::domain::shared::errors::{DomainError, DomainErrorKind};

/// Represents a company entity in the system.
/// 
/// A company is an aggregate root that can have multiple projects,
/// resources, and other business entities associated with it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Company {
    /// Unique identifier for the company
    pub id: String,
    /// Human-readable code for the company (e.g., "COMP-001")
    pub code: String,
    /// Official company name
    pub name: String,
    /// Optional company description
    pub description: Option<String>,
    /// Company's tax identification number (CNPJ in Brazil)
    pub tax_id: Option<String>,
    /// Company's legal address
    pub address: Option<String>,
    /// Company's contact email
    pub email: Option<String>,
    /// Company's phone number
    pub phone: Option<String>,
    /// Company's website
    pub website: Option<String>,
    /// Company's industry/sector
    pub industry: Option<String>,
    /// Company's size (Small, Medium, Large)
    pub size: CompanySize,
    /// Company's status (Active, Inactive, Suspended)
    pub status: CompanyStatus,
    /// When the company was created in the system
    pub created_at: DateTime<Utc>,
    /// When the company was last updated
    pub updated_at: DateTime<Utc>,
    /// Who created the company record
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompanySize {
    Small,      // 1-50 employees
    Medium,     // 51-250 employees
    Large,      // 251+ employees
}

impl std::fmt::Display for CompanySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompanySize::Small => write!(f, "Pequena (1-50 funcionários)"),
            CompanySize::Medium => write!(f, "Média (51-250 funcionários)"),
            CompanySize::Large => write!(f, "Grande (251+ funcionários)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompanyStatus {
    Active,
    Inactive,
    Suspended,
}

impl std::fmt::Display for CompanyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompanyStatus::Active => write!(f, "Ativa"),
            CompanyStatus::Inactive => write!(f, "Inativa"),
            CompanyStatus::Suspended => write!(f, "Suspensa"),
        }
    }
}

impl Company {
    /// Creates a new company instance.
    pub fn new(
        code: String,
        name: String,
        created_by: String,
    ) -> Result<Self, DomainError> {
        if code.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: "Company code cannot be empty".to_string(),
            }));
        }

        if name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Company name cannot be empty".to_string(),
            }));
        }

        let now = Utc::now();
        
        Ok(Self {
            id: uuid7::uuid7().to_string(),
            code,
            name,
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            size: CompanySize::Medium,
            status: CompanyStatus::Active,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }

    /// Updates the company name.
    pub fn update_name(&mut self, new_name: String) -> Result<(), DomainError> {
        if new_name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Company name cannot be empty".to_string(),
            }));
        }

        self.name = new_name;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Updates the company description.
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// Updates the company tax ID.
    pub fn update_tax_id(&mut self, tax_id: Option<String>) {
        self.tax_id = tax_id;
        self.updated_at = Utc::now();
    }

    /// Updates the company address.
    pub fn update_address(&mut self, address: Option<String>) {
        self.address = address;
        self.updated_at = Utc::now();
    }

    /// Updates the company email.
    pub fn update_email(&mut self, email: Option<String>) {
        self.email = email;
        self.updated_at = Utc::now();
    }

    /// Updates the company phone.
    pub fn update_phone(&mut self, phone: Option<String>) {
        self.phone = phone;
        self.updated_at = Utc::now();
    }

    /// Updates the company website.
    pub fn update_website(&mut self, website: Option<String>) {
        self.website = website;
        self.updated_at = Utc::now();
    }

    /// Updates the company industry.
    pub fn update_industry(&mut self, industry: Option<String>) {
        self.industry = industry;
        self.updated_at = Utc::now();
    }

    /// Updates the company size.
    pub fn update_size(&mut self, size: CompanySize) {
        self.size = size;
        self.updated_at = Utc::now();
    }

    /// Changes the company status.
    pub fn change_status(&mut self, new_status: CompanyStatus) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    /// Checks if the company is active.
    pub fn is_active(&self) -> bool {
        matches!(self.status, CompanyStatus::Active)
    }

    /// Gets the company code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Gets the company name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the company ID.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Default for CompanySize {
    fn default() -> Self {
        CompanySize::Medium
    }
}

impl Default for CompanyStatus {
    fn default() -> Self {
        CompanyStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_creation() {
        let company = Company::new(
            "COMP-001".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        assert_eq!(company.code(), "COMP-001");
        assert_eq!(company.name(), "TechConsulting Ltda");
        assert_eq!(company.created_by, "user@example.com");
        assert!(company.is_active());
        assert_eq!(company.size, CompanySize::Medium);
    }

    #[test]
    fn test_company_creation_empty_code() {
        let result = Company::new(
            "".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_company_creation_empty_name() {
        let result = Company::new(
            "COMP-001".to_string(),
            "".to_string(),
            "user@example.com".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_company_update_name() {
        let mut company = Company::new(
            "COMP-001".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        company.update_name("New Company Name".to_string()).unwrap();
        assert_eq!(company.name(), "New Company Name");
    }

    #[test]
    fn test_company_update_name_empty() {
        let mut company = Company::new(
            "COMP-001".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        let result = company.update_name("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_company_change_status() {
        let mut company = Company::new(
            "COMP-001".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        ).unwrap();

        company.change_status(CompanyStatus::Inactive);
        assert!(!company.is_active());
        assert_eq!(company.status, CompanyStatus::Inactive);
    }
}
