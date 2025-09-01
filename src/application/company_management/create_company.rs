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
        Self {
            company_repository,
        }
    }

    /// Executes the company creation use case.
    pub async fn execute(&self, args: CreateCompanyArgs) -> Result<Company, DomainError> {
        // Check if company code already exists
        let code_exists = self.company_repository.code_exists(&args.code).await?;
        if code_exists {
            return Err(DomainError::new(crate::domain::shared::errors::DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: "Company code already exists".to_string(),
            }));
        }

        // Check if company name already exists
        let name_exists = self.company_repository.name_exists(&args.name).await?;
        if name_exists {
            return Err(DomainError::new(crate::domain::shared::errors::DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Company name already exists".to_string(),
            }));
        }

        // Create the company
        let mut company = Company::new(
            args.code,
            args.name,
            args.created_by,
        )?;

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
        self.company_repository.save(company.clone()).await?;

        Ok(company)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_management::Company;
    use crate::domain::shared::errors::DomainError;
    use async_trait::async_trait;
    use mockall::mock;

    mock! {
        CompanyRepository {}

        #[async_trait]
        impl CompanyRepository for CompanyRepository {
            async fn save(&self, company: Company) -> Result<(), DomainError>;
            async fn find_by_id(&self, id: &str) -> Result<Option<Company>, DomainError>;
            async fn find_by_code(&self, code: &str) -> Result<Option<Company>, DomainError>;
            async fn find_by_name(&self, name: &str) -> Result<Option<Company>, DomainError>;
            async fn find_all(&self) -> Result<Vec<Company>, DomainError>;
            async fn update(&self, company: Company) -> Result<(), DomainError>;
            async fn delete(&self, id: &str) -> Result<(), DomainError>;
            async fn get_next_code(&self) -> Result<String, DomainError>;
            async fn code_exists(&self, code: &str) -> Result<bool, DomainError>;
            async fn name_exists(&self, name: &str) -> Result<bool, DomainError>;
        }
    }

    #[tokio::test]
    async fn test_create_company_success() {
        let mut mock_repo = MockCompanyRepository::new();
        
        // Setup expectations
        mock_repo.expect_code_exists()
            .with(mockall::predicate::eq("COMP-001"))
            .times(1)
            .returning(|_| Ok(false));
        
        mock_repo.expect_name_exists()
            .with(mockall::predicate::eq("TechConsulting Ltda"))
            .times(1)
            .returning(|_| Ok(false));
        
        mock_repo.expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let use_case = CreateCompanyUseCase::new(mock_repo);
        
        let args = CreateCompanyArgs {
            code: "COMP-001".to_string(),
            name: "TechConsulting Ltda".to_string(),
            description: Some("Technology consulting company".to_string()),
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "user@example.com".to_string(),
        };

        let result = use_case.execute(args).await;
        assert!(result.is_ok());
        
        let company = result.unwrap();
        assert_eq!(company.code(), "COMP-001");
        assert_eq!(company.name(), "TechConsulting Ltda");
        assert_eq!(company.description, Some("Technology consulting company".to_string()));
    }

    #[tokio::test]
    async fn test_create_company_code_exists() {
        let mut mock_repo = MockCompanyRepository::new();
        
        mock_repo.expect_code_exists()
            .with(mockall::predicate::eq("COMP-001"))
            .times(1)
            .returning(|_| Ok(true));

        let use_case = CreateCompanyUseCase::new(mock_repo);
        
        let args = CreateCompanyArgs {
            code: "COMP-001".to_string(),
            name: "TechConsulting Ltda".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "user@example.com".to_string(),
        };

        let result = use_case.execute(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_company_name_exists() {
        let mut mock_repo = MockCompanyRepository::new();
        
        mock_repo.expect_code_exists()
            .with(mockall::predicate::eq("COMP-001"))
            .times(1)
            .returning(|_| Ok(false));
        
        mock_repo.expect_name_exists()
            .with(mockall::predicate::eq("TechConsulting Ltda"))
            .times(1)
            .returning(|_| Ok(true));

        let use_case = CreateCompanyUseCase::new(mock_repo);
        
        let args = CreateCompanyArgs {
            code: "COMP-001".to_string(),
            name: "TechConsulting Ltda".to_string(),
            description: None,
            tax_id: None,
            address: None,
            email: None,
            phone: None,
            website: None,
            industry: None,
            created_by: "user@example.com".to_string(),
        };

        let result = use_case.execute(args).await;
        assert!(result.is_err());
    }
}
