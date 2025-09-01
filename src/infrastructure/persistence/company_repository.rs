use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::company_management::{Company, CompanyRepository};
use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use crate::infrastructure::persistence::manifests::company_manifest::CompanyManifest;

/// File-based implementation of CompanyRepository.
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
        self.base_path.join("companies").join(format!("{}.yaml", code))
    }

    fn get_companies_dir(&self) -> PathBuf {
        self.base_path.join("companies")
    }

    async fn load_companies_from_disk(&self) -> Result<(), DomainError> {
        let companies_dir = self.get_companies_dir();
        
        if !companies_dir.exists() {
            fs::create_dir_all(&companies_dir).map_err(|e| {
                DomainError::new(
                    DomainErrorKind::RepositoryError {
                        operation: "create_companies_directory".to_string(),
                        details: format!("Failed to create companies directory: {}", e),
                    }
                )
            })?;
            return Ok(());
        }

        let mut companies = HashMap::new();
        
        for entry in fs::read_dir(&companies_dir).map_err(|e| {
            DomainError::new(
                DomainErrorKind::RepositoryError {
                    operation: "read_companies_directory".to_string(),
                    details: format!("Failed to read companies directory: {}", e),
                }
            )
        })? {
            let entry = entry.map_err(|e| {
                DomainError::new(
                    DomainErrorKind::RepositoryError {
                        operation: "read_directory_entry".to_string(),
                        details: format!("Failed to read directory entry: {}", e),
                    }
                )
            })?;
            
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(&path).map_err(|e| {
                    DomainError::new(
                        DomainErrorKind::RepositoryError {
                            operation: "read_company_file".to_string(),
                            details: format!("Failed to read company file {}: {}", path.display(), e),
                        }
                    )
                })?;
                
                let manifest: CompanyManifest = serde_yaml::from_str(&content).map_err(|e| {
                    DomainError::new(
                        DomainErrorKind::Serialization {
                            format: "YAML".to_string(),
                            details: format!("Failed to parse company file {}: {}", path.display(), e),
                        }
                    )
                })?;
                
                let company = manifest.to();
                companies.insert(company.code.clone(), company);
            }
        }

        let mut companies_lock = self.companies.write().await;
        *companies_lock = companies;
        
        Ok(())
    }

    async fn save_company_to_disk(&self, company: &Company) -> Result<(), DomainError> {
        let companies_dir = self.get_companies_dir();
        
        if !companies_dir.exists() {
            fs::create_dir_all(&companies_dir).map_err(|e| {
                DomainError::new(
                    DomainErrorKind::RepositoryError {
                        operation: "create_companies_directory".to_string(),
                        details: format!("Failed to create companies directory: {}", e),
                    }
                )
            })?;
        }

        let manifest = CompanyManifest::from(company);
        let yaml_content = serde_yaml::to_string(&manifest).map_err(|e| {
            DomainError::new(
                DomainErrorKind::Serialization {
                    format: "YAML".to_string(),
                    details: format!("Failed to serialize company to YAML: {}", e),
                }
            )
        })?;

        let file_path = self.get_company_path(&company.code);
        fs::write(&file_path, yaml_content).map_err(|e| {
            DomainError::new(
                DomainErrorKind::RepositoryError {
                    operation: "write_company_file".to_string(),
                    details: format!("Failed to write company file {}: {}", file_path.display(), e),
                }
            )
        })?;

        Ok(())
    }

    async fn delete_company_from_disk(&self, code: &str) -> Result<(), DomainError> {
        let file_path = self.get_company_path(code);
        
        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                DomainError::new(
                    DomainErrorKind::RepositoryError {
                        operation: "delete_company_file".to_string(),
                        details: format!("Failed to delete company file {}: {}", file_path.display(), e),
                    }
                )
            })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl CompanyRepository for FileCompanyRepository {
    async fn save(&self, company: Company) -> Result<(), DomainError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk().await?;
        
        let company_code = company.code.clone();
        
        // Save to disk
        self.save_company_to_disk(&company).await?;
        
        // Update in-memory cache
        let mut companies = self.companies.write().await;
        companies.insert(company_code.clone(), company);
        
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<Company>, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        let company = companies.values().find(|c| c.id == id).cloned();
        
        Ok(company)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Company>, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        let company = companies.get(code).cloned();
        
        Ok(company)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Company>, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        let company = companies.values().find(|c| c.name == name).cloned();
        
        Ok(company)
    }

    async fn find_all(&self) -> Result<Vec<Company>, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        let companies_vec: Vec<Company> = companies.values().cloned().collect();
        
        Ok(companies_vec)
    }

    async fn update(&self, company: Company) -> Result<(), DomainError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk().await?;
        
        let company_code = company.code.clone();
        
        // Save updated company to disk
        self.save_company_to_disk(&company).await?;
        
        // Update in-memory cache
        let mut companies = self.companies.write().await;
        companies.insert(company_code.clone(), company);
        
        Ok(())
    }

    async fn delete(&self, code: &str) -> Result<(), DomainError> {
        // Load companies from disk first to ensure consistency
        self.load_companies_from_disk().await?;
        
        // Remove from disk
        self.delete_company_from_disk(code).await?;
        
        // Remove from in-memory cache
        let mut companies = self.companies.write().await;
        companies.remove(code);
        
        Ok(())
    }

    async fn get_next_code(&self) -> Result<String, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
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

    async fn code_exists(&self, code: &str) -> Result<bool, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        Ok(companies.contains_key(code))
    }

    async fn name_exists(&self, name: &str) -> Result<bool, DomainError> {
        self.load_companies_from_disk().await?;
        
        let companies = self.companies.read().await;
        Ok(companies.values().any(|c| c.name == name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::domain::company_management::company::{Company, CompanySize, CompanyStatus};

    async fn create_test_company(code: &str, name: &str) -> Company {
        Company::new(
            code.to_string(),
            name.to_string(),
            "test@example.com".to_string(),
        ).unwrap()
    }

    #[tokio::test]
    async fn test_save_and_find_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        let company = create_test_company("TEST-001", "Test Company").await;
        
        // Save company
        let result = repo.save(company.clone()).await;
        assert!(result.is_ok());
        
        // Find by code
        let found_company = repo.find_by_code("TEST-001").await.unwrap().unwrap();
        assert_eq!(found_company.name, "Test Company");
        
        // Find by name
        let found_by_name = repo.find_by_name("Test Company").await.unwrap().unwrap();
        assert_eq!(found_by_name.code, "TEST-001");
    }

    #[tokio::test]
    async fn test_update_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        let mut company = create_test_company("TEST-002", "Original Name").await;
        
        // Save original company
        repo.save(company.clone()).await.unwrap();
        
        // Update company
        company = company.update_name("Updated Name").unwrap();
        let result = repo.update(company.clone()).await;
        assert!(result.is_ok());
        
        // Verify update persisted
        let found_company = repo.find_by_code("TEST-002").await.unwrap().unwrap();
        assert_eq!(found_company.name, "Updated Name");
    }

    #[tokio::test]
    async fn test_delete_company() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        let company = create_test_company("TEST-003", "To Delete").await;
        
        // Save company
        repo.save(company.clone()).await.unwrap();
        
        // Verify it exists
        assert!(repo.find_by_code("TEST-003").await.unwrap().is_some());
        
        // Delete company
        repo.delete("TEST-003").await.unwrap();
        
        // Verify it's gone
        assert!(repo.find_by_code("TEST-003").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_next_code() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        // First code should be company-1
        let next_code = repo.get_next_code().await.unwrap();
        assert_eq!(next_code, "company-1");
        
        // Create a company with company-1
        let company = create_test_company("company-1", "First Company").await;
        repo.save(company).await.unwrap();
        
        // Next code should be company-2
        let next_code = repo.get_next_code().await.unwrap();
        assert_eq!(next_code, "company-2");
    }

    #[tokio::test]
    async fn test_code_and_name_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        let company = create_test_company("TEST-004", "Unique Name").await;
        
        // Initially, code and name don't exist
        assert!(!repo.code_exists("TEST-004").await.unwrap());
        assert!(!repo.name_exists("Unique Name").await.unwrap());
        
        // Save company
        repo.save(company).await.unwrap();
        
        // Now they exist
        assert!(repo.code_exists("TEST-004").await.unwrap());
        assert!(repo.name_exists("Unique Name").await.unwrap());
    }

    #[tokio::test]
    async fn test_find_all_companies() {
        let temp_dir = TempDir::new().unwrap();
        let repo = FileCompanyRepository::new(temp_dir.path());
        
        // Create multiple companies
        let company1 = create_test_company("COMP-001", "Company 1").await;
        let company2 = create_test_company("COMP-002", "Company 2").await;
        
        repo.save(company1).await.unwrap();
        repo.save(company2).await.unwrap();
        
        // Find all companies
        let all_companies = repo.find_all().await.unwrap();
        assert_eq!(all_companies.len(), 2);
        
        let codes: Vec<String> = all_companies.iter().map(|c| c.code.clone()).collect();
        assert!(codes.contains(&"COMP-001".to_string()));
        assert!(codes.contains(&"COMP-002".to_string()));
    }
}
