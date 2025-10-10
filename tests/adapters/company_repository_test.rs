use assert_fs::TempDir;

use task_task_revolution::domain::company_management::{Company, CompanyRepository};
use task_task_revolution::infrastructure::persistence::company_repository::FileCompanyRepository;

/// Test fixtures for CompanyRepository tests
struct CompanyRepositoryTestFixture {
    #[allow(dead_code)]
    temp_dir: TempDir,
    repository: FileCompanyRepository,
}

impl CompanyRepositoryTestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileCompanyRepository::new(temp_dir.path());

        Self { temp_dir, repository }
    }

    fn create_test_company(&self, code: &str, name: &str) -> Company {
        Company::new(code.to_string(), name.to_string(), "test@example.com".to_string()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find_by_code() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-001", "Test Company");

        // Save the company
        fixture.repository.save(company.clone()).unwrap();

        // Find by code
        let found = fixture.repository.find_by_code("TEST-001").unwrap();
        assert!(found.is_some());
        let found_company = found.unwrap();
        assert_eq!(found_company.code(), "TEST-001");
        assert_eq!(found_company.name(), "Test Company");
    }

    #[test]
    fn test_save_and_find_by_id() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-002", "Test Company 2");

        // Save the company
        fixture.repository.save(company.clone()).unwrap();

        // Find by ID
        let found = fixture.repository.find_by_id(company.id()).unwrap();
        assert!(found.is_some());
        let found_company = found.unwrap();
        assert_eq!(found_company.id(), company.id());
        assert_eq!(found_company.code(), "TEST-002");
    }

    #[test]
    fn test_find_all() {
        let fixture = CompanyRepositoryTestFixture::new();

        // Create multiple companies
        let company1 = fixture.create_test_company("TEST-001", "Test Company 1");
        let company2 = fixture.create_test_company("TEST-002", "Test Company 2");

        // Save companies
        fixture.repository.save(company1).unwrap();
        fixture.repository.save(company2).unwrap();

        // Find all
        let all_companies = fixture.repository.find_all().unwrap();
        assert_eq!(all_companies.len(), 2);

        let codes: Vec<&str> = all_companies.iter().map(|c| c.code()).collect();
        assert!(codes.contains(&"TEST-001"));
        assert!(codes.contains(&"TEST-002"));
    }

    #[test]
    fn test_find_by_name() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-003", "Unique Company Name");

        // Save the company
        fixture.repository.save(company).unwrap();

        // Find by name
        let found = fixture.repository.find_by_name("Unique Company Name").unwrap();
        assert!(found.is_some());
        let found_company = found.unwrap();
        assert_eq!(found_company.name(), "Unique Company Name");
    }

    #[test]
    fn test_update_company() {
        let fixture = CompanyRepositoryTestFixture::new();
        let mut company = fixture.create_test_company("TEST-004", "Original Name");

        // Save the company
        fixture.repository.save(company.clone()).unwrap();

        // Update the company
        company.update_name("Updated Name".to_string()).unwrap();
        fixture.repository.update(company.clone()).unwrap();

        // Verify the update
        let found = fixture.repository.find_by_code("TEST-004").unwrap();
        assert!(found.is_some());
        let found_company = found.unwrap();
        assert_eq!(found_company.name(), "Updated Name");
    }

    #[test]
    fn test_delete_company() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-005", "To Be Deleted");

        // Save the company
        fixture.repository.save(company.clone()).unwrap();

        // Verify it exists
        let found = fixture.repository.find_by_code("TEST-005").unwrap();
        assert!(found.is_some());

        // Delete the company
        fixture.repository.delete(company.code()).unwrap();

        // Verify it's deleted (hard delete - company should not exist)
        let found = fixture.repository.find_by_code("TEST-005").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_code_exists() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-006", "Code Exists Test");

        // Initially, code should not exist
        assert!(!fixture.repository.code_exists("TEST-006").unwrap());

        // Save the company
        fixture.repository.save(company).unwrap();

        // Now code should exist
        assert!(fixture.repository.code_exists("TEST-006").unwrap());
    }

    #[test]
    fn test_name_exists() {
        let fixture = CompanyRepositoryTestFixture::new();
        let company = fixture.create_test_company("TEST-007", "Name Exists Test");

        // Initially, name should not exist
        assert!(!fixture.repository.name_exists("Name Exists Test").unwrap());

        // Save the company
        fixture.repository.save(company).unwrap();

        // Now name should exist
        assert!(fixture.repository.name_exists("Name Exists Test").unwrap());
    }

    #[test]
    fn test_get_next_code() {
        let fixture = CompanyRepositoryTestFixture::new();

        // Get next code
        let next_code = fixture.repository.get_next_code().unwrap();
        assert!(!next_code.is_empty());
        assert!(next_code.starts_with("company-"));
    }

    #[test]
    fn test_find_nonexistent_company() {
        let fixture = CompanyRepositoryTestFixture::new();

        // Try to find a company that doesn't exist
        let found = fixture.repository.find_by_code("NONEXISTENT").unwrap();
        assert!(found.is_none());

        let found_by_id = fixture.repository.find_by_id("nonexistent-id").unwrap();
        assert!(found_by_id.is_none());

        let found_by_name = fixture.repository.find_by_name("Nonexistent Company").unwrap();
        assert!(found_by_name.is_none());
    }

    #[test]
    fn test_company_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create first repository instance and save a company
        {
            let repository = FileCompanyRepository::new(temp_path);
            let company = Company::new(
                "PERSIST-001".to_string(),
                "Persistent Company".to_string(),
                "persist@example.com".to_string(),
            )
            .unwrap();
            repository.save(company).unwrap();
        }

        // Create second repository instance and verify the company exists
        {
            let repository = FileCompanyRepository::new(temp_path);
            let found = repository.find_by_code("PERSIST-001").unwrap();
            assert!(found.is_some());
            let found_company = found.unwrap();
            assert_eq!(found_company.name(), "Persistent Company");
        }
    }
}
