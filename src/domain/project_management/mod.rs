pub mod advanced_dependencies;
pub mod any_project;
pub mod builder;
pub mod calculation_cache_system;
pub mod change_propagation_system;
pub mod conflict_validation_system;
pub mod dependency_calculation_engine;
pub mod gantt_chart;
pub mod layoff_period;
pub mod project;
pub mod repository;
pub mod state;
pub mod template;
pub mod vacation_rules;

pub use advanced_dependencies::{AdvancedDependency, AdvancedDependencyGraph, DependencyType, LagType, TaskNode};
pub use any_project::AnyProject;
pub use calculation_cache_system::{CacheConfig, CacheStats, CalculationCacheSystem};
pub use change_propagation_system::{
    ChangePropagationSystem, ChangeType, PropagationConfig, PropagationResult, PropagationStatus,
};
pub use conflict_validation_system::{
    ConflictReport, ConflictSeverity, ConflictType, ConflictValidationSystem, ValidationConfig, ValidationStatus,
};
pub use dependency_calculation_engine::{
    CalculationConfig, CalculationResult, DependencyCalculationEngine, TaskCalculationStatus,
};
pub use gantt_chart::{GanttChart, GanttConfig, GanttDependency, GanttTask, GanttViewType, TaskStatus};
pub use template::ProjectTemplate;
