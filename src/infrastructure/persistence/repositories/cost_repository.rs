use crate::domain::financial::CostEntry;
use crate::domain::shared::errors::DomainError;
use crate::infrastructure::persistence::manifests::cost_manifest::CostManifest;
use std::fs;
use std::path::{Path, PathBuf};

pub struct CostRepository {
    base_path: PathBuf,
}

impl CostRepository {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the path to the costs directory for a project
    fn costs_dir(&self, company_code: &str, project_code: &str) -> PathBuf {
        self.base_path
            .join("companies")
            .join(company_code)
            .join("projects")
            .join(project_code)
            .join("costs")
    }

    /// Get the path to a specific cost file
    fn cost_path(&self, company_code: &str, project_code: &str, cost_id: &str) -> PathBuf {
        self.costs_dir(company_code, project_code)
            .join(format!("{}.yaml", cost_id))
    }

    /// Save a cost entry to disk
    pub fn save(
        &self,
        company_code: &str,
        project_code: &str,
        cost: &CostEntry,
    ) -> Result<(), DomainError> {
        let costs_dir = self.costs_dir(company_code, project_code);

        // Ensure costs directory exists
        fs::create_dir_all(&costs_dir).map_err(|e| DomainError::ValidationError {
            field: "costs_dir".to_string(),
            message: format!("Failed to create costs directory: {}", e),
        })?;

        let path = self.cost_path(company_code, project_code, &cost.id);
        let manifest = CostManifest::from(cost.clone());
        let yaml = serde_yaml::to_string(&manifest).map_err(|e| DomainError::ValidationError {
            field: "cost".to_string(),
            message: format!("Failed to serialize cost: {}", e),
        })?;

        fs::write(&path, yaml).map_err(|e| DomainError::ValidationError {
            field: "cost_path".to_string(),
            message: format!("Failed to write cost file: {}", e),
        })?;

        Ok(())
    }

    /// Load a specific cost entry
    pub fn load(
        &self,
        company_code: &str,
        project_code: &str,
        cost_id: &str,
    ) -> Result<CostEntry, DomainError> {
        let path = self.cost_path(company_code, project_code, cost_id);

        if !path.exists() {
            return Err(DomainError::EntityNotFound {
                entity_type: "Cost".to_string(),
                identifier: cost_id.to_string(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| DomainError::ValidationError {
            field: "cost_path".to_string(),
            message: format!("Failed to read cost file: {}", e),
        })?;

        let manifest: CostManifest = serde_yaml::from_str(&content).map_err(|e| {
            DomainError::ValidationError {
                field: "cost".to_string(),
                message: format!("Failed to parse cost YAML: {}", e),
            }
        })?;

        let mut cost = CostEntry::try_from(manifest).map_err(|e| DomainError::ValidationError {
            field: "cost".to_string(),
            message: e,
        })?;

        // Set project_id from path
        cost.project_id = project_code.to_string();

        Ok(cost)
    }

    /// List all costs for a project
    pub fn list(
        &self,
        company_code: &str,
        project_code: &str,
    ) -> Result<Vec<CostEntry>, DomainError> {
        let costs_dir = self.costs_dir(company_code, project_code);

        if !costs_dir.exists() {
            return Ok(Vec::new());
        }

        let mut costs = Vec::new();

        for entry in fs::read_dir(&costs_dir).map_err(|e| DomainError::ValidationError {
            field: "costs_dir".to_string(),
            message: format!("Failed to read costs directory: {}", e),
        })? {
            let entry = entry.map_err(|e| DomainError::ValidationError {
                field: "costs_dir".to_string(),
                message: format!("Failed to read directory entry: {}", e),
            })?;

            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.ends_with(".yaml") {
                    let cost_id = file_name.trim_end_matches(".yaml");
                    if let Ok(cost) = self.load(company_code, project_code, cost_id) {
                        costs.push(cost);
                    }
                }
            }
        }

        Ok(costs)
    }

    /// Delete a cost entry
    pub fn delete(
        &self,
        company_code: &str,
        project_code: &str,
        cost_id: &str,
    ) -> Result<(), DomainError> {
        let path = self.cost_path(company_code, project_code, cost_id);

        if !path.exists() {
            return Err(DomainError::EntityNotFound {
                entity_type: "Cost".to_string(),
                identifier: cost_id.to_string(),
            });
        }

        fs::remove_file(&path).map_err(|e| DomainError::ValidationError {
            field: "cost_path".to_string(),
            message: format!("Failed to delete cost file: {}", e),
        })?;

        Ok(())
    }

    /// List all costs for a company (across all projects)
    pub fn list_by_company(&self, company_code: &str) -> Result<Vec<CostEntry>, DomainError> {
        let projects_dir = self.base_path
            .join("companies")
            .join(company_code)
            .join("projects");

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut all_costs = Vec::new();

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
                if let Ok(costs) = self.list(company_code, &project_code) {
                    all_costs.extend(costs);
                }
            }
        }

        Ok(all_costs)
    }

    /// List costs by resource
    pub fn list_by_resource(
        &self,
        company_code: &str,
        project_code: &str,
        resource_id: &str,
    ) -> Result<Vec<CostEntry>, DomainError> {
        let all_costs = self.list(company_code, project_code)?;
        Ok(all_costs
            .into_iter()
            .filter(|c| c.resource_id == resource_id)
            .collect())
    }

    /// List costs by task
    pub fn list_by_task(
        &self,
        company_code: &str,
        project_code: &str,
        task_id: &str,
    ) -> Result<Vec<CostEntry>, DomainError> {
        let all_costs = self.list(company_code, project_code)?;
        Ok(all_costs
            .into_iter()
            .filter(|c| c.task_id.as_ref().map(|t| t == task_id).unwrap_or(false))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::financial::CostType;
    use tempfile::TempDir;

    #[test]
    fn test_cost_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo = CostRepository::new(temp_dir.path());

        let cost = CostEntry::new(
            "RES-1".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            Some("Development".to_string()),
            "user1".to_string(),
        );

        let cost_id = cost.id.clone();
        repo.save("COMP-1", "PROJ-1", &cost).unwrap();

        let loaded = repo.load("COMP-1", "PROJ-1", &cost_id).unwrap();
        assert_eq!(loaded.amount, 100.0);
        assert_eq!(loaded.resource_id, "RES-1");
    }

    #[test]
    fn test_cost_list() {
        let temp_dir = TempDir::new().unwrap();
        let repo = CostRepository::new(temp_dir.path());

        let cost1 = CostEntry::new(
            "RES-1".to_string(),
            None,
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            None,
            "user1".to_string(),
        );

        repo.save("COMP-1", "PROJ-1", &cost1).unwrap();
        
        // Ensure unique timestamp-based IDs by adding small delay
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Create a new cost with guaranteed different timestamp
        let cost2 = CostEntry::new(
            "RES-2".to_string(),
            None,
            "PROJ-1".to_string(),
            200.0,
            CostType::Fixed,
            None,
            "user1".to_string(),
        );
        
        repo.save("COMP-1", "PROJ-1", &cost2).unwrap();

        let costs = repo.list("COMP-1", "PROJ-1").unwrap();
        assert_eq!(costs.len(), 2);
    }
}

