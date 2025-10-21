pub mod conflict_resolver;
pub mod critical_path;
pub mod resource_leveler;
pub mod schedule_optimizer;
pub mod scheduler;

// Re-export main types
pub use conflict_resolver::{ConflictResolutionReport, ConflictResolver, ResolutionStatistics, ResolutionStrategy};
pub use critical_path::{
    CriticalPathAnalysis, CriticalPathCalculator, DelayImpact, OptimizationSuggestion, SlackAnalysis,
};
pub use resource_leveler::{
    ConflictResolution, OptimizationResult as ResourceOptimizationResult, ResolutionMethod, ResourceLeveler,
    ResourceLevelingResult, ResourceUtilization,
};
pub use schedule_optimizer::{
    Improvement, OptimizationAlgorithm, OptimizationConstraint, OptimizationGoal, OptimizationResult, ScheduleOptimizer,
};
pub use scheduler::{ConflictSeverity, ProjectSchedule, ResourceConflict, ScheduledTask, Scheduler, TimeRange};
