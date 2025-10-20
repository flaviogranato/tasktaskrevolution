use crate::domain::financial::ProjectBudget;
use crate::domain::shared::errors::DomainError;
use crate::infrastructure::persistence::manifests::budget_manifest::BudgetManifest;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BudgetRepository {
    base_path: PathBuf,
}

impl BudgetRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the path to the budget file for a project
    fn budget_path(&self, company_code: &str, project_code: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("budget.yaml")
    }

    /// Save a budget to disk
    pub fn save(
        &self,
        company_code: &str,
        project_code: &str,
        budget: &ProjectBudget,
    ) -> Result<(), DomainError> {
        let path = self.budget_path(company_code, project_code);

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| DomainError::ValidationError {
                field: "budget_path".to_string(),
                message: format!("Failed to create budget directory: {}", e),
            })?;
        }

        let manifest = BudgetManifest::from(budget.clone());
        let yaml = serde_yaml::to_string(&manifest).map_err(|e| DomainError::ValidationError {
            field: "budget".to_string(),
            message: format!("Failed to serialize budget: {}", e),
        })?;

        fs::write(&path, yaml).map_err(|e| DomainError::ValidationError {
            field: "budget_path".to_string(),
            message: format!("Failed to write budget file: {}", e),
        })?;

        Ok(())
    }

    /// Load a budget from disk
    pub fn load(
        &self,
        company_code: &str,
        project_code: &str,
    ) -> Result<ProjectBudget, DomainError> {
        let path = self.budget_path(company_code, project_code);

        if !path.exists() {
            return Err(DomainError::EntityNotFound {
                entity_type: "Budget".to_string(),
                identifier: format!("{}/{}", company_code, project_code),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| DomainError::ValidationError {
            field: "budget_path".to_string(),
            message: format!("Failed to read budget file: {}", e),
        })?;

        let manifest: BudgetManifest = serde_yaml::from_str(&content).map_err(|e| {
            DomainError::ValidationError {
                field: "budget".to_string(),
                message: format!("Failed to parse budget YAML: {}", e),
            }
        })?;

        let mut budget = ProjectBudget::try_from(manifest)
            .map_err(|e| DomainError::ValidationError {
                field: "budget".to_string(),
                message: e,
            })?;

        // Set project_id from path
        budget.project_id = project_code.to_string();

        Ok(budget)
    }

    /// Check if a budget exists
    pub fn exists(&self, company_code: &str, project_code: &str) -> bool {
        self.budget_path(company_code, project_code).exists()
    }

    /// Delete a budget
    pub fn delete(&self, company_code: &str, project_code: &str) -> Result<(), DomainError> {
        let path = self.budget_path(company_code, project_code);

        if !path.exists() {
            return Err(DomainError::EntityNotFound {
                entity_type: "Budget".to_string(),
                identifier: format!("{}/{}", company_code, project_code),
            });
        }

        fs::remove_file(&path).map_err(|e| DomainError::ValidationError {
            field: "budget_path".to_string(),
            message: format!("Failed to delete budget file: {}", e),
        })?;

        Ok(())
    }

    /// List all budgets for a company
    pub fn list_by_company(&self, company_code: &str) -> Result<Vec<ProjectBudget>, DomainError> {
        let projects_dir = self.base_path
            .join("companies")
            .join(company_code)
            .join("projects");

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut budgets = Vec::new();

        for entry in fs::read_dir(&projects_dir).map_err(|e| DomainError::ValidationError {
            field: "projects_dir".to_string(),
            message: format!("Failed to read projects directory: {}", e),
        })? {
            let entry = entry.map_err(|e| DomainError::ValidationError {
                field: "projects_dir".to_string(),
                message: format!("Failed to read directory entry: {}", e),
            })?;

            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let project_code = entry.file_name().to_string_lossy().to_string();
                if let Ok(budget) = self.load(company_code, &project_code) {
                    budgets.push(budget);
                }
            }
        }

        Ok(budgets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_budget_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo = BudgetRepository::new(temp_dir.path());

        let budget = ProjectBudget::new(
            "PROJ-1".to_string(),
            10000.0,
            "USD".to_string(),
            "user1".to_string(),
        );

        repo.save("COMP-1", "PROJ-1", &budget).unwrap();
        assert!(repo.exists("COMP-1", "PROJ-1"));

        let loaded = repo.load("COMP-1", "PROJ-1").unwrap();
        assert_eq!(loaded.total_budget, 10000.0);
        assert_eq!(loaded.currency, "USD");
    }

    #[test]
    fn test_budget_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let repo = BudgetRepository::new(temp_dir.path());

        let result = repo.load("COMP-1", "PROJ-1");
        assert!(result.is_err());
    }
}

