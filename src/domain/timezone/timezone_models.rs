use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid7::Uuid;

/// Represents a timezone with complete information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneInfo {
    pub id: String,
    pub name: String,
    pub offset_seconds: i32,
    pub is_dst: bool,
    pub abbreviation: String,
    pub country: Option<String>,
    pub region: Option<String>,
}

impl TimezoneInfo {
    pub fn new(id: String, name: String, offset_seconds: i32, is_dst: bool, abbreviation: String) -> Self {
        Self {
            id,
            name,
            offset_seconds,
            is_dst,
            abbreviation,
            country: None,
            region: None,
        }
    }

    pub fn with_location(mut self, country: String, region: String) -> Self {
        self.country = Some(country);
        self.region = Some(region);
        self
    }

    /// Converte um DateTime<Utc> para este timezone
    pub fn convert_from_utc<T: TimeZone>(&self, utc_time: DateTime<Utc>) -> Result<DateTime<Tz>, String> {
        let tz: Tz = self.id.parse().map_err(|_| format!("Invalid timezone: {}", self.id))?;

        Ok(utc_time.with_timezone(&tz))
    }

    /// Converte um DateTime deste timezone para UTC
    pub fn convert_to_utc<T: TimeZone>(&self, local_time: DateTime<Tz>) -> DateTime<Utc> {
        local_time.with_timezone(&Utc)
    }
}

/// Represents a timezone conversion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneConversion {
    pub id: Uuid,
    pub source_timezone: String,
    pub target_timezone: String,
    pub source_time: DateTime<Utc>,
    pub target_time: DateTime<Utc>,
    pub offset_difference: i32,
    pub created_at: DateTime<Utc>,
}

impl TimezoneConversion {
    pub fn new(
        source_timezone: String,
        target_timezone: String,
        source_time: DateTime<Utc>,
        target_time: DateTime<Utc>,
    ) -> Self {
        let offset_difference = target_time.timestamp() - source_time.timestamp();

        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ),
            source_timezone,
            target_timezone,
            source_time,
            target_time,
            offset_difference: offset_difference as i32,
            created_at: Utc::now(),
        }
    }
}

/// Represents timezone preferences of a user
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezonePreferences {
    pub user_id: String,
    pub default_timezone: String,
    pub preferred_timezones: Vec<String>,
    pub auto_detect: bool,
    pub display_format: TimeDisplayFormat,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimezonePreferences {
    pub fn new(user_id: String, default_timezone: String) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            default_timezone: default_timezone.clone(),
            preferred_timezones: vec![default_timezone],
            auto_detect: true,
            display_format: TimeDisplayFormat::default(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_preferred_timezone(&mut self, timezone: String) {
        if !self.preferred_timezones.contains(&timezone) {
            self.preferred_timezones.push(timezone);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_preferred_timezone(&mut self, timezone: &str) {
        self.preferred_timezones.retain(|tz| tz != timezone);
        self.updated_at = Utc::now();
    }
}

/// Time display format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeDisplayFormat {
    /// 24-hour format (HH:MM:SS)
    TwentyFourHour,
    /// 12-hour format (h:MM:SS AM/PM)
    TwelveHour,
    /// ISO 8601 format
    Iso8601,
    /// Custom format
    Custom(String),
}

impl Default for TimeDisplayFormat {
    fn default() -> Self {
        Self::TwentyFourHour
    }
}

impl TimeDisplayFormat {
    pub fn format_time(&self, time: DateTime<Utc>) -> String {
        match self {
            TimeDisplayFormat::TwentyFourHour => time.format("%H:%M:%S").to_string(),
            TimeDisplayFormat::TwelveHour => time.format("%I:%M:%S %p").to_string(),
            TimeDisplayFormat::Iso8601 => time.to_rfc3339(),
            TimeDisplayFormat::Custom(format) => time.format(format).to_string(),
        }
    }
}

/// Representa um cronograma global
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSchedule {
    pub id: Uuid,
    pub project_id: String,
    pub timezone: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub participants: Vec<GlobalParticipant>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GlobalSchedule {
    pub fn new(project_id: String, timezone: String, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::from_fields_v7(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                0,
                0,
            ),
            project_id,
            timezone,
            start_time,
            end_time,
            participants: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_participant(&mut self, participant: GlobalParticipant) {
        self.participants.push(participant);
        self.updated_at = Utc::now();
    }

    pub fn get_duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    pub fn get_duration(&self) -> chrono::Duration {
        self.end_time - self.start_time
    }
}

/// Representa um participante global
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalParticipant {
    pub user_id: String,
    pub name: String,
    pub timezone: String,
    pub local_start_time: DateTime<Utc>,
    pub local_end_time: DateTime<Utc>,
    pub is_required: bool,
}

impl GlobalParticipant {
    pub fn new(
        user_id: String,
        name: String,
        timezone: String,
        local_start_time: DateTime<Utc>,
        local_end_time: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            name,
            timezone,
            local_start_time,
            local_end_time,
            is_required: true,
        }
    }

    pub fn optional(mut self) -> Self {
        self.is_required = false;
        self
    }
}

/// Represents global metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalMetrics {
    pub total_projects: usize,
    pub active_timezones: Vec<String>,
    pub timezone_distribution: HashMap<String, usize>,
    pub average_meeting_duration_minutes: f64,
    pub most_active_timezone: Option<String>,
    pub generated_at: DateTime<Utc>,
}

impl GlobalMetrics {
    pub fn new() -> Self {
        Self {
            total_projects: 0,
            active_timezones: Vec::new(),
            timezone_distribution: HashMap::new(),
            average_meeting_duration_minutes: 0.0,
            most_active_timezone: None,
            generated_at: Utc::now(),
        }
    }

    pub fn calculate_most_active_timezone(&mut self) {
        self.most_active_timezone = self
            .timezone_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(timezone, _)| timezone.clone());
    }
}

impl Default for GlobalMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Erro relacionado a timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimezoneError {
    InvalidTimezone(String),
    ConversionFailed(String),
    TimezoneNotFound(String),
    InvalidTimeFormat(String),
    UnsupportedOperation(String),
}

impl std::fmt::Display for TimezoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimezoneError::InvalidTimezone(msg) => write!(f, "Invalid timezone: {}", msg),
            TimezoneError::ConversionFailed(msg) => write!(f, "Conversion failed: {}", msg),
            TimezoneError::TimezoneNotFound(msg) => write!(f, "Timezone not found: {}", msg),
            TimezoneError::InvalidTimeFormat(msg) => write!(f, "Invalid time format: {}", msg),
            TimezoneError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
        }
    }
}

impl std::error::Error for TimezoneError {}

pub type TimezoneResult<T> = Result<T, TimezoneError>;
