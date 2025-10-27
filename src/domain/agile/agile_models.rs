use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid7::Uuid;

/// Sprint represents a time-boxed iteration in Scrum methodology
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub project_id: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub capacity: SprintCapacity,
    pub goals: Vec<String>,
    pub status: SprintStatus,
    pub tasks: Vec<String>, // Task IDs
    pub metrics: SprintMetrics,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sprint capacity configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SprintCapacity {
    pub total_story_points: u32,
    pub team_members: Vec<String>, // Resource IDs
    pub working_days: Vec<Weekday>,
    pub hours_per_day: f64,
    pub availability_percentage: f64, // 0.0 to 1.0
}

/// Sprint status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SprintStatus {
    Planning,
    Active,
    Review,
    Retrospective,
    Completed,
    Cancelled,
}

/// Sprint metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SprintMetrics {
    pub planned_story_points: u32,
    pub completed_story_points: u32,
    pub remaining_story_points: u32,
    pub velocity: f64,
    pub burndown_data: Vec<BurndownDataPoint>,
    pub team_velocity: f64,
    pub sprint_goal_completion: f64, // 0.0 to 1.0
}

/// Burndown data point for chart visualization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurndownDataPoint {
    pub date: DateTime<Utc>,
    pub remaining_story_points: u32,
    pub ideal_remaining: u32,
    pub completed_story_points: u32,
}

/// Kanban board configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub id: String,
    pub name: String,
    pub project_id: String,
    pub columns: Vec<KanbanColumn>,
    pub wip_limits: HashMap<String, u32>, // Column ID -> WIP limit
    pub flow_metrics: FlowMetrics,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Kanban column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanbanColumn {
    pub id: String,
    pub name: String,
    pub position: u32,
    pub wip_limit: Option<u32>,
    pub tasks: Vec<String>, // Task IDs
    pub color: Option<String>,
    pub description: Option<String>,
}

/// Flow metrics for Kanban
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowMetrics {
    pub cycle_time: Duration,
    pub lead_time: Duration,
    pub throughput: f64, // Tasks per day
    pub flow_efficiency: f64, // 0.0 to 1.0
    pub bottleneck_columns: Vec<String>,
    pub average_wip: f64,
}

/// Velocity tracking data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityData {
    pub team_id: String,
    pub sprint_id: String,
    pub story_points_completed: u32,
    pub story_points_committed: u32,
    pub velocity: f64,
    pub sprint_duration_days: u32,
    pub team_members: Vec<String>,
    pub completion_date: DateTime<Utc>,
}

/// Velocity trend analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VelocityTrend {
    pub team_id: String,
    pub sprints: Vec<VelocityData>,
    pub average_velocity: f64,
    pub velocity_trend: VelocityTrendDirection,
    pub volatility: f64, // Standard deviation
    pub prediction_next_sprint: f64,
}

/// Velocity trend direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VelocityTrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Agile metrics aggregation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgileMetrics {
    pub project_id: String,
    pub team_id: String,
    pub sprint_count: u32,
    pub total_story_points: u32,
    pub completed_story_points: u32,
    pub average_velocity: f64,
    pub sprint_success_rate: f64, // 0.0 to 1.0
    pub goal_completion_rate: f64, // 0.0 to 1.0
    pub cycle_time_avg: Duration,
    pub lead_time_avg: Duration,
    pub flow_efficiency: f64,
    pub team_satisfaction: Option<f64>, // 1.0 to 5.0
    pub last_updated: DateTime<Utc>,
}

/// Daily standup data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyStandup {
    pub id: String,
    pub sprint_id: String,
    pub date: DateTime<Utc>,
    pub team_member_id: String,
    pub yesterday_completed: Vec<String>, // Task IDs
    pub today_planned: Vec<String>, // Task IDs
    pub blockers: Vec<String>,
    pub notes: Option<String>,
}

/// Sprint retrospective data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SprintRetrospective {
    pub id: String,
    pub sprint_id: String,
    pub date: DateTime<Utc>,
    pub what_went_well: Vec<String>,
    pub what_could_improve: Vec<String>,
    pub action_items: Vec<ActionItem>,
    pub team_satisfaction: f64, // 1.0 to 5.0
    pub sprint_rating: f64, // 1.0 to 5.0
}

/// Action item from retrospective
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub owner: String, // Resource ID
    pub due_date: Option<DateTime<Utc>>,
    pub status: ActionItemStatus,
    pub priority: ActionItemPriority,
}

/// Action item status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionItemStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
}

/// Action item priority
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionItemPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Weekday enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Sprint {
    /// Create a new sprint
    pub fn new(
        name: String,
        project_id: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        capacity: SprintCapacity,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ).to_string(),
            name,
            description: None,
            project_id,
            start_date,
            end_date,
            capacity,
            goals: Vec::new(),
            status: SprintStatus::Planning,
            tasks: Vec::new(),
            metrics: SprintMetrics::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Get sprint duration in days
    pub fn duration_days(&self) -> u32 {
        (self.end_date - self.start_date).num_days() as u32
    }

    /// Check if sprint is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, SprintStatus::Active)
    }

    /// Check if sprint is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, SprintStatus::Completed)
    }

    /// Add task to sprint
    pub fn add_task(&mut self, task_id: String) {
        if !self.tasks.contains(&task_id) {
            self.tasks.push(task_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove task from sprint
    pub fn remove_task(&mut self, task_id: &str) {
        self.tasks.retain(|id| id != task_id);
        self.updated_at = Utc::now();
    }

    /// Update sprint status
    pub fn update_status(&mut self, status: SprintStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

impl Default for SprintMetrics {
    fn default() -> Self {
        Self {
            planned_story_points: 0,
            completed_story_points: 0,
            remaining_story_points: 0,
            velocity: 0.0,
            burndown_data: Vec::new(),
            team_velocity: 0.0,
            sprint_goal_completion: 0.0,
        }
    }
}

impl KanbanBoard {
    /// Create a new Kanban board
    pub fn new(name: String, project_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ).to_string(),
            name,
            project_id,
            columns: Vec::new(),
            wip_limits: HashMap::new(),
            flow_metrics: FlowMetrics::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add column to board
    pub fn add_column(&mut self, column: KanbanColumn) {
        self.columns.push(column);
        self.columns.sort_by_key(|c| c.position);
        self.updated_at = Utc::now();
    }

    /// Remove column from board
    pub fn remove_column(&mut self, column_id: &str) {
        self.columns.retain(|c| c.id != column_id);
        self.wip_limits.remove(column_id);
        self.updated_at = Utc::now();
    }

    /// Move task between columns
    pub fn move_task(&mut self, task_id: &str, from_column: &str, to_column: &str) -> Result<(), String> {
        // Find source column
        if let Some(source_col) = self.columns.iter_mut().find(|c| c.id == from_column) {
            source_col.tasks.retain(|id| id != task_id);
        } else {
            return Err(format!("Source column {} not found", from_column));
        }

        // Find target column
        if let Some(target_col) = self.columns.iter_mut().find(|c| c.id == to_column) {
            target_col.tasks.push(task_id.to_string());
        } else {
            return Err(format!("Target column {} not found", to_column));
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check WIP limits
    pub fn check_wip_limits(&self) -> Vec<String> {
        let mut violations = Vec::new();
        
        for column in &self.columns {
            if let Some(limit) = self.wip_limits.get(&column.id) {
                if column.tasks.len() as u32 > *limit {
                    violations.push(format!("Column {} exceeds WIP limit: {}/{}", 
                        column.name, column.tasks.len(), limit));
                }
            }
        }
        
        violations
    }
}

impl Default for FlowMetrics {
    fn default() -> Self {
        Self {
            cycle_time: Duration::zero(),
            lead_time: Duration::zero(),
            throughput: 0.0,
            flow_efficiency: 0.0,
            bottleneck_columns: Vec::new(),
            average_wip: 0.0,
        }
    }
}

impl VelocityData {
    /// Create new velocity data
    pub fn new(
        team_id: String,
        sprint_id: String,
        story_points_completed: u32,
        story_points_committed: u32,
        sprint_duration_days: u32,
        team_members: Vec<String>,
    ) -> Self {
        let velocity = if sprint_duration_days > 0 {
            story_points_completed as f64 / sprint_duration_days as f64
        } else {
            0.0
        };

        Self {
            team_id,
            sprint_id,
            story_points_completed,
            story_points_committed,
            velocity,
            sprint_duration_days,
            team_members,
            completion_date: Utc::now(),
        }
    }

    /// Calculate velocity per team member
    pub fn velocity_per_member(&self) -> f64 {
        if self.team_members.is_empty() {
            0.0
        } else {
            self.velocity / self.team_members.len() as f64
        }
    }
}

impl VelocityTrend {
    /// Calculate trend direction
    pub fn calculate_trend_direction(&mut self) {
        if self.sprints.len() < 2 {
            self.velocity_trend = VelocityTrendDirection::Stable;
            return;
        }

        let recent_velocities: Vec<f64> = self.sprints.iter()
            .map(|s| s.velocity)
            .collect();

        // Calculate trend using linear regression
        let n = recent_velocities.len() as f64;
        let sum_x: f64 = (0..recent_velocities.len()).sum::<usize>() as f64;
        let sum_y: f64 = recent_velocities.iter().sum();
        let sum_xy: f64 = recent_velocities.iter().enumerate()
            .map(|(i, &y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..recent_velocities.len())
            .map(|i| (i as f64).powi(2))
            .sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        
        // Calculate volatility (standard deviation)
        let mean = sum_y / n;
        let variance: f64 = recent_velocities.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / n;
        self.volatility = variance.sqrt();

        // Determine trend direction
        if self.volatility > mean * 0.3 {
            self.velocity_trend = VelocityTrendDirection::Volatile;
        } else if slope > 0.1 {
            self.velocity_trend = VelocityTrendDirection::Increasing;
        } else if slope < -0.1 {
            self.velocity_trend = VelocityTrendDirection::Decreasing;
        } else {
            self.velocity_trend = VelocityTrendDirection::Stable;
        }

        // Predict next sprint velocity
        self.prediction_next_sprint = mean + slope * n;
    }
}

impl DailyStandup {
    /// Create new daily standup
    pub fn new(sprint_id: String, team_member_id: String, date: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ).to_string(),
            sprint_id,
            date,
            team_member_id,
            yesterday_completed: Vec::new(),
            today_planned: Vec::new(),
            blockers: Vec::new(),
            notes: None,
        }
    }
}

impl SprintRetrospective {
    /// Create new sprint retrospective
    pub fn new(sprint_id: String, date: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ).to_string(),
            sprint_id,
            date,
            what_went_well: Vec::new(),
            what_could_improve: Vec::new(),
            action_items: Vec::new(),
            team_satisfaction: 3.0, // Default neutral
            sprint_rating: 3.0, // Default neutral
        }
    }
}

impl ActionItem {
    /// Create new action item
    pub fn new(description: String, owner: String, priority: ActionItemPriority) -> Self {
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ).to_string(),
            description,
            owner,
            due_date: None,
            status: ActionItemStatus::Open,
            priority,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprint_creation() {
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string(), "dev2".to_string()],
            working_days: vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
            hours_per_day: 8.0,
            availability_percentage: 0.8,
        };

        let sprint = Sprint::new(
            "Sprint 1".to_string(),
            "proj1".to_string(),
            start_date,
            end_date,
            capacity,
        );

        assert_eq!(sprint.name, "Sprint 1");
        assert_eq!(sprint.project_id, "proj1");
        assert_eq!(sprint.duration_days(), 14);
        assert!(matches!(sprint.status, SprintStatus::Planning));
    }

    #[test]
    fn test_kanban_board_operations() {
        let mut board = KanbanBoard::new("Test Board".to_string(), "proj1".to_string());
        
        let column = KanbanColumn {
            id: "col1".to_string(),
            name: "To Do".to_string(),
            position: 1,
            wip_limit: Some(5),
            tasks: vec!["task1".to_string()],
            color: Some("#blue".to_string()),
            description: Some("Tasks to be done".to_string()),
        };

        board.add_column(column);
        assert_eq!(board.columns.len(), 1);

        // Test WIP limit check
        let violations = board.check_wip_limits();
        assert!(violations.is_empty()); // 1 task < 5 limit
    }

    #[test]
    fn test_velocity_data_calculation() {
        let velocity_data = VelocityData::new(
            "team1".to_string(),
            "sprint1".to_string(),
            20,
            25,
            14,
            vec!["dev1".to_string(), "dev2".to_string()],
        );

        assert_eq!(velocity_data.velocity, 20.0 / 14.0);
        assert_eq!(velocity_data.velocity_per_member(), velocity_data.velocity / 2.0);
    }

    #[test]
    fn test_velocity_trend_calculation() {
        let mut trend = VelocityTrend {
            team_id: "team1".to_string(),
            sprints: vec![
                VelocityData::new("team1".to_string(), "sprint1".to_string(), 20, 25, 14, vec!["dev1".to_string()]),
                VelocityData::new("team1".to_string(), "sprint2".to_string(), 25, 30, 14, vec!["dev1".to_string()]),
                VelocityData::new("team1".to_string(), "sprint3".to_string(), 30, 35, 14, vec!["dev1".to_string()]),
            ],
            average_velocity: 0.0,
            velocity_trend: VelocityTrendDirection::Stable,
            volatility: 0.0,
            prediction_next_sprint: 0.0,
        };

        trend.calculate_trend_direction();
        assert!(matches!(trend.velocity_trend, VelocityTrendDirection::Increasing));
    }
}
