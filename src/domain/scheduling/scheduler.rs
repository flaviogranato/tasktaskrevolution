use crate::domain::{
    project_management::AnyProject,
    shared::errors::DomainError,
    task_management::{AnyTask, TaskDependency},
};
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

/// Represents a scheduled task with calculated dates and slack
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTask {
    pub task_id: String,
    pub early_start: DateTime<Utc>,
    pub early_finish: DateTime<Utc>,
    pub late_start: DateTime<Utc>,
    pub late_finish: DateTime<Utc>,
    pub total_slack: Duration,
    pub free_slack: Duration,
    pub is_critical: bool,
    pub duration: Duration,
}

impl ScheduledTask {
    pub fn new(task_id: String, early_start: DateTime<Utc>, late_start: DateTime<Utc>, duration: Duration) -> Self {
        let early_finish = early_start + duration;
        let late_finish = late_start + duration;
        let total_slack = late_start - early_start;
        let free_slack = Duration::zero(); // Will be calculated later

        Self {
            task_id,
            early_start,
            early_finish,
            late_start,
            late_finish,
            total_slack,
            free_slack,
            is_critical: total_slack.is_zero(),
            duration,
        }
    }

    /// Calculate free slack based on successor tasks
    pub fn calculate_free_slack(&mut self, successors: &[&ScheduledTask]) {
        if successors.is_empty() {
            self.free_slack = self.total_slack;
            return;
        }

        let earliest_successor_start = successors
            .iter()
            .map(|s| s.early_start)
            .min()
            .unwrap_or(self.early_finish);

        self.free_slack = earliest_successor_start - self.early_finish;
        if self.free_slack < Duration::zero() {
            self.free_slack = Duration::zero();
        }
    }
}

/// Represents a complete project schedule
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSchedule {
    pub project_id: String,
    pub tasks: Vec<ScheduledTask>,
    pub critical_path: Vec<String>,
    pub total_duration: Duration,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub resource_assignments: HashMap<String, Vec<String>>, // resource_id -> task_ids
    pub conflicts: Vec<ResourceConflict>,
}

impl ProjectSchedule {
    pub fn new(project_id: String, start_date: DateTime<Utc>) -> Self {
        Self {
            project_id,
            tasks: Vec::new(),
            critical_path: Vec::new(),
            total_duration: Duration::zero(),
            start_date,
            end_date: start_date,
            resource_assignments: HashMap::new(),
            conflicts: Vec::new(),
        }
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&ScheduledTask> {
        self.tasks.iter().find(|t| t.task_id == task_id)
    }

    /// Get critical tasks
    pub fn get_critical_tasks(&self) -> Vec<&ScheduledTask> {
        self.tasks.iter().filter(|t| t.is_critical).collect()
    }

    /// Get tasks by resource
    pub fn get_tasks_by_resource(&self, resource_id: &str) -> Vec<&ScheduledTask> {
        self.resource_assignments
            .get(resource_id)
            .map(|task_ids| task_ids.iter().filter_map(|id| self.get_task(id)).collect())
            .unwrap_or_default()
    }
}

/// Represents a resource conflict
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceConflict {
    pub resource_id: String,
    pub conflicting_tasks: Vec<String>,
    pub conflict_period: TimeRange,
    pub severity: ConflictSeverity,
    pub suggested_resolution: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ConflictSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictSeverity::Low => write!(f, "Low"),
            ConflictSeverity::Medium => write!(f, "Medium"),
            ConflictSeverity::High => write!(f, "High"),
            ConflictSeverity::Critical => write!(f, "Critical"),
        }
    }
}

/// Main scheduler that performs forward and backward pass
pub struct Scheduler {
    project: AnyProject,
    tasks: Vec<AnyTask>,
    dependencies: Vec<TaskDependency>,
}

impl Scheduler {
    pub fn new(project: AnyProject, tasks: Vec<AnyTask>, dependencies: Vec<TaskDependency>) -> Self {
        Self {
            project,
            tasks,
            dependencies,
        }
    }

    /// Create a complete project schedule
    pub fn create_schedule(&self, start_date: DateTime<Utc>) -> Result<ProjectSchedule, DomainError> {
        let mut schedule = ProjectSchedule::new(self.project.code().to_string(), start_date);

        // Step 1: Forward pass - calculate early start/finish
        let early_schedule = self.forward_pass(start_date)?;

        // Step 2: Backward pass - calculate late start/finish
        let late_schedule = self.backward_pass(&early_schedule)?;

        // Step 3: Calculate slack and identify critical path
        let mut scheduled_tasks = self.calculate_slack(&late_schedule)?;

        // Step 4: Calculate free slack
        self.calculate_free_slack(&mut scheduled_tasks)?;

        // Step 5: Build final schedule
        schedule.tasks = scheduled_tasks;
        schedule.critical_path = self.identify_critical_path(&schedule.tasks);
        schedule.total_duration = self.calculate_total_duration(&schedule.tasks);
        schedule.end_date = start_date + schedule.total_duration;

        // Step 6: Assign resources and detect conflicts
        self.assign_resources(&mut schedule)?;
        self.detect_conflicts(&mut schedule)?;

        Ok(schedule)
    }

    /// Forward pass - calculate early start and finish dates
    fn forward_pass(&self, start_date: DateTime<Utc>) -> Result<Vec<ScheduledTask>, DomainError> {
        let mut scheduled_tasks = Vec::new();
        let task_map: HashMap<String, &AnyTask> = self.tasks.iter().map(|t| (t.id().to_string(), t)).collect();
        let mut early_finish_map: HashMap<String, DateTime<Utc>> = HashMap::new();

        // Initialize all tasks with default values
        for task in &self.tasks {
            let duration = self.calculate_task_duration(task);
            let scheduled_task = ScheduledTask::new(task.id().to_string(), start_date, start_date, duration);
            scheduled_tasks.push(scheduled_task);
            early_finish_map.insert(task.id().to_string(), start_date + duration);
        }

        // Process tasks in topological order
        let mut processed = HashSet::new();
        let mut queue: Vec<String> = self.get_root_tasks();

        while let Some(task_id) = queue.pop() {
            if processed.contains(&task_id) {
                continue;
            }

            let task = task_map.get(&task_id).ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                identifier: task_id.clone(),
            })?;

            // Calculate early start based on predecessors
            let early_start = self.calculate_early_start(&task_id, &early_finish_map, start_date);
            let duration = self.calculate_task_duration(task);
            let early_finish = early_start + duration;

            // Update the scheduled task
            if let Some(scheduled_task) = scheduled_tasks.iter_mut().find(|t| t.task_id == task_id) {
                scheduled_task.early_start = early_start;
                scheduled_task.early_finish = early_finish;
            }

            early_finish_map.insert(task_id.clone(), early_finish);
            processed.insert(task_id.clone());

            // Add successors to queue
            for dep in &self.dependencies {
                if dep.predecessor == task_id {
                    queue.push(dep.successor.clone());
                }
            }
        }

        Ok(scheduled_tasks)
    }

    /// Backward pass - calculate late start and finish dates
    fn backward_pass(&self, early_schedule: &[ScheduledTask]) -> Result<Vec<ScheduledTask>, DomainError> {
        let mut late_schedule = early_schedule.to_vec();

        // Find project end date (maximum early finish)
        let project_end = early_schedule
            .iter()
            .map(|t| t.early_finish)
            .max()
            .unwrap_or(Utc::now());

        // Initialize late dates
        for task in &mut late_schedule {
            task.late_finish = project_end;
            task.late_start = project_end - task.duration;
        }

        // Process tasks in reverse topological order
        let mut processed = HashSet::new();
        let mut queue: Vec<String> = self.get_leaf_tasks();

        while let Some(task_id) = queue.pop() {
            if processed.contains(&task_id) {
                continue;
            }

            // Calculate late dates based on successors
            let late_finish = self.calculate_late_finish(&task_id, &late_schedule);
            let task_duration = self.calculate_task_duration_by_id(&task_id)?;
            let late_start = late_finish - task_duration;

            // Update the scheduled task
            if let Some(scheduled_task) = late_schedule.iter_mut().find(|t| t.task_id == task_id) {
                scheduled_task.late_finish = late_finish;
                scheduled_task.late_start = late_start;
            }

            processed.insert(task_id.clone());

            // Add predecessors to queue
            for dep in &self.dependencies {
                if dep.successor == task_id {
                    queue.push(dep.predecessor.clone());
                }
            }
        }

        Ok(late_schedule)
    }

    /// Calculate slack for all tasks
    fn calculate_slack(&self, schedule: &[ScheduledTask]) -> Result<Vec<ScheduledTask>, DomainError> {
        let mut scheduled_tasks = schedule.to_vec();

        for task in &mut scheduled_tasks {
            task.total_slack = task.late_start - task.early_start;
            task.is_critical = task.total_slack.is_zero();
        }

        Ok(scheduled_tasks)
    }

    /// Calculate free slack for all tasks
    fn calculate_free_slack(&self, scheduled_tasks: &mut [ScheduledTask]) -> Result<(), DomainError> {
        // Create a copy of the tasks for reading successors
        let tasks_copy: Vec<ScheduledTask> = scheduled_tasks.to_vec();

        for task in scheduled_tasks.iter_mut() {
            let task_id = &task.task_id;
            let successors: Vec<&ScheduledTask> = self.get_successors(task_id, &tasks_copy);
            task.calculate_free_slack(&successors);
        }
        Ok(())
    }

    /// Identify critical path
    fn identify_critical_path(&self, tasks: &[ScheduledTask]) -> Vec<String> {
        let mut critical_path = Vec::new();
        let mut current_task = tasks.iter().find(|t| {
            t.is_critical && t.early_start == tasks.iter().map(|t| t.early_start).min().unwrap_or(Utc::now())
        });

        while let Some(task) = current_task {
            critical_path.push(task.task_id.clone());

            // Find next critical task
            current_task = tasks.iter().find(|t| {
                t.is_critical && t.early_start >= task.early_finish && self.has_dependency(&task.task_id, &t.task_id)
            });
        }

        critical_path
    }

    /// Calculate total project duration
    fn calculate_total_duration(&self, tasks: &[ScheduledTask]) -> Duration {
        tasks.iter().map(|t| t.early_finish).max().unwrap_or(Utc::now())
            - tasks.iter().map(|t| t.early_start).min().unwrap_or(Utc::now())
    }

    /// Assign resources to tasks
    fn assign_resources(&self, schedule: &mut ProjectSchedule) -> Result<(), DomainError> {
        for task in &schedule.tasks {
            if let Some(resource_id) = self.get_task_resource(&task.task_id) {
                schedule
                    .resource_assignments
                    .entry(resource_id)
                    .or_default()
                    .push(task.task_id.clone());
            }
        }
        Ok(())
    }

    /// Detect resource conflicts
    fn detect_conflicts(&self, schedule: &mut ProjectSchedule) -> Result<(), DomainError> {
        for (resource_id, task_ids) in &schedule.resource_assignments {
            if task_ids.len() > 1 {
                let conflicts = self.find_resource_conflicts(resource_id, task_ids, &schedule.tasks);
                schedule.conflicts.extend(conflicts);
            }
        }
        Ok(())
    }

    // Helper methods
    fn get_root_tasks(&self) -> Vec<String> {
        let all_task_ids: HashSet<String> = self.tasks.iter().map(|t| t.id().to_string()).collect();
        let dependent_tasks: HashSet<String> = self.dependencies.iter().map(|d| d.successor.clone()).collect();

        all_task_ids.difference(&dependent_tasks).cloned().collect()
    }

    fn get_leaf_tasks(&self) -> Vec<String> {
        let all_task_ids: HashSet<String> = self.tasks.iter().map(|t| t.id().to_string()).collect();
        let predecessor_tasks: HashSet<String> = self.dependencies.iter().map(|d| d.predecessor.clone()).collect();

        all_task_ids.difference(&predecessor_tasks).cloned().collect()
    }

    fn calculate_early_start(
        &self,
        task_id: &str,
        early_finish_map: &HashMap<String, DateTime<Utc>>,
        project_start: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let predecessors = self.get_predecessors(task_id);

        if predecessors.is_empty() {
            return project_start;
        }

        predecessors
            .iter()
            .filter_map(|pred_id| early_finish_map.get(pred_id))
            .max()
            .copied()
            .unwrap_or(project_start)
    }

    fn calculate_late_finish(&self, task_id: &str, schedule: &[ScheduledTask]) -> DateTime<Utc> {
        let successors = self.get_successors(task_id, schedule);

        if successors.is_empty() {
            return schedule.iter().map(|t| t.early_finish).max().unwrap_or(Utc::now());
        }

        successors.iter().map(|s| s.late_start).min().unwrap_or(Utc::now())
    }

    fn get_predecessors(&self, task_id: &str) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|d| d.successor == task_id)
            .map(|d| d.predecessor.clone())
            .collect()
    }

    fn get_successors<'a>(&self, task_id: &str, schedule: &'a [ScheduledTask]) -> Vec<&'a ScheduledTask> {
        let successor_ids: Vec<String> = self
            .dependencies
            .iter()
            .filter(|d| d.predecessor == task_id)
            .map(|d| d.successor.clone())
            .collect();

        schedule.iter().filter(|t| successor_ids.contains(&t.task_id)).collect()
    }

    fn has_dependency(&self, predecessor_id: &str, successor_id: &str) -> bool {
        self.dependencies
            .iter()
            .any(|d| d.predecessor == predecessor_id && d.successor == successor_id)
    }

    fn calculate_task_duration(&self, _task: &AnyTask) -> Duration {
        // For now, use a default duration based on task type
        // This should be enhanced to use actual task duration if available
        Duration::days(1)
    }

    fn calculate_task_duration_by_id(&self, task_id: &str) -> Result<Duration, DomainError> {
        let task = self
            .tasks
            .iter()
            .find(|t| t.id().to_string() == task_id)
            .ok_or_else(|| DomainError::EntityNotFound {
                entity_type: "Task".to_string(),
                identifier: task_id.to_string(),
            })?;

        Ok(self.calculate_task_duration(task))
    }

    fn get_task_resource(&self, _task_id: &str) -> Option<String> {
        // This should be enhanced to get actual resource assignment
        // For now, return None
        None
    }

    fn find_resource_conflicts(
        &self,
        resource_id: &str,
        task_ids: &[String],
        tasks: &[ScheduledTask],
    ) -> Vec<ResourceConflict> {
        let mut conflicts = Vec::new();

        for i in 0..task_ids.len() {
            for j in (i + 1)..task_ids.len() {
                let task1_id = &task_ids[i];
                let task2_id = &task_ids[j];

                if let (Some(task1), Some(task2)) = (
                    tasks.iter().find(|t| t.task_id == *task1_id),
                    tasks.iter().find(|t| t.task_id == *task2_id),
                ) {
                    #[allow(clippy::collapsible_if)]
                    if self.tasks_overlap(task1, task2) {
                        let conflict = ResourceConflict {
                            resource_id: resource_id.to_string(),
                            conflicting_tasks: vec![task1_id.clone(), task2_id.clone()],
                            conflict_period: TimeRange {
                                start: task1.early_start.max(task2.early_start),
                                end: task1.early_finish.min(task2.early_finish),
                            },
                            severity: ConflictSeverity::High,
                            suggested_resolution: Some("Reschedule one of the tasks".to_string()),
                        };
                        conflicts.push(conflict);
                    }
                }
            }
        }

        conflicts
    }

    fn tasks_overlap(&self, task1: &ScheduledTask, task2: &ScheduledTask) -> bool {
        task1.early_start < task2.early_finish && task2.early_start < task1.early_finish
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::domain::task_management::builder::TaskBuilder;
    use crate::domain::project_management::builder::ProjectBuilder;

    #[test]
    fn test_scheduler_creation() {
        let project = ProjectBuilder::new()
            .code("TEST-PROJ".to_string())
            .name("Test Project".to_string())
            .company_code("TEST-COMP".to_string())
            .created_by("test".to_string())
            .build()
            .unwrap()
            .into();

        let scheduler = Scheduler::new(project, vec![], vec![]);
        assert_eq!(scheduler.project.code(), "TEST-PROJ");
    }

    #[test]
    fn test_scheduled_task_creation() {
        let start = Utc::now();
        let duration = Duration::days(2);

        let task = ScheduledTask::new("task-1".to_string(), start, start + Duration::days(1), duration);

        assert_eq!(task.task_id, "task-1");
        assert_eq!(task.early_start, start);
        assert_eq!(task.early_finish, start + duration);
        assert_eq!(task.duration, duration);
    }

    #[test]
    fn test_project_schedule_creation() {
        let start = Utc::now();
        let schedule = ProjectSchedule::new("PROJ-1".to_string(), start);

        assert_eq!(schedule.project_id, "PROJ-1");
        assert_eq!(schedule.start_date, start);
        assert!(schedule.tasks.is_empty());
        assert!(schedule.critical_path.is_empty());
    }
}
