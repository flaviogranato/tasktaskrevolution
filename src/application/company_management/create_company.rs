use crate::domain::company_management::{Company, CompanyRepository};
use crate::domain::shared::errors::DomainError;

/// Arguments for creating a new company.
#[derive(Debug, Clone)]
pub struct CreateCompanyArgs {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_id: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub created_by: String,
}

/// Use case for creating a new company.
pub struct CreateCompanyUseCase<R>
where
    R: CompanyRepository,
{
    company_repository: R,
}

impl<R> CreateCompanyUseCase<R>
where
    R: CompanyRepository,
{
    /// Creates a new instance of CreateCompanyUseCase.
    pub fn new(company_repository: R) -> Self {
        Self { company_repository }
    }

    /// Executes the company creation use case.
    pub fn execute(&self, args: CreateCompanyArgs) -> Result<Company, DomainError> {
        // Generate code automatically if not provided
        let code = if args.code.is_empty() {
            self.company_repository.get_next_code()?
        } else {
            args.code
        };

        // Check if company code already exists
        let code_exists = self.company_repository.code_exists(&code)?;
        if code_exists {
            return Err(DomainError::ValidationError {
                field: "code".to_string(),
                message: "Company code already exists".to_string(),
            });
        }

        // Check if company name already exists
        let name_exists = self.company_repository.name_exists(&args.name)?;
        if name_exists {
            return Err(DomainError::ValidationError {
                field: "name".to_string(),
                message: "Company name already exists".to_string(),
            });
        }

        // Create the company
        let mut company = Company::new(code, args.name, args.created_by)?;

        // Set optional fields
        if let Some(description) = args.description {
            company.update_description(Some(description));
        }
        if let Some(tax_id) = args.tax_id {
            company.update_tax_id(Some(tax_id));
        }
        if let Some(address) = args.address {
            company.update_address(Some(address));
        }
        if let Some(email) = args.email {
            company.update_email(Some(email));
        }
        if let Some(phone) = args.phone {
            company.update_phone(Some(phone));
        }
        if let Some(website) = args.website {
            company.update_website(Some(website));
        }
        if let Some(industry) = args.industry {
            company.update_industry(Some(industry));
        }

        // Save the company
        self.company_repository.save(company.clone())?;

        Ok(company)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::infrastructure::persistence::company_repository::FileCompanyRepository;
    use tempfile::TempDir;

    #[test]
    fn test_create_company_success() {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileCompanyRepository::new(temp_dir.path());
        let use_case = CreateCompanyUseCase::new(repository);

        let args = CreateCompanyArgs {
            code: "TEST-001".to_string(),
            name: "Test Company".to_string(),
            description: Some("A test company".to_string()),
            tax_id: Some("12.345.678/0001-90".to_string()),
            address: None,
            email: Some("test@company.com".to_string()),
            phone: None,
            website: None,
            industry: Some("Technology".to_string()),
            created_by: "test@example.com".to_string(),
        };

        let result = use_case.execute(args);
        assert!(result.is_ok());

        let company = result.unwrap();
        assert_eq!(company.code, "TEST-001");
        assert_eq!(company.name, "Test Company");
        assert_eq!(company.description, Some("A test company".to_string()));
        assert_eq!(company.tax_id, Some("12.345.678/0001-90".to_string()));
        assert_eq!(company.email, Some("test@company.com".to_string()));
        assert_eq!(company.industry, Some("Technology".to_string()));
        assert_eq!(company.created_by, "test@example.com");
    }

    #[test]
    fn test_create_company_duplicate_code() {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileCompanyRepository::new(temp_dir.path());
        let use_case = CreateCompanyUseCase::new(repository);

        let args1 = CreateCompanyArgs {
            code: "DUPLICATE".to_string(),
            name: "First Company".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "test@example.com".to_string(),
        };

        let args2 = CreateCompanyArgs {
            code: "DUPLICATE".to_string(),
            name: "Second Company".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "test@example.com".to_string(),
        };

        // First company should be created successfully
        let result1 = use_case.execute(args1);
        assert!(result1.is_ok());

        // Second company with same code should fail
        let result2 = use_case.execute(args2);
        assert!(result2.is_err());

        if let Err(error) = result2 {
            match error {
                DomainError::ValidationError { field, message } => {
                    assert_eq!(field, "code");
                    assert_eq!(message, "Company code already exists");
                }
                _ => panic!("Expected ValidationError"),
            }
        }
    }

    #[test]
    fn test_create_company_duplicate_name() {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileCompanyRepository::new(temp_dir.path());
        let use_case = CreateCompanyUseCase::new(repository);

        let args1 = CreateCompanyArgs {
            code: "COMP-001".to_string(),
            name: "Same Name".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "test@example.com".to_string(),
        };

        let args2 = CreateCompanyArgs {
            code: "COMP-002".to_string(),
            name: "Same Name".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "test@example.com".to_string(),
        };

        // First company should be created successfully
        let result1 = use_case.execute(args1);
        assert!(result1.is_ok());

        // Second company with same name should fail
        let result2 = use_case.execute(args2);
        assert!(result2.is_err());

        if let Err(error) = result2 {
            match error {
                DomainError::ValidationError { field, message } => {
                    assert_eq!(field, "name");
                    assert_eq!(message, "Company name already exists");
                }
                _ => panic!("Expected ValidationError"),
            }
        }
    }
}
