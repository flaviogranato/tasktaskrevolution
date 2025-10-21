use crate::application::financial::{
    AddCostUseCase, CalculateCostsUseCase, GenerateReportUseCase, ManageBudgetUseCase, ReportFormat,
};
use crate::domain::financial::CostType;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct CostArgs {
    #[command(subcommand)]
    pub command: CostCommand,
}

#[derive(Subcommand)]
pub enum CostCommand {
    /// Add a cost entry
    Add {
        /// Resource ID
        #[arg(short, long)]
        resource: String,

        /// Amount
        #[arg(short, long)]
        amount: f64,

        /// Cost type (hourly, fixed, material, travel)
        #[arg(short = 't', long)]
        cost_type: String,

        /// Task ID (optional)
        #[arg(short = 'k', long)]
        task: Option<String>,

        /// Project ID
        #[arg(short, long)]
        project: String,

        /// Description (optional)
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Calculate costs for a project
    Calculate {
        /// Project ID
        #[arg(short, long)]
        project: String,
    },

    /// Budget management
    Budget {
        #[command(subcommand)]
        command: BudgetCommand,
    },

    /// Generate cost report
    Report {
        /// Project ID
        #[arg(short, long)]
        project: String,

        /// Output format (table, json, csv)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum BudgetCommand {
    /// Set project budget
    Set {
        /// Project ID
        #[arg(short, long)]
        project: String,

        /// Budget amount
        #[arg(short, long)]
        amount: f64,

        /// Currency code
        #[arg(short, long, default_value = "USD")]
        currency: String,
    },

    /// Show budget status
    Status {
        /// Project ID
        #[arg(short, long)]
        project: String,
    },
}

impl CostCommand {
    pub fn execute(&self, base_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            CostCommand::Add {
                resource,
                amount,
                cost_type,
                task,
                project,
                description,
            } => self.handle_add_cost(base_path, resource, *amount, cost_type, task, project, description),
            CostCommand::Calculate { project } => self.handle_calculate_costs(base_path, project),
            CostCommand::Budget { command } => self.handle_budget_command(base_path, command),
            CostCommand::Report { project, format } => self.handle_generate_report(base_path, project, format),
        }
    }

    fn handle_add_cost(
        &self,
        base_path: &str,
        resource: &str,
        amount: f64,
        cost_type_str: &str,
        task: &Option<String>,
        project: &str,
        description: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse cost type
        let cost_type = match cost_type_str.to_lowercase().as_str() {
            "hourly" => CostType::Hourly,
            "fixed" => CostType::Fixed,
            "material" => CostType::Material,
            "travel" => CostType::Travel,
            _ => {
                eprintln!("Invalid cost type. Use: hourly, fixed, material, travel");
                return Err("Invalid cost type".into());
            }
        };

        let use_case = AddCostUseCase::new(PathBuf::from(base_path));
        let cost_entry = use_case.execute(
            resource.to_string(),
            task.clone(),
            project.to_string(),
            amount,
            cost_type,
            description.clone(),
            "system".to_string(),
        )?;

        println!("Cost entry added successfully!");
        println!("   ID: {}", cost_entry.id);
        println!("   Resource: {}", cost_entry.resource_id);
        println!("   Amount: ${:.2}", cost_entry.amount);
        println!("   Type: {}", cost_entry.cost_type);
        println!("   Project: {}", cost_entry.project_id);
        if let Some(task_id) = &cost_entry.task_id {
            println!("   Task: {}", task_id);
        }
        if let Some(desc) = &cost_entry.description {
            println!("   Description: {}", desc);
        }
        println!("   Date: {}", cost_entry.date);

        Ok(())
    }

    fn handle_calculate_costs(&self, base_path: &str, project: &str) -> Result<(), Box<dyn std::error::Error>> {
        let use_case = CalculateCostsUseCase::new(PathBuf::from(base_path));
        let summary = use_case.execute(project)?;

        println!("Cost calculation for project: {}", project);
        println!("\nCOST SUMMARY:");
        println!("   Total Cost: ${:.2}", summary.total_cost);

        if !summary.cost_by_type.is_empty() {
            println!("\nCOST BY TYPE:");
            for (cost_type, amount) in &summary.cost_by_type {
                println!("   {}: ${:.2}", cost_type, amount);
            }
        }

        if !summary.cost_by_resource.is_empty() {
            println!("\nCOST BY RESOURCE:");
            for (resource_id, amount) in &summary.cost_by_resource {
                println!("   {}: ${:.2}", resource_id, amount);
            }
        }

        Ok(())
    }

    fn handle_budget_command(
        &self,
        base_path: &str,
        command: &BudgetCommand,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let use_case = ManageBudgetUseCase::new(PathBuf::from(base_path));

        match command {
            BudgetCommand::Set {
                project,
                amount,
                currency,
            } => {
                let budget = use_case.set_budget(project.clone(), *amount, currency.clone(), "system".to_string())?;

                println!("Budget set successfully!");
                println!("   Project: {}", budget.project_id);
                println!("   Amount: ${:.2} {}", budget.total_budget, budget.currency);
                println!("   Currency: {}", budget.currency);
                println!("   Status: {}", budget.status);
            }

            BudgetCommand::Status { project } => match use_case.get_budget(project) {
                Ok(budget) => {
                    println!("Budget status for project: {}", project);
                    println!("\nBUDGET STATUS:");
                    println!("   Total Budget: ${:.2}", budget.total_budget);
                    println!("   Spent Amount: ${:.2}", budget.spent_amount);
                    println!("   Remaining: ${:.2}", budget.remaining_amount);
                    println!("   Utilization: {:.1}%", budget.get_utilization_percentage());
                    println!("   Status: {}", budget.status);

                    if !budget.alerts.is_empty() {
                        println!("\nALERTS:");
                        for alert in &budget.alerts {
                            println!("   {}: {}", alert.severity, alert.message);
                            if let Some(action) = &alert.suggested_action {
                                println!("   SUGGESTION: {}", action);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Budget not found for project {}: {}", project, e);
                    println!("\nTo set a budget, use:");
                    println!("   ttr cost budget set --project {} --amount <AMOUNT>", project);
                }
            },
        }

        Ok(())
    }

    fn handle_generate_report(
        &self,
        base_path: &str,
        project: &str,
        format_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let format = match format_str.to_lowercase().as_str() {
            "table" => ReportFormat::Table,
            "json" => ReportFormat::Json,
            "csv" => ReportFormat::Csv,
            _ => {
                eprintln!("Invalid format. Use: table, json, csv");
                return Err("Invalid format".into());
            }
        };

        let use_case = GenerateReportUseCase::new(PathBuf::from(base_path));
        let report = use_case.execute(project, format)?;

        println!("{}", report);

        Ok(())
    }
}
