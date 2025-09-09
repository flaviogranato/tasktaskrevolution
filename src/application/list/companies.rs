use crate::domain::company_management::{Company, repository::CompanyRepository};
use crate::application::errors::AppError;

pub struct ListCompaniesUseCase<R: CompanyRepository> {
    repository: R,
}

impl<R: CompanyRepository> ListCompaniesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<Company>, AppError> {
        self.repository.find_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_management::Company;

    struct MockCompanyRepository {
        companies: std::sync::RwLock<Vec<Company>>,
    }

    impl MockCompanyRepository {
        fn new(companies: Vec<Company>) -> Self {
            Self {
                companies: std::sync::RwLock::new(companies),
            }
        }
    }

    impl CompanyRepository for MockCompanyRepository {
        fn save(&self, company: Company) -> Result<(), AppError> {
            self.companies.write().unwrap().push(company);
            Ok(())
        }

        fn find_by_id(&self, _id: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn find_by_code(&self, _code: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn find_by_name(&self, _name: &str) -> Result<Option<Company>, AppError> {
            Ok(None)
        }

        fn find_all(&self) -> Result<Vec<Company>, AppError> {
            Ok(self.companies.read().unwrap().clone())
        }

        fn update(&self, _company: Company) -> Result<(), AppError> {
            Ok(())
        }

        fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn get_next_code(&self) -> Result<String, AppError> {
            Ok("COMP-001".to_string())
        }

        fn code_exists(&self, _code: &str) -> Result<bool, AppError> {
            Ok(false)
        }

        fn name_exists(&self, _name: &str) -> Result<bool, AppError> {
            Ok(false)
        }
    }

    fn create_test_company(code: &str, name: &str) -> Company {
        Company::new(
            code.to_string(),
            name.to_string(),
            "test-user".to_string(),
        ).expect("Failed to create test company")
    }

    #[test]
    fn test_list_companies_success() {
        let companies = vec![
            create_test_company("COMP-001", "Company A"),
            create_test_company("COMP-002", "Company B"),
        ];
        let mock_repo = MockCompanyRepository::new(companies);
        let use_case = ListCompaniesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name() == "Company A"));
        assert!(result.iter().any(|c| c.name() == "Company B"));
    }

    #[test]
    fn test_list_companies_empty() {
        let mock_repo = MockCompanyRepository::new(vec![]);
        let use_case = ListCompaniesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert!(result.is_empty());
    }
}
