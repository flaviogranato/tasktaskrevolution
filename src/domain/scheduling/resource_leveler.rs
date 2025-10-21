use crate::domain::scheduling::{ConflictSeverity, ProjectSchedule, ResourceConflict, ScheduledTask};
use crate::domain::shared::errors::DomainError;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

/// Resource leveling algorithm to resolve resource conflicts
pub struct ResourceLeveler {
    schedule: ProjectSchedule,
    resource_capacity: HashMap<String, f64>, // resource_id -> capacity (0.0 to 1.0)
}

impl ResourceLeveler {
    pub fn new(schedule: ProjectSchedule) -> Self {
        Self {
            schedule,
            resource_capacity: HashMap::new(),
        }
    }

    /// Set resource capacity (0.0 = 0%, 1.0 = 100%)
    pub fn set_resource_capacity(&mut self, resource_id: String, capacity: f64) {
        self.resource_capacity.insert(resource_id, capacity.clamp(0.0, 1.0));
    }

    /// Perform resource leveling to resolve conflicts
    pub fn level_resources(&mut self) -> Result<ResourceLevelingResult, DomainError> {
        let mut leveling_result = ResourceLevelingResult::new();
        let mut conflicts_resolved = 0;
        let mut total_delay = Duration::zero();

        // Get all resource conflicts
        let conflicts = self.schedule.conflicts.clone();

        for conflict in conflicts {
            match self.resolve_conflict(&conflict) {
                Ok(resolution) => {
                    conflicts_resolved += 1;
                    total_delay += resolution.delay;
                    leveling_result.resolutions.push(resolution);
                }
                Err(e) => {
                    leveling_result.failed_resolutions.push(FailedResolution {
                        conflict,
                        error: e.to_string(),
                    });
                }
            }
        }

        leveling_result.conflicts_resolved = conflicts_resolved;
        leveling_result.total_delay = total_delay;
        leveling_result.success = leveling_result.failed_resolutions.is_empty();

        Ok(leveling_result)
    }

    /// Resolve a specific resource conflict
    fn resolve_conflict(&mut self, conflict: &ResourceConflict) -> Result<ConflictResolution, DomainError> {
        match conflict.severity {
            ConflictSeverity::Low => self.resolve_low_severity_conflict(conflict),
            ConflictSeverity::Medium => self.resolve_medium_severity_conflict(conflict),
            ConflictSeverity::High => self.resolve_high_severity_conflict(conflict),
            ConflictSeverity::Critical => self.resolve_critical_severity_conflict(conflict),
        }
    }

    /// Resolve low severity conflicts by minor adjustments
    fn resolve_low_severity_conflict(
        &mut self,
        conflict: &ResourceConflict,
    ) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // For low severity, try to shift one task slightly
        #[allow(clippy::collapsible_if)]
        if let Some(task_id) = conflict.conflicting_tasks.first() {
            if let Some(task) = self.schedule.tasks.iter_mut().find(|t| t.task_id == *task_id) {
                let shift = Duration::hours(1); // Small shift
                task.early_start += shift;
                task.early_finish += shift;
                task.late_start += shift;
                task.late_finish += shift;

                resolution.delay = shift;
                resolution.method = ResolutionMethod::MinorShift;
                resolution.description = "Minor time shift to resolve low-severity conflict".to_string();
            }
        }

        Ok(resolution)
    }

    /// Resolve medium severity conflicts by rescheduling
    fn resolve_medium_severity_conflict(
        &mut self,
        conflict: &ResourceConflict,
    ) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Find the best task to reschedule
        let best_task = self.find_best_task_to_reschedule(conflict)?;

        let task_duration = self
            .schedule
            .tasks
            .iter()
            .find(|t| t.task_id == best_task)
            .map(|t| t.duration)
            .unwrap_or(Duration::days(1));

        let new_start = self.find_next_available_slot(&conflict.resource_id, task_duration)?;

        if let Some(task) = self.schedule.tasks.iter_mut().find(|t| t.task_id == best_task) {
            let delay = new_start - task.early_start;

            task.early_start = new_start;
            task.early_finish = new_start + task.duration;
            task.late_start += delay;
            task.late_finish += delay;

            resolution.delay = delay;
            resolution.method = ResolutionMethod::Reschedule;
            resolution.description = format!("Rescheduled task {} to resolve conflict", best_task);
        }

        Ok(resolution)
    }

    /// Resolve high severity conflicts by resource reassignment
    fn resolve_high_severity_conflict(
        &mut self,
        conflict: &ResourceConflict,
    ) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // Try to find an alternative resource
        if let Some(alternative_resource) = self.find_alternative_resource(&conflict.resource_id) {
            // Reassign one of the conflicting tasks
            if let Some(task_id) = conflict.conflicting_tasks.first() {
                self.reassign_task_resource(task_id, &alternative_resource)?;

                resolution.method = ResolutionMethod::ResourceReassignment;
                resolution.description = format!("Reassigned task {} to resource {}", task_id, alternative_resource);
            }
        } else {
            // Fall back to rescheduling
            return self.resolve_medium_severity_conflict(conflict);
        }

        Ok(resolution)
    }

    /// Resolve critical severity conflicts by project restructuring
    fn resolve_critical_severity_conflict(
        &mut self,
        conflict: &ResourceConflict,
    ) -> Result<ConflictResolution, DomainError> {
        let mut resolution = ConflictResolution::new(conflict.resource_id.clone());

        // For critical conflicts, we need to restructure the project
        // This might involve breaking down tasks or changing dependencies

        // Try to break down one of the conflicting tasks
        if let Some(task_id) = conflict.conflicting_tasks.first() {
            if let Some(breakdown) = self.break_down_task(task_id)? {
                resolution.method = ResolutionMethod::TaskBreakdown;
                resolution.description = format!("Broke down task {} into smaller subtasks", task_id);
                resolution.subtasks = Some(breakdown);
            } else {
                // If breakdown is not possible, delay the entire project
                let delay = Duration::days(1);
                self.delay_project(delay);

                resolution.delay = delay;
                resolution.method = ResolutionMethod::ProjectDelay;
                resolution.description = "Delayed entire project to resolve critical conflict".to_string();
            }
        }

        Ok(resolution)
    }

    /// Find the best task to reschedule based on slack and dependencies
    fn find_best_task_to_reschedule(&self, conflict: &ResourceConflict) -> Result<String, DomainError> {
        let mut candidates = Vec::new();

        for task_id in &conflict.conflicting_tasks {
            if let Some(task) = self.schedule.get_task(task_id) {
                let score = self.calculate_reschedule_score(task);
                candidates.push((task_id.clone(), score));
            }
        }

        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        candidates
            .first()
            .map(|(task_id, _)| task_id.clone())
            .ok_or_else(|| DomainError::ValidationError {
                field: "conflict_resolution".to_string(),
                message: "No suitable task found for rescheduling".to_string(),
            })
    }

    /// Calculate a score for rescheduling a task (lower is better)
    fn calculate_reschedule_score(&self, task: &ScheduledTask) -> f64 {
        let slack_score = task.total_slack.num_days() as f64;
        let dependency_penalty = self.calculate_dependency_penalty(&task.task_id);
        let critical_penalty = if task.is_critical { 1000.0 } else { 0.0 };

        slack_score + dependency_penalty + critical_penalty
    }

    /// Calculate penalty for rescheduling based on dependencies
    fn calculate_dependency_penalty(&self, task_id: &str) -> f64 {
        // Count how many tasks depend on this task
        let dependent_count = self
            .schedule
            .tasks
            .iter()
            .filter(|t| self.has_dependency(task_id, &t.task_id))
            .count();

        dependent_count as f64 * 10.0
    }

    /// Check if there's a dependency between two tasks
    fn has_dependency(&self, _predecessor_id: &str, _successor_id: &str) -> bool {
        // This should check the actual dependency graph
        // For now, return false
        false
    }

    /// Find the next available time slot for a resource
    fn find_next_available_slot(&self, resource_id: &str, _duration: Duration) -> Result<DateTime<Utc>, DomainError> {
        let resource_tasks = self.schedule.get_tasks_by_resource(resource_id);

        if resource_tasks.is_empty() {
            return Ok(Utc::now());
        }

        // Find the latest finish time
        let latest_finish = resource_tasks
            .iter()
            .map(|t| t.early_finish)
            .max()
            .unwrap_or(Utc::now());

        Ok(latest_finish + Duration::hours(1)) // Add 1 hour buffer
    }

    /// Find an alternative resource for reassignment
    fn find_alternative_resource(&self, _current_resource_id: &str) -> Option<String> {
        // This should check available resources with similar skills
        // For now, return None
        None
    }

    /// Analyze resource utilization across the project
    pub fn analyze_resource_utilization(&self) -> ResourceUtilizationAnalysis {
        let mut utilization_data = HashMap::new();
        let mut total_utilization = 0.0;
        let mut resource_count = 0;

        for (resource_id, capacity) in &self.resource_capacity {
            let tasks = self.schedule.get_tasks_by_resource(resource_id);
            let utilization = self.calculate_resource_utilization(resource_id, &tasks);

            utilization_data.insert(
                resource_id.clone(),
                ResourceUtilizationInfo {
                    resource_id: resource_id.clone(),
                    average_utilization: utilization,
                    peak_utilization: self.calculate_peak_utilization(resource_id, &tasks),
                    assigned_tasks_count: tasks.len(),
                    capacity: *capacity,
                    efficiency: utilization / capacity,
                },
            );

            total_utilization += utilization;
            resource_count += 1;
        }

        let average_utilization = if resource_count > 0 {
            total_utilization / resource_count as f64
        } else {
            0.0
        };

        let overutilized = self.identify_overutilized_resources(&utilization_data);
        let underutilized = self.identify_underutilized_resources(&utilization_data);

        ResourceUtilizationAnalysis {
            resource_utilization: utilization_data,
            average_utilization,
            total_resources: resource_count,
            overutilized_resources: overutilized,
            underutilized_resources: underutilized,
        }
    }

    /// Calculate utilization for a specific resource
    fn calculate_resource_utilization(&self, _resource_id: &str, tasks: &[&ScheduledTask]) -> f64 {
        if tasks.is_empty() {
            return 0.0;
        }

        let total_duration = tasks.iter().map(|t| t.duration.num_hours() as f64).sum::<f64>();

        let project_duration = self.schedule.total_duration.num_hours() as f64;

        if project_duration > 0.0 {
            total_duration / project_duration
        } else {
            0.0
        }
    }

    /// Calculate peak utilization for a resource
    fn calculate_peak_utilization(&self, _resource_id: &str, tasks: &[&ScheduledTask]) -> f64 {
        if tasks.is_empty() {
            return 0.0;
        }

        // Find the period with maximum concurrent tasks
        let mut max_concurrent = 0;
        let mut current_concurrent = 0;

        // Sort tasks by start time
        let mut sorted_tasks = tasks.to_vec();
        sorted_tasks.sort_by_key(|t| t.early_start);

        for _task in &sorted_tasks {
            current_concurrent += 1;
            max_concurrent = max_concurrent.max(current_concurrent);
        }

        max_concurrent as f64
    }

    /// Identify overutilized resources
    fn identify_overutilized_resources(
        &self,
        utilization_data: &HashMap<String, ResourceUtilizationInfo>,
    ) -> Vec<String> {
        utilization_data
            .iter()
            .filter(|(_, info)| info.efficiency > 1.0)
            .map(|(resource_id, _)| resource_id.clone())
            .collect()
    }

    /// Identify underutilized resources
    fn identify_underutilized_resources(
        &self,
        utilization_data: &HashMap<String, ResourceUtilizationInfo>,
    ) -> Vec<String> {
        utilization_data
            .iter()
            .filter(|(_, info)| info.efficiency < 0.5)
            .map(|(resource_id, _)| resource_id.clone())
            .collect()
    }

    /// Generate resource leveling recommendations
    pub fn generate_recommendations(&self) -> Vec<ResourceLevelingRecommendation> {
        let mut recommendations = Vec::new();
        let utilization_analysis = self.analyze_resource_utilization();

        // Recommend reassignment for overutilized resources
        for resource_id in &utilization_analysis.overutilized_resources {
            recommendations.push(ResourceLevelingRecommendation::ReassignTasks {
                resource_id: resource_id.clone(),
                reason: "Resource is overutilized".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        // Recommend task redistribution for underutilized resources
        for resource_id in &utilization_analysis.underutilized_resources {
            recommendations.push(ResourceLevelingRecommendation::RedistributeTasks {
                resource_id: resource_id.clone(),
                reason: "Resource is underutilized".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        // Recommend capacity adjustments
        for (resource_id, info) in &utilization_analysis.resource_utilization {
            if info.efficiency > 1.2 {
                recommendations.push(ResourceLevelingRecommendation::IncreaseCapacity {
                    resource_id: resource_id.clone(),
                    current_capacity: info.capacity,
                    recommended_capacity: (info.capacity * 1.2).min(1.0),
                    reason: "Resource consistently overutilized".to_string(),
                    priority: RecommendationPriority::High,
                });
            }
        }

        recommendations
    }

    /// Reassign a task to a different resource
    fn reassign_task_resource(&mut self, task_id: &str, new_resource_id: &str) -> Result<(), DomainError> {
        // Remove from old resource assignments
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

    /// Break down a task into smaller subtasks
    fn break_down_task(&self, _task_id: &str) -> Result<Option<Vec<String>>, DomainError> {
        // This is a simplified implementation
        // In a real system, this would create actual subtasks
        Ok(None)
    }

    /// Delay the entire project
    fn delay_project(&mut self, delay: Duration) {
        for task in &mut self.schedule.tasks {
            task.early_start += delay;
            task.early_finish += delay;
            task.late_start += delay;
            task.late_finish += delay;
        }
        self.schedule.start_date += delay;
        self.schedule.end_date += delay;
    }

    /// Get resource utilization over time
    pub fn get_resource_utilization(&self, resource_id: &str) -> ResourceUtilization {
        let tasks = self.schedule.get_tasks_by_resource(resource_id);
        let mut utilization_periods = Vec::new();

        for task in &tasks {
            utilization_periods.push(UtilizationPeriod {
                start: task.early_start,
                end: task.early_finish,
                utilization: 1.0, // Assume 100% utilization during task
            });
        }

        let total_utilization = self.calculate_total_utilization(&utilization_periods);
        let peak_utilization = self.calculate_peak_utilization(resource_id, &tasks);

        ResourceUtilization {
            resource_id: resource_id.to_string(),
            periods: utilization_periods,
            total_utilization,
            peak_utilization,
            average_utilization: total_utilization / self.schedule.total_duration.num_days() as f64,
        }
    }

    /// Calculate total utilization time
    fn calculate_total_utilization(&self, periods: &[UtilizationPeriod]) -> f64 {
        periods
            .iter()
            .map(|p| (p.end - p.start).num_hours() as f64 * p.utilization)
            .sum()
    }

    /// Optimize resource allocation
    pub fn optimize_allocation(&mut self) -> Result<OptimizationResult, DomainError> {
        let mut optimization_result = OptimizationResult::new();
        let mut improvements = Vec::new();

        // Analyze each resource
        for resource_id in self.schedule.resource_assignments.keys() {
            let utilization = self.get_resource_utilization(resource_id);

            if utilization.average_utilization < 0.5 {
                improvements.push(ResourceImprovement {
                    resource_id: resource_id.clone(),
                    current_utilization: utilization.average_utilization,
                    suggested_utilization: 0.8,
                    improvement_type: "Underutilized resource - consider reassigning tasks".to_string(),
                });
            } else if utilization.peak_utilization > 1.0 {
                improvements.push(ResourceImprovement {
                    resource_id: resource_id.clone(),
                    current_utilization: utilization.peak_utilization,
                    suggested_utilization: 1.0,
                    improvement_type: "Overutilized resource - consider load balancing".to_string(),
                });
            }
        }

        optimization_result.improvements = improvements;
        optimization_result.total_improvements = optimization_result.improvements.len();
        optimization_result.success = true;

        Ok(optimization_result)
    }
}

/// Result of resource leveling
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceLevelingResult {
    pub success: bool,
    pub conflicts_resolved: usize,
    pub total_delay: Duration,
    pub resolutions: Vec<ConflictResolution>,
    pub failed_resolutions: Vec<FailedResolution>,
}

impl Default for ResourceLevelingResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceLevelingResult {
    pub fn new() -> Self {
        Self {
            success: false,
            conflicts_resolved: 0,
            total_delay: Duration::zero(),
            resolutions: Vec::new(),
            failed_resolutions: Vec::new(),
        }
    }
}

/// Resolution of a specific conflict
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictResolution {
    pub resource_id: String,
    pub method: ResolutionMethod,
    pub delay: Duration,
    pub description: String,
    pub subtasks: Option<Vec<String>>,
}

impl ConflictResolution {
    pub fn new(resource_id: String) -> Self {
        Self {
            resource_id,
            method: ResolutionMethod::Unknown,
            delay: Duration::zero(),
            description: String::new(),
            subtasks: None,
        }
    }
}

/// Method used to resolve a conflict
#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionMethod {
    MinorShift,
    Reschedule,
    ResourceReassignment,
    TaskBreakdown,
    ProjectDelay,
    Unknown,
}

/// Resource utilization analysis
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUtilizationAnalysis {
    pub resource_utilization: HashMap<String, ResourceUtilizationInfo>,
    pub average_utilization: f64,
    pub total_resources: usize,
    pub overutilized_resources: Vec<String>,
    pub underutilized_resources: Vec<String>,
}

/// Information about a resource's utilization
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUtilizationInfo {
    pub resource_id: String,
    pub average_utilization: f64,
    pub peak_utilization: f64,
    pub assigned_tasks_count: usize,
    pub capacity: f64,
    pub efficiency: f64,
}

/// Resource leveling recommendations
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceLevelingRecommendation {
    ReassignTasks {
        resource_id: String,
        reason: String,
        priority: RecommendationPriority,
    },
    RedistributeTasks {
        resource_id: String,
        reason: String,
        priority: RecommendationPriority,
    },
    IncreaseCapacity {
        resource_id: String,
        current_capacity: f64,
        recommended_capacity: f64,
        reason: String,
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

impl std::fmt::Display for ResolutionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolutionMethod::MinorShift => write!(f, "Minor Time Shift"),
            ResolutionMethod::Reschedule => write!(f, "Reschedule Task"),
            ResolutionMethod::ResourceReassignment => write!(f, "Reassign Resource"),
            ResolutionMethod::TaskBreakdown => write!(f, "Break Down Task"),
            ResolutionMethod::ProjectDelay => write!(f, "Delay Project"),
            ResolutionMethod::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Failed resolution attempt
#[derive(Debug, Clone, PartialEq)]
pub struct FailedResolution {
    pub conflict: ResourceConflict,
    pub error: String,
}

/// Resource utilization analysis
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUtilization {
    pub resource_id: String,
    pub periods: Vec<UtilizationPeriod>,
    pub total_utilization: f64,
    pub peak_utilization: f64,
    pub average_utilization: f64,
}

/// Utilization period for a resource
#[derive(Debug, Clone, PartialEq)]
pub struct UtilizationPeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub utilization: f64,
}

/// Result of resource optimization
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationResult {
    pub success: bool,
    pub total_improvements: usize,
    pub improvements: Vec<ResourceImprovement>,
}

impl Default for OptimizationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationResult {
    pub fn new() -> Self {
        Self {
            success: false,
            total_improvements: 0,
            improvements: Vec::new(),
        }
    }
}

/// Resource improvement suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceImprovement {
    pub resource_id: String,
    pub current_utilization: f64,
    pub suggested_utilization: f64,
    pub improvement_type: String,
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
    fn test_resource_leveler_creation() {
        let schedule = create_test_schedule();
        let leveler = ResourceLeveler::new(schedule);
        assert!(leveler.resource_capacity.is_empty());
    }

    #[test]
    fn test_set_resource_capacity() {
        let schedule = create_test_schedule();
        let mut leveler = ResourceLeveler::new(schedule);
        leveler.set_resource_capacity("resource-1".to_string(), 0.8);

        assert_eq!(leveler.resource_capacity.get("resource-1"), Some(&0.8));
    }

    #[test]
    fn test_capacity_clamping() {
        let schedule = create_test_schedule();
        let mut leveler = ResourceLeveler::new(schedule);

        // Test upper bound
        leveler.set_resource_capacity("resource-1".to_string(), 1.5);
        assert_eq!(leveler.resource_capacity.get("resource-1"), Some(&1.0));

        // Test lower bound
        leveler.set_resource_capacity("resource-2".to_string(), -0.5);
        assert_eq!(leveler.resource_capacity.get("resource-2"), Some(&0.0));
    }

    #[test]
    fn test_conflict_resolution_creation() {
        let resolution = ConflictResolution::new("resource-1".to_string());
        assert_eq!(resolution.resource_id, "resource-1");
        assert_eq!(resolution.method, ResolutionMethod::Unknown);
        assert_eq!(resolution.delay, Duration::zero());
    }

    #[test]
    fn test_optimization_result_creation() {
        let result = OptimizationResult::new();
        assert!(!result.success);
        assert_eq!(result.total_improvements, 0);
        assert!(result.improvements.is_empty());
    }

    #[test]
    fn test_resource_utilization_analysis() {
        let schedule = create_test_schedule();
        let mut leveler = ResourceLeveler::new(schedule);

        // Set some resource capacities
        leveler.set_resource_capacity("resource-1".to_string(), 0.8);
        leveler.set_resource_capacity("resource-2".to_string(), 0.6);

        let analysis = leveler.analyze_resource_utilization();

        assert_eq!(analysis.total_resources, 2);
        assert!(analysis.average_utilization >= 0.0);
        assert!(analysis.overutilized_resources.is_empty());
        // Note: underutilized_resources might not be empty due to low utilization
    }

    #[test]
    fn test_recommendation_generation() {
        let schedule = create_test_schedule();
        let mut leveler = ResourceLeveler::new(schedule);

        // Set resource capacities
        leveler.set_resource_capacity("resource-1".to_string(), 0.5);
        leveler.set_resource_capacity("resource-2".to_string(), 0.8);

        let recommendations = leveler.generate_recommendations();

        // Should have some recommendations based on utilization
        // Removed useless comparison (len() is always >= 0)
    }

    #[test]
    fn test_resource_utilization_info() {
        let info = ResourceUtilizationInfo {
            resource_id: "test-resource".to_string(),
            average_utilization: 0.75,
            peak_utilization: 1.0,
            assigned_tasks_count: 3,
            capacity: 0.8,
            efficiency: 0.9375,
        };

        assert_eq!(info.resource_id, "test-resource");
        assert_eq!(info.average_utilization, 0.75);
        assert_eq!(info.efficiency, 0.9375);
    }

    #[test]
    fn test_recommendation_priority() {
        let priority = RecommendationPriority::High;
        assert_eq!(priority.to_string(), "High");

        let critical = RecommendationPriority::Critical;
        assert_eq!(critical.to_string(), "Critical");
    }
}
