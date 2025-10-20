use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CostType {
    Hourly,    // Custo por hora
    Fixed,     // Custo fixo
    Material,  // Custo de material
    Travel,    // Custo de viagem
}

impl std::fmt::Display for CostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CostType::Hourly => write!(f, "hourly"),
            CostType::Fixed => write!(f, "fixed"),
            CostType::Material => write!(f, "material"),
            CostType::Travel => write!(f, "travel"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEntry {
    pub id: String,
    pub resource_id: String,
    pub task_id: Option<String>,
    pub project_id: String,
    pub amount: f64,
    pub cost_type: CostType,
    pub date: chrono::NaiveDate,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
}

impl CostEntry {
    pub fn new(
        resource_id: String,
        task_id: Option<String>,
        project_id: String,
        amount: f64,
        cost_type: CostType,
        description: Option<String>,
        created_by: String,
    ) -> Self {
        Self {
            id: format!("cost-{}", chrono::Utc::now().timestamp_millis()),
            resource_id,
            task_id,
            project_id,
            amount,
            cost_type,
            date: chrono::Utc::now().date_naive(),
            description,
            created_at: chrono::Utc::now(),
            created_by,
        }
    }

    pub fn calculate_hourly_cost(&self, hours: f64) -> f64 {
        match self.cost_type {
            CostType::Hourly => self.amount * hours,
            _ => self.amount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost: f64,
    pub cost_by_type: HashMap<CostType, f64>,
    pub cost_by_resource: HashMap<String, f64>,
    pub cost_by_task: HashMap<String, f64>,
    pub daily_costs: HashMap<chrono::NaiveDate, f64>,
}

impl CostSummary {
    pub fn new() -> Self {
        Self {
            total_cost: 0.0,
            cost_by_type: HashMap::new(),
            cost_by_resource: HashMap::new(),
            cost_by_task: HashMap::new(),
            daily_costs: HashMap::new(),
        }
    }

    pub fn add_cost(&mut self, cost: &CostEntry) {
        self.total_cost += cost.amount;
        
        // Add to cost by type
        *self.cost_by_type.entry(cost.cost_type.clone()).or_insert(0.0) += cost.amount;
        
        // Add to cost by resource
        *self.cost_by_resource.entry(cost.resource_id.clone()).or_insert(0.0) += cost.amount;
        
        // Add to cost by task (if applicable)
        if let Some(task_id) = &cost.task_id {
            *self.cost_by_task.entry(task_id.clone()).or_insert(0.0) += cost.amount;
        }
        
        // Add to daily costs
        *self.daily_costs.entry(cost.date).or_insert(0.0) += cost.amount;
    }
}

impl Default for CostSummary {
    fn default() -> Self {
        Self::new()
    }
}

