use crate::domain::financial::ProjectBudget;
use crate::infrastructure::persistence::repositories::BudgetRepository;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct BudgetArgs {
    #[clap(subcommand)]
    pub command: BudgetCommand,
}

#[derive(Subcommand)]
pub enum BudgetCommand {
    /// Create a new budget for a project
    Create {
        /// Project code
        #[clap(long)]
        project: String,

        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,

        /// Total budget amount
        #[clap(long)]
        amount: f64,

        /// Currency (default: USD)
        #[clap(long, default_value = "USD")]
        currency: String,

        /// Created by (default: system)
        #[clap(long, default_value = "system")]
        created_by: String,
    },

    /// Show budget details for a project
    Show {
        /// Project code
        #[clap(long)]
        project: String,

        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,
    },

    /// Update budget amount
    Update {
        /// Project code
        #[clap(long)]
        project: String,

        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,

        /// New total budget amount
        #[clap(long)]
        amount: f64,
    },

    /// Show budget status and alerts
    Status {
        /// Project code
        #[clap(long)]
        project: String,

        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,
    },

    /// List all budgets
    List {
        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,
    },

    /// Delete a budget
    Delete {
        /// Project code
        #[clap(long)]
        project: String,

        /// Company code (defaults to current context)
        #[clap(long)]
        company: Option<String>,
    },
}

impl BudgetCommand {
    pub fn execute(&self, base_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let budget_repo = BudgetRepository::new(base_path);

        match self {
            BudgetCommand::Create {
                project,
                company,
                amount,
                currency,
                created_by,
            } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());

                let budget = ProjectBudget::new(project.clone(), *amount, currency.clone(), created_by.clone());

                budget_repo.save(&company_code, project, &budget)?;

                println!("✓ Budget created successfully");
                println!();
                println!("  Project:       {}", project);
                println!("  Total Budget:  {} {}", amount, currency);
                println!("  Status:        {}", budget.status);

                Ok(())
            }

            BudgetCommand::Show { project, company } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());
                let budget = budget_repo.load(&company_code, project)?;

                println!("Budget Details");
                println!("─────────────────────────────────────");
                println!("  Project:        {}", budget.project_id);
                println!("  Total Budget:   {} {}", budget.total_budget, budget.currency);
                println!("  Spent:          {} {}", budget.spent_amount, budget.currency);
                println!("  Remaining:      {} {}", budget.remaining_amount, budget.currency);
                println!("  Status:         {}", budget.status);
                println!("  Created By:     {}", budget.created_by);
                println!("  Created At:     {}", budget.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("  Updated At:     {}", budget.updated_at.format("%Y-%m-%d %H:%M:%S"));

                if !budget.alerts.is_empty() {
                    println!();
                    println!("Active Alerts:");
                    for alert in &budget.alerts {
                        let icon = match alert.severity.to_string().as_str() {
                            "Warning" => "⚠",
                            "Critical" => "✗",
                            _ => "ℹ",
                        };
                        println!("  {} {} - {}", icon, alert.severity, alert.message);
                    }
                }

                Ok(())
            }

            BudgetCommand::Update {
                project,
                company,
                amount,
            } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());
                let mut budget = budget_repo.load(&company_code, project)?;

                let old_amount = budget.total_budget;
                budget.total_budget = *amount;
                budget.remaining_amount = amount - budget.spent_amount;
                budget.updated_at = chrono::Utc::now();
                budget.status = budget.calculate_status();
                budget.alerts = budget.check_budget_alerts();

                budget_repo.save(&company_code, project, &budget)?;

                println!("✓ Budget updated successfully");
                println!();
                println!("  Project:       {}", project);
                println!("  Old Budget:    {} {}", old_amount, budget.currency);
                println!("  New Budget:    {} {}", amount, budget.currency);
                println!("  Status:        {}", budget.status);

                Ok(())
            }

            BudgetCommand::Status { project, company } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());
                let budget = budget_repo.load(&company_code, project)?;

                let utilization = if budget.total_budget > 0.0 {
                    (budget.spent_amount / budget.total_budget) * 100.0
                } else {
                    0.0
                };

                println!("Budget Status: {}", project);
                println!("════════════════════════════════════");
                println!();
                println!("  Total Budget:   {} {}", budget.total_budget, budget.currency);
                println!(
                    "  Spent:          {} {} ({:.1}%)",
                    budget.spent_amount, budget.currency, utilization
                );
                println!("  Remaining:      {} {}", budget.remaining_amount, budget.currency);
                println!("  Status:         {}", budget.status);
                println!();

                if budget.alerts.is_empty() {
                    println!("✓ No budget alerts");
                } else {
                    println!("Active Alerts:");
                    println!("─────────────────────────────────────");
                    for alert in &budget.alerts {
                        println!();
                        println!("  Severity:  {}", alert.severity);
                        println!("  Entity:    {}", alert.entity_type);
                        println!("  Title:     {}", alert.title);
                        println!("  Message:   {}", alert.message);
                        if let Some(action) = &alert.suggested_action {
                            println!("  Action:    {}", action);
                        }
                    }
                }

                Ok(())
            }

            BudgetCommand::List { company } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());
                let budgets = budget_repo.list_by_company(&company_code)?;

                if budgets.is_empty() {
                    println!("No budgets found for company: {}", company_code);
                    return Ok(());
                }

                println!("Budgets for Company: {}", company_code);
                println!("═══════════════════════════════════════════════════════════════════");
                println!(
                    "{:<20} {:<15} {:<15} {:<15} {:<12}",
                    "PROJECT", "TOTAL", "SPENT", "REMAINING", "STATUS"
                );
                println!("───────────────────────────────────────────────────────────────────");

                for budget in budgets {
                    println!(
                        "{:<20} {:<15} {:<15} {:<15} {:<12}",
                        budget.project_id,
                        format!("{} {}", budget.total_budget, budget.currency),
                        format!("{} {}", budget.spent_amount, budget.currency),
                        format!("{} {}", budget.remaining_amount, budget.currency),
                        budget.status
                    );
                }

                Ok(())
            }

            BudgetCommand::Delete { project, company } => {
                let company_code = company.clone().unwrap_or_else(|| "TECH-CORP".to_string());
                budget_repo.delete(&company_code, project)?;

                println!("✓ Budget deleted successfully");
                println!();
                println!("  Project:  {}", project);
                println!("  Company:  {}", company_code);

                Ok(())
            }
        }
    }
}
