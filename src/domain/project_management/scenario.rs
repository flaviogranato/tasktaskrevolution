#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ScenarioType {
    Baseline,
    Current,
    WhatIf,
    Historical,
}

impl std::fmt::Display for ScenarioType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScenarioType::Baseline => write!(f, "Baseline"),
            ScenarioType::Current => write!(f, "Current"),
            ScenarioType::WhatIf => write!(f, "What-If"),
            ScenarioType::Historical => write!(f, "Historical"),
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Represents a project scenario with specific dates, status, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScenario {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: ScenarioType,
    
    // Project data for this scenario
    pub project_id: String,
    pub project_code: String,
    pub project_name: String,
    pub status: String,
    pub priority: String,
    
    // Dates for this scenario
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    
    // Task data for this scenario
    pub tasks: HashMap<String, TaskScenario>,
    
    // Resource assignments for this scenario
    pub resource_assignments: HashMap<String, ResourceAssignmentScenario>,
    
    // Scenario-specific metadata
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub metadata: HashMap<String, String>,
}

/// Represents a task within a specific scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScenario {
    pub task_id: String,
    pub task_code: String,
    pub task_name: String,
    pub status: String,
    pub priority: String,
    
    // Dates for this scenario
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    
    // Dependencies for this scenario
    pub dependencies: Vec<String>,
    
    // Resource assignments for this task in this scenario
    pub resource_assignments: Vec<String>,
    
    // Scenario-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Represents a resource assignment within a specific scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignmentScenario {
    pub assignment_id: String,
    pub resource_id: String,
    pub task_id: String,
    pub allocation_percentage: u8,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    
    // Scenario-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Manages multiple scenarios for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioManager {
    pub project_id: String,
    pub project_code: String,
    pub scenarios: HashMap<String, ProjectScenario>,
    pub active_scenario_id: Option<String>,
    pub baseline_scenario_id: Option<String>,
}

impl ProjectScenario {
    pub fn new(
        name: String,
        description: Option<String>,
        scenario_type: ScenarioType,
        project_id: String,
        project_code: String,
        project_name: String,
        created_by: String,
    ) -> Self {
        Self {
            id: uuid7::uuid7().to_string(),
            name,
            description,
            scenario_type,
            project_id,
            project_code,
            project_name,
            status: "Planned".to_string(),
            priority: "Medium".to_string(),
            start_date: None,
            end_date: None,
            actual_start_date: None,
            actual_end_date: None,
            tasks: HashMap::new(),
            resource_assignments: HashMap::new(),
            created_at: Utc::now(),
            created_by,
            metadata: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: TaskScenario) {
        self.tasks.insert(task.task_id.clone(), task);
    }

    pub fn add_resource_assignment(&mut self, assignment: ResourceAssignmentScenario) {
        self.resource_assignments.insert(assignment.assignment_id.clone(), assignment);
    }

    pub fn update_dates(&mut self, start_date: Option<NaiveDate>, end_date: Option<NaiveDate>) {
        self.start_date = start_date;
        self.end_date = end_date;
    }

    pub fn update_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn update_priority(&mut self, priority: String) {
        self.priority = priority;
    }

    pub fn set_actual_dates(&mut self, actual_start: Option<NaiveDate>, actual_end: Option<NaiveDate>) {
        self.actual_start_date = actual_start;
        self.actual_end_date = actual_end;
    }

    pub fn get_task(&self, task_id: &str) -> Option<&TaskScenario> {
        self.tasks.get(task_id)
    }

    pub fn get_resource_assignment(&self, assignment_id: &str) -> Option<&ResourceAssignmentScenario> {
        self.resource_assignments.get(assignment_id)
    }

    pub fn is_baseline(&self) -> bool {
        matches!(self.scenario_type, ScenarioType::Baseline)
    }

    pub fn is_current(&self) -> bool {
        matches!(self.scenario_type, ScenarioType::Current)
    }

    pub fn duration_days(&self) -> Option<u32> {
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            Some((end - start).num_days() as u32)
        } else {
            None
        }
    }

    pub fn actual_duration_days(&self) -> Option<u32> {
        if let (Some(start), Some(end)) = (self.actual_start_date, self.actual_end_date) {
            Some((end - start).num_days() as u32)
        } else {
            None
        }
    }

    pub fn is_delayed(&self) -> bool {
        if let (Some(planned_end), Some(actual_end)) = (self.end_date, self.actual_end_date) {
            actual_end > planned_end
        } else {
            false
        }
    }

    pub fn delay_days(&self) -> Option<i32> {
        if let (Some(planned_end), Some(actual_end)) = (self.end_date, self.actual_end_date) {
            Some((actual_end - planned_end).num_days() as i32)
        } else {
            None
        }
    }
}

impl ScenarioManager {
    pub fn new(project_id: String, project_code: String) -> Self {
        Self {
            project_id,
            project_code,
            scenarios: HashMap::new(),
            active_scenario_id: None,
            baseline_scenario_id: None,
        }
    }

    pub fn add_scenario(&mut self, scenario: ProjectScenario) {
        let scenario_id = scenario.id.clone();
        let scenario_type = scenario.scenario_type;
        
        self.scenarios.insert(scenario_id.clone(), scenario);
        
        // Set as baseline if it's the first baseline scenario
        if matches!(scenario_type, ScenarioType::Baseline) && self.baseline_scenario_id.is_none() {
            self.baseline_scenario_id = Some(scenario_id.clone());
        }
        
        // Set as active if it's the first scenario
        if self.active_scenario_id.is_none() {
            self.active_scenario_id = Some(scenario_id);
        }
    }

    pub fn get_scenario(&self, scenario_id: &str) -> Option<&ProjectScenario> {
        self.scenarios.get(scenario_id)
    }

    pub fn get_scenario_mut(&mut self, scenario_id: &str) -> Option<&mut ProjectScenario> {
        self.scenarios.get_mut(scenario_id)
    }

    pub fn set_active_scenario(&mut self, scenario_id: &str) -> bool {
        if self.scenarios.contains_key(scenario_id) {
            self.active_scenario_id = Some(scenario_id.to_string());
            true
        } else {
            false
        }
    }

    pub fn set_baseline_scenario(&mut self, scenario_id: &str) -> bool {
        if let Some(scenario) = self.scenarios.get(scenario_id) {
            if matches!(scenario.scenario_type, ScenarioType::Baseline) {
                self.baseline_scenario_id = Some(scenario_id.to_string());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_active_scenario(&self) -> Option<&ProjectScenario> {
        self.active_scenario_id
            .as_ref()
            .and_then(|id| self.scenarios.get(id))
    }

    pub fn get_baseline_scenario(&self) -> Option<&ProjectScenario> {
        self.baseline_scenario_id
            .as_ref()
            .and_then(|id| self.scenarios.get(id))
    }

    pub fn compare_scenarios(&self, scenario1_id: &str, scenario2_id: &str) -> Option<ScenarioComparison> {
        let scenario1 = self.scenarios.get(scenario1_id)?;
        let scenario2 = self.scenarios.get(scenario2_id)?;

        Some(ScenarioComparison::new(scenario1, scenario2))
    }

    pub fn list_scenarios_by_type(&self, scenario_type: ScenarioType) -> Vec<&ProjectScenario> {
        self.scenarios
            .values()
            .filter(|s| s.scenario_type == scenario_type)
            .collect()
    }

    pub fn remove_scenario(&mut self, scenario_id: &str) -> Option<ProjectScenario> {
        // Don't allow removing the baseline scenario
        if self.baseline_scenario_id.as_ref() == Some(&scenario_id.to_string()) {
            return None;
        }

        // If removing the active scenario, set another as active
        if self.active_scenario_id.as_ref() == Some(&scenario_id.to_string()) {
            self.active_scenario_id = self.scenarios
                .keys()
                .find(|id| *id != scenario_id)
                .map(|id| id.clone());
        }

        self.scenarios.remove(scenario_id)
    }
}

/// Represents a comparison between two scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioComparison {
    pub scenario1_id: String,
    pub scenario1_name: String,
    pub scenario2_id: String,
    pub scenario2_name: String,
    
    // Date comparisons
    pub start_date_difference: Option<i32>,
    pub end_date_difference: Option<i32>,
    pub duration_difference: Option<i32>,
    
    // Status comparisons
    pub status_different: bool,
    pub priority_different: bool,
    
    // Task comparisons
    pub task_count_difference: i32,
    pub new_tasks: Vec<String>,
    pub removed_tasks: Vec<String>,
    pub modified_tasks: Vec<String>,
    
    // Resource assignment comparisons
    pub resource_assignment_differences: i32,
    
    // Summary
    pub is_delayed: bool,
    pub delay_days: Option<i32>,
    pub is_ahead_of_schedule: bool,
    pub ahead_days: Option<i32>,
}

impl ScenarioComparison {
    pub fn new(scenario1: &ProjectScenario, scenario2: &ProjectScenario) -> Self {
        let start_date_diff = if let (Some(s1), Some(s2)) = (scenario1.start_date, scenario2.start_date) {
            Some((s2 - s1).num_days() as i32)
        } else {
            None
        };

        let end_date_diff = if let (Some(e1), Some(e2)) = (scenario2.end_date, scenario2.end_date) {
            Some((e2 - e1).num_days() as i32)
        } else {
            None
        };

        let duration_diff = if let (Some(d1), Some(d2)) = (scenario1.duration_days(), scenario2.duration_days()) {
            Some(d2 as i32 - d1 as i32)
        } else {
            None
        };

        // Find task differences
        let mut new_tasks = Vec::new();
        let mut removed_tasks = Vec::new();
        let mut modified_tasks = Vec::new();

        for (task_id, task2) in &scenario2.tasks {
            if !scenario1.tasks.contains_key(task_id) {
                new_tasks.push(task_id.clone());
            } else if let Some(task1) = scenario1.tasks.get(task_id) {
                if task1.status != task2.status || 
                   task1.start_date != task2.start_date || 
                   task1.end_date != task2.end_date {
                    modified_tasks.push(task_id.clone());
                }
            }
        }

        for task_id in scenario1.tasks.keys() {
            if !scenario2.tasks.contains_key(task_id) {
                removed_tasks.push(task_id.clone());
            }
        }

        let task_count_diff = scenario2.tasks.len() as i32 - scenario1.tasks.len() as i32;
        let resource_assignment_diff = scenario2.resource_assignments.len() as i32 - scenario1.resource_assignments.len() as i32;

        let is_delayed = if let (Some(e1), Some(e2)) = (scenario1.end_date, scenario2.end_date) {
            e2 > e1
        } else {
            false
        };

        let delay_days = if is_delayed {
            if let (Some(e1), Some(e2)) = (scenario1.end_date, scenario2.end_date) {
                Some((e2 - e1).num_days() as i32)
            } else {
                None
            }
        } else {
            None
        };

        let is_ahead = if let (Some(e1), Some(e2)) = (scenario1.end_date, scenario2.end_date) {
            e2 < e1
        } else {
            false
        };

        let ahead_days = if is_ahead {
            if let (Some(e1), Some(e2)) = (scenario1.end_date, scenario2.end_date) {
                Some((e1 - e2).num_days() as i32)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            scenario1_id: scenario1.id.clone(),
            scenario1_name: scenario1.name.clone(),
            scenario2_id: scenario2.id.clone(),
            scenario2_name: scenario2.name.clone(),
            start_date_difference: start_date_diff,
            end_date_difference: end_date_diff,
            duration_difference: duration_diff,
            status_different: scenario1.status != scenario2.status,
            priority_different: scenario1.priority != scenario2.priority,
            task_count_difference: task_count_diff,
            new_tasks,
            removed_tasks,
            modified_tasks,
            resource_assignment_differences: resource_assignment_diff,
            is_delayed,
            delay_days,
            is_ahead_of_schedule: is_ahead,
            ahead_days,
        }
    }

    pub fn has_significant_changes(&self) -> bool {
        self.start_date_difference.is_some() ||
        self.end_date_difference.is_some() ||
        self.duration_difference.is_some() ||
        self.status_different ||
        self.priority_different ||
        self.task_count_difference != 0 ||
        !self.new_tasks.is_empty() ||
        !self.removed_tasks.is_empty() ||
        !self.modified_tasks.is_empty() ||
        self.resource_assignment_differences != 0
    }

    pub fn summary(&self) -> String {
        let mut summary = Vec::new();

        if let Some(diff) = self.start_date_difference {
            if diff > 0 {
                summary.push(format!("Started {} days later", diff));
            } else if diff < 0 {
                summary.push(format!("Started {} days earlier", -diff));
            }
        }

        if let Some(diff) = self.end_date_difference {
            if diff > 0 {
                summary.push(format!("Ends {} days later", diff));
            } else if diff < 0 {
                summary.push(format!("Ends {} days earlier", -diff));
            }
        }

        if self.task_count_difference > 0 {
            summary.push(format!("Added {} tasks", self.task_count_difference));
        } else if self.task_count_difference < 0 {
            summary.push(format!("Removed {} tasks", -self.task_count_difference));
        }

        if !self.new_tasks.is_empty() {
            summary.push(format!("{} new tasks", self.new_tasks.len()));
        }

        if !self.removed_tasks.is_empty() {
            summary.push(format!("{} removed tasks", self.removed_tasks.len()));
        }

        if !self.modified_tasks.is_empty() {
            summary.push(format!("{} modified tasks", self.modified_tasks.len()));
        }

        if summary.is_empty() {
            "No significant changes".to_string()
        } else {
            summary.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creation() {
        let scenario = ProjectScenario::new(
            "Baseline".to_string(),
            Some("Original plan".to_string()),
            ScenarioType::Baseline,
            "proj-1".to_string(),
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "user@example.com".to_string(),
        );

        assert_eq!(scenario.name, "Baseline");
        assert_eq!(scenario.scenario_type, ScenarioType::Baseline);
        assert_eq!(scenario.project_id, "proj-1");
        assert!(scenario.is_baseline());
        assert!(!scenario.is_current());
    }

    #[test]
    fn test_scenario_manager() {
        let mut manager = ScenarioManager::new("proj-1".to_string(), "PROJ-001".to_string());
        
        let baseline = ProjectScenario::new(
            "Baseline".to_string(),
            None,
            ScenarioType::Baseline,
            "proj-1".to_string(),
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "user@example.com".to_string(),
        );

        manager.add_scenario(baseline);
        
        assert_eq!(manager.scenarios.len(), 1);
        assert!(manager.baseline_scenario_id.is_some());
        assert!(manager.active_scenario_id.is_some());
    }

    #[test]
    fn test_scenario_comparison() {
        let mut scenario1 = ProjectScenario::new(
            "Baseline".to_string(),
            None,
            ScenarioType::Baseline,
            "proj-1".to_string(),
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "user@example.com".to_string(),
        );

        let mut scenario2 = ProjectScenario::new(
            "Current".to_string(),
            None,
            ScenarioType::Current,
            "proj-1".to_string(),
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "user@example.com".to_string(),
        );

        scenario1.update_dates(
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()),
        );

        scenario2.update_dates(
            Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 4, 5).unwrap()),
        );

        let comparison = ScenarioComparison::new(&scenario1, &scenario2);
        
        assert_eq!(comparison.start_date_difference, Some(4));
        assert_eq!(comparison.end_date_difference, Some(5));
        assert!(comparison.has_significant_changes());
    }
}
