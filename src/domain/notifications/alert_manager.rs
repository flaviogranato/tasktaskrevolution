use super::alert::{Alert, AlertSeverity, AlertSummary};
use std::collections::HashMap;

pub struct AlertManager {
    alerts: HashMap<String, Alert>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: HashMap::new(),
        }
    }

    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.insert(alert.id.clone(), alert);
    }

    pub fn get_alert(&self, id: &str) -> Option<&Alert> {
        self.alerts.get(id)
    }

    pub fn get_alerts_by_entity(&self, entity_id: &str) -> Vec<&Alert> {
        self.alerts
            .values()
            .filter(|alert| alert.entity_id == entity_id)
            .collect()
    }

    pub fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<&Alert> {
        self.alerts
            .values()
            .filter(|alert| alert.severity == severity)
            .collect()
    }

    pub fn get_unacknowledged_alerts(&self) -> Vec<&Alert> {
        self.alerts
            .values()
            .filter(|alert| !alert.acknowledged)
            .collect()
    }

    pub fn acknowledge_alert(&mut self, id: &str, acknowledged_by: String) -> bool {
        if let Some(alert) = self.alerts.get_mut(id) {
            alert.acknowledge(acknowledged_by);
            true
        } else {
            false
        }
    }

    pub fn get_summary(&self) -> AlertSummary {
        let mut summary = AlertSummary::new();
        
        for alert in self.alerts.values() {
            summary.add_alert(alert);
        }
        
        summary
    }

    pub fn get_all_alerts(&self) -> Vec<&Alert> {
        self.alerts.values().collect()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

