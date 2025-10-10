#![allow(dead_code)]

use super::super::shared::query_engine::Queryable;
use super::super::shared::query_parser::QueryValue;
use super::{
    resource::{Period, Resource, TimeOffEntry},
    state::{Assigned, Available, Inactive, ResourceState},
};
use chrono::Datelike;
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

    pub fn vacations(&self) -> Option<&[Period]> {
        // Otimizado: retorna slice em vez de &Vec
        match self {
            AnyResource::Available(r) => r.vacations.as_deref(),
            AnyResource::Assigned(r) => r.vacations.as_deref(),
            AnyResource::Inactive(r) => r.vacations.as_deref(),
        }
    }

    pub fn time_off_balance(&self) -> u32 {
        match self {
            AnyResource::Available(r) => r.time_off_balance,
            AnyResource::Assigned(r) => r.time_off_balance,
            AnyResource::Inactive(r) => r.time_off_balance,
        }
    }

    pub fn email(&self) -> Option<&str> {
        // Otimizado: retorna &str em vez de &String
        match self {
            AnyResource::Available(r) => r.email.as_deref(),
            AnyResource::Assigned(r) => r.email.as_deref(),
            AnyResource::Inactive(r) => r.email.as_deref(),
        }
    }

    #[allow(dead_code)]
    pub fn status(&self) -> &str {
        // Otimizado: removido 'static desnecessário
        match self {
            AnyResource::Available(_) => "Available",
            AnyResource::Assigned(_) => "Assigned",
            AnyResource::Inactive(_) => "Inactive",
        }
    }

    // --- Zero-copy accessors ---

    pub fn vacations_iter(&self) -> Option<impl Iterator<Item = &Period>> {
        self.vacations().map(|v| v.iter())
    }

    pub fn time_off_history_iter(&self) -> Option<impl Iterator<Item = &TimeOffEntry>> {
        match self {
            AnyResource::Available(r) => r.time_off_history.as_deref().map(|h| h.iter()),
            AnyResource::Assigned(r) => r.time_off_history.as_deref().map(|h| h.iter()),
            AnyResource::Inactive(r) => r.time_off_history.as_deref().map(|h| h.iter()),
        }
    }

    // --- State Transitions ---

    pub fn deactivate(self) -> Result<AnyResource, String> {
        let inactive_resource = match self {
            AnyResource::Available(r) => r.deactivate().into(),
            AnyResource::Assigned(r) => r.deactivate().into(),
            AnyResource::Inactive(_) => return Err("Resource is already inactive.".to_string()),
        };
        Ok(inactive_resource)
    }

    // --- Setters for updating fields ---

    pub fn set_name(&mut self, name: String) {
        match self {
            AnyResource::Available(r) => r.name = name,
            AnyResource::Assigned(r) => r.name = name,
            AnyResource::Inactive(r) => r.name = name,
        }
    }

    pub fn set_email(&mut self, email: Option<String>) {
        match self {
            AnyResource::Available(r) => r.email = email,
            AnyResource::Assigned(r) => r.email = email,
            AnyResource::Inactive(r) => r.email = email,
        }
    }

    pub fn set_resource_type(&mut self, resource_type: String) {
        match self {
            AnyResource::Available(r) => r.resource_type = resource_type,
            AnyResource::Assigned(r) => r.resource_type = resource_type,
            AnyResource::Inactive(r) => r.resource_type = resource_type,
        }
    }

    // --- Availability Methods ---

    pub fn is_available_on_date(&self, date: chrono::NaiveDate) -> bool {
        match self {
            AnyResource::Available(r) => self.check_resource_availability(r, date),
            AnyResource::Assigned(r) => self.check_resource_availability(r, date),
            AnyResource::Inactive(_) => false,
        }
    }

    pub fn is_available_during_period(&self, start_date: chrono::NaiveDate, end_date: chrono::NaiveDate) -> bool {
        let mut current_date = start_date;
        while current_date <= end_date {
            if !self.is_available_on_date(current_date) {
                return false;
            }
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }
        true
    }

    pub fn is_holiday(&self, date: chrono::NaiveDate) -> bool {
        if let Some(vacations) = self.vacations() {
            vacations
                .iter()
                .any(|period| date >= period.start_date.date_naive() && date <= period.end_date.date_naive())
        } else {
            false
        }
    }

    pub fn is_on_leave(&self, date: chrono::NaiveDate) -> bool {
        // For now, treat vacations as leave
        self.is_holiday(date)
    }

    pub fn is_working_day(&self, date: chrono::NaiveDate) -> bool {
        // Simple implementation: Monday to Friday are working days
        matches!(
            date.weekday(),
            chrono::Weekday::Mon
                | chrono::Weekday::Tue
                | chrono::Weekday::Wed
                | chrono::Weekday::Thu
                | chrono::Weekday::Fri
        )
    }

    pub fn max_allocation_percentage(&self) -> u32 {
        match self {
            AnyResource::Available(r) => r
                .wip_limits
                .as_ref()
                .map(|w| w.max_allocation_percentage as u32)
                .unwrap_or(100),
            AnyResource::Assigned(r) => r
                .wip_limits
                .as_ref()
                .map(|w| w.max_allocation_percentage as u32)
                .unwrap_or(100),
            AnyResource::Inactive(_) => 0,
        }
    }

    fn check_resource_availability<S: ResourceState>(&self, resource: &Resource<S>, date: chrono::NaiveDate) -> bool {
        // Check if resource is within active period
        if let Some(start_date) = resource.start_date
            && date < start_date
        {
            return false;
        }
        if let Some(end_date) = resource.end_date
            && date > end_date
        {
            return false;
        }

        // Check if it's a working day
        if !self.is_working_day(date) {
            return false;
        }

        // Check if it's a holiday/vacation
        if self.is_holiday(date) {
            return false;
        }

        true
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

impl Queryable for AnyResource {
    fn get_field_value(&self, field: &str) -> Option<QueryValue> {
        match self {
            AnyResource::Available(resource) => get_resource_field_value(resource, field, self.status()),
            AnyResource::Assigned(resource) => get_resource_field_value(resource, field, self.status()),
            AnyResource::Inactive(resource) => get_resource_field_value(resource, field, self.status()),
        }
    }

    fn entity_type() -> &'static str {
        "resource"
    }
}

fn get_resource_field_value(
    resource: &Resource<impl crate::domain::resource_management::state::ResourceState>,
    field: &str,
    status: &str,
) -> Option<QueryValue> {
    match field {
        "id" => Some(QueryValue::String(resource.id.to_string())),
        "code" => Some(QueryValue::String(resource.code.clone())),
        "name" => Some(QueryValue::String(resource.name.clone())),
        "email" => resource.email.as_ref().map(|e| QueryValue::String(e.clone())),
        "resource_type" => Some(QueryValue::String(resource.resource_type.clone())),
        "status" => Some(QueryValue::String(status.to_string())),
        "start_date" => resource.start_date.map(QueryValue::Date),
        "end_date" => resource.end_date.map(QueryValue::Date),
        "time_off_balance" => Some(QueryValue::Number(resource.time_off_balance as f64)),
        "is_active" => Some(QueryValue::Boolean(status == "Available" || status == "Assigned")),
        "has_email" => Some(QueryValue::Boolean(resource.email.is_some())),
        "vacation_count" => Some(QueryValue::Number(
            resource.vacations.as_ref().map(|v| v.len()).unwrap_or(0) as f64,
        )),
        "time_off_count" => Some(QueryValue::Number(
            resource.time_off_history.as_ref().map(|h| h.len()).unwrap_or(0) as f64,
        )),
        _ => None,
    }
}
