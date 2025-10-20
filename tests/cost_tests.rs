use task_task_revolution::domain::financial::{CostEntry, CostType, ProjectBudget, BudgetStatus, CostSummary};
use task_task_revolution::domain::notifications::{Alert, AlertSeverity, AlertType};

#[test]
fn test_cost_entry_creation() {
    let cost = CostEntry::new(
        "RES-1".to_string(),
        Some("TASK-1".to_string()),
        "PROJ-1".to_string(),
        100.0,
        CostType::Hourly,
        Some("Development work".to_string()),
        "user1".to_string(),
    );
    
    assert_eq!(cost.resource_id, "RES-1");
    assert_eq!(cost.amount, 100.0);
    assert_eq!(cost.cost_type, CostType::Hourly);
    assert_eq!(cost.project_id, "PROJ-1");
}

#[test]
fn test_hourly_cost_calculation() {
    let cost = CostEntry::new(
        "RES-1".to_string(),
        None,
        "PROJ-1".to_string(),
        50.0,
        CostType::Hourly,
        None,
        "user1".to_string(),
    );
    
    assert_eq!(cost.calculate_hourly_cost(8.0), 400.0);
    assert_eq!(cost.calculate_hourly_cost(0.0), 0.0);
}

#[test]
fn test_project_budget_creation() {
    let budget = ProjectBudget::new(
        "PROJ-1".to_string(),
        10000.0,
        "USD".to_string(),
        "user1".to_string(),
    );
    
    assert_eq!(budget.project_id, "PROJ-1");
    assert_eq!(budget.total_budget, 10000.0);
    assert_eq!(budget.spent_amount, 0.0);
    assert_eq!(budget.remaining_amount, 10000.0);
    assert_eq!(budget.status, BudgetStatus::Under);
}

#[test]
fn test_budget_status_calculation() {
    let mut budget = ProjectBudget::new(
        "PROJ-1".to_string(),
        10000.0,
        "USD".to_string(),
        "user1".to_string(),
    );
    
    // Test under budget
    budget.update_spending(5000.0);
    assert_eq!(budget.status, BudgetStatus::Under);
    
    // Test on track
    budget.update_spending(7500.0);
    assert_eq!(budget.status, BudgetStatus::OnTrack);
    
    // Test over budget
    budget.update_spending(9500.0);
    assert_eq!(budget.status, BudgetStatus::Over);
    
    // Test exceeded
    budget.update_spending(10500.0);
    assert_eq!(budget.status, BudgetStatus::Exceeded);
}

#[test]
fn test_budget_alerts_generation() {
    let mut budget = ProjectBudget::new(
        "PROJ-1".to_string(),
        10000.0,
        "USD".to_string(),
        "user1".to_string(),
    );
    
    // Test no alerts for under budget
    budget.update_spending(5000.0);
    assert!(budget.alerts.is_empty());
    
    // Test warning alert for 90% usage
    budget.update_spending(9000.0);
    assert_eq!(budget.alerts.len(), 1);
    assert_eq!(budget.alerts[0].severity, AlertSeverity::Warning);
    
    // Test critical alert for exceeded budget
    budget.update_spending(11000.0);
    assert_eq!(budget.alerts.len(), 1);
    assert_eq!(budget.alerts[0].severity, AlertSeverity::Critical);
}

#[test]
fn test_alert_acknowledgment() {
    let mut alert = Alert::new(
        AlertType::BudgetWarning,
        AlertSeverity::Warning,
        "Test Alert".to_string(),
        "Test message".to_string(),
        "PROJ-1".to_string(),
        "project".to_string(),
        Some("Test action".to_string()),
    );
    
    assert!(!alert.acknowledged);
    alert.acknowledge("user1".to_string());
    assert!(alert.acknowledged);
    assert_eq!(alert.acknowledged_by, Some("user1".to_string()));
}

#[test]
fn test_cost_summary_calculation() {
    let mut summary = CostSummary::new();
    
    let cost1 = CostEntry::new(
        "RES-1".to_string(),
        Some("TASK-1".to_string()),
        "PROJ-1".to_string(),
        100.0,
        CostType::Hourly,
        None,
        "user1".to_string(),
    );
    
    let cost2 = CostEntry::new(
        "RES-2".to_string(),
        Some("TASK-2".to_string()),
        "PROJ-1".to_string(),
        200.0,
        CostType::Fixed,
        None,
        "user1".to_string(),
    );
    
    summary.add_cost(&cost1);
    summary.add_cost(&cost2);
    
    assert_eq!(summary.total_cost, 300.0);
    assert_eq!(summary.cost_by_type.get(&CostType::Hourly), Some(&100.0));
    assert_eq!(summary.cost_by_type.get(&CostType::Fixed), Some(&200.0));
    assert_eq!(summary.cost_by_resource.get("RES-1"), Some(&100.0));
    assert_eq!(summary.cost_by_resource.get("RES-2"), Some(&200.0));
}

