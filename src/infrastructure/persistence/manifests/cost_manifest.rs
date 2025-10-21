use crate::domain::financial::{CostEntry, CostType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: CostMetadata,
    pub spec: CostSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostMetadata {
    pub id: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSpec {
    pub amount: f64,
    pub cost_type: String,
    pub date: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_url: Option<String>,
}

impl From<CostEntry> for CostManifest {
    fn from(cost: CostEntry) -> Self {
        let code = format!("COST-{}", &cost.id[5..13].to_uppercase());

        // Create labels with default categorization
        let mut labels = HashMap::new();
        labels.insert("category".to_string(), cost.cost_type.to_string().to_lowercase());
        labels.insert("phase".to_string(), "Development".to_string());

        CostManifest {
            api_version: API_VERSION.to_string(),
            kind: "Cost".to_string(),
            metadata: CostMetadata {
                id: cost.id.clone(),
                code,
                resource_code: Some(cost.resource_id),
                task_code: cost.task_id,
                labels: Some(labels),
                annotations: None,
                created_at: cost.created_at.to_rfc3339(),
                updated_at: Some(cost.created_at.to_rfc3339()),
                created_by: cost.created_by,
                approved_by: None,
                approved_at: None,
            },
            spec: CostSpec {
                amount: cost.amount,
                cost_type: cost.cost_type.to_string(),
                date: cost.date.to_string(),
                description: cost.description,
                currency: Some("USD".to_string()),
                hours: if matches!(cost.cost_type, crate::domain::financial::CostType::Hourly) {
                    Some(1.0) // Default 1 hour for hourly costs
                } else {
                    None
                },
                category: Some(cost.cost_type.to_string().to_lowercase()),
                status: Some("Pending".to_string()),
                billable: Some(true),
                receipt_url: None,
            },
        }
    }
}

impl TryFrom<CostManifest> for CostEntry {
    type Error = String;

    fn try_from(manifest: CostManifest) -> Result<Self, Self::Error> {
        // Validate amount
        if manifest.spec.amount < 0.0 {
            return Err("Amount cannot be negative".to_string());
        }

        // Parse cost type using FromStr
        let cost_type = manifest.spec.cost_type.parse::<CostType>()
            .map_err(|e| format!("Invalid cost type: {}", e))?;

        // Parse date with better error handling
        let date = chrono::NaiveDate::parse_from_str(&manifest.spec.date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid date format '{}': {}", manifest.spec.date, e))?;

        // Parse created_at with better error handling
        let created_at = chrono::DateTime::parse_from_rfc3339(&manifest.metadata.created_at)
            .map_err(|e| format!("Invalid created_at '{}': {}", manifest.metadata.created_at, e))?
            .with_timezone(&chrono::Utc);

        // Validate required fields
        if manifest.metadata.resource_code.is_none() {
            return Err("Resource code is required".to_string());
        }

        if manifest.metadata.created_by.trim().is_empty() {
            return Err("Created by cannot be empty".to_string());
        }

        Ok(CostEntry {
            id: manifest.metadata.id,
            resource_id: manifest.metadata.resource_code.unwrap(),
            task_id: manifest.metadata.task_code,
            project_id: "".to_string(), // Will be inferred from file path context
            amount: manifest.spec.amount,
            cost_type,
            date,
            description: manifest.spec.description,
            created_at,
            created_by: manifest.metadata.created_by,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_manifest_conversion() {
        let cost = CostEntry::new(
            "RES-1".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            Some("Development".to_string()),
            "user1".to_string(),
        ).unwrap();

        let manifest = CostManifest::from(cost.clone());
        assert_eq!(manifest.metadata.resource_code, Some("RES-1".to_string()));
        assert!(manifest.metadata.code.starts_with("COST-"));
        assert_eq!(manifest.spec.amount, 100.0);

        let cost_back = CostEntry::try_from(manifest).unwrap();
        assert_eq!(cost_back.resource_id, cost.resource_id);
        assert_eq!(cost_back.amount, cost.amount);
    }

    #[test]
    fn test_cost_validation() {
        // Test negative amount
        let result = CostEntry::new(
            "RES-1".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            -100.0,
            CostType::Hourly,
            Some("Development".to_string()),
            "user1".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Amount cannot be negative"));

        // Test empty resource_id
        let result = CostEntry::new(
            "".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            Some("Development".to_string()),
            "user1".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Resource ID cannot be empty"));

        // Test empty created_by
        let result = CostEntry::new(
            "RES-1".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            Some("Development".to_string()),
            "".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Created by cannot be empty"));
    }
}
