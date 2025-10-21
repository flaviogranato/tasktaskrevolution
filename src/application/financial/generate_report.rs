use crate::domain::financial::{CostSummary, ProjectBudget};
use crate::domain::shared::errors::DomainError;
use std::path::PathBuf;

pub enum ReportFormat {
    Table,
    Json,
    Csv,
}

pub struct FinancialReport {
    pub project_id: String,
    pub budget: Option<ProjectBudget>,
    pub cost_summary: CostSummary,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl FinancialReport {
    pub fn format_table(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("\nPROJECT FINANCIAL REPORT: {}\n", self.project_id));
        output.push_str(&format!(
            "Generated: {}\n\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));

        if let Some(budget) = &self.budget {
            output.push_str("BUDGET SUMMARY:\n");
            output.push_str(&format!("   Total Budget: ${:.2}\n", budget.total_budget));
            output.push_str(&format!("   Spent Amount: ${:.2}\n", budget.spent_amount));
            output.push_str(&format!("   Remaining: ${:.2}\n", budget.remaining_amount));
            output.push_str(&format!(
                "   Utilization: {:.1}%\n",
                budget.get_utilization_percentage()
            ));
            output.push_str(&format!("   Status: {}\n\n", budget.status));
        }

        output.push_str("COST SUMMARY:\n");
        output.push_str(&format!("   Total Cost: ${:.2}\n", self.cost_summary.total_cost));

        if !self.cost_summary.cost_by_type.is_empty() {
            output.push_str("\nCOST BY TYPE:\n");
            for (cost_type, amount) in &self.cost_summary.cost_by_type {
                output.push_str(&format!("   {}: ${:.2}\n", cost_type, amount));
            }
        }

        if !self.cost_summary.cost_by_resource.is_empty() {
            output.push_str("\nCOST BY RESOURCE:\n");
            for (resource_id, amount) in &self.cost_summary.cost_by_resource {
                output.push_str(&format!("   {}: ${:.2}\n", resource_id, amount));
            }
        }

        output
    }
}

pub struct GenerateReportUseCase {
    base_path: PathBuf,
}

impl GenerateReportUseCase {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn execute(&self, project_id: &str, format: ReportFormat) -> Result<String, DomainError> {
        let _ = &self.base_path; // Prevent unused warning
        // TODO: Load budget and costs from repository
        let budget = None;
        let cost_summary = CostSummary::new();

        let report = FinancialReport {
            project_id: project_id.to_string(),
            budget,
            cost_summary,
            generated_at: chrono::Utc::now(),
        };

        match format {
            ReportFormat::Table => Ok(report.format_table()),
            ReportFormat::Json => {
                // TODO: Implement JSON format
                Ok("{}".to_string())
            }
            ReportFormat::Csv => {
                // TODO: Implement CSV format
                Ok("".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_generate_report_table() {
        let use_case = GenerateReportUseCase::new(PathBuf::from("."));
        let result = use_case.execute("PROJ-1", ReportFormat::Table);

        assert!(result.is_ok());
        let report = result.unwrap();
        assert!(report.contains("PROJECT FINANCIAL REPORT"));
        assert!(report.contains("PROJ-1"));
    }
}
