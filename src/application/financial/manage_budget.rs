use crate::domain::financial::{BudgetStatus, ProjectBudget};
use crate::domain::shared::errors::DomainError;
use std::path::PathBuf;

pub struct ManageBudgetUseCase {
    base_path: PathBuf,
}

impl ManageBudgetUseCase {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn set_budget(
        &self,
        project_id: String,
        amount: f64,
        currency: String,
        created_by: String,
    ) -> Result<ProjectBudget, DomainError> {
        // Validate amount
        if amount <= 0.0 {
            return Err(DomainError::ValidationError {
                field: "amount".to_string(),
                message: "Budget amount must be greater than zero".to_string(),
            });
        }

        // Create budget
        let budget = ProjectBudget::new(project_id, amount, currency, created_by);

        // TODO: Save to repository
        Ok(budget)
    }

    pub fn get_budget(&self, project_id: &str) -> Result<ProjectBudget, DomainError> {
        // TODO: Load from repository
        Err(DomainError::EntityNotFound {
            entity_type: "Budget".to_string(),
            identifier: project_id.to_string(),
        })
    }

    pub fn update_spending(&self, project_id: &str, spent_amount: f64) -> Result<ProjectBudget, DomainError> {
        // Load budget
        let mut budget = self.get_budget(project_id)?;

        // Update spending
        budget.update_spending(spent_amount);

        // TODO: Save to repository
        Ok(budget)
    }

    pub fn get_status(&self, project_id: &str) -> Result<BudgetStatus, DomainError> {
        let budget = self.get_budget(project_id)?;
        Ok(budget.status)
    }

    pub fn check_alerts(&self, project_id: &str) -> Result<Vec<crate::domain::financial::BudgetAlert>, DomainError> {
        let budget = self.get_budget(project_id)?;
        Ok(budget.alerts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_set_budget_success() {
        let use_case = ManageBudgetUseCase::new(PathBuf::from("."));
        let result = use_case.set_budget("PROJ-1".to_string(), 50000.0, "USD".to_string(), "user1".to_string());

        assert!(result.is_ok());
        let budget = result.unwrap();
        assert_eq!(budget.total_budget, 50000.0);
        assert_eq!(budget.currency, "USD");
    }

    #[test]
    fn test_set_budget_negative_amount() {
        let use_case = ManageBudgetUseCase::new(PathBuf::from("."));
        let result = use_case.set_budget("PROJ-1".to_string(), -1000.0, "USD".to_string(), "user1".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_set_budget_zero_amount() {
        let use_case = ManageBudgetUseCase::new(PathBuf::from("."));
        let result = use_case.set_budget("PROJ-1".to_string(), 0.0, "USD".to_string(), "user1".to_string());

        assert!(result.is_err());
    }
}
