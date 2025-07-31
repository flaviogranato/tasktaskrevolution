use super::resource::ProjectAssignment;
use serde::Serialize;

/// A marker trait for all resource states.
pub trait ResourceState: Sized + std::fmt::Debug {}

/// State for a resource that is available for project assignments.
#[derive(Debug, Clone, Serialize)]
pub struct Available;
impl ResourceState for Available {}

/// State for a resource that is currently assigned to one or more projects.
#[derive(Debug, Clone, Serialize)]
pub struct Assigned {
    pub project_assignments: Vec<ProjectAssignment>,
}
impl ResourceState for Assigned {}

/// State for a resource that is no longer active.
#[derive(Debug, Clone, Serialize)]
pub struct Inactive;
impl ResourceState for Inactive {}
