use serde::{Deserialize, Serialize};
use crate::domain::notifications::AlertSeverity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetStatus {
    Under,      // Abaixo do orçamento
    OnTrack,    // No prazo
    Over,       // Acima do orçamento
    Exceeded,   // Excedido
}

impl std::fmt::Display for BudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetStatus::Under => write!(f, "UNDER"),
            BudgetStatus::OnTrack => write!(f, "ON_TRACK"),
            BudgetStatus::Over => write!(f, "OVER"),
            BudgetStatus::Exceeded => write!(f, "EXCEEDED"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectBudget {
    pub project_id: String,
    pub total_budget: f64,
    pub spent_amount: f64,
    pub remaining_amount: f64,
    pub status: BudgetStatus,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub alerts: Vec<BudgetAlert>,
}

impl ProjectBudget {
    pub fn new(
        project_id: String,
        total_budget: f64,
        currency: String,
        created_by: String,
    ) -> Self {
        Self {
            project_id,
            total_budget,
            spent_amount: 0.0,
            remaining_amount: total_budget,
            status: BudgetStatus::Under,
            currency,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by,
            alerts: Vec::new(),
        }
    }

    pub fn update_spending(&mut self, spent_amount: f64) {
        self.spent_amount = spent_amount;
        self.remaining_amount = self.total_budget - self.spent_amount;
        self.status = self.calculate_status();
        self.updated_at = chrono::Utc::now();
        self.alerts = self.check_budget_alerts();
    }

    pub fn calculate_status(&self) -> BudgetStatus {
        let utilization = self.spent_amount / self.total_budget;
        
        if utilization >= 1.0 {
            BudgetStatus::Exceeded
        } else if utilization >= 0.9 {
            BudgetStatus::Over
        } else if utilization >= 0.7 {
            BudgetStatus::OnTrack
        } else {
            BudgetStatus::Under
        }
    }

    pub fn check_budget_alerts(&self) -> Vec<BudgetAlert> {
        let mut alerts = Vec::new();
        let utilization = self.spent_amount / self.total_budget;
        
        if utilization >= 1.0 {
            alerts.push(BudgetAlert::new(
                format!("budget-exceeded-{}", self.project_id),
                AlertSeverity::Critical,
                "Budget Exceeded".to_string(),
                format!(
                    "Project {} has exceeded its budget by {:.2} {}",
                    self.project_id,
                    self.spent_amount - self.total_budget,
                    self.currency
                ),
                self.project_id.clone(),
                "project".to_string(),
                Some("Consider reducing scope or increasing budget".to_string()),
            ));
        } else if utilization >= 0.9 {
            alerts.push(BudgetAlert::new(
                format!("budget-warning-{}", self.project_id),
                AlertSeverity::Warning,
                "Budget Warning".to_string(),
                format!(
                    "Project {} has used {:.1}% of its budget ({:.2} {} / {:.2} {})",
                    self.project_id,
                    utilization * 100.0,
                    self.spent_amount,
                    self.currency,
                    self.total_budget,
                    self.currency
                ),
                self.project_id.clone(),
                "project".to_string(),
                Some("Monitor spending closely".to_string()),
            ));
        }
        
        alerts
    }

    pub fn get_utilization_percentage(&self) -> f64 {
        (self.spent_amount / self.total_budget) * 100.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: String,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub entity_id: String,
    pub entity_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub suggested_action: Option<String>,
}

impl BudgetAlert {
    pub fn new(
        id: String,
        severity: AlertSeverity,
        title: String,
        message: String,
        entity_id: String,
        entity_type: String,
        suggested_action: Option<String>,
    ) -> Self {
        Self {
            id,
            severity,
            title,
            message,
            entity_id,
            entity_type,
            created_at: chrono::Utc::now(),
            acknowledged: false,
            suggested_action,
        }
    }
}


