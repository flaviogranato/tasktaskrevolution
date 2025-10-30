use crate::domain::financial::{CostEntry, CostType};
use crate::domain::shared::errors::DomainError;
use std::path::PathBuf;

pub struct AddCostUseCase {
    #[allow(dead_code)]
    base_path: PathBuf,
}

impl AddCostUseCase {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn execute(
        &self,
        resource_id: String,
        task_id: Option<String>,
        project_id: String,
        amount: f64,
        cost_type: CostType,
        description: Option<String>,
        created_by: String,
    ) -> Result<CostEntry, DomainError> {
        // Validate amount
        if amount <= 0.0 {
            return Err(DomainError::ValidationError {
                field: "amount".to_string(),
                message: "Amount must be greater than zero".to_string(),
            });
        }

        // Create cost entry
        let cost = CostEntry::new(
            resource_id,
            task_id,
            project_id,
            amount,
            cost_type,
            description,
            created_by,
        )?;

        // TODO: Save to repository
        // For now, return the created cost entry
        Ok(cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add_cost_success() {
        let use_case = AddCostUseCase::new(PathBuf::from("."));
        let result = use_case.execute(
            "RES-1".to_string(),
            Some("TASK-1".to_string()),
            "PROJ-1".to_string(),
            100.0,
            CostType::Hourly,
            Some("Development work".to_string()),
            "user1".to_string(),
        );

        assert!(result.is_ok());
        let cost = result.unwrap();
        assert_eq!(cost.amount, 100.0);
        assert_eq!(cost.resource_id, "RES-1");
    }

    #[test]
    fn test_add_cost_negative_amount() {
        let use_case = AddCostUseCase::new(PathBuf::from("."));
        let result = use_case.execute(
            "RES-1".to_string(),
            None,
            "PROJ-1".to_string(),
            -50.0,
            CostType::Fixed,
            None,
            "user1".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_add_cost_zero_amount() {
        let use_case = AddCostUseCase::new(PathBuf::from("."));
        let result = use_case.execute(
            "RES-1".to_string(),
            None,
            "PROJ-1".to_string(),
            0.0,
            CostType::Fixed,
            None,
            "user1".to_string(),
        );

        assert!(result.is_err());
    }
}
