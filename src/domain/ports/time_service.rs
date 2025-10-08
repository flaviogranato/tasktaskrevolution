//! Time service port for domain time operations
//!
//! This module defines the time interface that the domain layer
//! requires from the infrastructure layer.

use crate::domain::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

/// Time service port for time operations
pub trait TimeServicePort: Send + Sync {
    /// Get the current UTC time
    fn now_utc(&self) -> DateTime<Utc>;

    /// Get the current local time
    fn now_local(&self) -> DateTime<chrono::Local>;

    /// Get the current date
    fn today(&self) -> NaiveDate;

    /// Parse a date string
    fn parse_date(&self, date_str: &str, format: &str) -> DomainResult<NaiveDate>;

    /// Parse a datetime string
    fn parse_datetime(&self, datetime_str: &str, format: &str) -> DomainResult<NaiveDateTime>;

    /// Format a date
    fn format_date(&self, date: &NaiveDate, format: &str) -> String;

    /// Format a datetime
    fn format_datetime(&self, datetime: &NaiveDateTime, format: &str) -> String;

    /// Get timezone information
    fn get_timezone(&self) -> String;

    /// Set timezone
    fn set_timezone(&self, timezone: &str) -> DomainResult<()>;

    /// Check if a date is a business day
    fn is_business_day(&self, date: &NaiveDate) -> bool;

    /// Get the next business day
    fn next_business_day(&self, date: &NaiveDate) -> NaiveDate;

    /// Get the previous business day
    fn previous_business_day(&self, date: &NaiveDate) -> NaiveDate;

    /// Calculate the difference between two dates
    fn date_diff(&self, start: &NaiveDate, end: &NaiveDate) -> i64;

    /// Add days to a date
    fn add_days(&self, date: &NaiveDate, days: i64) -> NaiveDate;

    /// Subtract days from a date
    fn subtract_days(&self, date: &NaiveDate, days: i64) -> NaiveDate;
}
