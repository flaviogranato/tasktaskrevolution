use super::timezone_models::*;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gerenciador de timezones
pub struct TimezoneManager {
    timezones: HashMap<String, TimezoneInfo>,
    default_timezone: String,
}

impl TimezoneManager {
    pub fn new() -> Self {
        let mut manager = Self {
            timezones: HashMap::new(),
            default_timezone: "UTC".to_string(),
        };

        // Inicializar com timezones comuns
        manager.initialize_common_timezones();
        manager
    }

    pub fn with_default_timezone(mut self, timezone: String) -> Self {
        self.default_timezone = timezone;
        self
    }

    /// Inicializa timezones comuns
    fn initialize_common_timezones(&mut self) {
        let common_timezones = vec![
            ("UTC", "UTC", 0, false, "UTC", None, None),
            (
                "America/New_York",
                "Eastern Time",
                -18000,
                false,
                "EST",
                Some("US"),
                Some("Eastern"),
            ),
            (
                "America/Chicago",
                "Central Time",
                -21600,
                false,
                "CST",
                Some("US"),
                Some("Central"),
            ),
            (
                "America/Denver",
                "Mountain Time",
                -25200,
                false,
                "MST",
                Some("US"),
                Some("Mountain"),
            ),
            (
                "America/Los_Angeles",
                "Pacific Time",
                -28800,
                false,
                "PST",
                Some("US"),
                Some("Pacific"),
            ),
            (
                "Europe/London",
                "London Time",
                0,
                false,
                "GMT",
                Some("UK"),
                Some("London"),
            ),
            (
                "Europe/Paris",
                "Paris Time",
                3600,
                false,
                "CET",
                Some("FR"),
                Some("Paris"),
            ),
            (
                "Europe/Berlin",
                "Berlin Time",
                3600,
                false,
                "CET",
                Some("DE"),
                Some("Berlin"),
            ),
            (
                "Asia/Tokyo",
                "Tokyo Time",
                32400,
                false,
                "JST",
                Some("JP"),
                Some("Tokyo"),
            ),
            (
                "Asia/Shanghai",
                "Shanghai Time",
                28800,
                false,
                "CST",
                Some("CN"),
                Some("Shanghai"),
            ),
            (
                "Asia/Kolkata",
                "India Time",
                19800,
                false,
                "IST",
                Some("IN"),
                Some("India"),
            ),
            (
                "Australia/Sydney",
                "Sydney Time",
                39600,
                false,
                "AEST",
                Some("AU"),
                Some("Sydney"),
            ),
        ];

        for (id, name, offset, is_dst, abbrev, country, region) in common_timezones {
            let mut timezone = TimezoneInfo::new(id.to_string(), name.to_string(), offset, is_dst, abbrev.to_string());

            if let (Some(country), Some(region)) = (country, region) {
                timezone = timezone.with_location(country.to_string(), region.to_string());
            }

            self.timezones.insert(id.to_string(), timezone);
        }
    }

    /// Adiciona um novo timezone
    pub fn add_timezone(&mut self, timezone: TimezoneInfo) -> TimezoneResult<()> {
        if self.timezones.contains_key(&timezone.id) {
            return Err(TimezoneError::InvalidTimezone(format!(
                "Timezone {} already exists",
                timezone.id
            )));
        }

        // Validar se o timezone é válido
        if timezone.id.parse::<Tz>().is_err() {
            return Err(TimezoneError::InvalidTimezone(format!(
                "Invalid timezone ID: {}",
                timezone.id
            )));
        }

        self.timezones.insert(timezone.id.clone(), timezone);
        Ok(())
    }

    /// Obtém informações de um timezone
    pub fn get_timezone(&self, id: &str) -> TimezoneResult<&TimezoneInfo> {
        self.timezones
            .get(id)
            .ok_or_else(|| TimezoneError::TimezoneNotFound(id.to_string()))
    }

    /// Lista todos os timezones disponíveis
    pub fn list_timezones(&self) -> Vec<&TimezoneInfo> {
        self.timezones.values().collect()
    }

    /// Busca timezones por país
    pub fn find_by_country(&self, country: &str) -> Vec<&TimezoneInfo> {
        self.timezones
            .values()
            .filter(|tz| tz.country.as_ref().map_or(false, |c| c == country))
            .collect()
    }

    /// Busca timezones por região
    pub fn find_by_region(&self, region: &str) -> Vec<&TimezoneInfo> {
        self.timezones
            .values()
            .filter(|tz| tz.region.as_ref().map_or(false, |r| r == region))
            .collect()
    }

    /// Obtém o timezone padrão
    pub fn get_default_timezone(&self) -> &str {
        &self.default_timezone
    }

    /// Define o timezone padrão
    pub fn set_default_timezone(&mut self, timezone: String) -> TimezoneResult<()> {
        if !self.timezones.contains_key(&timezone) {
            return Err(TimezoneError::TimezoneNotFound(timezone));
        }
        self.default_timezone = timezone;
        Ok(())
    }

    /// Detecta o timezone do sistema
    pub fn detect_system_timezone(&self) -> TimezoneResult<String> {
        // Em um ambiente real, isso usaria bibliotecas do sistema
        // Por enquanto, retornamos UTC como fallback
        Ok("UTC".to_string())
    }

    /// Valida se um timezone é válido
    pub fn validate_timezone(&self, timezone_id: &str) -> bool {
        timezone_id.parse::<Tz>().is_ok()
    }

    /// Obtém timezones próximos a um offset específico
    pub fn find_by_offset(&self, offset_seconds: i32, tolerance: i32) -> Vec<&TimezoneInfo> {
        self.timezones
            .values()
            .filter(|tz| (tz.offset_seconds - offset_seconds).abs() <= tolerance)
            .collect()
    }

    /// Obtém estatísticas dos timezones
    pub fn get_timezone_stats(&self) -> TimezoneStats {
        let total = self.timezones.len();
        let countries: std::collections::HashSet<_> =
            self.timezones.values().filter_map(|tz| tz.country.as_ref()).collect();
        let regions: std::collections::HashSet<_> =
            self.timezones.values().filter_map(|tz| tz.region.as_ref()).collect();

        TimezoneStats {
            total_timezones: total,
            unique_countries: countries.len(),
            unique_regions: regions.len(),
            default_timezone: self.default_timezone.clone(),
        }
    }
}

/// Estatísticas dos timezones
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneStats {
    pub total_timezones: usize,
    pub unique_countries: usize,
    pub unique_regions: usize,
    pub default_timezone: String,
}

impl Default for TimezoneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_manager_creation() {
        let manager = TimezoneManager::new();
        assert!(!manager.timezones.is_empty());
        assert_eq!(manager.default_timezone, "UTC");
    }

    #[test]
    fn test_get_timezone() {
        let manager = TimezoneManager::new();
        let utc = manager.get_timezone("UTC").unwrap();
        assert_eq!(utc.id, "UTC");
        assert_eq!(utc.name, "UTC");
    }

    #[test]
    fn test_get_nonexistent_timezone() {
        let manager = TimezoneManager::new();
        let result = manager.get_timezone("Nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_by_country() {
        let manager = TimezoneManager::new();
        let us_timezones = manager.find_by_country("US");
        assert!(!us_timezones.is_empty());
    }

    #[test]
    fn test_validate_timezone() {
        let manager = TimezoneManager::new();
        assert!(manager.validate_timezone("UTC"));
        assert!(manager.validate_timezone("America/New_York"));
        assert!(!manager.validate_timezone("Invalid/Timezone"));
    }

    #[test]
    fn test_timezone_stats() {
        let manager = TimezoneManager::new();
        let stats = manager.get_timezone_stats();
        assert!(stats.total_timezones > 0);
        assert_eq!(stats.default_timezone, "UTC");
    }
}
