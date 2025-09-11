use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::application::errors::AppError;
use crate::domain::company_management::{Company, CompanyRepository};
use crate::infrastructure::persistence::manifests::company_manifest::CompanyManifest;

/// File-based implementation of CompanyRepository.
#[derive(Clone)]
pub struct FileCompanyRepository {
    base_path: PathBuf,
    companies: Arc<RwLock<HashMap<String, Company>>>,
}

impl FileCompanyRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        let base_path = base_path.as_ref().to_path_buf();
        let companies = Arc::new(RwLock::new(HashMap::new()));

        Self { base_path, companies }
    }

    fn get_company_path(&self, code: &str) -> PathBuf {
        self.base_path.join("companies").join(code).join("company.yaml")
    }

    fn get_companies_dir(&self) -> PathBuf {
        self.base_path.join("companies")
    }

    fn get_company_dir(&self, code: &str) -> PathBuf {
        self.base_path.join("companies").join(code)
    }

    fn load_companies_from_disk(&self) -> Result<(), AppError> {
        let companies_dir = self.get_companies_dir();

        if !companies_dir.exists() {
            fs::create_dir_all(&companies_dir).map_err(|e| AppError::IoError {
                operation: "create directory".to_string(),
                details: e.to_string(),
            })?;
            return Ok(());
        }

        let mut companies = HashMap::new();

        for entry in fs::read_dir(&companies_dir).map_err(|e| AppError::IoErrorWithPath {
            operation: "read directory".to_string(),
            path: companies_dir.to_string_lossy().to_string(),
            details: e.to_string(),
        })? {
            let entry = entry.map_err(|e| AppError::IoError {
                operation: "read directory entry".to_string(),
                details: e.to_string(),
            })?;

            let path = entry.path();
            if path.is_dir() {
                let company_yaml_path = path.join("company.yaml");
                if company_yaml_path.exists() {
                    let content = fs::read_to_string(&company_yaml_path).map_err(|e| AppError::IoErrorWithPath {
                        operation: "file read".to_string(),
                        path: company_yaml_path.to_string_lossy().to_string(),
                        details: e.to_string(),
                    })?;

                    let manifest: CompanyManifest =
                        serde_yaml::from_str(&content).map_err(|e| AppError::SerializationError {
                            format: "YAML".to_string(),
                            details: format!("Failed to parse company file {}: {}", company_yaml_path.display(), e),
                        })?;

                    let company = manifest.to();
                    companies.insert(company.code.clone(), company);
                }
            }
        }

        let mut companies_lock = self.companies.write().unwrap();
        *companies_lock = companies;

        Ok(())
    }

    fn save_company_to_disk(&self, company: &Company) -> Result<(), AppError> {
        let companies_dir = self.get_companies_dir();
        let company_dir = self.get_company_dir(&company.code);

        // Create companies directory if it doesn't exist
        if !companies_dir.exists() {
            fs::create_dir_all(&companies_dir).map_err(|e| AppError::IoError {
                operation: "create directory".to_string(),
                details: e.to_string(),
            })?;
        }

        // Create company directory if it doesn't exist
        if !company_dir.exists() {
            fs::create_dir_all(&company_dir).map_err(|e| AppError::IoErrorWithPath {
                operation: "create directory".to_string(),
                path: company_dir.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;
        }

        let manifest = CompanyManifest::from(company);
        let yaml_content = serde_yaml::to_string(&manifest).map_err(|e| AppError::SerializationError {
            format: "YAML".to_string(),
            details: format!("Failed to serialize company to YAML: {}", e),
        })?;

        let file_path = self.get_company_path(&company.code);
        fs::write(&file_path, yaml_content).map_err(|e| AppError::IoErrorWithPath {
            operation: "file write".to_string(),
            path: file_path.to_string_lossy().to_string(),
            details: e.to_string(),
        })?;

        Ok(())
    }

    #[allow(dead_code)]
    fn delete_company_from_disk(&self, code: &str) -> Result<(), AppError> {
        let company_dir = self.get_company_dir(code);

        if company_dir.exists() {
            fs::remove_dir_all(&company_dir).map_err(|e| AppError::IoErrorWithPath {
                operation: "delete directory".to_string(),
                path: company_dir.to_string_lossy().to_string(),
                details: e.to_string(),
            })?;
        }

        Ok(())
    }
}

impl CompanyRepository for FileCompanyRepository {
    fn save(&self, company: Company) -> Result<(), AppError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk()?;

        let company_code = company.code.clone();

        // Save to disk
        self.save_company_to_disk(&company)?;

        // Update in-memory cache
        let mut companies = self.companies.write().unwrap();
        companies.insert(company_code.clone(), company);

        Ok(())
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Company>, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        let company = companies.values().find(|c| c.id == id).cloned();

        Ok(company)
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Company>, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        let company = companies.get(code).cloned();

        Ok(company)
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Company>, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        let company = companies.values().find(|c| c.name == name).cloned();

        Ok(company)
    }

    fn find_all(&self) -> Result<Vec<Company>, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        let companies_vec: Vec<Company> = companies.values().cloned().collect();

        Ok(companies_vec)
    }

    fn update(&self, company: Company) -> Result<(), AppError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk()?;

        let company_code = company.code.clone();

        // Save updated company to disk
        self.save_company_to_disk(&company)?;

        // Update in-memory cache
        let mut companies = self.companies.write().unwrap();
        companies.insert(company_code.clone(), company);

        Ok(())
    }

    fn delete(&self, code: &str) -> Result<(), AppError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk()?;

        // Remove from disk
        self.delete_company_from_disk(code)?;

        // Remove from in-memory cache
        let mut companies = self.companies.write().unwrap();
        companies.remove(code);

        Ok(())
    }

    fn get_next_code(&self) -> Result<String, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        let existing_codes: Vec<&String> = companies.keys().collect();

        let mut counter = 1;
        loop {
            let new_code = format!("company-{}", counter);
            if !existing_codes.contains(&&new_code) {
                return Ok(new_code);
            }
            counter += 1;
        }
    }

    fn code_exists(&self, code: &str) -> Result<bool, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        Ok(companies.contains_key(code))
    }

    fn name_exists(&self, name: &str) -> Result<bool, AppError> {
        self.load_companies_from_disk()?;

        let companies = self.companies.read().unwrap();
        Ok(companies.values().any(|c| c.name == name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_management::company::Company;
    use tempfile::TempDir;

    fn create_test_company(code: &str, name: &str) -> Company {
        Company::new(code.to_string(), name.to_string(), "test@example.com".to_string()).unwrap()
    }

    #[test]
    fn test_save_and_find_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        let company = create_test_company("TEST-001", "Test Company");

        // Save company
        let result = repo.save(company.clone());
        assert!(result.is_ok());

        // Find by code
        let found_company = repo.find_by_code("TEST-001").unwrap().unwrap();
        assert_eq!(found_company.name, "Test Company");

        // Find by name
        let found_by_name = repo.find_by_name("Test Company").unwrap().unwrap();
        assert_eq!(found_by_name.code, "TEST-001");
    }

    #[test]
    fn test_update_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        let mut company = create_test_company("TEST-002", "Original Name");

        // Save original company
        repo.save(company.clone()).unwrap();

        // Update company
        company.update_name("Updated Name".to_string()).unwrap();
        let result = repo.update(company.clone());
        assert!(result.is_ok());

        // Verify update persisted
        let found_company = repo.find_by_code("TEST-002").unwrap().unwrap();
        assert_eq!(found_company.name, "Updated Name");
    }

    #[test]
    fn test_delete_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        let company = create_test_company("TEST-003", "To Delete");

        // Save company
        repo.save(company.clone()).unwrap();

        // Verify it exists
        assert!(repo.find_by_code("TEST-003").unwrap().is_some());

        // Delete company
        repo.delete("TEST-003").unwrap();

        // Verify it's gone
        assert!(repo.find_by_code("TEST-003").unwrap().is_none());
    }

    #[test]
    fn test_get_next_code() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        // First code should be company-1
        let next_code = repo.get_next_code().unwrap();
        assert_eq!(next_code, "company-1");

        // Create a company with company-1
        let company = create_test_company("company-1", "First Company");
        repo.save(company).unwrap();

        // Next code should be company-2
        let next_code = repo.get_next_code().unwrap();
        assert_eq!(next_code, "company-2");
    }

    #[test]
    fn test_code_and_name_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        let company = create_test_company("TEST-004", "Unique Name");

        // Initially, code and name don't exist
        assert!(!repo.code_exists("TEST-004").unwrap());
        assert!(!repo.name_exists("Unique Name").unwrap());

        // Save company
        repo.save(company).unwrap();

        // Now they exist
        assert!(repo.code_exists("TEST-004").unwrap());
        assert!(repo.name_exists("Unique Name").unwrap());
    }

    #[test]
    fn test_find_all_companies() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());

        // Create multiple companies
        let company1 = create_test_company("COMP-001", "Company 1");
        let company2 = create_test_company("COMP-002", "Company 2");

        repo.save(company1).unwrap();
        repo.save(company2).unwrap();

        // Find all companies
        let all_companies = repo.find_all().unwrap();
        assert_eq!(all_companies.len(), 2);

        let codes: Vec<String> = all_companies.iter().map(|c| c.code.clone()).collect();
        assert!(codes.contains(&"COMP-001".to_string()));
        assert!(codes.contains(&"COMP-002".to_string()));
    }
}
