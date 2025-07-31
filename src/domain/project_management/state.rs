use serde::Serialize;

/// A marker trait for all project states.
pub trait ProjectState: Sized + std::fmt::Debug {}

/// State for a project that has been planned but not yet started.
#[derive(Debug, Clone, Serialize)]
pub struct Planned;
impl ProjectState for Planned {}

/// State for a project that is currently in progress.
#[derive(Debug, Clone, Serialize)]
pub struct InProgress;
impl ProjectState for InProgress {}

/// State for a project that has been completed.
#[derive(Debug, Clone, Serialize)]
pub struct Completed;
impl ProjectState for Completed {}

/// State for a project that has been cancelled.
#[derive(Debug, Clone, Serialize)]
pub struct Cancelled;
impl ProjectState for Cancelled {}
