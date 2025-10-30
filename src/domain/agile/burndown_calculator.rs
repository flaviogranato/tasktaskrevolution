use crate::domain::agile::agile_models::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// BurndownCalculator handles burndown chart calculations and analysis
#[derive(Debug, Clone)]
pub struct BurndownCalculator {
    sprint_data: HashMap<String, Sprint>,
}

impl BurndownCalculator {
    /// Create a new BurndownCalculator
    pub fn new() -> Self {
        Self {
            sprint_data: HashMap::new(),
        }
    }

    /// Add sprint data for calculation
    pub fn add_sprint(&mut self, sprint: Sprint) {
        self.sprint_data.insert(sprint.id.clone(), sprint);
    }

    /// Calculate ideal burndown line
    pub fn calculate_ideal_burndown(
        &self,
        sprint_id: &str,
        total_story_points: u32,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Vec<BurndownDataPoint> {
        let mut ideal_points = Vec::new();
        let total_days = (end_date - start_date).num_days() as u32;
        
        if total_days == 0 {
            return ideal_points;
        }

        let daily_burn = total_story_points as f64 / total_days as f64;
        let mut current_date = start_date;
        let mut remaining_points = total_story_points as f64;

        while current_date <= end_date {
            ideal_points.push(BurndownDataPoint {
                date: current_date,
                remaining_story_points: remaining_points as u32,
                ideal_remaining: remaining_points as u32,
                completed_story_points: (total_story_points as f64 - remaining_points) as u32,
            });

            remaining_points -= daily_burn;
            current_date += Duration::days(1);
        }

        ideal_points
    }

    /// Calculate actual burndown from sprint data
    pub fn calculate_actual_burndown(&self, sprint_id: &str) -> Result<Vec<BurndownDataPoint>, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            Ok(sprint.metrics.burndown_data.clone())
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Generate burndown data points for a sprint
    pub fn generate_burndown_data(
        &self,
        sprint_id: &str,
        daily_completions: Vec<DailyCompletion>,
    ) -> Result<Vec<BurndownDataPoint>, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            let mut burndown_points = Vec::new();
            let total_story_points = sprint.metrics.planned_story_points;
            let mut completed_points = 0u32;
            let mut current_date = sprint.start_date;

            // Sort daily completions by date
            let mut sorted_completions = daily_completions;
            sorted_completions.sort_by_key(|d| d.date);

            // Generate ideal burndown for comparison
            let ideal_burndown = self.calculate_ideal_burndown(
                sprint_id,
                total_story_points,
                sprint.start_date,
                sprint.end_date,
            );

            let mut completion_index = 0;

            while current_date <= sprint.end_date {
                // Check if there are completions for this date
                while completion_index < sorted_completions.len() 
                    && sorted_completions[completion_index].date.date_naive() == current_date.date_naive() {
                    completed_points += sorted_completions[completion_index].story_points;
                    completion_index += 1;
                }

                let remaining_points = total_story_points.saturating_sub(completed_points);
                
                // Find corresponding ideal point
                let ideal_remaining = ideal_burndown.iter()
                    .find(|p| p.date.date_naive() == current_date.date_naive())
                    .map(|p| p.ideal_remaining)
                    .unwrap_or(0);

                burndown_points.push(BurndownDataPoint {
                    date: current_date,
                    remaining_story_points: remaining_points,
                    ideal_remaining,
                    completed_story_points: completed_points,
                });

                current_date += Duration::days(1);
            }

            Ok(burndown_points)
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Calculate burndown velocity
    pub fn calculate_burndown_velocity(&self, sprint_id: &str) -> Result<BurndownVelocity, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            let burndown_data = &sprint.metrics.burndown_data;
            
            if burndown_data.len() < 2 {
                return Ok(BurndownVelocity::default());
            }

            // Prefer planned points; fallback to capacity if not set in metrics
            let mut total_story_points = sprint.metrics.planned_story_points as f64;
            if total_story_points == 0.0 {
                total_story_points = sprint.capacity.total_story_points as f64;
            }
            let total_days = sprint.duration_days() as f64;
            
            // Calculate ideal velocity (story points per day)
            let ideal_velocity = if total_days > 0.0 {
                total_story_points / total_days
            } else {
                0.0
            };

            // Calculate actual velocity from burndown data
            let first_point = burndown_data.first().unwrap();
            let last_point = burndown_data.last().unwrap();
            let actual_days = (last_point.date - first_point.date).num_days() as f64;
            
            let actual_velocity = if actual_days > 0.0 {
                (first_point.remaining_story_points as f64 - last_point.remaining_story_points as f64) / actual_days
            } else {
                0.0
            };

            // Calculate variance
            let variance = actual_velocity - ideal_velocity;
            let variance_percentage = if ideal_velocity > 0.0 {
                (variance / ideal_velocity) * 100.0
            } else {
                0.0
            };

            // Determine trend
            let trend = if variance_percentage > 10.0 {
                BurndownTrend::Ahead
            } else if variance_percentage < -10.0 {
                BurndownTrend::Behind
            } else {
                BurndownTrend::OnTrack
            };

            // Calculate completion rate
            let completion_rate = if total_story_points > 0.0 {
                (total_story_points - last_point.remaining_story_points as f64) / total_story_points
            } else {
                0.0
            };

            Ok(BurndownVelocity {
                ideal_velocity,
                actual_velocity,
                variance,
                variance_percentage,
                trend,
                completion_rate,
                total_story_points: total_story_points as u32,
                remaining_story_points: last_point.remaining_story_points,
            })
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Predict sprint completion
    pub fn predict_sprint_completion(&self, sprint_id: &str) -> Result<SprintPrediction, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            let burndown_data = &sprint.metrics.burndown_data;
            
            if burndown_data.len() < 2 {
                return Ok(SprintPrediction::default());
            }

            let velocity = self.calculate_burndown_velocity(sprint_id)?;
            let current_date = Utc::now();
            let remaining_points = velocity.remaining_story_points as f64;

            // Predict completion date based on current velocity
            let predicted_completion_days = if velocity.actual_velocity > 0.0 {
                remaining_points / velocity.actual_velocity
            } else {
                f64::INFINITY
            };

            let predicted_completion_date = current_date + Duration::days(predicted_completion_days as i64);
            
            // Check if prediction is within sprint bounds
            let on_time = predicted_completion_date <= sprint.end_date;
            let days_ahead_behind = if on_time {
                (sprint.end_date - predicted_completion_date).num_days()
            } else {
                -(predicted_completion_date - sprint.end_date).num_days()
            };

            // Calculate confidence based on data points
            let confidence = if burndown_data.len() >= 5 {
                0.8
            } else if burndown_data.len() >= 3 {
                0.6
            } else {
                0.4
            };

            Ok(SprintPrediction {
                predicted_completion_date,
                on_time,
                days_ahead_behind,
                confidence,
                remaining_story_points: velocity.remaining_story_points,
                current_velocity: velocity.actual_velocity,
            })
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Calculate sprint health score
    pub fn calculate_sprint_health(&self, sprint_id: &str) -> Result<SprintHealthScore, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            let velocity = self.calculate_burndown_velocity(sprint_id)?;
            let prediction = self.predict_sprint_completion(sprint_id)?;
            
            let mut health_score = 100.0;

            // Penalize for being behind schedule
            if !prediction.on_time {
                health_score -= (prediction.days_ahead_behind.abs() as f64) * 5.0;
            }

            // Penalize for low completion rate
            if velocity.completion_rate < 0.5 {
                health_score -= (0.5 - velocity.completion_rate) * 100.0;
            }

            // Penalize for high variance
            if velocity.variance_percentage.abs() > 20.0 {
                health_score -= (velocity.variance_percentage.abs() - 20.0) * 0.5;
            }

            // Ensure score is between 0 and 100
            health_score = health_score.max(0.0).min(100.0);

            let health_level = if health_score >= 80.0 {
                HealthLevel::Excellent
            } else if health_score >= 60.0 {
                HealthLevel::Good
            } else if health_score >= 40.0 {
                HealthLevel::Fair
            } else {
                HealthLevel::Poor
            };

            let recommendations = self.generate_recommendations(&velocity, &prediction);
            
            Ok(SprintHealthScore {
                score: health_score,
                level: health_level,
                velocity,
                prediction,
                recommendations,
            })
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Generate recommendations based on burndown analysis
    fn generate_recommendations(&self, velocity: &BurndownVelocity, prediction: &SprintPrediction) -> Vec<String> {
        let mut recommendations = Vec::new();

        if velocity.trend == BurndownTrend::Behind {
            recommendations.push("Consider reducing scope or adding team members to catch up".to_string());
        }

        if velocity.variance_percentage.abs() > 20.0 {
            recommendations.push("Velocity is inconsistent - review task estimation and team capacity".to_string());
        }

        if !prediction.on_time {
            recommendations.push(format!("Sprint is predicted to be {} days behind schedule", prediction.days_ahead_behind.abs()));
        }

        if velocity.completion_rate < 0.3 {
            recommendations.push("Low completion rate - consider breaking down tasks into smaller pieces".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Sprint is on track - continue current approach".to_string());
        }

        recommendations
    }

    /// Get burndown chart data for visualization
    pub fn get_burndown_chart_data(&self, sprint_id: &str) -> Result<BurndownChartData, String> {
        if let Some(sprint) = self.sprint_data.get(sprint_id) {
            let ideal_burndown = self.calculate_ideal_burndown(
                sprint_id,
                sprint.metrics.planned_story_points,
                sprint.start_date,
                sprint.end_date,
            );

            let actual_burndown = sprint.metrics.burndown_data.clone();
            let velocity = self.calculate_burndown_velocity(sprint_id)?;
            let prediction = self.predict_sprint_completion(sprint_id)?;

            Ok(BurndownChartData {
                sprint_id: sprint_id.to_string(),
                sprint_name: sprint.name.clone(),
                ideal_burndown,
                actual_burndown,
                velocity,
                prediction,
                chart_type: BurndownChartType::StoryPoints,
            })
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }
}

/// Daily completion data
#[derive(Debug, Clone)]
pub struct DailyCompletion {
    pub date: DateTime<Utc>,
    pub story_points: u32,
    pub task_count: u32,
}

/// Burndown velocity analysis
#[derive(Debug, Clone)]
pub struct BurndownVelocity {
    pub ideal_velocity: f64,
    pub actual_velocity: f64,
    pub variance: f64,
    pub variance_percentage: f64,
    pub trend: BurndownTrend,
    pub completion_rate: f64,
    pub total_story_points: u32,
    pub remaining_story_points: u32,
}

/// Burndown trend
#[derive(Debug, Clone, PartialEq)]
pub enum BurndownTrend {
    Ahead,
    OnTrack,
    Behind,
}

/// Sprint prediction
#[derive(Debug, Clone)]
pub struct SprintPrediction {
    pub predicted_completion_date: DateTime<Utc>,
    pub on_time: bool,
    pub days_ahead_behind: i64,
    pub confidence: f64,
    pub remaining_story_points: u32,
    pub current_velocity: f64,
}

/// Sprint health score
#[derive(Debug, Clone)]
pub struct SprintHealthScore {
    pub score: f64,
    pub level: HealthLevel,
    pub velocity: BurndownVelocity,
    pub prediction: SprintPrediction,
    pub recommendations: Vec<String>,
}

/// Health level
#[derive(Debug, Clone, PartialEq)]
pub enum HealthLevel {
    Excellent,
    Good,
    Fair,
    Poor,
}

/// Burndown chart data
#[derive(Debug, Clone)]
pub struct BurndownChartData {
    pub sprint_id: String,
    pub sprint_name: String,
    pub ideal_burndown: Vec<BurndownDataPoint>,
    pub actual_burndown: Vec<BurndownDataPoint>,
    pub velocity: BurndownVelocity,
    pub prediction: SprintPrediction,
    pub chart_type: BurndownChartType,
}

/// Burndown chart type
#[derive(Debug, Clone, PartialEq)]
pub enum BurndownChartType {
    StoryPoints,
    TaskCount,
    Hours,
}

impl Default for BurndownCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BurndownVelocity {
    fn default() -> Self {
        Self {
            ideal_velocity: 0.0,
            actual_velocity: 0.0,
            variance: 0.0,
            variance_percentage: 0.0,
            trend: BurndownTrend::OnTrack,
            completion_rate: 0.0,
            total_story_points: 0,
            remaining_story_points: 0,
        }
    }
}

impl Default for SprintPrediction {
    fn default() -> Self {
        Self {
            predicted_completion_date: Utc::now(),
            on_time: true,
            days_ahead_behind: 0,
            confidence: 0.0,
            remaining_story_points: 0,
            current_velocity: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burndown_calculator_creation() {
        let calculator = BurndownCalculator::new();
        assert_eq!(calculator.sprint_data.len(), 0);
    }

    #[test]
    fn test_calculate_ideal_burndown() {
        let calculator = BurndownCalculator::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        
        let ideal_points = calculator.calculate_ideal_burndown(
            "sprint1",
            40,
            start_date,
            end_date,
        );

        assert_eq!(ideal_points.len(), 15); // 14 days + start day
        assert_eq!(ideal_points[0].remaining_story_points, 40);
        assert_eq!(ideal_points[14].remaining_story_points, 0);
    }

    #[test]
    fn test_generate_burndown_data() {
        let mut calculator = BurndownCalculator::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string()],
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

        let sprint_id = sprint.id.clone();
        calculator.add_sprint(sprint);

        let daily_completions = vec![
            DailyCompletion {
                date: start_date + Duration::days(1),
                story_points: 5,
                task_count: 2,
            },
            DailyCompletion {
                date: start_date + Duration::days(2),
                story_points: 8,
                task_count: 3,
            },
        ];

        let result = calculator.generate_burndown_data(&sprint_id, daily_completions);
        assert!(result.is_ok());
        
        let burndown_data = result.unwrap();
        assert!(!burndown_data.is_empty());
    }

    #[test]
    fn test_calculate_burndown_velocity() {
        let mut calculator = BurndownCalculator::new();
        let start_date = Utc::now();
        let end_date = start_date + Duration::days(14);
        let capacity = SprintCapacity {
            total_story_points: 40,
            team_members: vec!["dev1".to_string()],
            working_days: vec![Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday, Weekday::Thursday, Weekday::Friday],
            hours_per_day: 8.0,
            availability_percentage: 0.8,
        };

        let mut sprint = Sprint::new(
            "Sprint 1".to_string(),
            "proj1".to_string(),
            start_date,
            end_date,
            capacity,
        );

        // Add some burndown data
        sprint.metrics.burndown_data = vec![
            BurndownDataPoint {
                date: start_date,
                remaining_story_points: 40,
                ideal_remaining: 40,
                completed_story_points: 0,
            },
            BurndownDataPoint {
                date: start_date + Duration::days(7),
                remaining_story_points: 20,
                ideal_remaining: 20,
                completed_story_points: 20,
            },
        ];

        let sprint_id = sprint.id.clone();
        calculator.add_sprint(sprint);

        let velocity = calculator.calculate_burndown_velocity(&sprint_id).unwrap();
        assert!(velocity.actual_velocity > 0.0);
        assert_eq!(velocity.total_story_points, 40);
    }
}
