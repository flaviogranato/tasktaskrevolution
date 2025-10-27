use crate::domain::agile::agile_models::*;
use chrono::{DateTime, Utc, Datelike};
use std::collections::HashMap;

/// VelocityTracker handles team velocity tracking and analysis
#[derive(Debug, Clone)]
pub struct VelocityTracker {
    velocity_data: HashMap<String, Vec<VelocityData>>,
    team_metrics: HashMap<String, TeamVelocityMetrics>,
}

impl VelocityTracker {
    /// Create a new VelocityTracker
    pub fn new() -> Self {
        Self {
            velocity_data: HashMap::new(),
            team_metrics: HashMap::new(),
        }
    }

    /// Record velocity data for a sprint
    pub fn record_sprint_velocity(&mut self, velocity_data: VelocityData) {
        let team_key = velocity_data.team_id.clone();
        self.velocity_data.entry(team_key.clone()).or_insert_with(Vec::new).push(velocity_data);
        
        // Update team metrics
        self.update_team_metrics(&team_key);
    }

    /// Get velocity history for a team
    pub fn get_team_velocity_history(&self, team_id: &str) -> Option<&Vec<VelocityData>> {
        self.velocity_data.get(team_id)
    }

    /// Calculate velocity trend for a team
    pub fn calculate_velocity_trend(&self, team_id: &str) -> Option<VelocityTrend> {
        if let Some(velocity_history) = self.velocity_data.get(team_id) {
            if velocity_history.len() < 2 {
                return None;
            }

            let mut trend = VelocityTrend {
                team_id: team_id.to_string(),
                sprints: velocity_history.clone(),
                average_velocity: 0.0,
                velocity_trend: VelocityTrendDirection::Stable,
                volatility: 0.0,
                prediction_next_sprint: 0.0,
            };

            // Calculate average velocity
            trend.average_velocity = velocity_history.iter()
                .map(|v| v.velocity)
                .sum::<f64>() / velocity_history.len() as f64;

            trend.calculate_trend_direction();
            Some(trend)
        } else {
            None
        }
    }

    /// Calculate team velocity metrics
    pub fn calculate_team_metrics(&self, team_id: &str) -> Option<TeamVelocityMetrics> {
        if let Some(velocity_history) = self.velocity_data.get(team_id) {
            if velocity_history.is_empty() {
                return None;
            }

            let velocities: Vec<f64> = velocity_history.iter().map(|v| v.velocity).collect();
            let story_points: Vec<u32> = velocity_history.iter().map(|v| v.story_points_completed).collect();

            // Calculate basic statistics
            let average_velocity = velocities.iter().sum::<f64>() / velocities.len() as f64;
            let min_velocity = velocities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_velocity = velocities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            // Calculate standard deviation (volatility)
            let variance = velocities.iter()
                .map(|v| (v - average_velocity).powi(2))
                .sum::<f64>() / velocities.len() as f64;
            let volatility = variance.sqrt();

            // Calculate trend
            let trend = if velocities.len() >= 3 {
                let recent_avg = velocities[velocities.len()-3..].iter().sum::<f64>() / 3.0;
                let earlier_avg = velocities[..3].iter().sum::<f64>() / 3.0;
                
                if recent_avg > earlier_avg * 1.1 {
                    VelocityTrendDirection::Increasing
                } else if recent_avg < earlier_avg * 0.9 {
                    VelocityTrendDirection::Decreasing
                } else {
                    VelocityTrendDirection::Stable
                }
            } else {
                VelocityTrendDirection::Stable
            };

            // Calculate completion rate
            let total_committed: u32 = velocity_history.iter().map(|v| v.story_points_committed).sum();
            let total_completed: u32 = velocity_history.iter().map(|v| v.story_points_completed).sum();
            let completion_rate = if total_committed > 0 {
                total_completed as f64 / total_committed as f64
            } else {
                0.0
            };

            // Calculate sprint consistency
            let consistency_score = if volatility > 0.0 {
                (1.0 - (volatility / average_velocity).min(1.0)) * 100.0
            } else {
                100.0
            };

            // Predict next sprint velocity
            let prediction = self.predict_next_sprint_velocity(team_id);

            Some(TeamVelocityMetrics {
                team_id: team_id.to_string(),
                sprint_count: velocity_history.len(),
                average_velocity,
                min_velocity,
                max_velocity,
                volatility,
                trend,
                completion_rate,
                consistency_score,
                total_story_points_completed: total_completed,
                total_story_points_committed: total_committed,
                prediction_next_sprint: prediction,
                last_updated: Utc::now(),
            })
        } else {
            None
        }
    }

    /// Predict next sprint velocity
    pub fn predict_next_sprint_velocity(&self, team_id: &str) -> f64 {
        if let Some(velocity_history) = self.velocity_data.get(team_id) {
            if velocity_history.len() < 2 {
                return 0.0;
            }

            // Use weighted average with more weight on recent sprints
            let weights: Vec<f64> = (1..=velocity_history.len())
                .map(|i| i as f64)
                .collect();
            
            let total_weight: f64 = weights.iter().sum();
            let weighted_sum: f64 = velocity_history.iter()
                .zip(weights.iter())
                .map(|(v, w)| v.velocity * w)
                .sum();

            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    /// Get velocity distribution analysis
    pub fn get_velocity_distribution(&self, team_id: &str) -> Option<VelocityDistribution> {
        if let Some(velocity_history) = self.velocity_data.get(team_id) {
            if velocity_history.is_empty() {
                return None;
            }

            let velocities: Vec<f64> = velocity_history.iter().map(|v| v.velocity).collect();
            let average = velocities.iter().sum::<f64>() / velocities.len() as f64;

            // Calculate quartiles
            let mut sorted_velocities = velocities.clone();
            sorted_velocities.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let q1_index = sorted_velocities.len() / 4;
            let q2_index = sorted_velocities.len() / 2;
            let q3_index = (sorted_velocities.len() * 3) / 4;

            let q1 = sorted_velocities[q1_index];
            let median = sorted_velocities[q2_index];
            let q3 = sorted_velocities[q3_index];

            // Calculate velocity ranges
            let low_velocity = average * 0.7;
            let high_velocity = average * 1.3;

            let low_count = velocities.iter().filter(|&&v| v < low_velocity).count();
            let normal_count = velocities.iter().filter(|&&v| v >= low_velocity && v <= high_velocity).count();
            let high_count = velocities.iter().filter(|&&v| v > high_velocity).count();

            Some(VelocityDistribution {
                team_id: team_id.to_string(),
                average_velocity: average,
                median_velocity: median,
                q1_velocity: q1,
                q3_velocity: q3,
                low_velocity_count: low_count,
                normal_velocity_count: normal_count,
                high_velocity_count: high_count,
                velocity_range: VelocityRange {
                    min: sorted_velocities[0],
                    max: sorted_velocities[sorted_velocities.len() - 1],
                    low_threshold: low_velocity,
                    high_threshold: high_velocity,
                },
            })
        } else {
            None
        }
    }

    /// Analyze velocity patterns
    pub fn analyze_velocity_patterns(&self, team_id: &str) -> Option<VelocityPatternAnalysis> {
        if let Some(velocity_history) = self.velocity_data.get(team_id) {
            if velocity_history.len() < 3 {
                return None;
            }

            let velocities: Vec<f64> = velocity_history.iter().map(|v| v.velocity).collect();
            
            // Detect patterns
            let mut patterns = Vec::new();

            // Check for increasing trend
            if self.detect_increasing_trend(&velocities) {
                patterns.push(VelocityPattern::IncreasingTrend);
            }

            // Check for decreasing trend
            if self.detect_decreasing_trend(&velocities) {
                patterns.push(VelocityPattern::DecreasingTrend);
            }

            // Check for volatility
            if self.detect_high_volatility(&velocities) {
                patterns.push(VelocityPattern::HighVolatility);
            }

            // Check for stability
            if self.detect_stability(&velocities) {
                patterns.push(VelocityPattern::Stable);
            }

            // Check for seasonal patterns
            if self.detect_seasonal_pattern(&velocity_history) {
                patterns.push(VelocityPattern::Seasonal);
            }

            // Generate insights
            let insights = self.generate_velocity_insights(&velocities, &patterns);

            Some(VelocityPatternAnalysis {
                team_id: team_id.to_string(),
                patterns: patterns.clone(),
                insights,
                confidence: self.calculate_pattern_confidence(&velocities),
                recommendations: self.generate_recommendations(&patterns),
            })
        } else {
            None
        }
    }

    /// Update team metrics
    fn update_team_metrics(&mut self, team_id: &str) {
        if let Some(metrics) = self.calculate_team_metrics(team_id) {
            self.team_metrics.insert(team_id.to_string(), metrics);
        }
    }

    /// Detect increasing trend
    fn detect_increasing_trend(&self, velocities: &[f64]) -> bool {
        if velocities.len() < 3 {
            return false;
        }

        let recent_avg = velocities[velocities.len()-3..].iter().sum::<f64>() / 3.0;
        let earlier_avg = velocities[..3].iter().sum::<f64>() / 3.0;
        
        recent_avg > earlier_avg * 1.15
    }

    /// Detect decreasing trend
    fn detect_decreasing_trend(&self, velocities: &[f64]) -> bool {
        if velocities.len() < 3 {
            return false;
        }

        let recent_avg = velocities[velocities.len()-3..].iter().sum::<f64>() / 3.0;
        let earlier_avg = velocities[..3].iter().sum::<f64>() / 3.0;
        
        recent_avg < earlier_avg * 0.85
    }

    /// Detect high volatility
    fn detect_high_volatility(&self, velocities: &[f64]) -> bool {
        if velocities.len() < 3 {
            return false;
        }

        let average = velocities.iter().sum::<f64>() / velocities.len() as f64;
        let variance = velocities.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / velocities.len() as f64;
        let volatility = variance.sqrt();

        volatility > average * 0.3
    }

    /// Detect stability
    fn detect_stability(&self, velocities: &[f64]) -> bool {
        if velocities.len() < 3 {
            return false;
        }

        let average = velocities.iter().sum::<f64>() / velocities.len() as f64;
        let variance = velocities.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / velocities.len() as f64;
        let volatility = variance.sqrt();

        volatility < average * 0.15
    }

    /// Detect seasonal pattern
    fn detect_seasonal_pattern(&self, velocity_history: &[VelocityData]) -> bool {
        if velocity_history.len() < 6 {
            return false;
        }

        // Simple seasonal detection based on month patterns
        let mut monthly_velocities: HashMap<u32, Vec<f64>> = HashMap::new();
        
        for data in velocity_history {
            let month = data.completion_date.month();
            monthly_velocities.entry(month).or_insert_with(Vec::new).push(data.velocity);
        }

        // Check if there's significant variation between months
        let monthly_averages: Vec<f64> = monthly_velocities.values()
            .map(|velocities| velocities.iter().sum::<f64>() / velocities.len() as f64)
            .collect();

        if monthly_averages.len() >= 3 {
            let overall_avg = monthly_averages.iter().sum::<f64>() / monthly_averages.len() as f64;
            let max_deviation = monthly_averages.iter()
                .map(|&avg| (avg - overall_avg).abs())
                .fold(0.0, f64::max);

            max_deviation > overall_avg * 0.2
        } else {
            false
        }
    }

    /// Generate velocity insights
    fn generate_velocity_insights(&self, velocities: &[f64], patterns: &[VelocityPattern]) -> Vec<String> {
        let mut insights = Vec::new();

        if patterns.contains(&VelocityPattern::IncreasingTrend) {
            insights.push("Team velocity is showing a positive upward trend".to_string());
        }

        if patterns.contains(&VelocityPattern::DecreasingTrend) {
            insights.push("Team velocity is declining - consider investigating blockers".to_string());
        }

        if patterns.contains(&VelocityPattern::HighVolatility) {
            insights.push("High velocity volatility suggests inconsistent sprint planning or execution".to_string());
        }

        if patterns.contains(&VelocityPattern::Stable) {
            insights.push("Team shows consistent velocity - good for planning and forecasting".to_string());
        }

        if patterns.contains(&VelocityPattern::Seasonal) {
            insights.push("Seasonal patterns detected in velocity - consider capacity planning".to_string());
        }

        // Add general insights
        let average = velocities.iter().sum::<f64>() / velocities.len() as f64;
        if average > 20.0 {
            insights.push("High average velocity indicates strong team performance".to_string());
        } else if average < 10.0 {
            insights.push("Low average velocity - consider team capacity and task sizing".to_string());
        }

        insights
    }

    /// Calculate pattern confidence
    fn calculate_pattern_confidence(&self, velocities: &[f64]) -> f64 {
        if velocities.len() < 3 {
            return 0.0;
        }

        // Confidence based on data points and consistency
        let data_points_score = (velocities.len() as f64 / 10.0).min(1.0);
        
        let average = velocities.iter().sum::<f64>() / velocities.len() as f64;
        let variance = velocities.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / velocities.len() as f64;
        let volatility = variance.sqrt();
        
        let consistency_score = if volatility > 0.0 {
            (1.0 - (volatility / average).min(1.0)).max(0.0)
        } else {
            1.0
        };

        (data_points_score + consistency_score) / 2.0
    }

    /// Generate recommendations
    fn generate_recommendations(&self, patterns: &[VelocityPattern]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if patterns.contains(&VelocityPattern::DecreasingTrend) {
            recommendations.push("Investigate and address blockers affecting team productivity".to_string());
            recommendations.push("Review sprint planning and task estimation accuracy".to_string());
        }

        if patterns.contains(&VelocityPattern::HighVolatility) {
            recommendations.push("Improve sprint planning consistency and task sizing".to_string());
            recommendations.push("Consider implementing velocity smoothing techniques".to_string());
        }

        if patterns.contains(&VelocityPattern::Seasonal) {
            recommendations.push("Plan capacity adjustments for seasonal variations".to_string());
            recommendations.push("Consider team member availability during different periods".to_string());
        }

        if patterns.contains(&VelocityPattern::Stable) {
            recommendations.push("Maintain current practices - stability is valuable for planning".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Continue monitoring velocity trends for continuous improvement".to_string());
        }

        recommendations
    }

    /// Get all team metrics
    pub fn get_all_team_metrics(&self) -> Vec<&TeamVelocityMetrics> {
        self.team_metrics.values().collect()
    }

    /// Get team metrics by ID
    pub fn get_team_metrics(&self, team_id: &str) -> Option<&TeamVelocityMetrics> {
        self.team_metrics.get(team_id)
    }
}

/// Team velocity metrics
#[derive(Debug, Clone)]
pub struct TeamVelocityMetrics {
    pub team_id: String,
    pub sprint_count: usize,
    pub average_velocity: f64,
    pub min_velocity: f64,
    pub max_velocity: f64,
    pub volatility: f64,
    pub trend: VelocityTrendDirection,
    pub completion_rate: f64,
    pub consistency_score: f64,
    pub total_story_points_completed: u32,
    pub total_story_points_committed: u32,
    pub prediction_next_sprint: f64,
    pub last_updated: DateTime<Utc>,
}

/// Velocity distribution analysis
#[derive(Debug, Clone)]
pub struct VelocityDistribution {
    pub team_id: String,
    pub average_velocity: f64,
    pub median_velocity: f64,
    pub q1_velocity: f64,
    pub q3_velocity: f64,
    pub low_velocity_count: usize,
    pub normal_velocity_count: usize,
    pub high_velocity_count: usize,
    pub velocity_range: VelocityRange,
}

/// Velocity range
#[derive(Debug, Clone)]
pub struct VelocityRange {
    pub min: f64,
    pub max: f64,
    pub low_threshold: f64,
    pub high_threshold: f64,
}

/// Velocity pattern analysis
#[derive(Debug, Clone)]
pub struct VelocityPatternAnalysis {
    pub team_id: String,
    pub patterns: Vec<VelocityPattern>,
    pub insights: Vec<String>,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}

/// Velocity patterns
#[derive(Debug, Clone, PartialEq)]
pub enum VelocityPattern {
    IncreasingTrend,
    DecreasingTrend,
    HighVolatility,
    Stable,
    Seasonal,
}

impl Default for VelocityTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_tracker_creation() {
        let tracker = VelocityTracker::new();
        assert_eq!(tracker.velocity_data.len(), 0);
    }

    #[test]
    fn test_record_sprint_velocity() {
        let mut tracker = VelocityTracker::new();
        let velocity_data = VelocityData::new(
            "team1".to_string(),
            "sprint1".to_string(),
            20,
            25,
            14,
            vec!["dev1".to_string(), "dev2".to_string()],
        );

        tracker.record_sprint_velocity(velocity_data);
        assert_eq!(tracker.velocity_data.len(), 1);
        assert!(tracker.velocity_data.contains_key("team1"));
    }

    #[test]
    fn test_calculate_velocity_trend() {
        let mut tracker = VelocityTracker::new();
        
        // Add increasing velocity data
        for i in 1..=5 {
            let velocity_data = VelocityData::new(
                "team1".to_string(),
                format!("sprint{}", i),
                i * 5,
                i * 6,
                14,
                vec!["dev1".to_string()],
            );
            tracker.record_sprint_velocity(velocity_data);
        }

        let trend = tracker.calculate_velocity_trend("team1").unwrap();
        assert_eq!(trend.team_id, "team1");
        assert_eq!(trend.sprints.len(), 5);
        assert!(trend.average_velocity > 0.0);
    }

    #[test]
    fn test_calculate_team_metrics() {
        let mut tracker = VelocityTracker::new();
        
        for i in 1..=3 {
            let velocity_data = VelocityData::new(
                "team1".to_string(),
                format!("sprint{}", i),
                20,
                25,
                14,
                vec!["dev1".to_string()],
            );
            tracker.record_sprint_velocity(velocity_data);
        }

        let metrics = tracker.calculate_team_metrics("team1").unwrap();
        assert_eq!(metrics.team_id, "team1");
        assert_eq!(metrics.sprint_count, 3);
        assert!(metrics.average_velocity > 0.0);
        assert!(metrics.consistency_score >= 0.0);
    }

    #[test]
    fn test_predict_next_sprint_velocity() {
        let mut tracker = VelocityTracker::new();
        
        for i in 1..=5 {
            let velocity_data = VelocityData::new(
                "team1".to_string(),
                format!("sprint{}", i),
                20 + i,
                25 + i,
                14,
                vec!["dev1".to_string()],
            );
            tracker.record_sprint_velocity(velocity_data);
        }

        let prediction = tracker.predict_next_sprint_velocity("team1");
        assert!(prediction > 0.0);
    }

    #[test]
    fn test_get_velocity_distribution() {
        let mut tracker = VelocityTracker::new();
        
        let velocities = vec![10.0, 15.0, 20.0, 25.0, 30.0];
        for (i, velocity) in velocities.iter().enumerate() {
            let velocity_data = VelocityData::new(
                "team1".to_string(),
                format!("sprint{}", i + 1),
                *velocity as u32,
                (*velocity * 1.2) as u32,
                14,
                vec!["dev1".to_string()],
            );
            tracker.record_sprint_velocity(velocity_data);
        }

        let distribution = tracker.get_velocity_distribution("team1").unwrap();
        assert_eq!(distribution.team_id, "team1");
        assert_eq!(distribution.average_velocity, 20.0);
        assert_eq!(distribution.median_velocity, 20.0);
    }
}
