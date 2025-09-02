#![allow(dead_code)]

pub(crate) use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub id: Option<String>,
    pub manager_name: String,
    pub manager_email: String,
    pub default_timezone: String,
    pub company_name: Option<String>,
    pub work_hours_start: Option<String>,
    pub work_hours_end: Option<String>,
    pub work_days: Vec<WorkDay>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl WorkDay {
    pub fn from_str(day: &str) -> Option<Self> {
        match day.to_lowercase().as_str() {
            "monday" | "segunda" | "seg" => Some(WorkDay::Monday),
            "tuesday" | "terça" | "ter" => Some(WorkDay::Tuesday),
            "wednesday" | "quarta" | "qua" => Some(WorkDay::Wednesday),
            "thursday" | "quinta" | "qui" => Some(WorkDay::Thursday),
            "friday" | "sexta" | "sex" => Some(WorkDay::Friday),
            "saturday" | "sábado" | "sab" => Some(WorkDay::Saturday),
            "sunday" | "domingo" | "dom" => Some(WorkDay::Sunday),
            _ => None,
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            WorkDay::Monday => "Monday".to_string(),
            WorkDay::Tuesday => "Tuesday".to_string(),
            WorkDay::Wednesday => "Wednesday".to_string(),
            WorkDay::Thursday => "Thursday".to_string(),
            WorkDay::Friday => "Friday".to_string(),
            WorkDay::Saturday => "Saturday".to_string(),
            WorkDay::Sunday => "Sunday".to_string(),
        }
    }
}

impl Config {
    pub fn new(manager_name: String, manager_email: String, default_timezone: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: None,
            manager_name,
            manager_email,
            default_timezone,
            company_name: None,
            work_hours_start: None,
            work_hours_end: None,
            work_days: vec![
                WorkDay::Monday,
                WorkDay::Tuesday,
                WorkDay::Wednesday,
                WorkDay::Thursday,
                WorkDay::Friday,
            ],
            created_at: Some(now),
            updated_at: Some(now),
        }
    }

    pub fn with_company_name(mut self, company_name: String) -> Self {
        self.company_name = Some(company_name);
        self
    }

    pub fn with_work_hours(mut self, start: String, end: String) -> Self {
        self.work_hours_start = Some(start);
        self.work_hours_end = Some(end);
        self
    }

    pub fn with_work_days(mut self, work_days: Vec<WorkDay>) -> Self {
        self.work_days = work_days;
        self
    }

    pub fn update_work_days(&mut self, work_days: Vec<WorkDay>) {
        self.work_days = work_days;
        self.updated_at = Some(chrono::Utc::now());
    }

    pub fn is_work_day(&self, day: &WorkDay) -> bool {
        self.work_days.contains(day)
    }

    /// Checks if a given time is within work hours
    pub fn is_work_hours(&self, time: &str) -> bool {
        if let (Some(start), Some(end)) = (&self.work_hours_start, &self.work_hours_end) {
            time >= start.as_str() && time <= end.as_str()
        } else {
            true // Se não há horário definido, considera sempre horário de trabalho
        }
    }

    /// Updates the company name
    pub fn update_company_name(&mut self, company_name: String) {
        self.company_name = Some(company_name);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Updates the manager information
    pub fn update_manager(&mut self, name: String, email: String) {
        self.manager_name = name;
        self.manager_email = email;
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Updates the default timezone
    pub fn update_timezone(&mut self, timezone: String) {
        self.default_timezone = timezone;
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Updates work hours
    pub fn update_work_hours(&mut self, start: String, end: String) {
        self.work_hours_start = Some(start);
        self.work_hours_end = Some(end);
        self.updated_at = Some(chrono::Utc::now());
    }

    /// Validates if the configuration is complete and valid
    pub fn is_valid(&self) -> bool {
        !self.manager_name.trim().is_empty()
            && !self.manager_email.trim().is_empty()
            && !self.default_timezone.trim().is_empty()
            && !self.work_days.is_empty()
    }

    /// Gets the current work schedule as a formatted string
    pub fn work_schedule_display(&self) -> String {
        let days = self
            .work_days
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let hours = if let (Some(start), Some(end)) = (&self.work_hours_start, &self.work_hours_end) {
            format!("{} - {}", start, end)
        } else {
            "Not configured".to_string()
        };

        format!("Days: {} | Hours: {}", days, hours)
    }

    /// Checks if a given timezone is valid
    pub fn is_valid_timezone(&self) -> bool {
        // Basic timezone validation - can be enhanced with chrono-tz
        let valid_timezones = [
            "UTC",
            "GMT",
            "EST",
            "PST",
            "CST",
            "MST",
            "America/New_York",
            "America/Los_Angeles",
            "America/Chicago",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Asia/Dubai",
            "America/Sao_Paulo",
            "America/Argentina/Buenos_Aires",
        ];

        valid_timezones.contains(&self.default_timezone.as_str())
    }

    /// Gets the company display name
    pub fn display_name(&self) -> String {
        self.company_name
            .clone()
            .unwrap_or_else(|| "Unnamed Company".to_string())
    }

    /// Creates a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "Company: {} | Manager: {} | Timezone: {} | Work Schedule: {}",
            self.display_name(),
            self.manager_name,
            self.default_timezone,
            self.work_schedule_display()
        )
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ name: {}, email: {}, timezone: {} }}",
            self.manager_name, self.manager_email, self.default_timezone
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_display() {
        let config = Config {
            id: None,
            manager_name: "Admin User".to_string(),
            manager_email: "admin@example.com".to_string(),
            default_timezone: "UTC".to_string(),
            company_name: None,
            work_hours_start: None,
            work_hours_end: None,
            work_days: vec![],
            created_at: None,
            updated_at: None,
        };
        let expected = "Config { name: Admin User, email: admin@example.com, timezone: UTC }";
        assert_eq!(config.to_string(), expected);
    }

    #[test]
    fn test_work_day_from_str() {
        assert_eq!(WorkDay::from_str("monday"), Some(WorkDay::Monday));
        assert_eq!(WorkDay::from_str("segunda"), Some(WorkDay::Monday));
        assert_eq!(WorkDay::from_str("seg"), Some(WorkDay::Monday));
        assert_eq!(WorkDay::from_str("invalid"), None);
    }

    #[test]
    fn test_work_day_to_string() {
        assert_eq!(WorkDay::Monday.to_string(), "Monday");
        assert_eq!(WorkDay::Friday.to_string(), "Friday");
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = Config::new(
            "John Doe".to_string(),
            "john@company.com".to_string(),
            "America/Sao_Paulo".to_string(),
        )
        .with_company_name("Tech Corp".to_string())
        .with_work_hours("09:00".to_string(), "18:00".to_string())
        .with_work_days(vec![WorkDay::Monday, WorkDay::Tuesday, WorkDay::Wednesday]);

        assert_eq!(config.company_name, Some("Tech Corp".to_string()));
        assert_eq!(config.work_hours_start, Some("09:00".to_string()));
        assert_eq!(config.work_hours_end, Some("18:00".to_string()));
        assert_eq!(config.work_days.len(), 3);
    }

    #[test]
    fn test_config_work_day_checks() {
        let mut config = Config::new("Admin".to_string(), "admin@company.com".to_string(), "UTC".to_string());

        assert!(config.is_work_day(&WorkDay::Monday));
        assert!(!config.is_work_day(&WorkDay::Sunday));

        config.update_work_days(vec![WorkDay::Monday, WorkDay::Tuesday]);
        assert!(config.is_work_day(&WorkDay::Monday));
        assert!(!config.is_work_day(&WorkDay::Wednesday));
    }

    #[test]
    fn test_config_work_hours_checks() {
        let config = Config::new("Admin".to_string(), "admin@company.com".to_string(), "UTC".to_string())
            .with_work_hours("09:00".to_string(), "18:00".to_string());

        assert!(config.is_work_hours("10:00"));
        assert!(config.is_work_hours("18:00"));
        assert!(!config.is_work_hours("20:00"));
    }
}
