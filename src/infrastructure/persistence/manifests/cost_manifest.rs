use serde::{Deserialize, Serialize};
use crate::domain::financial::{CostEntry, CostType};
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
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSpec {
    pub amount: f64,
    pub cost_type: String,
    pub date: String,
    pub description: Option<String>,
}

impl From<CostEntry> for CostManifest {
    fn from(cost: CostEntry) -> Self {
        let code = format!("COST-{}", &cost.id[5..13].to_uppercase());
        
        CostManifest {
            api_version: API_VERSION.to_string(),
            kind: "Cost".to_string(),
            metadata: CostMetadata {
                id: cost.id.clone(),
                code,
                resource_code: Some(cost.resource_id),
                task_code: cost.task_id,
                labels: None,
                annotations: None,
                created_at: cost.created_at.to_rfc3339(),
                created_by: cost.created_by,
            },
            spec: CostSpec {
                amount: cost.amount,
                cost_type: cost.cost_type.to_string(),
                date: cost.date.to_string(),
                description: cost.description,
            },
        }
    }
}

impl TryFrom<CostManifest> for CostEntry {
    type Error = String;

    fn try_from(manifest: CostManifest) -> Result<Self, Self::Error> {
        let cost_type = match manifest.spec.cost_type.to_lowercase().as_str() {
            "hourly" => CostType::Hourly,
            "fixed" => CostType::Fixed,
            "material" => CostType::Material,
            "travel" => CostType::Travel,
            _ => return Err(format!("Invalid cost type: {}", manifest.spec.cost_type)),
        };

        let date = manifest.spec.date.parse()
            .map_err(|e| format!("Invalid date: {}", e))?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&manifest.metadata.created_at)
            .map_err(|e| format!("Invalid created_at: {}", e))?
            .with_timezone(&chrono::Utc);

        Ok(CostEntry {
            id: manifest.metadata.id,
            resource_id: manifest.metadata.resource_code.unwrap_or_default(),
            task_id: manifest.metadata.task_code,
            project_id: "".to_string(), // Will be inferred from file path
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
        );

        let manifest = CostManifest::from(cost.clone());
        assert_eq!(manifest.metadata.resource_code, Some("RES-1".to_string()));
        assert!(manifest.metadata.code.starts_with("COST-"));
        assert_eq!(manifest.spec.amount, 100.0);

        let cost_back = CostEntry::try_from(manifest).unwrap();
        assert_eq!(cost_back.resource_id, cost.resource_id);
        assert_eq!(cost_back.amount, cost.amount);
    }
}

