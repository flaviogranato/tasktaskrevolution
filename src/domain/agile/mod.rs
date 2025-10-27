//! Agile methodologies support module
//!
//! This module provides comprehensive support for agile methodologies including
//! Scrum and Kanban with sprints, burndown charts, velocity tracking, and
//! agile-specific features.

pub mod sprint_manager;
pub mod kanban_board;
pub mod burndown_calculator;
pub mod velocity_tracker;
pub mod agile_reporter;
pub mod agile_models;

pub use sprint_manager::SprintManager;
pub use kanban_board::KanbanBoardManager;
pub use burndown_calculator::BurndownCalculator;
pub use velocity_tracker::VelocityTracker;
pub use agile_reporter::AgileReporter;
pub use agile_models::*;
