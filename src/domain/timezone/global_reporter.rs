// use super::global_scheduler::GlobalScheduler;
// use super::timezone_metrics::TimezoneMetricsAggregator;
use super::timezone_models::*;
use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global timezone report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalReporter {
    metrics: GlobalMetrics,
    timezone_analytics: TimezoneAnalytics,
    performance_metrics: PerformanceMetrics,
    collaboration_metrics: CollaborationMetrics,
    efficiency_metrics: EfficiencyMetrics,
}

impl GlobalReporter {
    pub fn new() -> Self {
        Self {
            metrics: GlobalMetrics::new(),
            timezone_analytics: TimezoneAnalytics::new(),
            performance_metrics: PerformanceMetrics::new(),
            collaboration_metrics: CollaborationMetrics::new(),
            efficiency_metrics: EfficiencyMetrics::new(),
        }
    }

    /// Generate complete timezone report
    pub fn generate_global_report(&mut self, schedules: &[GlobalSchedule]) -> GlobalReport {
        self.analyze_schedules(schedules);
        self.calculate_performance_metrics(schedules);
        self.calculate_collaboration_metrics(schedules);
        self.calculate_efficiency_metrics(schedules);

        GlobalReport {
            generated_at: Utc::now(),
            total_projects: schedules.len(),
            timezone_distribution: self.timezone_analytics.timezone_distribution.clone(),
            most_active_timezone: self.timezone_analytics.most_active_timezone.clone(),
            average_meeting_duration: self.timezone_analytics.average_meeting_duration,
            timezone_utilization: self.timezone_analytics.timezone_utilization.clone(),
            performance_metrics: self.performance_metrics.clone(),
            collaboration_metrics: self.collaboration_metrics.clone(),
            efficiency_metrics: self.efficiency_metrics.clone(),
            global_insights: self.generate_insights(),
            recommendations: self.generate_recommendations(),
            trend_analysis: self.generate_trend_analysis(schedules),
            risk_assessment: self.generate_risk_assessment(schedules),
        }
    }

    /// Analyze schedules to extract metrics
    fn analyze_schedules(&mut self, schedules: &[GlobalSchedule]) {
        self.timezone_analytics = TimezoneAnalytics::new();

        for schedule in schedules {
            // Count timezones
            *self
                .timezone_analytics
                .timezone_distribution
                .entry(schedule.timezone.clone())
                .or_insert(0) += 1;

            // Analyze participants
            for participant in &schedule.participants {
                *self
                    .timezone_analytics
                    .timezone_distribution
                    .entry(participant.timezone.clone())
                    .or_insert(0) += 1;
            }

            // Calculate average duration
            let duration = schedule.get_duration_minutes();
            self.timezone_analytics.total_duration += duration;
            self.timezone_analytics.meeting_count += 1;
        }

        // Calculate derived metrics
        if self.timezone_analytics.meeting_count > 0 {
            self.timezone_analytics.average_meeting_duration =
                self.timezone_analytics.total_duration as f64 / self.timezone_analytics.meeting_count as f64;
        }

        // Find most active timezone
        self.timezone_analytics.most_active_timezone = self
            .timezone_analytics
            .timezone_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(timezone, _)| timezone.clone());

        // Calculate utilization by timezone
        self.calculate_timezone_utilization(schedules);
    }

    /// Calculate utilization by timezone
    fn calculate_timezone_utilization(&mut self, schedules: &[GlobalSchedule]) {
        let mut timezone_hours: HashMap<String, f64> = HashMap::new();

        for schedule in schedules {
            let duration_hours = schedule.get_duration_minutes() as f64 / 60.0;

            // Add to schedule timezone
            *timezone_hours.entry(schedule.timezone.clone()).or_insert(0.0) += duration_hours;

            // Add to participant timezones
            for participant in &schedule.participants {
                *timezone_hours.entry(participant.timezone.clone()).or_insert(0.0) += duration_hours;
            }
        }

        // Calculate percentage utilization
        let total_hours: f64 = timezone_hours.values().sum();

        for (timezone, hours) in timezone_hours {
            let utilization = if total_hours > 0.0 {
                (hours / total_hours) * 100.0
            } else {
                0.0
            };

            self.timezone_analytics
                .timezone_utilization
                .insert(timezone, utilization);
        }
    }

    /// Generate global insights
    fn generate_insights(&self) -> Vec<GlobalInsight> {
        let mut insights = Vec::new();

        // Insight about most active timezone
        if let Some(ref most_active) = self.timezone_analytics.most_active_timezone {
            insights.push(GlobalInsight {
                category: InsightCategory::TimezoneUsage,
                title: "Most Active Timezone".to_string(),
                description: format!("The timezone {} is the most used in schedules", most_active),
                impact: InsightImpact::High,
                confidence: 0.9,
            });
        }

        // Insight about timezone distribution
        let timezone_count = self.timezone_analytics.timezone_distribution.len();
        if timezone_count > 5 {
            insights.push(GlobalInsight {
                category: InsightCategory::Complexity,
                title: "High Timezone Diversity".to_string(),
                description: format!(
                    "The project involves {} different timezones, indicating high global complexity",
                    timezone_count
                ),
                impact: InsightImpact::Medium,
                confidence: 0.8,
            });
        }

        // Insight about average duration
        if self.timezone_analytics.average_meeting_duration > 120.0 {
            insights.push(GlobalInsight {
                category: InsightCategory::Efficiency,
                title: "Long Meetings".to_string(),
                description: format!(
                    "The average meeting duration is {:.1} minutes, considered long",
                    self.timezone_analytics.average_meeting_duration
                ),
                impact: InsightImpact::Medium,
                confidence: 0.7,
            });
        }

        insights
    }

    /// Generate recommendations
    fn generate_recommendations(&self) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Recommendation about default timezone
        if let Some(ref most_active) = self.timezone_analytics.most_active_timezone {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Optimization,
                title: "Set Default Timezone".to_string(),
                description: format!(
                    "Consider setting {} as default timezone to simplify coordination",
                    most_active
                ),
                priority: RecommendationPriority::High,
                effort: EffortLevel::Low,
            });
        }

        // Recommendation about schedules
        recommendations.push(Recommendation {
            category: RecommendationCategory::Scheduling,
            title: "Optimize Meeting Schedules".to_string(),
            description: "Consider scheduling meetings at times that work well for most participants".to_string(),
            priority: RecommendationPriority::Medium,
            effort: EffortLevel::Medium,
        });

        // Recommendation about tools
        recommendations.push(Recommendation {
            category: RecommendationCategory::Tools,
            title: "Use Synchronization Tools".to_string(),
            description: "Implement automatic tools for timezone synchronization and notifications".to_string(),
            priority: RecommendationPriority::Low,
            effort: EffortLevel::High,
        });

        recommendations
    }

    /// Export report to different formats
    pub fn export_report(&self, format: ReportFormat) -> Result<String, String> {
        match format {
            ReportFormat::Json => {
                serde_json::to_string_pretty(self).map_err(|e| format!("Erro ao exportar JSON: {}", e))
            }
            ReportFormat::Yaml => serde_yaml::to_string(self).map_err(|e| format!("Erro ao exportar YAML: {}", e)),
            ReportFormat::Csv => self.export_to_csv(),
        }
    }

    /// Exporta para CSV
    fn export_to_csv(&self) -> Result<String, String> {
        let mut csv = String::from("timezone,utilization_percent,meeting_count\n");

        for (timezone, utilization) in &self.timezone_analytics.timezone_utilization {
            let count = self
                .timezone_analytics
                .timezone_distribution
                .get(timezone)
                .unwrap_or(&0);

            csv.push_str(&format!("{},{:.2},{}\n", timezone, utilization, count));
        }

        Ok(csv)
    }

    /// Calculate performance metrics
    fn calculate_performance_metrics(&mut self, schedules: &[GlobalSchedule]) {
        self.performance_metrics = PerformanceMetrics::new();

        let mut total_meetings = 0;
        let mut total_duration = Duration::zero();
        let mut timezone_conflicts = 0;
        let mut scheduling_delays = 0;

        for schedule in schedules {
            total_meetings += 1;
            total_duration += schedule.get_duration();

            // Detectar conflitos de timezone
            if self.has_timezone_conflicts(schedule) {
                timezone_conflicts += 1;
            }

            // Detectar atrasos de agendamento
            if self.has_scheduling_delays(schedule) {
                scheduling_delays += 1;
            }
        }

        self.performance_metrics.total_meetings = total_meetings;
        self.performance_metrics.total_duration = total_duration;
        self.performance_metrics.timezone_conflicts = timezone_conflicts;
        self.performance_metrics.scheduling_delays = scheduling_delays;
        self.performance_metrics.conflict_rate = if total_meetings > 0 {
            timezone_conflicts as f64 / total_meetings as f64
        } else {
            0.0
        };
        self.performance_metrics.delay_rate = if total_meetings > 0 {
            scheduling_delays as f64 / total_meetings as f64
        } else {
            0.0
        };
    }

    /// Calculate collaboration metrics
    fn calculate_collaboration_metrics(&mut self, schedules: &[GlobalSchedule]) {
        self.collaboration_metrics = CollaborationMetrics::new();

        let mut total_participants = 0;
        let mut cross_timezone_meetings = 0;
        let mut timezone_diversity = HashMap::new();

        for schedule in schedules {
            total_participants += schedule.participants.len();

            // Check if there are participants from different timezones
            let unique_timezones: std::collections::HashSet<String> =
                schedule.participants.iter().map(|p| p.timezone.clone()).collect();

            if unique_timezones.len() > 1 {
                cross_timezone_meetings += 1;
            }

            // Contar diversidade de timezones
            for timezone in unique_timezones {
                *timezone_diversity.entry(timezone).or_insert(0) += 1;
            }
        }

        self.collaboration_metrics.total_participants = total_participants;
        self.collaboration_metrics.cross_timezone_meetings = cross_timezone_meetings;
        self.collaboration_metrics.timezone_diversity = timezone_diversity;
        self.collaboration_metrics.collaboration_index = if !schedules.is_empty() {
            cross_timezone_meetings as f64 / schedules.len() as f64
        } else {
            0.0
        };
    }

    /// Calculate efficiency metrics
    fn calculate_efficiency_metrics(&mut self, schedules: &[GlobalSchedule]) {
        self.efficiency_metrics = EfficiencyMetrics::new();

        let mut total_productive_time = Duration::zero();
        let mut total_waiting_time = Duration::zero();
        let mut optimal_scheduling_score = 0.0;

        for schedule in schedules {
            let duration = schedule.get_duration();
            total_productive_time += duration;

            // Calcular tempo de espera (simplificado)
            let waiting_time = self.calculate_waiting_time(schedule);
            total_waiting_time += waiting_time;

            // Calcular score de agendamento otimizado
            let score = self.calculate_scheduling_score(schedule);
            optimal_scheduling_score += score;
        }

        self.efficiency_metrics.total_productive_time = total_productive_time;
        self.efficiency_metrics.total_waiting_time = total_waiting_time;
        self.efficiency_metrics.efficiency_ratio = if total_productive_time > Duration::zero() {
            total_productive_time.num_seconds() as f64
                / (total_productive_time + total_waiting_time).num_seconds() as f64
        } else {
            0.0
        };
        self.efficiency_metrics.average_scheduling_score = if !schedules.is_empty() {
            optimal_scheduling_score / schedules.len() as f64
        } else {
            0.0
        };
    }

    /// Check if there are timezone conflicts
    fn has_timezone_conflicts(&self, schedule: &GlobalSchedule) -> bool {
        let unique_timezones: std::collections::HashSet<String> =
            schedule.participants.iter().map(|p| p.timezone.clone()).collect();

        // Conflict if there are many different timezones in a meeting
        unique_timezones.len() > 3
    }

    /// Check if there are scheduling delays
    fn has_scheduling_delays(&self, schedule: &GlobalSchedule) -> bool {
        // Simulation: delay if meeting was scheduled with less than 24h notice
        let now = Utc::now();
        let time_until_meeting = schedule.start_time.signed_duration_since(now);
        time_until_meeting < Duration::hours(24)
    }

    /// Calcula tempo de espera
    fn calculate_waiting_time(&self, schedule: &GlobalSchedule) -> Duration {
        // Simulation: waiting time based on timezone complexity
        let timezone_count = schedule.participants.len();
        Duration::minutes(timezone_count as i64 * 5) // 5 min por timezone
    }

    /// Calcula score de agendamento
    fn calculate_scheduling_score(&self, schedule: &GlobalSchedule) -> f64 {
        let mut score = 1.0;

        // Penalizar por muitos timezones
        let timezone_count = schedule.participants.len();
        if timezone_count > 3 {
            score *= 0.8;
        }

        // Penalize for bad schedules (simulation)
        let hour = schedule.start_time.hour();
        if !(6..=22).contains(&hour) {
            score *= 0.7;
        }

        score
    }

    /// Generate trend analysis
    fn generate_trend_analysis(&self, schedules: &[GlobalSchedule]) -> TrendAnalysis {
        let mut trends = Vec::new();

        // Meeting growth trend
        if schedules.len() > 10 {
            trends.push(Trend {
                metric: "Meeting Volume".to_string(),
                direction: TrendDirection::Increasing,
                confidence: 0.8,
                description: "Increase in global meeting volume".to_string(),
            });
        }

        // Timezone diversity trend
        let avg_timezones_per_meeting: f64 =
            schedules.iter().map(|s| s.participants.len()).sum::<usize>() as f64 / schedules.len() as f64;

        if avg_timezones_per_meeting > 2.5 {
            trends.push(Trend {
                metric: "Timezone Diversity".to_string(),
                direction: TrendDirection::Increasing,
                confidence: 0.9,
                description: "Aumento na diversidade de timezones".to_string(),
            });
        }

        let overall_trend = if trends.iter().any(|t| t.direction == TrendDirection::Increasing) {
            TrendDirection::Increasing
        } else {
            TrendDirection::Stable
        };
        
        TrendAnalysis {
            trends,
            overall_trend,
        }
    }

    /// Generate risk assessment
    fn generate_risk_assessment(&self, _schedules: &[GlobalSchedule]) -> RiskAssessment {
        let mut risks = Vec::new();

        // Risco de conflitos de timezone
        if self.performance_metrics.conflict_rate > 0.3 {
            risks.push(Risk {
                category: RiskCategory::Scheduling,
                severity: RiskSeverity::High,
                description: "Alta taxa de conflitos de timezone".to_string(),
                mitigation: "Implement automatic coordination tools".to_string(),
            });
        }

        // Low efficiency risk
        if self.efficiency_metrics.efficiency_ratio < 0.7 {
            risks.push(Risk {
                category: RiskCategory::Efficiency,
                severity: RiskSeverity::Medium,
                description: "Low scheduling efficiency".to_string(),
                mitigation: "Optimize schedules and reduce waiting time".to_string(),
            });
        }

        // Always add a basic risk if no specific risks are found
        if risks.is_empty() {
            risks.push(Risk {
                category: RiskCategory::Scheduling,
                severity: RiskSeverity::Low,
                description: "Basic scheduling coordination needed".to_string(),
                mitigation: "Monitor timezone coordination".to_string(),
            });
        }

        let overall_risk_level = if risks.iter().any(|r| r.severity == RiskSeverity::High) {
            RiskLevel::High
        } else if risks.iter().any(|r| r.severity == RiskSeverity::Medium) {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };
        
        RiskAssessment {
            risks,
            overall_risk_level,
        }
    }
}

/// Global report
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalReport {
    pub generated_at: DateTime<Utc>,
    pub total_projects: usize,
    pub timezone_distribution: HashMap<String, usize>,
    pub most_active_timezone: Option<String>,
    pub average_meeting_duration: f64,
    pub timezone_utilization: HashMap<String, f64>,
    pub performance_metrics: PerformanceMetrics,
    pub collaboration_metrics: CollaborationMetrics,
    pub efficiency_metrics: EfficiencyMetrics,
    pub global_insights: Vec<GlobalInsight>,
    pub recommendations: Vec<Recommendation>,
    pub trend_analysis: TrendAnalysis,
    pub risk_assessment: RiskAssessment,
}

/// Analytics de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneAnalytics {
    pub timezone_distribution: HashMap<String, usize>,
    pub timezone_utilization: HashMap<String, f64>,
    pub most_active_timezone: Option<String>,
    pub average_meeting_duration: f64,
    pub total_duration: i64,
    pub meeting_count: usize,
}

impl Default for TimezoneAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

impl TimezoneAnalytics {
    pub fn new() -> Self {
        Self {
            timezone_distribution: HashMap::new(),
            timezone_utilization: HashMap::new(),
            most_active_timezone: None,
            average_meeting_duration: 0.0,
            total_duration: 0,
            meeting_count: 0,
        }
    }
}

/// Insight global
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalInsight {
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub impact: InsightImpact,
    pub confidence: f64,
}

/// Categoria do insight
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightCategory {
    TimezoneUsage,
    Complexity,
    Efficiency,
    Collaboration,
    Scheduling,
}

/// Impacto do insight
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightImpact {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub effort: EffortLevel,
}

/// Recommendation category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Optimization,
    Scheduling,
    Tools,
    Process,
    Communication,
}

/// Recommendation priority
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Effort level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// Report format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Yaml,
    Csv,
}

/// Performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_meetings: usize,
    pub total_duration: Duration,
    pub timezone_conflicts: usize,
    pub scheduling_delays: usize,
    pub conflict_rate: f64,
    pub delay_rate: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            total_meetings: 0,
            total_duration: Duration::zero(),
            timezone_conflicts: 0,
            scheduling_delays: 0,
            conflict_rate: 0.0,
            delay_rate: 0.0,
        }
    }
}

/// Collaboration metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollaborationMetrics {
    pub total_participants: usize,
    pub cross_timezone_meetings: usize,
    pub timezone_diversity: HashMap<String, usize>,
    pub collaboration_index: f64,
}

impl Default for CollaborationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CollaborationMetrics {
    pub fn new() -> Self {
        Self {
            total_participants: 0,
            cross_timezone_meetings: 0,
            timezone_diversity: HashMap::new(),
            collaboration_index: 0.0,
        }
    }
}

/// Efficiency metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    pub total_productive_time: Duration,
    pub total_waiting_time: Duration,
    pub efficiency_ratio: f64,
    pub average_scheduling_score: f64,
}

impl Default for EfficiencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl EfficiencyMetrics {
    pub fn new() -> Self {
        Self {
            total_productive_time: Duration::zero(),
            total_waiting_time: Duration::zero(),
            efficiency_ratio: 0.0,
            average_scheduling_score: 0.0,
        }
    }
}

/// Trend analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trends: Vec<Trend>,
    pub overall_trend: TrendDirection,
}

/// Individual trend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trend {
    pub metric: String,
    pub direction: TrendDirection,
    pub confidence: f64,
    pub description: String,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Risk assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub risks: Vec<Risk>,
    pub overall_risk_level: RiskLevel,
}

/// Individual risk
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Risk {
    pub category: RiskCategory,
    pub severity: RiskSeverity,
    pub description: String,
    pub mitigation: String,
}

/// Risk category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskCategory {
    Scheduling,
    Efficiency,
    Collaboration,
    Technical,
}

/// Risk severity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk level
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for GlobalReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_reporter_creation() {
        let reporter = GlobalReporter::new();
        assert_eq!(reporter.metrics.total_projects, 0);
    }

    #[test]
    fn test_generate_global_report() {
        let mut reporter = GlobalReporter::new();
        let schedules = vec![GlobalSchedule::new(
            "project1".to_string(),
            "UTC".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(2),
        )];

        let report = reporter.generate_global_report(&schedules);
        assert_eq!(report.total_projects, 1);
        assert!(!report.timezone_distribution.is_empty());
    }

    #[test]
    fn test_export_report_json() {
        let reporter = GlobalReporter::new();
        let result = reporter.export_report(ReportFormat::Json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_report_csv() {
        let reporter = GlobalReporter::new();
        let result = reporter.export_report(ReportFormat::Csv);
        assert!(result.is_ok());
    }

    #[test]
    fn test_comprehensive_analytics() {
        let mut reporter = GlobalReporter::new();
        
        // Create complex schedule data
        let base_time = Utc::now();
        let mut schedules = Vec::new();
        
        // Create more than 10 schedules to trigger trend analysis
        for i in 0..15 {
            let schedule = GlobalSchedule::new(
                format!("project{}", i),
                if i % 2 == 0 { "UTC".to_string() } else { "America/New_York".to_string() },
                base_time + Duration::hours(i),
                base_time + Duration::hours(i + 2)
            );
            schedules.push(schedule);
        }

        let report = reporter.generate_global_report(&schedules);
        
        // Test all metrics are populated
        assert!(report.performance_metrics.total_meetings > 0);
        // total_participants is non-negative by type; no need to assert lower bound
        assert!(report.efficiency_metrics.total_productive_time > Duration::zero());
        assert!(!report.global_insights.is_empty());
        assert!(!report.recommendations.is_empty());
        assert!(!report.trend_analysis.trends.is_empty());
        assert!(!report.risk_assessment.risks.is_empty());
    }

    #[test]
    fn test_performance_metrics_calculation() {
        let mut reporter = GlobalReporter::new();
        
        // Create schedules with conflicts
        let base_time = Utc::now();
        let schedules = vec![
            GlobalSchedule::new("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(2)),
            GlobalSchedule::new("project2".to_string(), "UTC".to_string(), base_time + Duration::minutes(30), base_time + Duration::hours(3)), // Overlapping
        ];

        let report = reporter.generate_global_report(&schedules);
        
        // Test performance metrics
        assert_eq!(report.performance_metrics.total_meetings, 2);
        assert!(report.performance_metrics.total_duration > Duration::zero());
        // timezone_conflicts is non-negative by type; no need to assert lower bound
        assert!(report.performance_metrics.conflict_rate >= 0.0);
        assert!(report.performance_metrics.delay_rate >= 0.0);
    }

    #[test]
    fn test_collaboration_metrics_analysis() {
        let mut reporter = GlobalReporter::new();
        
        // Create schedules with participants from different timezones
        let base_time = Utc::now();
        let mut schedule1 = GlobalSchedule::new("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(2));
        let mut schedule2 = GlobalSchedule::new("project2".to_string(), "America/New_York".to_string(), base_time, base_time + Duration::hours(2));
        
        // Add participants
        schedule1.add_participant(GlobalParticipant {
            user_id: "user1".to_string(),
            name: "User 1".to_string(),
            timezone: "UTC".to_string(),
            local_start_time: base_time,
            local_end_time: base_time + Duration::hours(2),
            is_required: true,
        });
        
        schedule2.add_participant(GlobalParticipant {
            user_id: "user2".to_string(),
            name: "User 2".to_string(),
            timezone: "America/New_York".to_string(),
            local_start_time: base_time,
            local_end_time: base_time + Duration::hours(2),
            is_required: true,
        });

        let schedules = vec![schedule1, schedule2];
        let report = reporter.generate_global_report(&schedules);
        
        // Test collaboration metrics
        assert!(report.collaboration_metrics.total_participants > 0);
        // cross_timezone_meetings is non-negative by type; no need to assert lower bound
        assert!(!report.collaboration_metrics.timezone_diversity.is_empty());
        assert!(report.collaboration_metrics.collaboration_index >= 0.0);
    }

    #[test]
    fn test_efficiency_metrics_calculation() {
        let mut reporter = GlobalReporter::new();
        
        // Create schedules with different durations
        let base_time = Utc::now();
        let schedules = vec![
            GlobalSchedule::new("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(1)),
            GlobalSchedule::new("project2".to_string(), "UTC".to_string(), base_time + Duration::hours(2), base_time + Duration::hours(4)),
        ];

        let report = reporter.generate_global_report(&schedules);
        
        // Test efficiency metrics
        assert!(report.efficiency_metrics.total_productive_time > Duration::zero());
        assert!(report.efficiency_metrics.total_waiting_time >= Duration::zero());
        assert!(report.efficiency_metrics.efficiency_ratio >= 0.0 && report.efficiency_metrics.efficiency_ratio <= 1.0);
        assert!(report.efficiency_metrics.average_scheduling_score >= 0.0);
    }

    #[test]
    fn test_trend_analysis_generation() {
        let mut reporter = GlobalReporter::new();
        
        // Create multiple schedules to generate trends
        let base_time = Utc::now();
        let mut schedules = Vec::new();
        
        for i in 0..15 { // Enough to trigger trend analysis
            schedules.push(GlobalSchedule::new(
                format!("project{}", i),
                "UTC".to_string(),
                base_time + Duration::hours(i),
                base_time + Duration::hours(i + 1)
            ));
        }

        let report = reporter.generate_global_report(&schedules);
        
        // Test trend analysis
        assert!(!report.trend_analysis.trends.is_empty());
        assert!(matches!(report.trend_analysis.overall_trend, TrendDirection::Increasing | TrendDirection::Stable));
        
        // Verify trend data
        for trend in &report.trend_analysis.trends {
            assert!(!trend.metric.is_empty());
            assert!(!trend.description.is_empty());
            assert!(trend.confidence >= 0.0 && trend.confidence <= 1.0);
        }
    }

    #[test]
    fn test_risk_assessment_generation() {
        let mut reporter = GlobalReporter::new();
        
        // Create schedules that might trigger risk assessment
        let base_time = Utc::now();
        let schedules = vec![
            GlobalSchedule::new("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(2)),
            GlobalSchedule::new("project2".to_string(), "UTC".to_string(), base_time + Duration::minutes(30), base_time + Duration::hours(3)), // Overlapping
        ];

        let report = reporter.generate_global_report(&schedules);
        
        // Test risk assessment
        assert!(!report.risk_assessment.risks.is_empty());
        assert!(matches!(report.risk_assessment.overall_risk_level, RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical));
        
        // Verify risk data
        for risk in &report.risk_assessment.risks {
            assert!(!risk.description.is_empty());
            assert!(!risk.mitigation.is_empty());
        }
    }

    #[test]
    fn test_insights_and_recommendations() {
        let mut reporter = GlobalReporter::new();
        
        // Create diverse schedule data
        let base_time = Utc::now();
        let schedules = vec![
            GlobalSchedule::new("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(2)),
            GlobalSchedule::new("project2".to_string(), "America/New_York".to_string(), base_time, base_time + Duration::hours(2)),
            GlobalSchedule::new("project3".to_string(), "Europe/London".to_string(), base_time, base_time + Duration::hours(2)),
        ];

        let report = reporter.generate_global_report(&schedules);
        
        // Test insights
        assert!(!report.global_insights.is_empty());
        for insight in &report.global_insights {
            assert!(!insight.title.is_empty());
            assert!(!insight.description.is_empty());
            assert!(insight.confidence >= 0.0 && insight.confidence <= 1.0);
        }
        
        // Test recommendations
        assert!(!report.recommendations.is_empty());
        for recommendation in &report.recommendations {
            assert!(!recommendation.title.is_empty());
            assert!(!recommendation.description.is_empty());
        }
    }

    #[test]
    fn test_export_formats() {
        let mut reporter = GlobalReporter::new();
        let base_time = Utc::now();
        let schedules = vec![GlobalSchedule::new(
            "project1".to_string(),
            "UTC".to_string(),
            base_time,
            base_time + Duration::hours(2),
        )];

        reporter.generate_global_report(&schedules);
        
        // Test all export formats
        let formats = vec![ReportFormat::Json, ReportFormat::Yaml, ReportFormat::Csv];
        
        for format in formats {
            let result = reporter.export_report(format.clone());
            assert!(result.is_ok(), "Failed to export in format: {:?}", format);
            
            let output = result.unwrap();
            assert!(!output.is_empty(), "Export output should not be empty for format: {:?}", format);
        }
    }
}
