pub mod repository;
pub mod resource;
pub mod state;

use self::state::{Assigned, Available, Inactive};
use serde::Serialize;

pub use resource::{Period, PeriodType, ProjectAssignment, Resource, TimeOffEntry};

/// An enum to represent a Resource in any of its possible states.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state")]
pub enum AnyResource {
    Available(Resource<Available>),
    Assigned(Resource<Assigned>),
    Inactive(Resource<Inactive>),
}

impl AnyResource {
    pub fn id(&self) -> Option<&str> {
        match self {
            AnyResource::Available(r) => r.id.as_deref(),
            AnyResource::Assigned(r) => r.id.as_deref(),
            AnyResource::Inactive(r) => r.id.as_deref(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AnyResource::Available(r) => &r.name,
            AnyResource::Assigned(r) => &r.name,
            AnyResource::Inactive(r) => &r.name,
        }
    }

    pub fn vacations(&self) -> Option<&Vec<Period>> {
        match self {
            AnyResource::Available(r) => r.vacations.as_ref(),
            AnyResource::Assigned(r) => r.vacations.as_ref(),
            AnyResource::Inactive(r) => r.vacations.as_ref(),
        }
    }

    pub fn time_off_balance(&self) -> u32 {
        match self {
            AnyResource::Available(r) => r.time_off_balance,
            AnyResource::Assigned(r) => r.time_off_balance,
            AnyResource::Inactive(r) => r.time_off_balance,
        }
    }
}

impl From<Resource<Available>> for AnyResource {
    fn from(resource: Resource<Available>) -> Self {
        AnyResource::Available(resource)
    }
}

impl From<Resource<Assigned>> for AnyResource {
    fn from(resource: Resource<Assigned>) -> Self {
        AnyResource::Assigned(resource)
    }
}

impl From<Resource<Inactive>> for AnyResource {
    fn from(resource: Resource<Inactive>) -> Self {
        AnyResource::Inactive(resource)
    }
}
