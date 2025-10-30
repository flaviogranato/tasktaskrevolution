use super::timezone_models::*;
use super::timezone_validator::{ValidationError, ValidationResult, ValidationSeverity};
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuração de timezone para um projeto
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneConfig {
    pub project_id: String,
    pub default_timezone: String,
    pub allowed_timezones: Vec<String>,
    pub working_hours: WorkingHours,
    pub timezone_rules: Vec<TimezoneRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimezoneConfig {
    pub fn new(project_id: String, default_timezone: String) -> Self {
        let now = Utc::now();
        Self {
            project_id,
            default_timezone: default_timezone.clone(),
            allowed_timezones: vec![default_timezone],
            working_hours: WorkingHours::default(),
            timezone_rules: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_allowed_timezone(&mut self, timezone: String) {
        if !self.allowed_timezones.contains(&timezone) {
            self.allowed_timezones.push(timezone);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_allowed_timezone(&mut self, timezone: &str) {
        self.allowed_timezones.retain(|tz| tz != timezone);
        self.updated_at = Utc::now();
    }

    pub fn add_timezone_rule(&mut self, rule: TimezoneRule) {
        self.timezone_rules.push(rule);
        self.updated_at = Utc::now();
    }

    pub fn is_timezone_allowed(&self, timezone: &str) -> bool {
        self.allowed_timezones.contains(&timezone.to_string())
    }

    pub fn validate_timezone(&self, timezone: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let warnings = Vec::new();

        if !self.is_timezone_allowed(timezone) {
            errors.push(ValidationError {
                code: "TIMEZONE_NOT_ALLOWED".to_string(),
                message: format!("Timezone {} is not allowed for this project", timezone),
                severity: ValidationSeverity::Error,
            });
        }

        // Aplicar regras de timezone
        for rule in &self.timezone_rules {
            if !rule.is_applicable(timezone) {
                continue;
            }

            match rule.validate(timezone) {
                Ok(()) => {}
                Err(rule_errors) => {
                    errors.extend(rule_errors);
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Horário de trabalho
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkingHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub timezone: String,
    pub working_days: Vec<WorkingDay>,
    pub holidays: Vec<Holiday>,
}

impl WorkingHours {
    pub fn new(start_hour: u8, end_hour: u8, timezone: String) -> Self {
        Self {
            start_hour,
            end_hour,
            timezone,
            working_days: vec![
                WorkingDay::Monday,
                WorkingDay::Tuesday,
                WorkingDay::Wednesday,
                WorkingDay::Thursday,
                WorkingDay::Friday,
            ],
            holidays: Vec::new(),
        }
    }

    pub fn is_working_time(&self, time: DateTime<Utc>) -> bool {
        let tz: chrono_tz::Tz = match self.timezone.parse() {
            Ok(tz) => tz,
            Err(_) => return false,
        };

        let local_time = time.with_timezone(&tz);
        let hour = local_time.hour() as u8;
        let weekday = local_time.weekday();

        // Verificar se está no horário de trabalho
        if hour < self.start_hour || hour >= self.end_hour {
            return false;
        }

        // Verificar se é dia útil
        let is_working_day = self.working_days.iter().any(|day| match day {
            WorkingDay::Monday => weekday == chrono::Weekday::Mon,
            WorkingDay::Tuesday => weekday == chrono::Weekday::Tue,
            WorkingDay::Wednesday => weekday == chrono::Weekday::Wed,
            WorkingDay::Thursday => weekday == chrono::Weekday::Thu,
            WorkingDay::Friday => weekday == chrono::Weekday::Fri,
            WorkingDay::Saturday => weekday == chrono::Weekday::Sat,
            WorkingDay::Sunday => weekday == chrono::Weekday::Sun,
        });

        if !is_working_day {
            return false;
        }

        // Verificar se não é feriado
        let date = local_time.date_naive();
        !self.holidays.iter().any(|holiday| holiday.matches_date(date))
    }

    pub fn add_holiday(&mut self, holiday: Holiday) {
        self.holidays.push(holiday);
    }

    pub fn remove_holiday(&mut self, holiday_name: &str) {
        self.holidays.retain(|h| h.name != holiday_name);
    }
}

impl Default for WorkingHours {
    fn default() -> Self {
        Self::new(9, 17, "UTC".to_string())
    }
}

/// Dia da semana
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorkingDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Feriado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    pub name: String,
    pub date: chrono::NaiveDate,
    pub is_recurring: bool,
    pub description: Option<String>,
}

impl Holiday {
    pub fn new(name: String, date: chrono::NaiveDate, is_recurring: bool) -> Self {
        Self {
            name,
            date,
            is_recurring,
            description: None,
        }
    }

    pub fn matches_date(&self, date: chrono::NaiveDate) -> bool {
        if self.is_recurring {
            // Para feriados recorrentes, verificar mês e dia
            date.month() == self.date.month() && date.day() == self.date.day()
        } else {
            // Para feriados fixos, verificar data exata
            date == self.date
        }
    }
}

/// Regra de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneRule {
    pub name: String,
    pub description: String,
    pub timezone_pattern: String,
    pub rule_type: TimezoneRuleType,
    pub parameters: HashMap<String, String>,
    pub is_active: bool,
}

impl TimezoneRule {
    pub fn new(name: String, description: String, timezone_pattern: String, rule_type: TimezoneRuleType) -> Self {
        Self {
            name,
            description,
            timezone_pattern,
            rule_type,
            parameters: HashMap::new(),
            is_active: true,
        }
    }

    pub fn is_applicable(&self, timezone: &str) -> bool {
        if !self.is_active {
            return false;
        }

        // Verificar se o timezone corresponde ao padrão
        self.timezone_pattern == "*" || timezone.contains(&self.timezone_pattern)
    }

    pub fn validate(&self, timezone: &str) -> Result<(), Vec<ValidationError>> {
        if !self.is_applicable(timezone) {
            return Ok(());
        }

        let errors = Vec::new();

        match self.rule_type {
            TimezoneRuleType::WorkingHoursOnly => {
                // Implementar validação de horário de trabalho
                // Por enquanto, sempre válido
            }
            TimezoneRuleType::WeekdaysOnly => {
                // Implementar validação de dias úteis
                // Por enquanto, sempre válido
            }
            TimezoneRuleType::Custom => {
                // Implementar validação customizada baseada nos parâmetros
                // Por enquanto, sempre válido
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

/// Tipo de regra de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimezoneRuleType {
    WorkingHoursOnly,
    WeekdaysOnly,
    Custom,
}

/// Configuração global de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalTimezoneConfig {
    pub default_timezone: String,
    pub auto_detect_timezone: bool,
    pub timezone_display_format: TimeDisplayFormat,
    pub global_working_hours: WorkingHours,
    pub timezone_configs: HashMap<String, TimezoneConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GlobalTimezoneConfig {
    pub fn new(default_timezone: String) -> Self {
        let now = Utc::now();
        Self {
            default_timezone,
            auto_detect_timezone: true,
            timezone_display_format: TimeDisplayFormat::default(),
            global_working_hours: WorkingHours::default(),
            timezone_configs: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_project_config(&mut self, config: TimezoneConfig) {
        self.timezone_configs.insert(config.project_id.clone(), config);
        self.updated_at = Utc::now();
    }

    pub fn get_project_config(&self, project_id: &str) -> Option<&TimezoneConfig> {
        self.timezone_configs.get(project_id)
    }

    pub fn remove_project_config(&mut self, project_id: &str) -> Option<TimezoneConfig> {
        let result = self.timezone_configs.remove(project_id);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }

    pub fn update_global_settings(&mut self, settings: GlobalTimezoneSettings) {
        if let Some(timezone) = settings.default_timezone {
            self.default_timezone = timezone;
        }
        if let Some(auto_detect) = settings.auto_detect_timezone {
            self.auto_detect_timezone = auto_detect;
        }
        if let Some(format) = settings.timezone_display_format {
            self.timezone_display_format = format;
        }
        self.updated_at = Utc::now();
    }
}

/// Configurações globais de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalTimezoneSettings {
    pub default_timezone: Option<String>,
    pub auto_detect_timezone: Option<bool>,
    pub timezone_display_format: Option<TimeDisplayFormat>,
}

impl Default for GlobalTimezoneConfig {
    fn default() -> Self {
        Self::new("UTC".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_config_creation() {
        let config = TimezoneConfig::new("project1".to_string(), "UTC".to_string());
        assert_eq!(config.project_id, "project1");
        assert_eq!(config.default_timezone, "UTC");
    }

    #[test]
    fn test_add_allowed_timezone() {
        let mut config = TimezoneConfig::new("project1".to_string(), "UTC".to_string());
        config.add_allowed_timezone("America/New_York".to_string());
        assert!(config.is_timezone_allowed("America/New_York"));
    }

    #[test]
    fn test_working_hours_creation() {
        let working_hours = WorkingHours::new(9, 17, "UTC".to_string());
        assert_eq!(working_hours.start_hour, 9);
        assert_eq!(working_hours.end_hour, 17);
    }

    #[test]
    fn test_holiday_creation() {
        let holiday = Holiday::new(
            "New Year".to_string(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            true,
        );
        assert_eq!(holiday.name, "New Year");
        assert!(holiday.is_recurring);
    }

    #[test]
    fn test_global_timezone_config() {
        let global_config = GlobalTimezoneConfig::new("UTC".to_string());
        assert_eq!(global_config.default_timezone, "UTC");
        assert!(global_config.auto_detect_timezone);
    }
}
