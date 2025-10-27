use crate::domain::agile::agile_models::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// SprintManager handles sprint lifecycle management
#[derive(Debug, Clone)]
pub struct SprintManager {
    sprints: HashMap<String, Sprint>,
    velocity_history: HashMap<String, Vec<VelocityData>>,
}

impl SprintManager {
    /// Create a new SprintManager
    pub fn new() -> Self {
        Self {
            sprints: HashMap::new(),
            velocity_history: HashMap::new(),
        }
    }

    /// Create a new sprint
    pub fn create_sprint(
        &mut self,
        name: String,
        project_id: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        capacity: SprintCapacity,
    ) -> Result<String, String> {
        // Validate sprint dates
        if end_date <= start_date {
            return Err("End date must be after start date".to_string());
        }

        // Check for overlapping sprints in the same project
        for sprint in self.sprints.values() {
            if sprint.project_id == project_id && sprint.is_active() {
                if (start_date >= sprint.start_date && start_date <= sprint.end_date) ||
                   (end_date >= sprint.start_date && end_date <= sprint.end_date) {
                    return Err(format!("Sprint overlaps with existing active sprint: {}", sprint.name));
                }
            }
        }

        let sprint = Sprint::new(name.clone(), project_id, start_date, end_date, capacity);
        let sprint_id = sprint.id.clone();
        self.sprints.insert(sprint_id.clone(), sprint);
        
        Ok(sprint_id)
    }

    /// Get sprint by ID
    pub fn get_sprint(&self, sprint_id: &str) -> Option<&Sprint> {
        self.sprints.get(sprint_id)
    }

    /// Get sprint by ID (mutable)
    pub fn get_sprint_mut(&mut self, sprint_id: &str) -> Option<&mut Sprint> {
        self.sprints.get_mut(sprint_id)
    }

    /// List all sprints for a project
    pub fn list_sprints(&self, project_id: &str) -> Vec<&Sprint> {
        self.sprints.values()
            .filter(|sprint| sprint.project_id == project_id)
            .collect()
    }

    /// List active sprints
    pub fn list_active_sprints(&self) -> Vec<&Sprint> {
        self.sprints.values()
            .filter(|sprint| sprint.is_active())
            .collect()
    }

    /// Update sprint status
    pub fn update_sprint_status(&mut self, sprint_id: &str, status: SprintStatus) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.update_status(status);
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Add task to sprint
    pub fn add_task_to_sprint(&mut self, sprint_id: &str, task_id: String) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.add_task(task_id);
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Remove task from sprint
    pub fn remove_task_from_sprint(&mut self, sprint_id: &str, task_id: &str) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.remove_task(task_id);
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Plan sprint capacity
    pub fn plan_sprint_capacity(&mut self, sprint_id: &str, capacity: SprintCapacity) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.capacity = capacity;
            sprint.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Add sprint goal
    pub fn add_sprint_goal(&mut self, sprint_id: &str, goal: String) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.goals.push(goal);
            sprint.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Remove sprint goal
    pub fn remove_sprint_goal(&mut self, sprint_id: &str, goal_index: usize) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            if goal_index < sprint.goals.len() {
                sprint.goals.remove(goal_index);
                sprint.updated_at = Utc::now();
                Ok(())
            } else {
                Err("Goal index out of range".to_string())
            }
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Start sprint
    pub fn start_sprint(&mut self, sprint_id: &str) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            if matches!(sprint.status, SprintStatus::Planning) {
                sprint.update_status(SprintStatus::Active);
                Ok(())
            } else {
                Err("Sprint must be in Planning status to start".to_string())
            }
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Complete sprint
    pub fn complete_sprint(&mut self, sprint_id: &str) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            if matches!(sprint.status, SprintStatus::Active) {
                sprint.update_status(SprintStatus::Completed);
                
                // Record velocity data
                let velocity_data = VelocityData::new(
                    sprint.capacity.team_members.join(","),
                    sprint_id.to_string(),
                    sprint.metrics.completed_story_points,
                    sprint.metrics.planned_story_points,
                    sprint.duration_days(),
                    sprint.capacity.team_members.clone(),
                );

                let team_key = sprint.capacity.team_members.join(",");
                self.velocity_history.entry(team_key).or_insert_with(Vec::new).push(velocity_data);
                
                Ok(())
            } else {
                Err("Sprint must be Active to complete".to_string())
            }
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Update sprint metrics
    pub fn update_sprint_metrics(&mut self, sprint_id: &str, metrics: SprintMetrics) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.metrics = metrics;
            sprint.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Add burndown data point
    pub fn add_burndown_point(&mut self, sprint_id: &str, data_point: BurndownDataPoint) -> Result<(), String> {
        if let Some(sprint) = self.sprints.get_mut(sprint_id) {
            sprint.metrics.burndown_data.push(data_point);
            sprint.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Get velocity history for team
    pub fn get_velocity_history(&self, team_id: &str) -> Option<&Vec<VelocityData>> {
        self.velocity_history.get(team_id)
    }

    /// Calculate team velocity trend
    pub fn calculate_velocity_trend(&self, team_id: &str) -> Option<VelocityTrend> {
        if let Some(velocity_data) = self.velocity_history.get(team_id) {
            let mut trend = VelocityTrend {
                team_id: team_id.to_string(),
                sprints: velocity_data.clone(),
                average_velocity: 0.0,
                velocity_trend: VelocityTrendDirection::Stable,
                volatility: 0.0,
                prediction_next_sprint: 0.0,
            };

            // Calculate average velocity
            if !velocity_data.is_empty() {
                trend.average_velocity = velocity_data.iter()
                    .map(|v| v.velocity)
                    .sum::<f64>() / velocity_data.len() as f64;
            }

            trend.calculate_trend_direction();
            Some(trend)
        } else {
            None
        }
    }

    /// Get sprint progress percentage
    pub fn get_sprint_progress(&self, sprint_id: &str) -> Result<f64, String> {
        if let Some(sprint) = self.sprints.get(sprint_id) {
            if sprint.metrics.planned_story_points == 0 {
                return Ok(0.0);
            }

            let progress = sprint.metrics.completed_story_points as f64 / sprint.metrics.planned_story_points as f64;
            Ok(progress.min(1.0))
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Get sprint health status
    pub fn get_sprint_health(&self, sprint_id: &str) -> Result<SprintHealth, String> {
        if let Some(sprint) = self.sprints.get(sprint_id) {
            let progress = self.get_sprint_progress(sprint_id)?;
            let days_remaining = (sprint.end_date - Utc::now()).num_days();
            let total_days = sprint.duration_days() as i64;
            let days_elapsed = total_days - days_remaining;
            let expected_progress = if total_days > 0 {
                days_elapsed as f64 / total_days as f64
            } else {
                0.0
            };

            let health = if progress >= expected_progress * 0.9 {
                SprintHealth::OnTrack
            } else if progress >= expected_progress * 0.7 {
                SprintHealth::AtRisk
            } else {
                SprintHealth::Behind
            };

            Ok(health)
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Delete sprint
    pub fn delete_sprint(&mut self, sprint_id: &str) -> Result<(), String> {
        if self.sprints.remove(sprint_id).is_some() {
            Ok(())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Get all sprints
    pub fn get_all_sprints(&self) -> Vec<&Sprint> {
        self.sprints.values().collect()
    }

    /// Get sprint statistics
    pub fn get_sprint_statistics(&self, project_id: Option<&str>) -> SprintStatistics {
        let sprints: Vec<&Sprint> = if let Some(pid) = project_id {
            self.sprints.values().filter(|s| s.project_id == *pid).collect()
        } else {
            self.sprints.values().collect()
        };

        let total_sprints = sprints.len();
        let completed_sprints = sprints.iter().filter(|s| s.is_completed()).count();
        let active_sprints = sprints.iter().filter(|s| s.is_active()).count();
        
        let total_story_points = sprints.iter()
            .map(|s| s.metrics.planned_story_points)
            .sum();
        
        let completed_story_points = sprints.iter()
            .map(|s| s.metrics.completed_story_points)
            .sum();

        let average_velocity = if completed_sprints > 0 {
            sprints.iter()
                .filter(|s| s.is_completed())
                .map(|s| s.metrics.velocity)
                .sum::<f64>() / completed_sprints as f64
        } else {
            0.0
        };

        SprintStatistics {
            total_sprints,
            completed_sprints,
            active_sprints,
            total_story_points,
            completed_story_points,
            average_velocity,
            completion_rate: if total_sprints > 0 {
                completed_sprints as f64 / total_sprints as f64
            } else {
                0.0
            },
        }
    }
}

/// Sprint health status
#[derive(Debug, Clone, PartialEq)]
pub enum SprintHealth {
    OnTrack,
    AtRisk,
    Behind,
}

/// Sprint statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct SprintStatistics {
    pub total_sprints: usize,
    pub completed_sprints: usize,
    pub active_sprints: usize,
    pub total_story_points: u32,
    pub completed_story_points: u32,
    pub average_velocity: f64,
    pub completion_rate: f64,
}

impl Default for SprintManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprint_manager_creation() {
        let manager = SprintManager::new();
        assert_eq!(manager.sprints.len(), 0);
    }

    #[test]
    fn test_create_sprint() {
        let mut manager = SprintManager::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string()],
            working_days: vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
            hours_per_day: 8.0,
            availability_percentage: 0.8,
        };

        let result = manager.create_sprint(
            "Sprint 1".to_string(),
            "proj1".to_string(),
            start_date,
            end_date,
            capacity,
        );

        assert!(result.is_ok());
        let sprint_id = result.unwrap();
        assert!(manager.get_sprint(&sprint_id).is_some());
    }

    #[test]
    fn test_sprint_lifecycle() {
        let mut manager = SprintManager::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string()],
            working_days: vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
            hours_per_day: 8.0,
            availability_percentage: 0.8,
        };

        let sprint_id = manager.create_sprint(
            "Sprint 1".to_string(),
            "proj1".to_string(),
            start_date,
            end_date,
            capacity,
        ).unwrap();

        // Start sprint
        assert!(manager.start_sprint(&sprint_id).is_ok());
        let sprint = manager.get_sprint(&sprint_id).unwrap();
        assert!(matches!(sprint.status, SprintStatus::Active));

        // Complete sprint
        assert!(manager.complete_sprint(&sprint_id).is_ok());
        let sprint = manager.get_sprint(&sprint_id).unwrap();
        assert!(matches!(sprint.status, SprintStatus::Completed));
    }

    #[test]
    fn test_sprint_statistics() {
        let mut manager = SprintManager::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string()],
            working_days: vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
            hours_per_day: 8.0,
            availability_percentage: 0.8,
        };

        let sprint_id = manager.create_sprint(
            "Sprint 1".to_string(),
            "proj1".to_string(),
            start_date,
            end_date,
            capacity,
        ).unwrap();

        let stats = manager.get_sprint_statistics(Some("proj1"));
        assert_eq!(stats.total_sprints, 1);
        assert_eq!(stats.active_sprints, 0); // Not started yet
    }
}
