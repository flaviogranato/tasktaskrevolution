use serde::{Deserialize, Serialize};
use crate::domain::financial::{ProjectBudget, BudgetStatus};
use std::collections::HashMap;

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: BudgetMetadata,
    pub spec: BudgetSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetMetadata {
    pub id: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSpec {
    pub total_budget: f64,
    pub spent_amount: f64,
    pub remaining_amount: f64,
    pub currency: String,
    pub status: String,
}

impl From<ProjectBudget> for BudgetManifest {
    fn from(budget: ProjectBudget) -> Self {
        let id = uuid7::uuid7().to_string();
        let code = format!("BUDGET-{}", &id[0..8].to_uppercase());
        
        BudgetManifest {
            api_version: API_VERSION.to_string(),
            kind: "Budget".to_string(),
            metadata: BudgetMetadata {
                id,
                code,
                labels: None,
                annotations: None,
                created_at: budget.created_at.to_rfc3339(),
                updated_at: budget.updated_at.to_rfc3339(),
                created_by: budget.created_by,
            },
            spec: BudgetSpec {
                total_budget: budget.total_budget,
                spent_amount: budget.spent_amount,
                remaining_amount: budget.remaining_amount,
                currency: budget.currency,
                status: budget.status.to_string(),
            },
        }
    }
}

impl TryFrom<BudgetManifest> for ProjectBudget {
    type Error = String;

    fn try_from(manifest: BudgetManifest) -> Result<Self, Self::Error> {
        let status = match manifest.spec.status.to_uppercase().as_str() {
            "UNDER" => BudgetStatus::Under,
            "ON_TRACK" => BudgetStatus::OnTrack,
            "OVER" => BudgetStatus::Over,
            "EXCEEDED" => BudgetStatus::Exceeded,
            _ => return Err(format!("Invalid budget status: {}", manifest.spec.status)),
        };

        let created_at = chrono::DateTime::parse_from_rfc3339(&manifest.metadata.created_at)
            .map_err(|e| format!("Invalid created_at: {}", e))?
            .with_timezone(&chrono::Utc);

        let updated_at = chrono::DateTime::parse_from_rfc3339(&manifest.metadata.updated_at)
            .map_err(|e| format!("Invalid updated_at: {}", e))?
            .with_timezone(&chrono::Utc);

        let mut budget = ProjectBudget::new(
            "".to_string(), // Will be inferred from file path
            manifest.spec.total_budget,
            manifest.spec.currency,
            manifest.metadata.created_by,
        );

        budget.spent_amount = manifest.spec.spent_amount;
        budget.remaining_amount = manifest.spec.remaining_amount;
        budget.status = status;
        budget.created_at = created_at;
        budget.updated_at = updated_at;
        budget.alerts = budget.check_budget_alerts();

        Ok(budget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_manifest_conversion() {
        let budget = ProjectBudget::new(
            "PROJ-1".to_string(),
            10000.0,
            "USD".to_string(),
            "user1".to_string(),
        );

        let manifest = BudgetManifest::from(budget.clone());
        assert!(!manifest.metadata.id.is_empty());
        assert!(manifest.metadata.code.starts_with("BUDGET-"));
        assert_eq!(manifest.spec.total_budget, 10000.0);
        assert_eq!(manifest.spec.currency, "USD");

        let budget_back = ProjectBudget::try_from(manifest).unwrap();
        assert_eq!(budget_back.total_budget, budget.total_budget);
        assert_eq!(budget_back.currency, budget.currency);
    }
}

