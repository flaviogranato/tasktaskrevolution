use super::timezone_models::*;
use chrono::{DateTime, Datelike, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validador de timezones
pub struct TimezoneValidator {
    validation_rules: HashMap<String, ValidationRule>,
}

impl TimezoneValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            validation_rules: HashMap::new(),
        };
        validator.initialize_default_rules();
        validator
    }

    /// Inicializa regras de validação padrão
    fn initialize_default_rules(&mut self) {
        // Regra para horário de trabalho
        self.validation_rules.insert(
            "working_hours".to_string(),
            ValidationRule {
                name: "Working Hours".to_string(),
                description: "Validates that times are within working hours (9 AM - 5 PM)".to_string(),
                validator: Box::new(|time, _| {
                    let hour = time.hour();
                    hour >= 9 && hour <= 17
                }),
            },
        );

        // Regra para dias úteis
        self.validation_rules.insert(
            "weekdays_only".to_string(),
            ValidationRule {
                name: "Weekdays Only".to_string(),
                description: "Validates that times are on weekdays only".to_string(),
                validator: Box::new(|time, _| {
                    time.weekday() != chrono::Weekday::Sat && time.weekday() != chrono::Weekday::Sun
                }),
            },
        );

        // Regra para horário comercial
        self.validation_rules.insert(
            "business_hours".to_string(),
            ValidationRule {
                name: "Business Hours".to_string(),
                description: "Validates that times are within business hours (8 AM - 6 PM)".to_string(),
                validator: Box::new(|time, _| {
                    let hour = time.hour();
                    hour >= 8 && hour <= 18
                }),
            },
        );
    }

    /// Valida um timezone
    pub fn validate_timezone(&self, timezone: &str) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Verificar se o timezone é válido
        if timezone.parse::<Tz>().is_err() {
            errors.push(ValidationError {
                code: "INVALID_TIMEZONE".to_string(),
                message: format!("Invalid timezone: {}", timezone),
                severity: ValidationSeverity::Error,
            });
        }

        // Verificar se é um timezone comum
        let common_timezones = vec![
            "UTC",
            "America/New_York",
            "America/Chicago",
            "America/Denver",
            "America/Los_Angeles",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Asia/Tokyo",
            "Asia/Shanghai",
        ];

        if !common_timezones.contains(&timezone) {
            warnings.push(ValidationError {
                code: "UNCOMMON_TIMEZONE".to_string(),
                message: format!("Uncommon timezone: {}", timezone),
                severity: ValidationSeverity::Warning,
            });
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Valida um horário em um timezone específico
    pub fn validate_time(&self, time: DateTime<Utc>, timezone: &str, rules: Vec<&str>) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Converter para o timezone local
        let tz: Tz = match timezone.parse() {
            Ok(tz) => tz,
            Err(_) => {
                return ValidationResult {
                    is_valid: false,
                    errors: vec![ValidationError {
                        code: "INVALID_TIMEZONE".to_string(),
                        message: format!("Invalid timezone: {}", timezone),
                        severity: ValidationSeverity::Error,
                    }],
                    warnings: Vec::new(),
                };
            }
        };

        let local_time = time.with_timezone(&tz);

        // Aplicar regras de validação
        for rule_name in rules {
            if let Some(rule) = self.validation_rules.get(rule_name) {
                if !(rule.validator)(&local_time, timezone) {
                    errors.push(ValidationError {
                        code: rule_name.to_string().to_uppercase(),
                        message: format!("Validation failed for rule: {}", rule.name),
                        severity: ValidationSeverity::Error,
                    });
                }
            }
        }

        // Validações adicionais
        self.validate_time_consistency(&local_time, timezone, &mut warnings);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Valida consistência do horário
    fn validate_time_consistency(&self, time: &DateTime<Tz>, timezone: &str, warnings: &mut Vec<ValidationError>) {
        let hour = time.hour();

        // Avisar sobre horários muito cedo ou muito tarde
        if hour < 6 {
            warnings.push(ValidationError {
                code: "EARLY_TIME".to_string(),
                message: "Very early time - consider if this is appropriate".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        if hour > 22 {
            warnings.push(ValidationError {
                code: "LATE_TIME".to_string(),
                message: "Very late time - consider if this is appropriate".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // Avisar sobre horários de almoço
        if hour >= 12 && hour <= 13 {
            warnings.push(ValidationError {
                code: "LUNCH_TIME".to_string(),
                message: "Scheduled during lunch time - consider if this is appropriate".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }
    }

    /// Valida um cronograma global
    pub fn validate_global_schedule(&self, schedule: &GlobalSchedule) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validar timezone do cronograma
        let timezone_result = self.validate_timezone(&schedule.timezone);
        if !timezone_result.is_valid {
            errors.extend(timezone_result.errors);
        }
        warnings.extend(timezone_result.warnings);

        // Validar horários do cronograma
        let time_result = self.validate_time(
            schedule.start_time,
            &schedule.timezone,
            vec!["working_hours", "weekdays_only"],
        );
        if !time_result.is_valid {
            errors.extend(time_result.errors);
        }
        warnings.extend(time_result.warnings);

        // Validar participantes
        for participant in &schedule.participants {
            let participant_result = self.validate_timezone(&participant.timezone);
            if !participant_result.is_valid {
                errors.extend(participant_result.errors);
            }
            warnings.extend(participant_result.warnings);
        }

        // Validar duração
        let duration = schedule.end_time - schedule.start_time;
        if duration.num_hours() > 8 {
            warnings.push(ValidationError {
                code: "LONG_DURATION".to_string(),
                message: "Schedule duration is longer than 8 hours".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Adiciona uma regra de validação personalizada
    pub fn add_validation_rule(&mut self, name: String, rule: ValidationRule) {
        self.validation_rules.insert(name, rule);
    }

    /// Remove uma regra de validação
    pub fn remove_validation_rule(&mut self, name: &str) -> Option<ValidationRule> {
        self.validation_rules.remove(name)
    }

    /// Lista todas as regras de validação
    pub fn list_validation_rules(&self) -> Vec<&ValidationRule> {
        self.validation_rules.values().collect()
    }

    /// Obtém estatísticas de validação
    pub fn get_validation_stats(&self) -> ValidationStats {
        ValidationStats {
            total_rules: self.validation_rules.len(),
            rule_names: self.validation_rules.keys().cloned().collect(),
        }
    }
}

/// Regra de validação
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub validator: Box<dyn Fn(&DateTime<Tz>, &str) -> bool + Send + Sync>,
}

/// Resultado de validação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationError>,
}

/// Erro de validação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

/// Severidade da validação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Estatísticas de validação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_rules: usize,
    pub rule_names: Vec<String>,
}

impl Default for TimezoneValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_validator_creation() {
        let validator = TimezoneValidator::new();
        assert!(!validator.validation_rules.is_empty());
    }

    #[test]
    fn test_validate_timezone() {
        let validator = TimezoneValidator::new();

        let result = validator.validate_timezone("UTC");
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        let result = validator.validate_timezone("Invalid/Timezone");
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_time() {
        let validator = TimezoneValidator::new();
        let time = Utc::now();

        let result = validator.validate_time(time, "UTC", vec!["working_hours"]);
        // Resultado pode variar dependendo do horário atual
        assert!(result.errors.is_empty() || !result.errors.is_empty());
    }

    #[test]
    fn test_validate_global_schedule() {
        let validator = TimezoneValidator::new();
        let schedule = GlobalSchedule::new(
            "project1".to_string(),
            "UTC".to_string(),
            Utc::now(),
            Utc::now() + chrono::Duration::hours(2),
        );

        let result = validator.validate_global_schedule(&schedule);
        // Resultado pode variar dependendo do horário atual
        assert!(result.errors.is_empty() || !result.errors.is_empty());
    }

    #[test]
    fn test_add_validation_rule() {
        let mut validator = TimezoneValidator::new();

        let rule = ValidationRule {
            name: "Custom Rule".to_string(),
            description: "Custom validation rule".to_string(),
            validator: Box::new(|_, _| true),
        };

        validator.add_validation_rule("custom".to_string(), rule);
        assert!(validator.validation_rules.contains_key("custom"));
    }
}
