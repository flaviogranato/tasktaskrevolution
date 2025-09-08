use chrono::{DateTime, NaiveDate, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid7::Uuid;

use crate::domain::shared::errors::{AppError, AppErrorKind};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ReportType {
    ProjectStatus,
    ResourceUtilization,
    TaskProgress,
    CostAnalysis,
    TimelineAnalysis,
    RiskAssessment,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ReportFormat {
    Html,
    Csv,
    Json,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum MetricType {
    Percentage,
    Duration,
    Currency,
    Count,
    Ratio,
    Custom(String),
}

// ============================================================================
// STRUCTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDashboard {
    pub id: String,
    pub project_id: String,
    pub company_code: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub metrics: ProjectMetrics,
    pub charts: Vec<Chart>,
    pub alerts: Vec<Alert>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub schedule_performance: ScheduleMetrics,
    pub cost_performance: CostMetrics,
    pub resource_performance: ResourceMetrics,
    pub quality_metrics: QualityMetrics,
    pub risk_metrics: RiskMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetrics {
    pub planned_duration: Duration,
    pub actual_duration: Option<Duration>,
    pub remaining_duration: Option<Duration>,
    pub completion_percentage: f64,
    pub schedule_variance: Option<Duration>,
    pub schedule_performance_index: Option<f64>,
    pub critical_path_length: Duration,
    pub slack_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    pub planned_cost: f64,
    pub actual_cost: Option<f64>,
    pub remaining_cost: Option<f64>,
    pub cost_variance: Option<f64>,
    pub cost_performance_index: Option<f64>,
    pub budget_at_completion: f64,
    pub estimate_at_completion: Option<f64>,
    pub estimate_to_complete: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub total_resources: u32,
    pub allocated_resources: u32,
    pub resource_utilization: f64,
    pub overallocation_count: u32,
    pub underallocation_count: u32,
    pub skill_gap_count: u32,
    pub average_workload: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub defect_count: u32,
    pub defect_rate: f64,
    pub rework_hours: f64,
    pub customer_satisfaction: Option<f64>,
    pub quality_gates_passed: u32,
    pub quality_gates_total: u32,
    pub quality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub risk_count: u32,
    pub high_risk_count: u32,
    pub medium_risk_count: u32,
    pub low_risk_count: u32,
    pub risk_exposure: f64,
    pub mitigation_effectiveness: f64,
    pub contingency_budget_used: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub id: String,
    pub name: String,
    pub chart_type: ChartType,
    pub data: ChartData,
    pub options: ChartOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ChartType {
    Gantt,
    Burndown,
    ResourceUtilization,
    CostTrend,
    RiskMatrix,
    Timeline,
    Pie,
    Bar,
    Line,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: Option<String>,
    pub border_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    pub title: Option<String>,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    pub show_legend: bool,
    pub responsive: bool,
    pub animation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum AlertCategory {
    Schedule,
    Cost,
    Resource,
    Quality,
    Risk,
    Security,
    Compliance,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: String,
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub impact: RecommendationImpact,
    pub effort: RecommendationEffort,
    pub implementation_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum RecommendationCategory {
    ScheduleOptimization,
    CostReduction,
    ResourceAllocation,
    QualityImprovement,
    RiskMitigation,
    ProcessImprovement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationImpact {
    pub schedule_impact: Duration,
    pub cost_impact: f64,
    pub quality_impact: f64,
    pub risk_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationEffort {
    pub estimated_hours: f64,
    pub required_skills: Vec<String>,
    pub dependencies: Vec<String>,
    pub timeline: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub content: String,
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub parameters: HashMap<String, String>,
    pub file_size: Option<u64>,
    pub checksum: Option<String>,
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl ProjectDashboard {
    pub fn new(
        project_id: String,
        company_code: String,
        generated_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            project_id,
            company_code,
            generated_at: Utc::now(),
            generated_by,
            metrics: ProjectMetrics::default(),
            charts: Vec::new(),
            alerts: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    pub fn add_chart(&mut self, chart: Chart) {
        self.charts.push(chart);
    }

    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }

    pub fn add_recommendation(&mut self, recommendation: Recommendation) {
        self.recommendations.push(recommendation);
    }

    pub fn get_critical_alerts(&self) -> Vec<&Alert> {
        self.alerts.iter()
            .filter(|alert| alert.severity == AlertSeverity::Critical)
            .collect()
    }

    pub fn get_unacknowledged_alerts(&self) -> Vec<&Alert> {
        self.alerts.iter()
            .filter(|alert| !alert.acknowledged)
            .collect()
    }

    pub fn get_high_priority_recommendations(&self) -> Vec<&Recommendation> {
        self.recommendations.iter()
            .filter(|rec| matches!(rec.priority, RecommendationPriority::High | RecommendationPriority::Critical))
            .collect()
    }

    pub fn calculate_overall_health_score(&self) -> f64 {
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        // Schedule Performance (30%)
        let schedule_score = self.calculate_schedule_score();
        total_score += schedule_score * 0.3;
        weight_sum += 0.3;

        // Cost Performance (25%)
        let cost_score = self.calculate_cost_score();
        total_score += cost_score * 0.25;
        weight_sum += 0.25;

        // Resource Performance (20%)
        let resource_score = self.calculate_resource_score();
        total_score += resource_score * 0.2;
        weight_sum += 0.2;

        // Quality Performance (15%)
        let quality_score = self.calculate_quality_score();
        total_score += quality_score * 0.15;
        weight_sum += 0.15;

        // Risk Performance (10%)
        let risk_score = self.calculate_risk_score();
        total_score += risk_score * 0.1;
        weight_sum += 0.1;

        if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            0.0
        }
    }

    fn calculate_schedule_score(&self) -> f64 {
        if let Some(spi) = self.metrics.schedule_performance.schedule_performance_index {
            if spi >= 1.0 {
                100.0
            } else if spi >= 0.9 {
                90.0 + (spi - 0.9) * 100.0
            } else if spi >= 0.8 {
                80.0 + (spi - 0.8) * 100.0
            } else {
                (spi * 100.0).max(0.0)
            }
        } else {
            50.0 // Default score if SPI not available
        }
    }

    fn calculate_cost_score(&self) -> f64 {
        if let Some(cpi) = self.metrics.cost_performance.cost_performance_index {
            if cpi >= 1.0 {
                100.0
            } else if cpi >= 0.9 {
                90.0 + (cpi - 0.9) * 100.0
            } else if cpi >= 0.8 {
                80.0 + (cpi - 0.8) * 100.0
            } else {
                (cpi * 100.0).max(0.0)
            }
        } else {
            50.0 // Default score if CPI not available
        }
    }

    fn calculate_resource_score(&self) -> f64 {
        let utilization = self.metrics.resource_performance.resource_utilization;
        let overallocation_penalty = self.metrics.resource_performance.overallocation_count as f64 * 5.0;
        let underallocation_penalty = self.metrics.resource_performance.underallocation_count as f64 * 2.0;

        let base_score = utilization * 100.0;
        let final_score = base_score - overallocation_penalty - underallocation_penalty;

        final_score.max(0.0).min(100.0)
    }

    fn calculate_quality_score(&self) -> f64 {
        let quality_score = self.metrics.quality_metrics.quality_score;
        let defect_penalty = self.metrics.quality_metrics.defect_rate * 50.0;

        let final_score = quality_score - defect_penalty;
        final_score.max(0.0).min(100.0)
    }

    fn calculate_risk_score(&self) -> f64 {
        let base_score = 100.0;
        let high_risk_penalty = self.metrics.risk_metrics.high_risk_count as f64 * 15.0;
        let medium_risk_penalty = self.metrics.risk_metrics.medium_risk_count as f64 * 8.0;
        let low_risk_penalty = self.metrics.risk_metrics.low_risk_count as f64 * 3.0;

        let final_score = base_score - high_risk_penalty - medium_risk_penalty - low_risk_penalty;
        final_score.max(0.0).min(100.0)
    }
}

impl Alert {
    pub fn new(
        severity: AlertSeverity,
        category: AlertCategory,
        title: String,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            severity,
            category,
            title,
            message,
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }

    pub fn acknowledge(&mut self, user_id: String) {
        self.acknowledged = true;
        self.acknowledged_by = Some(user_id);
        self.acknowledged_at = Some(Utc::now());
    }

    pub fn is_critical(&self) -> bool {
        self.severity == AlertSeverity::Critical
    }

    pub fn requires_immediate_action(&self) -> bool {
        matches!(self.severity, AlertSeverity::Error | AlertSeverity::Critical)
    }
}

impl Recommendation {
    pub fn new(
        priority: RecommendationPriority,
        category: RecommendationCategory,
        title: String,
        description: String,
        impact: RecommendationImpact,
        effort: RecommendationEffort,
        implementation_steps: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            priority,
            category,
            title,
            description,
            impact,
            effort,
            implementation_steps,
        }
    }

    pub fn is_high_priority(&self) -> bool {
        matches!(self.priority, RecommendationPriority::High | RecommendationPriority::Critical)
    }

    pub fn calculate_roi(&self) -> f64 {
        let total_cost_impact = self.impact.cost_impact.abs();
        let effort_cost = self.effort.estimated_hours * 100.0; // Assuming $100/hour

        if effort_cost > 0.0 {
            total_cost_impact / effort_cost
        } else {
            0.0
        }
    }

    pub fn get_implementation_summary(&self) -> String {
        format!(
            "Effort: {:.1} hours, Timeline: {} days, ROI: {:.2}",
            self.effort.estimated_hours,
            self.effort.timeline.num_days(),
            self.calculate_roi()
        )
    }
}

impl Report {
    pub fn new(
        name: String,
        report_type: ReportType,
        format: ReportFormat,
        content: String,
        generated_by: String,
        parameters: HashMap<String, String>,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            name,
            report_type,
            format,
            content,
            generated_at: Utc::now(),
            generated_by,
            parameters,
            file_size: Some(content.len() as u64),
            checksum: Some(calculate_checksum(&content)),
        }
    }

    pub fn get_file_extension(&self) -> &'static str {
        match self.format {
            ReportFormat::Html => "html",
            ReportFormat::Csv => "csv",
            ReportFormat::Json => "json",
            ReportFormat::Text => "txt",
        }
    }

    pub fn get_mime_type(&self) -> &'static str {
        match self.format {
            ReportFormat::Html => "text/html",
            ReportFormat::Csv => "text/csv",
            ReportFormat::Json => "application/json",
            ReportFormat::Text => "text/plain",
        }
    }

    pub fn is_exportable(&self) -> bool {
        matches!(self.format, ReportFormat::Csv | ReportFormat::Json | ReportFormat::Text)
    }
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for ProjectMetrics {
    fn default() -> Self {
        Self {
            schedule_performance: ScheduleMetrics::default(),
            cost_performance: CostMetrics::default(),
            resource_performance: ResourceMetrics::default(),
            quality_metrics: QualityMetrics::default(),
            risk_metrics: RiskMetrics::default(),
        }
    }
}

impl Default for ScheduleMetrics {
    fn default() -> Self {
        Self {
            planned_duration: Duration::days(0),
            actual_duration: None,
            remaining_duration: None,
            completion_percentage: 0.0,
            schedule_variance: None,
            schedule_performance_index: None,
            critical_path_length: Duration::days(0),
            slack_time: Duration::days(0),
        }
    }
}

impl Default for CostMetrics {
    fn default() -> Self {
        Self {
            planned_cost: 0.0,
            actual_cost: None,
            remaining_cost: None,
            cost_variance: None,
            cost_performance_index: None,
            budget_at_completion: 0.0,
            estimate_at_completion: None,
            estimate_to_complete: None,
        }
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            total_resources: 0,
            allocated_resources: 0,
            resource_utilization: 0.0,
            overallocation_count: 0,
            underallocation_count: 0,
            skill_gap_count: 0,
            average_workload: 0.0,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            defect_count: 0,
            defect_rate: 0.0,
            rework_hours: 0.0,
            customer_satisfaction: None,
            quality_gates_passed: 0,
            quality_gates_total: 0,
            quality_score: 100.0,
        }
    }
}

impl Default for RiskMetrics {
    fn default() -> Self {
        Self {
            risk_count: 0,
            high_risk_count: 0,
            medium_risk_count: 0,
            low_risk_count: 0,
            risk_exposure: 0.0,
            mitigation_effectiveness: 100.0,
            contingency_budget_used: 0.0,
        }
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn calculate_checksum(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_dashboard_creation() {
        let dashboard = ProjectDashboard::new(
            "PROJ-001".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );

        assert_eq!(dashboard.project_id, "PROJ-001");
        assert_eq!(dashboard.company_code, "COMP-001");
        assert_eq!(dashboard.generated_by, "user-001");
        assert!(dashboard.charts.is_empty());
        assert!(dashboard.alerts.is_empty());
        assert!(dashboard.recommendations.is_empty());
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            AlertSeverity::Warning,
            AlertCategory::Schedule,
            "Project Delay".to_string(),
            "Project is behind schedule by 5 days".to_string(),
        );

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.category, AlertCategory::Schedule);
        assert_eq!(alert.title, "Project Delay");
        assert!(!alert.acknowledged);
        assert!(alert.acknowledged_by.is_none());
    }

    #[test]
    fn test_alert_acknowledgment() {
        let mut alert = Alert::new(
            AlertSeverity::Error,
            AlertCategory::Cost,
            "Budget Overrun".to_string(),
            "Project costs exceed budget by 15%".to_string(),
        );

        assert!(!alert.acknowledged);

        alert.acknowledge("manager-001".to_string());

        assert!(alert.acknowledged);
        assert_eq!(alert.acknowledged_by, Some("manager-001".to_string()));
        assert!(alert.acknowledged_at.is_some());
    }

    #[test]
    fn test_alert_severity_checks() {
        let critical_alert = Alert::new(
            AlertSeverity::Critical,
            AlertCategory::Risk,
            "Critical Risk".to_string(),
            "High-risk event detected".to_string(),
        );

        let warning_alert = Alert::new(
            AlertSeverity::Warning,
            AlertCategory::Quality,
            "Quality Issue".to_string(),
            "Minor quality concern".to_string(),
        );

        assert!(critical_alert.is_critical());
        assert!(!warning_alert.is_critical());

        assert!(critical_alert.requires_immediate_action());
        assert!(!warning_alert.requires_immediate_action());
    }

    #[test]
    fn test_recommendation_creation() {
        let impact = RecommendationImpact {
            schedule_impact: Duration::days(2),
            cost_impact: -5000.0,
            quality_impact: 0.1,
            risk_impact: -0.2,
        };

        let effort = RecommendationEffort {
            estimated_hours: 40.0,
            required_skills: vec!["Project Management".to_string()],
            dependencies: vec!["Approval from Stakeholders".to_string()],
            timeline: Duration::days(5),
        };

        let recommendation = Recommendation::new(
            RecommendationPriority::High,
            RecommendationCategory::CostReduction,
            "Optimize Resource Allocation".to_string(),
            "Reduce costs by optimizing resource allocation".to_string(),
            impact,
            effort,
            vec![
                "Analyze current allocation".to_string(),
                "Identify optimization opportunities".to_string(),
                "Implement changes".to_string(),
            ],
        );

        assert_eq!(recommendation.priority, RecommendationPriority::High);
        assert_eq!(recommendation.category, RecommendationCategory::CostReduction);
        assert!(recommendation.is_high_priority());

        let roi = recommendation.calculate_roi();
        assert!(roi > 0.0);

        let summary = recommendation.get_implementation_summary();
        assert!(summary.contains("40.0 hours"));
        assert!(summary.contains("5 days"));
    }

    #[test]
    fn test_report_creation() {
        let mut parameters = HashMap::new();
        parameters.insert("start_date".to_string(), "2024-01-01".to_string());
        parameters.insert("end_date".to_string(), "2024-12-31".to_string());

        let report = Report::new(
            "Monthly Status Report".to_string(),
            ReportType::ProjectStatus,
            ReportFormat::Html,
            "<html><body>Report content</body></html>".to_string(),
            "user-001".to_string(),
            parameters,
        );

        assert_eq!(report.name, "Monthly Status Report");
        assert_eq!(report.report_type, ReportType::ProjectStatus);
        assert_eq!(report.format, ReportFormat::Html);
        assert_eq!(report.get_file_extension(), "html");
        assert_eq!(report.get_mime_type(), "text/html");
        assert!(!report.is_exportable());

        assert_eq!(report.parameters.len(), 2);
        assert_eq!(report.parameters.get("start_date"), Some(&"2024-01-01".to_string()));
    }

    #[test]
    fn test_csv_report_export() {
        let report = Report::new(
            "Resource Utilization".to_string(),
            ReportType::ResourceUtilization,
            ReportFormat::Csv,
            "Name,Allocation,Utilization\nJohn,80,0.8\nJane,60,0.6".to_string(),
            "user-001".to_string(),
            HashMap::new(),
        );

        assert_eq!(report.get_file_extension(), "csv");
        assert_eq!(report.get_mime_type(), "text/csv");
        assert!(report.is_exportable());
    }

    #[test]
    fn test_dashboard_health_score_calculation() {
        let mut dashboard = ProjectDashboard::new(
            "PROJ-001".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );

        // Configurar métricas para um projeto saudável
        dashboard.metrics.schedule_performance.schedule_performance_index = Some(1.1);
        dashboard.metrics.cost_performance.cost_performance_index = Some(1.05);
        dashboard.metrics.resource_performance.resource_utilization = 0.85;
        dashboard.metrics.resource_performance.overallocation_count = 0;
        dashboard.metrics.resource_performance.underallocation_count = 1;
        dashboard.metrics.quality_metrics.quality_score = 95.0;
        dashboard.metrics.quality_metrics.defect_rate = 0.02;
        dashboard.metrics.risk_metrics.high_risk_count = 0;
        dashboard.metrics.risk_metrics.medium_risk_count = 1;
        dashboard.metrics.risk_metrics.low_risk_count = 2;

        let health_score = dashboard.calculate_overall_health_score();

        // Score deve estar entre 0 e 100
        assert!(health_score >= 0.0 && health_score <= 100.0);

        // Para um projeto com métricas boas, o score deve ser alto
        assert!(health_score > 70.0);
    }

    #[test]
    fn test_dashboard_critical_alerts() {
        let mut dashboard = ProjectDashboard::new(
            "PROJ-001".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );

        let critical_alert = Alert::new(
            AlertSeverity::Critical,
            AlertCategory::Risk,
            "Critical Risk".to_string(),
            "High-risk event detected".to_string(),
        );

        let warning_alert = Alert::new(
            AlertSeverity::Warning,
            AlertCategory::Schedule,
            "Minor Delay".to_string(),
            "Project slightly behind schedule".to_string(),
        );

        dashboard.add_alert(critical_alert);
        dashboard.add_alert(warning_alert);

        let critical_alerts = dashboard.get_critical_alerts();
        assert_eq!(critical_alerts.len(), 1);
        assert_eq!(critical_alerts[0].severity, AlertSeverity::Critical);

        let unacknowledged = dashboard.get_unacknowledged_alerts();
        assert_eq!(unacknowledged.len(), 2);
    }
}
