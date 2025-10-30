use crate::domain::financial::CostSummary;
use crate::domain::shared::errors::DomainError;
use std::path::PathBuf;

pub struct CalculateCostsUseCase {
    #[allow(dead_code)]
    base_path: PathBuf,
}

impl CalculateCostsUseCase {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn execute(&self, _project_id: &str) -> Result<CostSummary, DomainError> {
        // TODO: Load costs from repository
        // For now, return empty summary
        let summary = CostSummary::new();

        Ok(summary)
    }

    pub fn calculate_for_task(&self, _task_id: &str) -> Result<f64, DomainError> {
        // TODO: Load costs for specific task
        Ok(0.0)
    }

    pub fn calculate_for_resource(&self, _resource_id: &str) -> Result<f64, DomainError> {
        // TODO: Load costs for specific resource
        Ok(0.0)
    }

    pub fn calculate_project_total(&self, project_id: &str) -> Result<f64, DomainError> {
        let summary = self.execute(project_id)?;
        Ok(summary.total_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_calculate_costs_empty_project() {
        let use_case = CalculateCostsUseCase::new(PathBuf::from("."));
        let result = use_case.execute("PROJ-1");

        assert!(result.is_ok());
        let summary = result.unwrap();
        assert_eq!(summary.total_cost, 0.0);
    }

    #[test]
    fn test_calculate_project_total() {
        let use_case = CalculateCostsUseCase::new(PathBuf::from("."));
        let result = use_case.calculate_project_total("PROJ-1");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }
}
