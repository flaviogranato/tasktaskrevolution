use serde::Serialize;

/// A marker trait for all task states.
pub trait TaskState: Sized + std::fmt::Debug {}

/// State for a task that has been planned but not yet started.
#[derive(Debug, Clone, Serialize)]
pub struct Planned;
impl TaskState for Planned {}

/// State for a task that is currently in progress.
#[derive(Debug, Clone, Serialize)]
pub struct InProgress {
    pub progress: u8,
}
impl TaskState for InProgress {}

/// State for a task that is blocked.
#[derive(Debug, Clone, Serialize)]
pub struct Blocked {
    pub reason: String,
}
impl TaskState for Blocked {}

/// State for a task that has been completed.
#[derive(Debug, Clone, Serialize)]
pub struct Completed;
impl TaskState for Completed {}

/// State for a task that has been cancelled.
#[derive(Debug, Clone, Serialize)]
pub struct Cancelled;
impl TaskState for Cancelled {}
