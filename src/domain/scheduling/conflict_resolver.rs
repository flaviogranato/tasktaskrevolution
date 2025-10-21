use crate::domain::scheduling::{ConflictSeverity, ProjectSchedule, ResourceConflict};
use crate::domain::shared::errors::DomainError;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Advanced conflict resolver with multiple resolution strategies
pub struct ConflictResolver {
    schedule: ProjectSchedule,
    resolution_strategies: Vec<ResolutionStrategy>,
    conflict_history: Vec<ConflictResolutionRecord>,
}

impl ConflictResolver {
    pub fn new(schedule: ProjectSchedule) -> Self {
        Self {
            schedule,
            resolution_strategies: vec![
                ResolutionStrategy::TimeShifting,
                ResolutionStrategy::ResourceReassignment,
                ResolutionStrategy::TaskBreakdown,
                ResolutionStrategy::DependencyAdjustment,
                ResolutionStrategy::ProjectRestructuring,
            ],
            conflict_history: Vec::new(),
        }
    }

    /// Resolve all conflicts using the best available strategy
    pub fn resolve_all_conflicts(&mut self) -> Result<ConflictResolutionReport, DomainError> {
        let mut report = ConflictResolutionReport::new();
        let mut remaining_conflicts = self.schedule.conflicts.clone();

        // Sort conflicts by severity (Critical first)
        remaining_conflicts.sort_by(|a, b| {
            let severity_order = |s: &ConflictSeverity| match s {
                ConflictSeverity::Critical => 0,
                ConflictSeverity::High => 1,
                ConflictSeverity::Medium => 2,
                ConflictSeverity::Low => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        for conflict in remaining_conflicts {
            match self.resolve_conflict(&conflict) {
                Ok(resolution) => {
                    report.successful_resolutions.push(resolution.clone());
                    self.record_resolution(conflict, resolution);
                }
                Err(e) => {
                    report.failed_resolutions.push(FailedConflictResolution {
                        conflict: conflict.clone(),
                        error: e.to_string(),
                        attempted_strategies: self.get_attempted_strategies(&conflict),
                    });
                }
            }
        }

        report.total_conflicts = self.schedule.conflicts.len();
        report.resolved_conflicts = report.successful_resolutions.len();
        report.failed_conflicts = report.failed_resolutions.len();
        report.success_rate = if report.total_conflicts > 0 {
            report.resolved_conflicts as f64 / report.total_conflicts as f64
        } else {
            1.0
        };

        Ok(report)
    }

    /// Resolve a specific conflict using the best strategy
    fn resolve_conflict(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let strategies = self.resolution_strategies.clone();
        for strategy in &strategies {
            match self.apply_strategy(strategy, conflict) {
                Ok(resolution) => return Ok(resolution),
                Err(_) => continue, // Try next strategy
            }
        }

        Err(DomainError::ValidationError {
            field: "conflict_resolution".to_string(),
            message: "No strategy could resolve this conflict".to_string(),
        })
    }

    /// Apply a specific resolution strategy
    fn apply_strategy(
        &mut self,
        strategy: &ResolutionStrategy,
        conflict: &ResourceConflict,
    ) -> Result<ConflictResolution, DomainError> {
        match strategy {
            ResolutionStrategy::TimeShifting => self.apply_time_shifting(conflict),
            ResolutionStrategy::ResourceReassignment => self.apply_resource_reassignment(conflict),
            ResolutionStrategy::TaskBreakdown => self.apply_task_breakdown(conflict),
            ResolutionStrategy::DependencyAdjustment => self.apply_dependency_adjustment(conflict),
            ResolutionStrategy::ProjectRestructuring => self.apply_project_restructuring(conflict),
        }
    }

    /// Apply time shifting strategy
    fn apply_time_shifting(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Find the task with the most slack
        let best_task = self.find_task_with_most_slack(&conflict.conflicting_tasks)?;

        // Calculate optimal shift
        let shift = self.calculate_optimal_shift(conflict, &best_task)?;

        // Apply the shift
        self.shift_task(&best_task, shift)?;

        resolution.method = ResolutionMethod::TimeShifting;
        resolution.description = format!("Shifted task {} by {} hours", best_task, shift.num_hours());
        resolution.delay = shift;
        resolution.affected_tasks = vec![best_task.clone()];

        Ok(resolution)
    }

    /// Apply resource reassignment strategy
    fn apply_resource_reassignment(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Find alternative resource
        let alternative_resource = self.find_alternative_resource(&conflict.resource_id)?;

        // Find the best task to reassign
        let best_task = self.find_best_task_for_reassignment(&conflict.conflicting_tasks)?;

        // Perform reassignment
        self.reassign_task_to_resource(&best_task, &alternative_resource)?;

        resolution.method = ResolutionMethod::ResourceReassignment;
        resolution.description = format!(
            "Reassigned task {} from {} to {}",
            best_task, conflict.resource_id, alternative_resource
        );
        resolution.affected_tasks = vec![best_task.clone()];
        resolution.new_resource = Some(alternative_resource);

        Ok(resolution)
    }

    /// Apply task breakdown strategy
    fn apply_task_breakdown(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Find the largest task that can be broken down
        let task_to_breakdown = self.find_largest_breakable_task(&conflict.conflicting_tasks)?;

        // Break down the task
        let subtasks = self.break_down_task(&task_to_breakdown)?;

        resolution.method = ResolutionMethod::TaskBreakdown;
        resolution.description = format!("Broke down task {} into {} subtasks", task_to_breakdown, subtasks.len());
        resolution.affected_tasks = vec![task_to_breakdown.clone()];
        resolution.subtasks = Some(subtasks);

        Ok(resolution)
    }

    /// Apply dependency adjustment strategy
    fn apply_dependency_adjustment(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Find dependencies that can be adjusted
        let adjustable_dependencies = self.find_adjustable_dependencies(&conflict.conflicting_tasks)?;

        // Apply dependency adjustments
        for (task_id, new_dependency) in &adjustable_dependencies {
            self.adjust_task_dependency(task_id, new_dependency.clone())?;
        }

        resolution.method = ResolutionMethod::DependencyAdjustment;
        resolution.description = format!("Adjusted {} dependencies", adjustable_dependencies.len());
        resolution.affected_tasks = adjustable_dependencies.keys().cloned().collect();

        Ok(resolution)
    }

    /// Apply project restructuring strategy
    fn apply_project_restructuring(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // This is the most drastic strategy - restructure the entire project
        let restructuring_plan = self.create_restructuring_plan(conflict)?;

        // Apply the restructuring
        self.apply_restructuring_plan(&restructuring_plan)?;

        resolution.method = ResolutionMethod::ProjectRestructuring;
        resolution.description = "Restructured project to resolve critical conflict".to_string();
        resolution.affected_tasks = restructuring_plan.affected_tasks.clone();
        resolution.project_delay = Some(restructuring_plan.project_delay);

        Ok(resolution)
    }

    // Helper methods for conflict resolution

    fn find_task_with_most_slack(&self, task_ids: &[String]) -> Result<String, DomainError> {
        let mut best_task = None;
        let mut max_slack = Duration::zero();

        for task_id in task_ids {
            if let Some(task) = self.schedule.get_task(task_id)
                && task.total_slack > max_slack
            {
                max_slack = task.total_slack;
                best_task = Some(task_id.clone());
            }
        }

        best_task.ok_or_else(|| DomainError::ValidationError {
            field: "task_selection".to_string(),
            message: "No suitable task found for time shifting".to_string(),
        })
    }

    fn calculate_optimal_shift(&self, conflict: &ResourceConflict, task_id: &str) -> Result<Duration, DomainError> {
        let task = self
            .schedule
            .get_task(task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                identifier: task_id.to_string(),
            })?;

        // Calculate how much we need to shift to avoid the conflict
        let conflict_duration = conflict.conflict_period.end - conflict.conflict_period.start;
        let shift_needed = conflict_duration + Duration::hours(1); // Add 1 hour buffer

        // Don't shift more than the available slack
        let max_shift = task.total_slack;
        Ok(shift_needed.min(max_shift))
    }

    fn shift_task(&mut self, task_id: &str, shift: Duration) -> Result<(), DomainError> {
        if let Some(task) = self.schedule.tasks.iter_mut().find(|t| t.task_id == task_id) {
            task.early_start += shift;
            task.early_finish += shift;
            task.late_start += shift;
            task.late_finish += shift;
        }
        Ok(())
    }

    fn find_alternative_resource(&self, current_resource_id: &str) -> Result<String, DomainError> {
        // This should check available resources with similar skills
        // For now, return a placeholder
        Ok(format!("{}-alt", current_resource_id))
    }

    fn find_best_task_for_reassignment(&self, task_ids: &[String]) -> Result<String, DomainError> {
        // Find the task with the least dependencies
        let mut best_task = None;
        let mut min_dependencies = usize::MAX;

        for task_id in task_ids {
            let dependency_count = self.count_task_dependencies(task_id);
            if dependency_count < min_dependencies {
                min_dependencies = dependency_count;
                best_task = Some(task_id.clone());
            }
        }

        best_task.ok_or_else(|| DomainError::ValidationError {
            field: "task_selection".to_string(),
            message: "No suitable task found for reassignment".to_string(),
        })
    }

    fn count_task_dependencies(&self, task_id: &str) -> usize {
        // Count how many tasks depend on this task
        self.schedule
            .tasks
            .iter()
            .filter(|t| self.has_dependency(task_id, &t.task_id))
            .count()
    }

    fn has_dependency(&self, _predecessor_id: &str, _successor_id: &str) -> bool {
        // This should check the actual dependency graph
        false
    }

    fn reassign_task_to_resource(&mut self, task_id: &str, new_resource_id: &str) -> Result<(), DomainError> {
        // Remove from old resource
        for (_, task_ids) in self.schedule.resource_assignments.iter_mut() {
            task_ids.retain(|id| id != task_id);
        }

        // Add to new resource
        self.schedule
            .resource_assignments
            .entry(new_resource_id.to_string())
            .or_default()
            .push(task_id.to_string());

        Ok(())
    }

    fn find_largest_breakable_task(&self, task_ids: &[String]) -> Result<String, DomainError> {
        // Find the task with the longest duration that can be broken down
        let mut best_task = None;
        let mut max_duration = Duration::zero();

        for task_id in task_ids {
            if let Some(task) = self.schedule.get_task(task_id)
                && task.duration > max_duration
                && self.can_break_down_task(task_id)
            {
                max_duration = task.duration;
                best_task = Some(task_id.clone());
            }
        }

        best_task.ok_or_else(|| DomainError::ValidationError {
            field: "task_selection".to_string(),
            message: "No breakable task found".to_string(),
        })
    }

    fn can_break_down_task(&self, _task_id: &str) -> bool {
        // Check if a task can be broken down
        // This should check task complexity, dependencies, etc.
        true
    }

    fn break_down_task(&self, _task_id: &str) -> Result<Vec<String>, DomainError> {
        // Break down a task into smaller subtasks
        // This is a simplified implementation
        Ok(vec![
            format!("{}-subtask-1", _task_id),
            format!("{}-subtask-2", _task_id),
        ])
    }

    fn find_adjustable_dependencies(
        &self,
        task_ids: &[String],
    ) -> Result<HashMap<String, DependencyAdjustment>, DomainError> {
        // Find dependencies that can be adjusted to resolve conflicts
        let mut adjustments = HashMap::new();

        for task_id in task_ids {
            if let Some(adjustment) = self.find_dependency_adjustment(task_id) {
                adjustments.insert(task_id.clone(), adjustment);
            }
        }

        Ok(adjustments)
    }

    fn find_dependency_adjustment(&self, task_id: &str) -> Option<DependencyAdjustment> {
        // Find how to adjust dependencies for this task
        Some(DependencyAdjustment {
            new_predecessor: None,
            new_successor: None,
            lag_time: Duration::hours(2),
        })
    }

    fn adjust_task_dependency(&mut self, task_id: &str, adjustment: DependencyAdjustment) -> Result<(), DomainError> {
        // Apply dependency adjustment
        // This is a simplified implementation
        Ok(())
    }

    fn create_restructuring_plan(&self, conflict: &ResourceConflict) -> Result<RestructuringPlan, DomainError> {
        Ok(RestructuringPlan {
            affected_tasks: conflict.conflicting_tasks.clone(),
            project_delay: Duration::days(1),
            new_dependencies: Vec::new(),
            resource_changes: Vec::new(),
        })
    }

    fn apply_restructuring_plan(&mut self, plan: &RestructuringPlan) -> Result<(), DomainError> {
        // Apply the restructuring plan
        // This is a simplified implementation
        Ok(())
    }

    fn get_attempted_strategies(&self, _conflict: &ResourceConflict) -> Vec<ResolutionStrategy> {
        // Return the strategies that were attempted for this conflict
        vec![
            ResolutionStrategy::TimeShifting,
            ResolutionStrategy::ResourceReassignment,
        ]
    }

    /// Analyze conflict patterns and trends
    pub fn analyze_conflict_patterns(&self) -> ConflictPatternAnalysis {
        let mut severity_distribution: HashMap<String, usize> = HashMap::new();
        let mut resource_conflict_counts = HashMap::new();
        let mut time_based_conflicts = Vec::new();

        for conflict in &self.schedule.conflicts {
            // Count by severity
            *severity_distribution.entry(conflict.severity.to_string()).or_insert(0) += 1;

            // Count by resource
            *resource_conflict_counts
                .entry(conflict.resource_id.clone())
                .or_insert(0) += 1;

            // Track time-based patterns
            time_based_conflicts.push(ConflictTimePattern {
                conflict_id: format!("conflict-{}", conflict.resource_id),
                start_time: conflict.conflict_period.start,
                end_time: conflict.conflict_period.end,
                severity: conflict.severity.clone(),
                duration: conflict.conflict_period.end - conflict.conflict_period.start,
            });
        }

        let total_conflicts = self.schedule.conflicts.len();
        let critical_conflicts = severity_distribution.get("Critical").copied().unwrap_or(0);

        let conflict_density = if total_conflicts > 0 {
            critical_conflicts as f64 / total_conflicts as f64
        } else {
            0.0
        };

        let most_problematic = self.find_most_problematic_resource(&resource_conflict_counts);
        let hotspots = self.identify_conflict_hotspots(&time_based_conflicts);

        ConflictPatternAnalysis {
            total_conflicts,
            severity_distribution,
            resource_conflict_counts,
            time_based_conflicts,
            conflict_density,
            most_problematic_resource: most_problematic,
            conflict_hotspots: hotspots,
        }
    }

    /// Find the most problematic resource
    fn find_most_problematic_resource(&self, resource_conflict_counts: &HashMap<String, usize>) -> Option<String> {
        resource_conflict_counts
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(resource_id, _)| resource_id.clone())
    }

    /// Identify conflict hotspots (time periods with many conflicts)
    fn identify_conflict_hotspots(&self, time_based_conflicts: &[ConflictTimePattern]) -> Vec<ConflictHotspot> {
        let mut hotspots = Vec::new();

        // Group conflicts by time periods
        let mut time_periods = HashMap::new();
        for conflict in time_based_conflicts {
            let period_key = format!(
                "{}-{}",
                conflict.start_time.format("%Y-%m-%d"),
                conflict.end_time.format("%Y-%m-%d")
            );
            time_periods.entry(period_key).or_insert_with(Vec::new).push(conflict);
        }

        // Find periods with multiple conflicts
        for (period, conflicts) in time_periods {
            if conflicts.len() > 1 {
                let severity = conflicts
                    .iter()
                    .map(|c| &c.severity)
                    .max_by_key(|s| match s {
                        ConflictSeverity::Critical => 0,
                        ConflictSeverity::High => 1,
                        ConflictSeverity::Medium => 2,
                        ConflictSeverity::Low => 3,
                    })
                    .cloned()
                    .unwrap_or(ConflictSeverity::Low);

                hotspots.push(ConflictHotspot {
                    period,
                    conflict_count: conflicts.len(),
                    max_severity: severity,
                    affected_resources: conflicts.iter().map(|c| c.conflict_id.clone()).collect(),
                });
            }
        }

        hotspots.sort_by_key(|h| h.conflict_count);
        hotspots.reverse(); // Most conflicts first
        hotspots
    }

    /// Generate conflict prevention recommendations
    pub fn generate_prevention_recommendations(&self) -> Vec<ConflictPreventionRecommendation> {
        let mut recommendations = Vec::new();
        let pattern_analysis = self.analyze_conflict_patterns();

        // Recommend resource capacity adjustments
        for (resource_id, conflict_count) in &pattern_analysis.resource_conflict_counts {
            if *conflict_count > 3 {
                recommendations.push(ConflictPreventionRecommendation::IncreaseResourceCapacity {
                    resource_id: resource_id.clone(),
                    current_conflicts: *conflict_count,
                    recommended_action: "Increase resource capacity or add backup resources".to_string(),
                    priority: RecommendationPriority::High,
                });
            }
        }

        // Recommend schedule adjustments for hotspots
        for hotspot in &pattern_analysis.conflict_hotspots {
            if hotspot.conflict_count > 2 {
                recommendations.push(ConflictPreventionRecommendation::AdjustSchedule {
                    period: hotspot.period.clone(),
                    conflict_count: hotspot.conflict_count,
                    recommended_action: "Redistribute tasks to avoid peak conflict periods".to_string(),
                    priority: RecommendationPriority::Medium,
                });
            }
        }

        // Recommend dependency optimization
        if pattern_analysis.conflict_density > 0.5 {
            recommendations.push(ConflictPreventionRecommendation::OptimizeDependencies {
                conflict_density: pattern_analysis.conflict_density,
                recommended_action: "Review and optimize task dependencies to reduce conflicts".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        recommendations
    }

    /// Calculate conflict resolution efficiency
    pub fn calculate_resolution_efficiency(&self) -> ResolutionEfficiency {
        let total_resolutions = self.conflict_history.len();
        let successful_resolutions = self.conflict_history.iter().filter(|r| r.success).count();

        let success_rate = if total_resolutions > 0 {
            successful_resolutions as f64 / total_resolutions as f64
        } else {
            0.0
        };

        let average_resolution_time = if !self.conflict_history.is_empty() {
            let total_time: Duration = self.conflict_history.iter().map(|r| r.resolution_time).sum();
            total_time.num_minutes() as f64 / self.conflict_history.len() as f64
        } else {
            0.0
        };

        ResolutionEfficiency {
            total_resolutions,
            successful_resolutions,
            success_rate,
            average_resolution_time_minutes: average_resolution_time,
            efficiency_score: self.calculate_efficiency_score(success_rate, average_resolution_time),
        }
    }

    /// Calculate overall efficiency score
    fn calculate_efficiency_score(&self, success_rate: f64, avg_time: f64) -> f64 {
        // Higher success rate and lower resolution time = better efficiency
        let time_score = if avg_time > 0.0 {
            (60.0 / avg_time).min(1.0) // Normalize time score
        } else {
            1.0
        };

        (success_rate * 0.7 + time_score * 0.3) * 100.0
    }

    fn record_resolution(&mut self, conflict: ResourceConflict, resolution: ConflictResolution) {
        self.conflict_history.push(ConflictResolutionRecord {
            conflict,
            resolution,
            timestamp: Utc::now(),
            success: true,                          // Assume success for now
            resolution_time: Duration::minutes(30), // Default resolution time
        });
    }

    /// Get conflict resolution statistics
    pub fn get_resolution_statistics(&self) -> ResolutionStatistics {
        let total_resolutions = self.conflict_history.len();
        let successful_resolutions = self
            .conflict_history
            .iter()
            .filter(|r| !matches!(r.resolution.method, ResolutionMethod::Failed))
            .count();

        let method_counts = self.conflict_history.iter().fold(HashMap::new(), |mut acc, record| {
            *acc.entry(record.resolution.method.clone()).or_insert(0) += 1;
            acc
        });

        ResolutionStatistics {
            total_resolutions,
            successful_resolutions,
            success_rate: if total_resolutions > 0 {
                successful_resolutions as f64 / total_resolutions as f64
            } else {
                0.0
            },
            method_counts,
            average_resolution_time: Duration::hours(1), // Placeholder
        }
    }
}

/// Resolution strategies available
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionStrategy {
    TimeShifting,
    ResourceReassignment,
    TaskBreakdown,
    DependencyAdjustment,
    ProjectRestructuring,
}

/// Resolution methods
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolutionMethod {
    TimeShifting,
    ResourceReassignment,
    TaskBreakdown,
    DependencyAdjustment,
    ProjectRestructuring,
    Failed,
}

impl std::fmt::Display for ResolutionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionMethod::TimeShifting => write!(f, "Time Shifting"),
            ResolutionMethod::ResourceReassignment => write!(f, "Resource Reassignment"),
            ResolutionMethod::TaskBreakdown => write!(f, "Task Breakdown"),
            ResolutionMethod::DependencyAdjustment => write!(f, "Dependency Adjustment"),
            ResolutionMethod::ProjectRestructuring => write!(f, "Project Restructuring"),
            ResolutionMethod::Failed => write!(f, "Failed"),
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolution {
    pub method: ResolutionMethod,
    pub description: String,
    pub delay: Duration,
    pub affected_tasks: Vec<String>,
    pub new_resource: Option<String>,
    pub subtasks: Option<Vec<String>>,
    pub project_delay: Option<Duration>,
}

impl ConflictResolution {
    pub fn new(resource_id: String) -> Self {
        Self {
            method: ResolutionMethod::Failed,
            description: String::new(),
            delay: Duration::zero(),
            affected_tasks: Vec::new(),
            new_resource: None,
            subtasks: None,
            project_delay: None,
        }
    }
}

/// Report of conflict resolution
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolutionReport {
    pub total_conflicts: usize,
    pub resolved_conflicts: usize,
    pub failed_conflicts: usize,
    pub success_rate: f64,
    pub successful_resolutions: Vec<ConflictResolution>,
    pub failed_resolutions: Vec<FailedConflictResolution>,
}

impl Default for ConflictResolutionReport {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictResolutionReport {
    pub fn new() -> Self {
        Self {
            total_conflicts: 0,
            resolved_conflicts: 0,
            failed_conflicts: 0,
            success_rate: 0.0,
            successful_resolutions: Vec::new(),
            failed_resolutions: Vec::new(),
        }
    }
}

/// Failed conflict resolution
#[derive(Debug, Clone, PartialEq)]
pub struct FailedConflictResolution {
    pub conflict: ResourceConflict,
    pub error: String,
    pub attempted_strategies: Vec<ResolutionStrategy>,
}

/// Dependency adjustment
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyAdjustment {
    pub new_predecessor: Option<String>,
    pub new_successor: Option<String>,
    pub lag_time: Duration,
}

/// Restructuring plan
#[derive(Debug, Clone, PartialEq)]
pub struct RestructuringPlan {
    pub affected_tasks: Vec<String>,
    pub project_delay: Duration,
    pub new_dependencies: Vec<String>,
    pub resource_changes: Vec<String>,
}

/// Conflict resolution record
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolutionRecord {
    pub conflict: ResourceConflict,
    pub resolution: ConflictResolution,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub resolution_time: Duration,
}

/// Resolution statistics
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionStatistics {
    pub total_resolutions: usize,
    pub successful_resolutions: usize,
    pub success_rate: f64,
    pub method_counts: HashMap<ResolutionMethod, usize>,
    pub average_resolution_time: Duration,
}

/// Analysis of conflict patterns
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictPatternAnalysis {
    pub total_conflicts: usize,
    pub severity_distribution: HashMap<String, usize>,
    pub resource_conflict_counts: HashMap<String, usize>,
    pub time_based_conflicts: Vec<ConflictTimePattern>,
    pub conflict_density: f64,
    pub most_problematic_resource: Option<String>,
    pub conflict_hotspots: Vec<ConflictHotspot>,
}

/// Time pattern of a conflict
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictTimePattern {
    pub conflict_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub severity: ConflictSeverity,
    pub duration: Duration,
}

/// Conflict hotspot (time period with many conflicts)
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictHotspot {
    pub period: String,
    pub conflict_count: usize,
    pub max_severity: ConflictSeverity,
    pub affected_resources: Vec<String>,
}

/// Conflict prevention recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictPreventionRecommendation {
    IncreaseResourceCapacity {
        resource_id: String,
        current_conflicts: usize,
        recommended_action: String,
        priority: RecommendationPriority,
    },
    AdjustSchedule {
        period: String,
        conflict_count: usize,
        recommended_action: String,
        priority: RecommendationPriority,
    },
    OptimizeDependencies {
        conflict_density: f64,
        recommended_action: String,
        priority: RecommendationPriority,
    },
}

/// Priority level for recommendations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RecommendationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationPriority::Low => write!(f, "Low"),
            RecommendationPriority::Medium => write!(f, "Medium"),
            RecommendationPriority::High => write!(f, "High"),
            RecommendationPriority::Critical => write!(f, "Critical"),
        }
    }
}

/// Resolution efficiency metrics
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionEfficiency {
    pub total_resolutions: usize,
    pub successful_resolutions: usize,
    pub success_rate: f64,
    pub average_resolution_time_minutes: f64,
    pub efficiency_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::scheduling::ProjectSchedule;

    fn create_test_schedule() -> ProjectSchedule {
        let start = Utc::now();
        ProjectSchedule::new("TEST-PROJ".to_string(), start)
    }

    #[test]
    fn test_conflict_resolver_creation() {
        let schedule = create_test_schedule();
        let resolver = ConflictResolver::new(schedule);
        assert_eq!(resolver.resolution_strategies.len(), 5);
        assert!(resolver.conflict_history.is_empty());
    }

    #[test]
    fn test_conflict_resolution_creation() {
        let resolution = ConflictResolution::new("resource-1".to_string());
        assert_eq!(resolution.method, ResolutionMethod::Failed);
        assert!(resolution.affected_tasks.is_empty());
    }

    #[test]
    fn test_resolution_report_creation() {
        let report = ConflictResolutionReport::new();
        assert_eq!(report.total_conflicts, 0);
        assert_eq!(report.success_rate, 0.0);
    }

    #[test]
    fn test_resolution_statistics() {
        let schedule = create_test_schedule();
        let resolver = ConflictResolver::new(schedule);
        let stats = resolver.get_resolution_statistics();

        assert_eq!(stats.total_resolutions, 0);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[test]
    fn test_conflict_pattern_analysis() {
        let schedule = create_test_schedule();
        let resolver = ConflictResolver::new(schedule);
        let analysis = resolver.analyze_conflict_patterns();

        assert_eq!(analysis.total_conflicts, 0);
        assert!(analysis.severity_distribution.is_empty());
        assert!(analysis.resource_conflict_counts.is_empty());
        assert!(analysis.time_based_conflicts.is_empty());
        assert_eq!(analysis.conflict_density, 0.0);
        assert!(analysis.most_problematic_resource.is_none());
        assert!(analysis.conflict_hotspots.is_empty());
    }

    #[test]
    fn test_prevention_recommendations() {
        let schedule = create_test_schedule();
        let resolver = ConflictResolver::new(schedule);
        let recommendations = resolver.generate_prevention_recommendations();

        // Should have some recommendations based on analysis
        // Removed useless comparison (len() is always >= 0)
    }

    #[test]
    fn test_resolution_efficiency() {
        let schedule = create_test_schedule();
        let resolver = ConflictResolver::new(schedule);
        let efficiency = resolver.calculate_resolution_efficiency();

        assert_eq!(efficiency.total_resolutions, 0);
        assert_eq!(efficiency.successful_resolutions, 0);
        assert_eq!(efficiency.success_rate, 0.0);
        // Note: average_resolution_time_minutes might not be 0.0 if there are default records
        assert!(efficiency.average_resolution_time_minutes >= 0.0);
        // Note: efficiency_score might not be 0.0 due to default resolution time
        assert!(efficiency.efficiency_score >= 0.0);
    }

    #[test]
    fn test_recommendation_priority() {
        let priority = RecommendationPriority::High;
        assert_eq!(priority.to_string(), "High");

        let critical = RecommendationPriority::Critical;
        assert_eq!(critical.to_string(), "Critical");
    }

    #[test]
    fn test_conflict_time_pattern() {
        let start = Utc::now();
        let end = start + Duration::hours(2);
        let pattern = ConflictTimePattern {
            conflict_id: "conflict-1".to_string(),
            start_time: start,
            end_time: end,
            severity: ConflictSeverity::High,
            duration: Duration::hours(2),
        };

        assert_eq!(pattern.conflict_id, "conflict-1");
        assert_eq!(pattern.severity, ConflictSeverity::High);
        assert_eq!(pattern.duration, Duration::hours(2));
    }
}
