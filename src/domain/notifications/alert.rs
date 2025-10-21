use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AlertType {
    BudgetWarning,
    BudgetExceeded,
    CostAnomaly,
    ResourceOveruse,
    TaskOverrun,
    ProjectDelay,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::BudgetWarning => write!(f, "BUDGET_WARNING"),
            AlertType::BudgetExceeded => write!(f, "BUDGET_EXCEEDED"),
            AlertType::CostAnomaly => write!(f, "COST_ANOMALY"),
            AlertType::ResourceOveruse => write!(f, "RESOURCE_OVERUSE"),
            AlertType::TaskOverrun => write!(f, "TASK_OVERRUN"),
            AlertType::ProjectDelay => write!(f, "PROJECT_DELAY"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,     // Informação
    Warning,  // Aviso
    Critical, // Crítico
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub entity_id: String,
    pub entity_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
    pub acknowledged_by: Option<String>,
    pub suggested_action: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Alert {
    pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        title: String,
        message: String,
        entity_id: String,
        entity_type: String,
        suggested_action: Option<String>,
    ) -> Self {
        Self {
            id: format!("alert-{}", chrono::Utc::now().timestamp_millis()),
            alert_type,
            severity,
            title,
            message,
            entity_id,
            entity_type,
            created_at: chrono::Utc::now(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
            suggested_action,
            metadata: HashMap::new(),
        }
    }

    pub fn acknowledge(&mut self, acknowledged_by: String) {
        self.acknowledged = true;
        self.acknowledged_at = Some(chrono::Utc::now());
        self.acknowledged_by = Some(acknowledged_by);
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub unacknowledged_count: usize,
    pub alerts_by_type: HashMap<AlertType, usize>,
}

impl AlertSummary {
    pub fn new() -> Self {
        Self {
            total_alerts: 0,
            critical_count: 0,
            warning_count: 0,
            info_count: 0,
            unacknowledged_count: 0,
            alerts_by_type: HashMap::new(),
        }
    }

    pub fn add_alert(&mut self, alert: &Alert) {
        self.total_alerts += 1;

        match alert.severity {
            AlertSeverity::Critical => self.critical_count += 1,
            AlertSeverity::Warning => self.warning_count += 1,
            AlertSeverity::Info => self.info_count += 1,
        }

        if !alert.acknowledged {
            self.unacknowledged_count += 1;
        }

        *self.alerts_by_type.entry(alert.alert_type.clone()).or_insert(0) += 1;
    }
}

impl Default for AlertSummary {
    fn default() -> Self {
        Self::new()
    }
}
