use super::{
    resource::{Period, Resource},
    state::{Assigned, Available, Inactive},
};
use serde::Serialize;
use uuid7::Uuid;

/// An enum to represent a Resource in any of its possible states.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state")]
pub enum AnyResource {
    Available(Resource<Available>),
    Assigned(Resource<Assigned>),
    Inactive(Resource<Inactive>),
}

impl AnyResource {
    #[allow(dead_code)]
    pub fn id(&self) -> &Uuid {
        match self {
            AnyResource::Available(r) => &r.id,
            AnyResource::Assigned(r) => &r.id,
            AnyResource::Inactive(r) => &r.id,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            AnyResource::Available(r) => &r.code,
            AnyResource::Assigned(r) => &r.code,
            AnyResource::Inactive(r) => &r.code,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AnyResource::Available(r) => &r.name,
            AnyResource::Assigned(r) => &r.name,
            AnyResource::Inactive(r) => &r.name,
        }
    }

    pub fn resource_type(&self) -> &str {
        match self {
            AnyResource::Available(r) => &r.resource_type,
            AnyResource::Assigned(r) => &r.resource_type,
            AnyResource::Inactive(r) => &r.resource_type,
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
