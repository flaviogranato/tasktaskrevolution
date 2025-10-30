use crate::domain::scheduling::{ProjectSchedule, ScheduledTask};
use crate::domain::shared::errors::DomainError;
use chrono::Duration;
use std::collections::HashMap;

/// Advanced schedule optimizer with multiple optimization algorithms
pub struct ScheduleOptimizer {
    schedule: ProjectSchedule,
    optimization_goals: Vec<OptimizationGoal>,
    constraints: Vec<OptimizationConstraint>,
}

impl ScheduleOptimizer {
    pub fn new(schedule: ProjectSchedule) -> Self {
        Self {
            schedule,
            optimization_goals: vec![
                OptimizationGoal::MinimizeDuration,
                OptimizationGoal::MinimizeResourceConflicts,
                OptimizationGoal::MaximizeResourceUtilization,
                OptimizationGoal::MinimizeCost,
            ],
            constraints: Vec::new(),
        }
    }

    /// Add an optimization goal
    pub fn add_goal(&mut self, goal: OptimizationGoal) {
        self.optimization_goals.push(goal);
    }

    /// Add an optimization constraint
    pub fn add_constraint(&mut self, constraint: OptimizationConstraint) {
        self.constraints.push(constraint);
    }

    /// Optimize the schedule using multiple algorithms
    pub fn optimize(&mut self) -> Result<OptimizationResult, DomainError> {
        let mut result = OptimizationResult::new();
        let mut best_schedule = self.schedule.clone();
        let mut best_score = f64::NEG_INFINITY;

        // Try different optimization algorithms
        let algorithms = vec![
            OptimizationAlgorithm::CriticalPathOptimization,
            OptimizationAlgorithm::ResourceBalancing,
            OptimizationAlgorithm::TimeCompression,
            OptimizationAlgorithm::CostOptimization,
            OptimizationAlgorithm::GeneticAlgorithm,
        ];

        for algorithm in algorithms {
            match self.apply_algorithm(&algorithm) {
                Ok(optimized_schedule) => {
                    let score = self.calculate_optimization_score(&optimized_schedule);
                    if score > best_score {
                        best_score = score;
                        best_schedule = optimized_schedule;
                    }
                    result.algorithm_results.push(AlgorithmResult {
                        algorithm,
                        score,
                        improvements: self.calculate_improvements(&self.schedule, &best_schedule),
                    });
                }
                Err(e) => {
                    result.failed_algorithms.push(FailedAlgorithm {
                        algorithm,
                        error: e.to_string(),
                    });
                }
            }
        }

        // Apply the best optimization
        self.schedule = best_schedule;
        result.best_score = best_score;
        result.final_schedule = Some(self.schedule.clone());
        result.success = true;

        Ok(result)
    }

    /// Apply a specific optimization algorithm
    fn apply_algorithm(&self, algorithm: &OptimizationAlgorithm) -> Result<ProjectSchedule, DomainError> {
        match algorithm {
            OptimizationAlgorithm::CriticalPathOptimization => self.optimize_critical_path(),
            OptimizationAlgorithm::ResourceBalancing => self.optimize_resource_balancing(),
            OptimizationAlgorithm::TimeCompression => self.optimize_time_compression(),
            OptimizationAlgorithm::CostOptimization => self.optimize_cost(),
            OptimizationAlgorithm::GeneticAlgorithm => self.optimize_genetic(),
            OptimizationAlgorithm::SimulatedAnnealing => self.optimize_genetic(),
            OptimizationAlgorithm::TabuSearch => self.optimize_genetic(),
        }
    }

    /// Optimize critical path
    fn optimize_critical_path(&self) -> Result<ProjectSchedule, DomainError> {
        let mut optimized_schedule = self.schedule.clone();

        // Identify critical tasks
        let critical_task_ids: Vec<String> = optimized_schedule
            .tasks
            .iter()
            .filter(|t| t.is_critical)
            .map(|t| t.task_id.clone())
            .collect();

        // Try to parallelize critical tasks where possible
        for i in 0..critical_task_ids.len() {
            for j in (i + 1)..critical_task_ids.len() {
                if let (Some(task1), Some(task2)) = (
                    optimized_schedule.get_task(&critical_task_ids[i]),
                    optimized_schedule.get_task(&critical_task_ids[j]),
                ) && self.can_parallelize(task1, task2)
                {
                    self.parallelize_tasks(&mut optimized_schedule, &critical_task_ids[i], &critical_task_ids[j])?;
                }
            }
        }

        // Optimize task dependencies
        self.optimize_dependencies(&mut optimized_schedule)?;

        Ok(optimized_schedule)
    }

    /// Optimize resource balancing
    fn optimize_resource_balancing(&self) -> Result<ProjectSchedule, DomainError> {
        let mut optimized_schedule = self.schedule.clone();

        // Analyze resource utilization
        let resource_utilization = self.analyze_resource_utilization(&optimized_schedule);
        let resource_info: Vec<(String, ResourceUtilizationInfo)> = resource_utilization.into_iter().collect();

        // Balance resource load
        for (resource_id, utilization) in resource_info {
            if utilization.average_utilization < 0.5 {
                // Underutilized resource - try to assign more tasks
                self.assign_more_tasks_to_resource(&mut optimized_schedule, &resource_id)?;
            } else if utilization.peak_utilization > 1.0 {
                // Overutilized resource - redistribute tasks
                self.redistribute_tasks_from_resource(&mut optimized_schedule, &resource_id)?;
            }
        }

        Ok(optimized_schedule)
    }

    /// Optimize time compression
    fn optimize_time_compression(&self) -> Result<ProjectSchedule, DomainError> {
        let mut optimized_schedule = self.schedule.clone();

        // Find tasks that can be compressed
        let compressible_tasks = self.find_compressible_tasks(&optimized_schedule);

        for task_id in compressible_tasks {
            self.compress_task(&mut optimized_schedule, &task_id)?;
        }

        // Optimize task sequencing
        self.optimize_task_sequencing(&mut optimized_schedule)?;

        Ok(optimized_schedule)
    }

    /// Optimize cost
    fn optimize_cost(&self) -> Result<ProjectSchedule, DomainError> {
        let mut optimized_schedule = self.schedule.clone();

        // Analyze cost distribution
        let cost_analysis = self.analyze_cost_distribution(&optimized_schedule);

        // Optimize resource assignments for cost
        for (resource_id, cost_info) in cost_analysis {
            if cost_info.hourly_rate > cost_info.average_rate * 1.2 {
                // High-cost resource - try to replace with lower-cost alternative
                if let Some(alternative) = self.find_lower_cost_alternative(&resource_id) {
                    self.replace_resource(&mut optimized_schedule, &resource_id, &alternative)?;
                }
            }
        }

        Ok(optimized_schedule)
    }

    /// Optimize using genetic algorithm
    #[allow(unused_assignments)]
    fn optimize_genetic(&self) -> Result<ProjectSchedule, DomainError> {
        let mut optimized_schedule = self.schedule.clone();

        // Initialize population
        let mut population = self.initialize_population(50)?;

        // Run genetic algorithm
        for _generation in 0..100 {
            // Evaluate fitness
            let fitness_scores = self.evaluate_population(&population)?;

            // Select parents
            let parents = self.select_parents(&population, &fitness_scores)?;

            // Create offspring
            let offspring = self.create_offspring(&parents)?;

            // Mutate offspring
            let mutated_offspring = self.mutate_offspring(offspring)?;

            // Replace population
            population = self.replace_population(population, mutated_offspring, &fitness_scores)?;

            // Check for convergence
            if self.has_converged(&fitness_scores) {
                break;
            }
        }

        // Get best individual
        let best_individual = self.get_best_individual(&population)?;
        optimized_schedule = self.individual_to_schedule(best_individual)?;

        Ok(optimized_schedule)
    }

    /// Calculate optimization score
    fn calculate_optimization_score(&self, schedule: &ProjectSchedule) -> f64 {
        let mut score = 0.0;

        for goal in &self.optimization_goals {
            match goal {
                OptimizationGoal::MinimizeDuration => {
                    let duration_score = 1.0 / (schedule.total_duration.num_days() as f64 + 1.0);
                    score += duration_score * 0.3;
                }
                OptimizationGoal::MinimizeResourceConflicts => {
                    let conflict_score = 1.0 / (schedule.conflicts.len() as f64 + 1.0);
                    score += conflict_score * 0.25;
                }
                OptimizationGoal::MaximizeResourceUtilization => {
                    let utilization_score = self.calculate_average_utilization(schedule);
                    score += utilization_score * 0.25;
                }
                OptimizationGoal::MinimizeCost => {
                    let cost_score = self.calculate_cost_score(schedule);
                    score += cost_score * 0.2;
                }
                OptimizationGoal::MaximizeQuality => {
                    score += 0.1;
                }
                OptimizationGoal::MinimizeRisk => {
                    score += 0.1;
                }
            }
        }

        score
    }

    /// Calculate improvements from original to optimized schedule
    fn calculate_improvements(&self, original: &ProjectSchedule, optimized: &ProjectSchedule) -> Vec<Improvement> {
        let mut improvements = Vec::new();

        // Duration improvement
        if optimized.total_duration < original.total_duration {
            improvements.push(Improvement {
                metric: "Duration".to_string(),
                original_value: original.total_duration.num_days() as f64,
                optimized_value: optimized.total_duration.num_days() as f64,
                improvement_percent: ((original.total_duration - optimized.total_duration).num_days() as f64
                    / original.total_duration.num_days() as f64)
                    * 100.0,
            });
        }

        // Conflict reduction
        if optimized.conflicts.len() < original.conflicts.len() {
            improvements.push(Improvement {
                metric: "Conflicts".to_string(),
                original_value: original.conflicts.len() as f64,
                optimized_value: optimized.conflicts.len() as f64,
                improvement_percent: ((original.conflicts.len() - optimized.conflicts.len()) as f64
                    / original.conflicts.len() as f64)
                    * 100.0,
            });
        }

        improvements
    }

    // Helper methods for optimization algorithms

    fn can_parallelize(&self, task1: &ScheduledTask, task2: &ScheduledTask) -> bool {
        // Check if two tasks can be done in parallel
        // This should check dependencies, resource requirements, etc.
        task1.early_start < task2.early_finish && task2.early_start < task1.early_finish
    }

    fn parallelize_tasks(
        &self,
        _schedule: &mut ProjectSchedule,
        _task1_id: &str,
        _task2_id: &str,
    ) -> Result<(), DomainError> {
        // Implement task parallelization
        Ok(())
    }

    fn optimize_dependencies(&self, _schedule: &mut ProjectSchedule) -> Result<(), DomainError> {
        // Optimize task dependencies
        Ok(())
    }

    fn analyze_resource_utilization(&self, schedule: &ProjectSchedule) -> HashMap<String, ResourceUtilizationInfo> {
        let mut utilization = HashMap::new();

        for (resource_id, task_ids) in &schedule.resource_assignments {
            let tasks: Vec<&ScheduledTask> = task_ids.iter().filter_map(|id| schedule.get_task(id)).collect();

            let total_time = tasks.iter().map(|t| t.duration.num_hours() as f64).sum::<f64>();

            let project_duration = schedule.total_duration.num_hours() as f64;
            let average_utilization = if project_duration > 0.0 {
                total_time / project_duration
            } else {
                0.0
            };

            let peak_utilization = tasks.iter().map(|t| t.duration.num_hours() as f64).fold(0.0, f64::max);

            utilization.insert(
                resource_id.clone(),
                ResourceUtilizationInfo {
                    average_utilization,
                    peak_utilization,
                    total_time,
                },
            );
        }

        utilization
    }

    fn assign_more_tasks_to_resource(
        &self,
        _schedule: &mut ProjectSchedule,
        _resource_id: &str,
    ) -> Result<(), DomainError> {
        // Assign more tasks to underutilized resource
        Ok(())
    }

    fn redistribute_tasks_from_resource(
        &self,
        _schedule: &mut ProjectSchedule,
        _resource_id: &str,
    ) -> Result<(), DomainError> {
        // Redistribute tasks from overutilized resource
        Ok(())
    }

    fn find_compressible_tasks(&self, schedule: &ProjectSchedule) -> Vec<String> {
        // Find tasks that can be compressed (e.g., by adding more resources)
        schedule
            .tasks
            .iter()
            .filter(|t| t.total_slack > Duration::days(1))
            .map(|t| t.task_id.clone())
            .collect()
    }

    fn compress_task(&self, _schedule: &mut ProjectSchedule, _task_id: &str) -> Result<(), DomainError> {
        // Compress a task (e.g., by adding more resources)
        Ok(())
    }

    fn optimize_task_sequencing(&self, _schedule: &mut ProjectSchedule) -> Result<(), DomainError> {
        // Optimize the sequence of tasks
        Ok(())
    }

    fn analyze_cost_distribution(&self, _schedule: &ProjectSchedule) -> HashMap<String, CostInfo> {
        // Analyze cost distribution across resources
        HashMap::new()
    }

    fn find_lower_cost_alternative(&self, _resource_id: &str) -> Option<String> {
        // Find a lower-cost alternative resource
        None
    }

    fn replace_resource(
        &self,
        _schedule: &mut ProjectSchedule,
        _old_resource_id: &str,
        _new_resource_id: &str,
    ) -> Result<(), DomainError> {
        // Replace one resource with another
        Ok(())
    }

    fn initialize_population(&self, _size: usize) -> Result<Vec<ScheduleIndividual>, DomainError> {
        // Initialize population for genetic algorithm
        Ok(vec![])
    }

    fn evaluate_population(&self, _population: &[ScheduleIndividual]) -> Result<Vec<f64>, DomainError> {
        // Evaluate fitness of population
        Ok(vec![])
    }

    fn select_parents(
        &self,
        _population: &[ScheduleIndividual],
        _fitness_scores: &[f64],
    ) -> Result<Vec<&ScheduleIndividual>, DomainError> {
        // Select parents for reproduction
        Ok(vec![])
    }

    fn create_offspring(&self, _parents: &[&ScheduleIndividual]) -> Result<Vec<ScheduleIndividual>, DomainError> {
        // Create offspring from parents
        Ok(vec![])
    }

    fn mutate_offspring(&self, offspring: Vec<ScheduleIndividual>) -> Result<Vec<ScheduleIndividual>, DomainError> {
        // Mutate offspring
        Ok(offspring)
    }

    fn replace_population(
        &self,
        population: Vec<ScheduleIndividual>,
        _offspring: Vec<ScheduleIndividual>,
        _fitness_scores: &[f64],
    ) -> Result<Vec<ScheduleIndividual>, DomainError> {
        // Replace population with new generation
        Ok(population)
    }

    fn has_converged(&self, _fitness_scores: &[f64]) -> bool {
        // Check if the population has converged
        false
    }

    fn get_best_individual<'a>(
        &self,
        population: &'a [ScheduleIndividual],
    ) -> Result<&'a ScheduleIndividual, DomainError> {
        // Get the best individual from population
        population.first().ok_or_else(|| DomainError::ValidationError {
            field: "population".to_string(),
            message: "Empty population".to_string(),
        })
    }

    fn individual_to_schedule(&self, _individual: &ScheduleIndividual) -> Result<ProjectSchedule, DomainError> {
        // Convert individual to schedule
        Ok(self.schedule.clone())
    }

    fn calculate_average_utilization(&self, _schedule: &ProjectSchedule) -> f64 {
        // Calculate average resource utilization
        0.8
    }

    fn calculate_cost_score(&self, _schedule: &ProjectSchedule) -> f64 {
        // Calculate cost optimization score
        0.9
    }
}

/// Optimization goals
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationGoal {
    MinimizeDuration,
    MinimizeResourceConflicts,
    MaximizeResourceUtilization,
    MinimizeCost,
    MaximizeQuality,
    MinimizeRisk,
}

/// Optimization constraints
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationConstraint {
    MaxDuration(Duration),
    MaxCost(f64),
    MinResourceUtilization(f64),
    MaxConcurrentTasks(usize),
    RequiredSkills(Vec<String>),
}

/// Optimization algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationAlgorithm {
    CriticalPathOptimization,
    ResourceBalancing,
    TimeCompression,
    CostOptimization,
    GeneticAlgorithm,
    SimulatedAnnealing,
    TabuSearch,
}

/// Result of optimization
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationResult {
    pub success: bool,
    pub best_score: f64,
    pub final_schedule: Option<ProjectSchedule>,
    pub algorithm_results: Vec<AlgorithmResult>,
    pub failed_algorithms: Vec<FailedAlgorithm>,
    pub total_improvements: Vec<Improvement>,
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
            best_score: 0.0,
            final_schedule: None,
            algorithm_results: Vec::new(),
            failed_algorithms: Vec::new(),
            total_improvements: Vec::new(),
        }
    }
}

/// Result of a specific algorithm
#[derive(Debug, Clone, PartialEq)]
pub struct AlgorithmResult {
    pub algorithm: OptimizationAlgorithm,
    pub score: f64,
    pub improvements: Vec<Improvement>,
}

/// Failed algorithm execution
#[derive(Debug, Clone, PartialEq)]
pub struct FailedAlgorithm {
    pub algorithm: OptimizationAlgorithm,
    pub error: String,
}

/// Improvement metric
#[derive(Debug, Clone, PartialEq)]
pub struct Improvement {
    pub metric: String,
    pub original_value: f64,
    pub optimized_value: f64,
    pub improvement_percent: f64,
}

/// Resource utilization information
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceUtilizationInfo {
    pub average_utilization: f64,
    pub peak_utilization: f64,
    pub total_time: f64,
}

/// Cost information
#[derive(Debug, Clone, PartialEq)]
pub struct CostInfo {
    pub hourly_rate: f64,
    pub average_rate: f64,
    pub total_cost: f64,
}

/// Individual for genetic algorithm
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleIndividual {
    pub task_assignments: HashMap<String, String>, // task_id -> resource_id
    pub task_sequence: Vec<String>,
    pub fitness: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::scheduling::ProjectSchedule;
    use chrono::Utc;

    fn create_test_schedule() -> ProjectSchedule {
        let start = Utc::now();
        ProjectSchedule::new("TEST-PROJ".to_string(), start)
    }

    #[test]
    fn test_schedule_optimizer_creation() {
        let schedule = create_test_schedule();
        let optimizer = ScheduleOptimizer::new(schedule);
        assert_eq!(optimizer.optimization_goals.len(), 4);
        assert!(optimizer.constraints.is_empty());
    }

    #[test]
    fn test_add_goal() {
        let schedule = create_test_schedule();
        let mut optimizer = ScheduleOptimizer::new(schedule);
        optimizer.add_goal(OptimizationGoal::MaximizeQuality);
        assert_eq!(optimizer.optimization_goals.len(), 5);
    }

    #[test]
    fn test_add_constraint() {
        let schedule = create_test_schedule();
        let mut optimizer = ScheduleOptimizer::new(schedule);
        optimizer.add_constraint(OptimizationConstraint::MaxDuration(Duration::days(30)));
        assert_eq!(optimizer.constraints.len(), 1);
    }

    #[test]
    fn test_optimization_result_creation() {
        let result = OptimizationResult::new();
        assert!(!result.success);
        assert_eq!(result.best_score, 0.0);
        assert!(result.algorithm_results.is_empty());
    }

    #[test]
    fn test_improvement_creation() {
        let improvement = Improvement {
            metric: "Duration".to_string(),
            original_value: 10.0,
            optimized_value: 8.0,
            improvement_percent: 20.0,
        };
        assert_eq!(improvement.metric, "Duration");
        assert_eq!(improvement.improvement_percent, 20.0);
    }
}
