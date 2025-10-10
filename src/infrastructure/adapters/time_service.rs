//! Time service adapter implementation
//!
//! This module provides a concrete implementation of the TimeServicePort
//! using chrono for time operations.

use crate::domain::ports::time_service::TimeServicePort;
use crate::domain::shared::errors::{DomainError, DomainResult};
use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, Utc, Weekday};

/// Standard time service adapter
pub struct StandardTimeServiceAdapter {
    timezone: String,
}

impl StandardTimeServiceAdapter {
    pub fn new() -> Self {
        Self {
            timezone: "UTC".to_string(),
        }
    }

    pub fn with_timezone(timezone: String) -> Self {
        Self { timezone }
    }
}

impl Default for StandardTimeServiceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeServicePort for StandardTimeServiceAdapter {
    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn now_local(&self) -> DateTime<chrono::Local> {
        Local::now()
    }

    fn today(&self) -> NaiveDate {
        Utc::now().date_naive()
    }

    fn parse_date(&self, date_str: &str, format: &str) -> DomainResult<NaiveDate> {
        NaiveDate::parse_from_str(date_str, format).map_err(|e| DomainError::ValidationError {
            field: "date".to_string(),
            message: format!("Failed to parse date '{}': {}", date_str, e),
        })
    }

    fn parse_datetime(&self, datetime_str: &str, format: &str) -> DomainResult<NaiveDateTime> {
        NaiveDateTime::parse_from_str(datetime_str, format).map_err(|e| DomainError::ValidationError {
            field: "datetime".to_string(),
            message: format!("Failed to parse datetime '{}': {}", datetime_str, e),
        })
    }

    fn format_date(&self, date: &NaiveDate, format: &str) -> String {
        date.format(format).to_string()
    }

    fn format_datetime(&self, datetime: &NaiveDateTime, format: &str) -> String {
        datetime.format(format).to_string()
    }

    fn get_timezone(&self) -> String {
        self.timezone.clone()
    }

    fn set_timezone(&self, _timezone: &str) -> DomainResult<()> {
        // In a real implementation, this would validate the timezone
        // For now, we'll just accept any string
        Ok(())
    }

    fn is_business_day(&self, date: &NaiveDate) -> bool {
        let weekday = date.weekday();
        weekday != Weekday::Sat && weekday != Weekday::Sun
    }

    fn next_business_day(&self, date: &NaiveDate) -> NaiveDate {
        let mut next = *date;
        loop {
            next = next.succ_opt().unwrap_or(next);
            if self.is_business_day(&next) {
                break;
            }
        }
        next
    }

    fn previous_business_day(&self, date: &NaiveDate) -> NaiveDate {
        let mut prev = *date;
        loop {
            prev = prev.pred_opt().unwrap_or(prev);
            if self.is_business_day(&prev) {
                break;
            }
        }
        prev
    }

    fn date_diff(&self, start: &NaiveDate, end: &NaiveDate) -> i64 {
        end.signed_duration_since(*start).num_days()
    }

    fn add_days(&self, date: &NaiveDate, days: i64) -> NaiveDate {
        *date + chrono::Duration::days(days)
    }

    fn subtract_days(&self, date: &NaiveDate, days: i64) -> NaiveDate {
        *date - chrono::Duration::days(days)
    }
}
