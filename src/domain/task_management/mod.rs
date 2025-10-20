#![allow(unused_imports)]

// Module declarations
pub mod any_task;
pub mod builder;
pub mod category;
pub mod dependency;
pub mod dependency_graph;
pub mod dependency_validator;
pub mod priority;
pub mod repository;
pub mod resource_assignment;
pub mod state;
pub mod task;

// Re-export public items from sub-modules
pub use any_task::AnyTask;
pub use builder::TaskBuilder;
pub use category::Category;
pub use dependency::{DependencyError, DependencyStatus, DependencyType, TaskDependency};
pub use dependency_graph::{DependencyGraph, DependencyGraphSummary};
pub use dependency_validator::{DependencyValidator, ValidationSummary};
pub use priority::Priority;
pub use task::{DateRange, Task};
