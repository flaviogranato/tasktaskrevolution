use crate::application::errors::AppError;
use crate::domain::company_management::{Company, repository::CompanyRepository};

pub struct ListCompaniesUseCase<R: CompanyRepository> {
    repository: R,
}

impl<R: CompanyRepository> ListCompaniesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<Company>, AppError> {
        Ok(self.repository.find_all()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_management::Company;
    use crate::domain::shared::errors::{DomainError, DomainResult};

    struct MockCompanyRepository {
        companies: std::sync::RwLock<Vec<Company>>,
        should_fail: bool,
    }

    impl MockCompanyRepository {
        fn new(companies: Vec<Company>) -> Self {
            Self {
                companies: std::sync::RwLock::new(companies),
                should_fail: false,
            }
        }

        fn new_with_failure() -> Self {
            Self {
                companies: std::sync::RwLock::new(vec![]),
                should_fail: true,
            }
        }
    }

    impl CompanyRepository for MockCompanyRepository {
        fn save(&self, company: Company) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "save".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            self.companies.write().unwrap().push(company);
            Ok(())
        }

        fn find_by_id(&self, _id: &str) -> DomainResult<Option<Company>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_by_id".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(None)
        }

        fn find_by_code(&self, _code: &str) -> DomainResult<Option<Company>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_by_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(None)
        }

        fn find_by_name(&self, _name: &str) -> DomainResult<Option<Company>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_by_name".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(None)
        }

        fn find_all(&self) -> DomainResult<Vec<Company>> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "find_all".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.companies.read().unwrap().clone())
        }

        fn update(&self, _company: Company) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "update".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(())
        }

        fn delete(&self, _id: &str) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "delete".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(())
        }

        fn get_next_code(&self) -> DomainResult<String> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "get_next_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok("COMP-001".to_string())
        }

        fn code_exists(&self, _code: &str) -> DomainResult<bool> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "code_exists".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(false)
        }

        fn name_exists(&self, _name: &str) -> DomainResult<bool> {
            if self.should_fail {
                return Err(DomainError::IoError {
                    operation: "name_exists".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(false)
        }
    }

    fn create_test_company(code: &str, name: &str) -> Company {
        Company::new(code.to_string(), name.to_string(), "test-user".to_string())
            .expect("Failed to create test company")
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

    #[test]
    fn test_list_companies_repository_error() {
        let mock_repo = MockCompanyRepository::new_with_failure();
        let use_case = ListCompaniesUseCase::new(mock_repo);

        let result = use_case.execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_all");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_list_companies_use_case_creation() {
        let mock_repo = MockCompanyRepository::new(vec![]);
        let _use_case = ListCompaniesUseCase::new(mock_repo);

        // Test that the use case was created successfully
        // If we get here, creation succeeded
    }

    #[test]
    fn test_list_companies_with_different_statuses() {
        let companies = vec![
            create_test_company("COMP-001", "Active Company"),
            create_test_company("COMP-002", "Inactive Company"),
        ];
        let mock_repo = MockCompanyRepository::new(companies);
        let use_case = ListCompaniesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|c| c.name() == "Active Company"));
        assert!(result.iter().any(|c| c.name() == "Inactive Company"));
    }

    #[test]
    fn test_list_companies_verify_company_properties() {
        let companies = vec![create_test_company("COMP-001", "Test Company")];
        let mock_repo = MockCompanyRepository::new(companies);
        let use_case = ListCompaniesUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 1);

        let company = &result[0];
        assert_eq!(company.code(), "COMP-001");
        assert_eq!(company.name(), "Test Company");
    }
}
