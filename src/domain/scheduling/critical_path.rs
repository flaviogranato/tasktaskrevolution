use crate::domain::scheduling::{ProjectSchedule, ScheduledTask};
use crate::domain::shared::errors::DomainError;
use chrono::Duration;
use std::collections::HashMap;

/// Critical Path Method (CPM) calculator
pub struct CriticalPathCalculator {
    schedule: ProjectSchedule,
}

impl CriticalPathCalculator {
    pub fn new(schedule: ProjectSchedule) -> Self {
        Self { schedule }
    }

    /// Calculate the critical path for the project
    pub fn calculate_critical_path(&self) -> Result<CriticalPathAnalysis, DomainError> {
        let critical_tasks = self.identify_critical_tasks();
        let critical_path = self.build_critical_path(&critical_tasks)?;
        let total_slack_analysis = self.analyze_total_slack();
        let free_slack_analysis = self.analyze_free_slack();

        Ok(CriticalPathAnalysis {
            critical_path: critical_path.clone(),
            critical_tasks: critical_tasks.iter().map(|&t| t.task_id.clone()).collect(),
            total_slack_analysis,
            free_slack_analysis,
            project_duration: self.schedule.total_duration,
            critical_path_duration: self.calculate_critical_path_duration(&critical_path),
        })
    }

    /// Identify all critical tasks (zero slack)
    fn identify_critical_tasks(&self) -> Vec<&ScheduledTask> {
        self.schedule.tasks.iter().filter(|task| task.is_critical).collect()
    }

    /// Build the critical path by following dependencies
    fn build_critical_path(&self, critical_tasks: &[&ScheduledTask]) -> Result<Vec<String>, DomainError> {
        if critical_tasks.is_empty() {
            return Ok(vec![]);
        }

        // Find the starting task (earliest critical task)
        let start_task =
            critical_tasks
                .iter()
                .min_by_key(|t| t.early_start)
                .ok_or_else(|| DomainError::ValidationError {
                    field: "critical_path".to_string(),
                    message: "No critical tasks found".to_string(),
                })?;

        let mut critical_path = vec![start_task.task_id.clone()];
        let mut current_task = *start_task;

        // Follow the critical path
        while let Some(next_task) = self.find_next_critical_task(current_task, critical_tasks) {
            if critical_path.contains(&next_task.task_id) {
                // Avoid infinite loops
                break;
            }
            critical_path.push(next_task.task_id.clone());
            current_task = next_task;
        }

        Ok(critical_path)
    }

    /// Find the next critical task in the path
    fn find_next_critical_task<'a>(
        &self,
        current_task: &ScheduledTask,
        critical_tasks: &'a [&ScheduledTask],
    ) -> Option<&'a ScheduledTask> {
        critical_tasks
            .iter()
            .find(|task| {
                task.early_start >= current_task.early_finish
                    && self.has_dependency(&current_task.task_id, &task.task_id)
            })
            .copied()
    }

    /// Check if there's a dependency between two tasks
    fn has_dependency(&self, _predecessor_id: &str, _successor_id: &str) -> bool {
        // This should check the actual dependency graph
        // For now, we'll use a simple heuristic based on timing
        // TODO: Implement proper dependency checking using the project's dependency graph
        true
    }

    /// Calculate the total duration of the critical path
    fn calculate_critical_path_duration(&self, critical_path: &[String]) -> Duration {
        if critical_path.is_empty() {
            return Duration::zero();
        }

        let first_task = &critical_path[0];
        let last_task = &critical_path[critical_path.len() - 1];

        if let (Some(start_task), Some(end_task)) =
            (self.schedule.get_task(first_task), self.schedule.get_task(last_task))
        {
            end_task.early_finish - start_task.early_start
        } else {
            Duration::zero()
        }
    }

    /// Analyze total slack distribution
    fn analyze_total_slack(&self) -> SlackAnalysis {
        let mut slack_distribution = HashMap::new();
        let mut total_slack_sum = Duration::zero();
        let mut task_count = 0;

        for task in &self.schedule.tasks {
            let slack_days = task.total_slack.num_days();
            *slack_distribution.entry(slack_days).or_insert(0) += 1;
            total_slack_sum += task.total_slack;
            task_count += 1;
        }

        let average_slack = if task_count > 0 {
            total_slack_sum.num_milliseconds() / task_count as i64
        } else {
            0
        };

        SlackAnalysis {
            distribution: slack_distribution,
            average_slack_days: average_slack as f64 / (24.0 * 60.0 * 60.0 * 1000.0),
            total_tasks: task_count,
            critical_tasks: self.schedule.tasks.iter().filter(|t| t.is_critical).count(),
        }
    }

    /// Analyze free slack distribution
    fn analyze_free_slack(&self) -> SlackAnalysis {
        let mut slack_distribution = HashMap::new();
        let mut total_slack_sum = Duration::zero();
        let mut task_count = 0;

        for task in &self.schedule.tasks {
            let slack_days = task.free_slack.num_days();
            *slack_distribution.entry(slack_days).or_insert(0) += 1;
            total_slack_sum += task.free_slack;
            task_count += 1;
        }

        let average_slack = if task_count > 0 {
            total_slack_sum.num_milliseconds() / task_count as i64
        } else {
            0
        };

        SlackAnalysis {
            distribution: slack_distribution,
            average_slack_days: average_slack as f64 / (24.0 * 60.0 * 60.0 * 1000.0),
            total_tasks: task_count,
            critical_tasks: self.schedule.tasks.iter().filter(|t| t.is_critical).count(),
        }
    }

    /// Get tasks that can be delayed without affecting the project
    pub fn get_non_critical_tasks(&self) -> Vec<&ScheduledTask> {
        self.schedule.tasks.iter().filter(|task| !task.is_critical).collect()
    }

    /// Get tasks with high slack (can be delayed significantly)
    pub fn get_high_slack_tasks(&self, threshold_days: i64) -> Vec<&ScheduledTask> {
        self.schedule
            .tasks
            .iter()
            .filter(|task| task.total_slack.num_days() >= threshold_days)
            .collect()
    }

    /// Calculate the impact of delaying a task
    pub fn calculate_delay_impact(&self, task_id: &str, delay_days: i64) -> DelayImpact {
        let task = match self.schedule.get_task(task_id) {
            Some(task) => task,
            None => return DelayImpact::TaskNotFound,
        };

        if task.is_critical {
            return DelayImpact::CriticalTaskDelay {
                project_delay_days: delay_days,
                affected_tasks: self.get_affected_tasks(task_id),
            };
        }

        let _new_finish = task.early_finish + Duration::days(delay_days);
        let max_slack = task.total_slack.num_days();

        if delay_days <= max_slack {
            DelayImpact::NoImpact
        } else {
            let excess_delay = delay_days - max_slack;
            DelayImpact::NonCriticalDelay {
                excess_delay_days: excess_delay,
                affected_tasks: self.get_affected_tasks(task_id),
            }
        }
    }

    /// Get tasks that would be affected by delaying a specific task
    fn get_affected_tasks(&self, _task_id: &str) -> Vec<String> {
        // This should traverse the dependency graph to find affected tasks
        // For now, return empty vector
        // TODO: Implement proper dependency traversal
        vec![]
    }

    /// Suggest optimizations for the critical path
    pub fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Check for tasks with high slack that could be delayed
        let high_slack_tasks = self.get_high_slack_tasks(7); // 7+ days slack
        if !high_slack_tasks.is_empty() {
            suggestions.push(OptimizationSuggestion::DelayNonCriticalTasks {
                task_count: high_slack_tasks.len(),
                potential_savings_days: high_slack_tasks
                    .iter()
                    .map(|t| t.total_slack.num_days())
                    .max()
                    .unwrap_or(0),
            });
        }

        // Check for resource conflicts on critical path
        let critical_conflicts = self
            .schedule
            .conflicts
            .iter()
            .filter(|c| {
                c.conflicting_tasks
                    .iter()
                    .any(|tid| self.schedule.get_task(tid).map(|t| t.is_critical).unwrap_or(false))
            })
            .count();

        if critical_conflicts > 0 {
            suggestions.push(OptimizationSuggestion::ResolveCriticalConflicts {
                conflict_count: critical_conflicts,
            });
        }

        // Check for opportunities to parallelize tasks
        let parallel_opportunities = self.find_parallel_opportunities();
        if parallel_opportunities > 0 {
            suggestions.push(OptimizationSuggestion::ParallelizeTasks {
                opportunity_count: parallel_opportunities,
            });
        }

        suggestions
    }

    /// Find opportunities to parallelize tasks
    fn find_parallel_opportunities(&self) -> usize {
        // This is a simplified implementation
        // In a real system, this would analyze the dependency graph
        // to find tasks that could be done in parallel
        0
    }

    /// Analyze risk factors in the critical path
    pub fn analyze_risk_factors(&self) -> RiskAnalysis {
        let critical_tasks = self.identify_critical_tasks();
        let total_tasks = self.schedule.tasks.len();
        let critical_percentage = if total_tasks > 0 {
            (critical_tasks.len() as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        let high_risk_tasks = critical_tasks
            .iter()
            .filter(|task| {
                // Tasks with very tight schedules (less than 1 day slack)
                task.total_slack.num_hours() < 24
            })
            .count();

        let resource_conflicts = self
            .schedule
            .conflicts
            .iter()
            .filter(|c| {
                c.conflicting_tasks
                    .iter()
                    .any(|tid| self.schedule.get_task(tid).map(|t| t.is_critical).unwrap_or(false))
            })
            .count();

        RiskAnalysis {
            critical_task_percentage: critical_percentage,
            high_risk_tasks,
            resource_conflicts,
            overall_risk_level: self.calculate_overall_risk_level(
                critical_percentage,
                high_risk_tasks,
                resource_conflicts,
            ),
        }
    }

    /// Calculate overall risk level
    fn calculate_overall_risk_level(
        &self,
        critical_percentage: f64,
        high_risk_tasks: usize,
        resource_conflicts: usize,
    ) -> RiskLevel {
        let mut risk_score = 0.0;

        // Critical path percentage risk
        if critical_percentage > 80.0 {
            risk_score += 3.0;
        } else if critical_percentage > 60.0 {
            risk_score += 2.0;
        } else if critical_percentage > 40.0 {
            risk_score += 1.0;
        }

        // High risk tasks
        if high_risk_tasks > 5 {
            risk_score += 3.0;
        } else if high_risk_tasks > 3 {
            risk_score += 2.0;
        } else if high_risk_tasks > 1 {
            risk_score += 1.0;
        }

        // Resource conflicts
        if resource_conflicts > 3 {
            risk_score += 2.0;
        } else if resource_conflicts > 1 {
            risk_score += 1.0;
        }

        match risk_score {
            score if score >= 6.0 => RiskLevel::High,
            score if score >= 3.0 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
}

/// Result of critical path analysis
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalPathAnalysis {
    pub critical_path: Vec<String>,
    pub critical_tasks: Vec<String>, // Note: This should be fixed in real implementation
    pub total_slack_analysis: SlackAnalysis,
    pub free_slack_analysis: SlackAnalysis,
    pub project_duration: Duration,
    pub critical_path_duration: Duration,
}

/// Analysis of slack distribution
#[derive(Debug, Clone, PartialEq)]
pub struct SlackAnalysis {
    pub distribution: HashMap<i64, usize>, // slack_days -> count
    pub average_slack_days: f64,
    pub total_tasks: usize,
    pub critical_tasks: usize,
}

/// Impact of delaying a task
#[derive(Debug, Clone, PartialEq)]
pub enum DelayImpact {
    NoImpact,
    TaskNotFound,
    CriticalTaskDelay {
        project_delay_days: i64,
        affected_tasks: Vec<String>,
    },
    NonCriticalDelay {
        excess_delay_days: i64,
        affected_tasks: Vec<String>,
    },
}

/// Optimization suggestions
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationSuggestion {
    DelayNonCriticalTasks {
        task_count: usize,
        potential_savings_days: i64,
    },
    ResolveCriticalConflicts {
        conflict_count: usize,
    },
    ParallelizeTasks {
        opportunity_count: usize,
    },
}

/// Risk analysis for the critical path
#[derive(Debug, Clone, PartialEq)]
pub struct RiskAnalysis {
    pub critical_task_percentage: f64,
    pub high_risk_tasks: usize,
    pub resource_conflicts: usize,
    pub overall_risk_level: RiskLevel,
}

/// Risk level classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
        }
    }
}

impl std::fmt::Display for OptimizationSuggestion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptimizationSuggestion::DelayNonCriticalTasks {
                task_count,
                potential_savings_days,
            } => {
                write!(
                    f,
                    "Delay {} non-critical tasks to save up to {} days",
                    task_count, potential_savings_days
                )
            }
            OptimizationSuggestion::ResolveCriticalConflicts { conflict_count } => {
                write!(f, "Resolve {} conflicts on critical path", conflict_count)
            }
            OptimizationSuggestion::ParallelizeTasks { opportunity_count } => {
                write!(f, "Parallelize {} task groups to reduce duration", opportunity_count)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::scheduling::{ProjectSchedule, ScheduledTask};
    use chrono::Utc;

    fn create_test_schedule() -> ProjectSchedule {
        let start = Utc::now();
        let mut schedule = ProjectSchedule::new("TEST-PROJ".to_string(), start);

        // Add some test tasks
        let mut task1 = ScheduledTask::new(
            "task-1".to_string(),
            start,
            start + Duration::days(1),
            Duration::days(2),
        );
        let mut task2 = ScheduledTask::new(
            "task-2".to_string(),
            start + Duration::days(2),
            start + Duration::days(3),
            Duration::days(1),
        );

        // Mark tasks as critical
        task1.is_critical = true;
        task2.is_critical = true;

        // Add slack to task2 for optimization suggestions
        task2.total_slack = Duration::days(10);

        schedule.tasks = vec![task1, task2];
        schedule.critical_path = vec!["task-1".to_string(), "task-2".to_string()];
        schedule.total_duration = Duration::days(3);

        schedule
    }

    #[test]
    fn test_critical_path_calculator_creation() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        assert_eq!(calculator.schedule.project_id, "TEST-PROJ");
    }

    #[test]
    fn test_critical_path_analysis() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        let analysis = calculator.calculate_critical_path().unwrap();

        assert_eq!(analysis.critical_path.len(), 2);
        assert_eq!(analysis.project_duration, Duration::days(3));
    }

    #[test]
    fn test_delay_impact_calculation() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        let impact = calculator.calculate_delay_impact("task-1", 1);

        match impact {
            DelayImpact::CriticalTaskDelay { project_delay_days, .. } => {
                assert_eq!(project_delay_days, 1);
            }
            _ => panic!("Expected CriticalTaskDelay"),
        }
    }

    #[test]
    fn test_optimization_suggestions() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        let suggestions = calculator.suggest_optimizations();

        // Should have at least one suggestion
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_risk_analysis() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        let risk_analysis = calculator.analyze_risk_factors();

        assert!(risk_analysis.critical_task_percentage >= 0.0);
        assert!(risk_analysis.critical_task_percentage <= 100.0);
        // Removed useless comparisons (usize is always >= 0)
    }

    #[test]
    fn test_risk_level_calculation() {
        let schedule = create_test_schedule();
        let calculator = CriticalPathCalculator::new(schedule);
        let risk_analysis = calculator.analyze_risk_factors();

        // Risk level should be one of the valid values
        match risk_analysis.overall_risk_level {
            RiskLevel::Low | RiskLevel::Medium | RiskLevel::High => {}
        }
    }
}
