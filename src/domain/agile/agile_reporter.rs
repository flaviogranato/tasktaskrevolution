use crate::domain::agile::agile_models::*;
use crate::domain::agile::sprint_manager::{SprintManager, SprintHealth, SprintStatistics};
use crate::domain::agile::kanban_board::{KanbanBoardManager, BoardStatistics};
use crate::domain::agile::velocity_tracker::{VelocityTracker, TeamVelocityMetrics, VelocityDistribution, VelocityPatternAnalysis};
use crate::domain::agile::burndown_calculator::{BurndownCalculator, BurndownChartData, BurndownVelocity, SprintPrediction, BurndownChartType};
use chrono::{DateTime, Utc};
use serde::Serialize;

/// AgileReporter handles agile-specific reporting and analytics
#[derive(Debug, Clone)]
pub struct AgileReporter {
    sprint_manager: SprintManager,
    kanban_manager: KanbanBoardManager,
    velocity_tracker: VelocityTracker,
    burndown_calculator: BurndownCalculator,
}

impl AgileReporter {
    /// Create a new AgileReporter
    pub fn new() -> Self {
        Self {
            sprint_manager: SprintManager::new(),
            kanban_manager: KanbanBoardManager::new(),
            velocity_tracker: VelocityTracker::new(),
            burndown_calculator: BurndownCalculator::new(),
        }
    }

    /// Generate sprint report
    pub fn generate_sprint_report(&self, sprint_id: &str) -> Result<SprintReport, String> {
        if let Some(sprint) = self.sprint_manager.get_sprint(sprint_id) {
            let progress = self.sprint_manager.get_sprint_progress(sprint_id)?;
            let health = self.sprint_manager.get_sprint_health(sprint_id)?;
            
            let burndown_data = self.burndown_calculator.get_burndown_chart_data(sprint_id)?;
            let velocity_analysis = self.burndown_calculator.calculate_burndown_velocity(sprint_id)?;
            let prediction = self.burndown_calculator.predict_sprint_completion(sprint_id)?;

            Ok(SprintReport {
                sprint_id: sprint_id.to_string(),
                sprint_name: sprint.name.clone(),
                project_id: sprint.project_id.clone(),
                status: sprint.status.clone(),
                start_date: sprint.start_date,
                end_date: sprint.end_date,
                duration_days: sprint.duration_days(),
                progress_percentage: progress * 100.0,
                health_status: health,
                metrics: sprint.metrics.clone(),
                burndown_data,
                velocity_analysis: velocity_analysis.clone(),
                prediction: prediction.clone(),
                team_capacity: sprint.capacity.clone(),
                goals: sprint.goals.clone(),
                task_count: sprint.tasks.len(),
                generated_at: Utc::now(),
            })
        } else {
            Err(format!("Sprint {} not found", sprint_id))
        }
    }

    /// Generate team velocity report
    pub fn generate_velocity_report(&self, team_id: &str) -> Result<VelocityReport, String> {
        if let Some(velocity_history) = self.velocity_tracker.get_team_velocity_history(team_id) {
            if velocity_history.is_empty() {
                return Err("No velocity data available for team".to_string());
            }

            let trend = self.velocity_tracker.calculate_velocity_trend(team_id)
                .ok_or("Unable to calculate velocity trend")?;
            
            let metrics = self.velocity_tracker.calculate_team_metrics(team_id)
                .ok_or("Unable to calculate team metrics")?;
            
            let distribution = self.velocity_tracker.get_velocity_distribution(team_id);
            let pattern_analysis = self.velocity_tracker.analyze_velocity_patterns(team_id);

            Ok(VelocityReport {
                team_id: team_id.to_string(),
                sprint_count: velocity_history.len(),
                velocity_history: velocity_history.clone(),
                trend,
                metrics,
                distribution,
                pattern_analysis,
                generated_at: Utc::now(),
            })
        } else {
            Err(format!("Team {} not found", team_id))
        }
    }

    /// Generate Kanban board report
    pub fn generate_kanban_report(&mut self, board_id: &str) -> Result<KanbanReport, String> {
        // Get board info first
        let (board_name, project_id, columns) = {
            if let Some(board) = self.kanban_manager.get_board(board_id) {
                (board.name.clone(), board.project_id.clone(), board.columns.clone())
            } else {
                return Err(format!("Board {} not found", board_id));
            }
        };
        
        // Now do mutable borrows
        let statistics = self.kanban_manager.get_board_statistics(board_id)?;
        let wip_violations = self.kanban_manager.check_wip_violations(board_id)?;
        let flow_metrics = self.kanban_manager.calculate_flow_metrics(board_id)?;

        Ok(KanbanReport {
            board_id: board_id.to_string(),
            board_name,
            project_id,
            statistics,
            wip_violations,
            flow_metrics,
            columns,
            generated_at: Utc::now(),
        })
    }

    /// Generate agile metrics dashboard
    pub fn generate_agile_dashboard(&self, project_id: Option<&str>) -> AgileDashboard {
        let sprint_stats = self.sprint_manager.get_sprint_statistics(project_id);
        let all_boards = self.kanban_manager.get_all_boards();
        let all_team_metrics = self.velocity_tracker.get_all_team_metrics();

        // Filter boards by project if specified
        let project_boards: Vec<&KanbanBoard> = if let Some(pid) = project_id {
            all_boards.iter().filter(|b| b.project_id == *pid).copied().collect()
        } else {
            all_boards
        };

        // Calculate overall metrics
        let total_boards = project_boards.len();
        let total_wip_violations: usize = project_boards.iter()
            .map(|b| self.kanban_manager.check_wip_violations(&b.id).unwrap_or_default().len())
            .sum();

        let average_velocity = if !all_team_metrics.is_empty() {
            all_team_metrics.iter()
                .map(|m| m.average_velocity)
                .sum::<f64>() / all_team_metrics.len() as f64
        } else {
            0.0
        };

        let total_story_points = all_team_metrics.iter()
            .map(|m| m.total_story_points_completed)
            .sum();

        AgileDashboard {
            project_id: project_id.map(|s| s.to_string()),
            sprint_statistics: sprint_stats,
            total_boards,
            total_wip_violations,
            average_velocity,
            total_story_points,
            team_count: all_team_metrics.len(),
            active_sprints: self.sprint_manager.list_active_sprints().len(),
            generated_at: Utc::now(),
        }
    }

    /// Generate retrospective report
    pub fn generate_retrospective_report(&self, sprint_id: &str, retrospective: SprintRetrospective) -> RetrospectiveReport {
        let sprint_report = self.generate_sprint_report(sprint_id).unwrap_or_else(|_| {
            SprintReport {
                sprint_id: sprint_id.to_string(),
                sprint_name: "Unknown".to_string(),
                project_id: "Unknown".to_string(),
                status: SprintStatus::Completed,
                start_date: Utc::now(),
                end_date: Utc::now(),
                duration_days: 0,
                progress_percentage: 0.0,
                health_status: SprintHealth::OnTrack,
                metrics: SprintMetrics::default(),
                burndown_data: BurndownChartData {
                    sprint_id: sprint_id.to_string(),
                    sprint_name: "Unknown".to_string(),
                    ideal_burndown: Vec::new(),
                    actual_burndown: Vec::new(),
                    velocity: BurndownVelocity::default(),
                    prediction: SprintPrediction::default(),
                    chart_type: BurndownChartType::StoryPoints,
                },
                velocity_analysis: BurndownVelocity::default(),
                prediction: SprintPrediction::default(),
                team_capacity: SprintCapacity {
                    total_story_points: 0,
                    team_members: Vec::new(),
                    working_days: Vec::new(),
                    hours_per_day: 0.0,
                    availability_percentage: 0.0,
                },
                goals: Vec::new(),
                task_count: 0,
                generated_at: Utc::now(),
            }
        });

        let action_items_summary = ActionItemsSummary {
            total_action_items: retrospective.action_items.len(),
            open_action_items: retrospective.action_items.iter()
                .filter(|ai| matches!(ai.status, ActionItemStatus::Open))
                .count(),
            in_progress_action_items: retrospective.action_items.iter()
                .filter(|ai| matches!(ai.status, ActionItemStatus::InProgress))
                .count(),
            completed_action_items: retrospective.action_items.iter()
                .filter(|ai| matches!(ai.status, ActionItemStatus::Completed))
                .count(),
            high_priority_items: retrospective.action_items.iter()
                .filter(|ai| matches!(ai.priority, ActionItemPriority::High | ActionItemPriority::Critical))
                .count(),
        };
        
        let team_satisfaction = retrospective.team_satisfaction;
        let sprint_rating = retrospective.sprint_rating;
        
        RetrospectiveReport {
            sprint_report,
            retrospective,
            action_items_summary,
            team_satisfaction,
            sprint_rating,
            generated_at: Utc::now(),
        }
    }

    /// Generate burndown chart data
    pub fn generate_burndown_chart(&self, sprint_id: &str) -> Result<BurndownChartData, String> {
        self.burndown_calculator.get_burndown_chart_data(sprint_id)
    }

    /// Generate velocity trend chart data
    pub fn generate_velocity_trend_chart(&self, team_id: &str) -> Result<VelocityTrendChartData, String> {
        if let Some(trend) = self.velocity_tracker.calculate_velocity_trend(team_id) {
            let chart_data: Vec<VelocityChartPoint> = trend.sprints.iter().map(|s| VelocityChartPoint {
                sprint_id: s.sprint_id.clone(),
                velocity: s.velocity,
                story_points: s.story_points_completed,
                date: s.completion_date,
            }).collect();
            
            Ok(VelocityTrendChartData {
                team_id: team_id.to_string(),
                trend,
                chart_data,
                generated_at: Utc::now(),
            })
        } else {
            Err(format!("No velocity trend data for team {}", team_id))
        }
    }

    /// Generate flow efficiency report
    pub fn generate_flow_efficiency_report(&mut self, board_id: &str) -> Result<FlowEfficiencyReport, String> {
        // Get board info first
        let board_name = {
            if let Some(board) = self.kanban_manager.get_board(board_id) {
                board.name.clone()
            } else {
                return Err(format!("Board {} not found", board_id));
            }
        };
        
        // Now do mutable borrows
        let flow_metrics = self.kanban_manager.calculate_flow_metrics(board_id)?;
        let movements = self.kanban_manager.get_task_movements(board_id).cloned().unwrap_or_default();

        // Calculate flow efficiency metrics
        let total_tasks = movements.len();
        let completed_tasks = movements.iter()
            .filter(|m| m.to_column == "Done" || m.to_column == "Completed")
            .count();

        let efficiency_percentage = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        let bottleneck_analysis = BottleneckAnalysis {
            bottleneck_columns: flow_metrics.bottleneck_columns.clone(),
            average_wip: flow_metrics.average_wip,
            recommendations: self.generate_flow_recommendations(&flow_metrics),
        };
        
        Ok(FlowEfficiencyReport {
            board_id: board_id.to_string(),
            board_name,
            flow_metrics,
            total_tasks,
            completed_tasks,
            efficiency_percentage,
            bottleneck_analysis,
            generated_at: Utc::now(),
        })
    }

    /// Generate flow recommendations
    fn generate_flow_recommendations(&self, flow_metrics: &FlowMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !flow_metrics.bottleneck_columns.is_empty() {
            recommendations.push(format!("Address bottlenecks in columns: {}", 
                flow_metrics.bottleneck_columns.join(", ")));
        }

        if flow_metrics.flow_efficiency < 0.5 {
            recommendations.push("Low flow efficiency - consider reducing WIP limits and improving handoffs".to_string());
        }

        if flow_metrics.cycle_time.num_days() > 14 {
            recommendations.push("Long cycle time - consider breaking down tasks and improving process flow".to_string());
        }

        if flow_metrics.throughput < 1.0 {
            recommendations.push("Low throughput - review team capacity and task sizing".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Flow efficiency is good - continue current practices".to_string());
        }

        recommendations
    }

    /// Export report to different formats
    pub fn export_report<T: AgileReportExport + Serialize>(&self, report: &T, format: ReportFormat) -> Result<String, String> {
        match format {
            ReportFormat::Json => {
                serde_json::to_string_pretty(report)
                    .map_err(|e| format!("JSON export failed: {}", e))
            },
            ReportFormat::Csv => {
                report.to_csv()
                    .map_err(|e| format!("CSV export failed: {}", e))
            },
            ReportFormat::Html => {
                report.to_html()
                    .map_err(|e| format!("HTML export failed: {}", e))
            },
        }
    }
}

/// Sprint report
#[derive(Debug, Clone)]
pub struct SprintReport {
    pub sprint_id: String,
    pub sprint_name: String,
    pub project_id: String,
    pub status: SprintStatus,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_days: u32,
    pub progress_percentage: f64,
    pub health_status: SprintHealth,
    pub metrics: SprintMetrics,
    pub burndown_data: BurndownChartData,
    pub velocity_analysis: BurndownVelocity,
    pub prediction: SprintPrediction,
    pub team_capacity: SprintCapacity,
    pub goals: Vec<String>,
    pub task_count: usize,
    pub generated_at: DateTime<Utc>,
}

/// Velocity report
#[derive(Debug, Clone)]
pub struct VelocityReport {
    pub team_id: String,
    pub sprint_count: usize,
    pub velocity_history: Vec<VelocityData>,
    pub trend: VelocityTrend,
    pub metrics: TeamVelocityMetrics,
    pub distribution: Option<VelocityDistribution>,
    pub pattern_analysis: Option<VelocityPatternAnalysis>,
    pub generated_at: DateTime<Utc>,
}

/// Kanban report
#[derive(Debug, Clone)]
pub struct KanbanReport {
    pub board_id: String,
    pub board_name: String,
    pub project_id: String,
    pub statistics: BoardStatistics,
    pub wip_violations: Vec<String>,
    pub flow_metrics: FlowMetrics,
    pub columns: Vec<KanbanColumn>,
    pub generated_at: DateTime<Utc>,
}

/// Agile dashboard
#[derive(Debug, Clone, Serialize)]
pub struct AgileDashboard {
    pub project_id: Option<String>,
    pub sprint_statistics: SprintStatistics,
    pub total_boards: usize,
    pub total_wip_violations: usize,
    pub average_velocity: f64,
    pub total_story_points: u32,
    pub team_count: usize,
    pub active_sprints: usize,
    pub generated_at: DateTime<Utc>,
}

/// Retrospective report
#[derive(Debug, Clone)]
pub struct RetrospectiveReport {
    pub sprint_report: SprintReport,
    pub retrospective: SprintRetrospective,
    pub action_items_summary: ActionItemsSummary,
    pub team_satisfaction: f64,
    pub sprint_rating: f64,
    pub generated_at: DateTime<Utc>,
}

/// Action items summary
#[derive(Debug, Clone)]
pub struct ActionItemsSummary {
    pub total_action_items: usize,
    pub open_action_items: usize,
    pub in_progress_action_items: usize,
    pub completed_action_items: usize,
    pub high_priority_items: usize,
}

/// Velocity trend chart data
#[derive(Debug, Clone)]
pub struct VelocityTrendChartData {
    pub team_id: String,
    pub trend: VelocityTrend,
    pub chart_data: Vec<VelocityChartPoint>,
    pub generated_at: DateTime<Utc>,
}

/// Velocity chart point
#[derive(Debug, Clone)]
pub struct VelocityChartPoint {
    pub sprint_id: String,
    pub velocity: f64,
    pub story_points: u32,
    pub date: DateTime<Utc>,
}

/// Flow efficiency report
#[derive(Debug, Clone)]
pub struct FlowEfficiencyReport {
    pub board_id: String,
    pub board_name: String,
    pub flow_metrics: FlowMetrics,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub efficiency_percentage: f64,
    pub bottleneck_analysis: BottleneckAnalysis,
    pub generated_at: DateTime<Utc>,
}

/// Bottleneck analysis
#[derive(Debug, Clone)]
pub struct BottleneckAnalysis {
    pub bottleneck_columns: Vec<String>,
    pub average_wip: f64,
    pub recommendations: Vec<String>,
}

/// Report format
#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
}

/// Trait for report export functionality
pub trait AgileReportExport {
    fn to_csv(&self) -> Result<String, String>;
    fn to_html(&self) -> Result<String, String>;
}

impl AgileReportExport for AgileDashboard {
    fn to_csv(&self) -> Result<String, String> {
        let mut csv = String::from("metric,value\n");
        csv.push_str(&format!("total_sprints,{}\n", self.sprint_statistics.total_sprints));
        csv.push_str(&format!("completed_sprints,{}\n", self.sprint_statistics.completed_sprints));
        csv.push_str(&format!("active_sprints,{}\n", self.active_sprints));
        csv.push_str(&format!("total_boards,{}\n", self.total_boards));
        csv.push_str(&format!("total_wip_violations,{}\n", self.total_wip_violations));
        csv.push_str(&format!("average_velocity,{}\n", self.average_velocity));
        csv.push_str(&format!("total_story_points,{}\n", self.total_story_points));
        csv.push_str(&format!("team_count,{}\n", self.team_count));
        Ok(csv)
    }

    fn to_html(&self) -> Result<String, String> {
        Ok(format!(
            r#"<div class="agile-dashboard">
                <h2>Agile Dashboard</h2>
                <div class="metrics">
                    <div class="metric"><span class="label">Total Sprints:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">Completed Sprints:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">Active Sprints:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">Total Boards:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">WIP Violations:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">Average Velocity:</span> <span class="value">{:.2}</span></div>
                    <div class="metric"><span class="label">Total Story Points:</span> <span class="value">{}</span></div>
                    <div class="metric"><span class="label">Team Count:</span> <span class="value">{}</span></div>
                </div>
            </div>"#,
            self.sprint_statistics.total_sprints,
            self.sprint_statistics.completed_sprints,
            self.active_sprints,
            self.total_boards,
            self.total_wip_violations,
            self.average_velocity,
            self.total_story_points,
            self.team_count
        ))
    }
}

impl Default for AgileReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agile_reporter_creation() {
        let reporter = AgileReporter::new();
        // Test that reporter is created successfully
        assert!(true);
    }

    #[test]
    fn test_generate_agile_dashboard() {
        let reporter = AgileReporter::new();
        let dashboard = reporter.generate_agile_dashboard(None);
        
        assert_eq!(dashboard.total_boards, 0);
        assert_eq!(dashboard.total_wip_violations, 0);
        assert_eq!(dashboard.average_velocity, 0.0);
        assert_eq!(dashboard.team_count, 0);
    }

    #[test]
    fn test_export_report_formats() {
        let reporter = AgileReporter::new();
        let dashboard = reporter.generate_agile_dashboard(None);
        
        // Test JSON export
        let json_result = reporter.export_report(&dashboard, ReportFormat::Json);
        assert!(json_result.is_ok());
        
        // Test that JSON contains expected fields
        let json_str = json_result.unwrap();
        assert!(json_str.contains("total_boards"));
        assert!(json_str.contains("average_velocity"));
    }
}
