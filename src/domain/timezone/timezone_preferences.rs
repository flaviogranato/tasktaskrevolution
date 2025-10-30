use super::timezone_models::*;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gerenciador de preferências de timezone
pub struct TimezonePreferencesManager {
    preferences: HashMap<String, TimezonePreferences>,
    default_preferences: TimezonePreferences,
}

impl TimezonePreferencesManager {
    pub fn new() -> Self {
        Self {
            preferences: HashMap::new(),
            default_preferences: TimezonePreferences::new("default".to_string(), "UTC".to_string()),
        }
    }

    /// Obtém preferências de um usuário
    pub fn get_user_preferences(&self, user_id: &str) -> &TimezonePreferences {
        self.preferences.get(user_id).unwrap_or(&self.default_preferences)
    }

    /// Define preferências de um usuário
    pub fn set_user_preferences(&mut self, user_id: String, preferences: TimezonePreferences) {
        self.preferences.insert(user_id, preferences);
    }

    /// Remove preferências de um usuário
    pub fn remove_user_preferences(&mut self, user_id: &str) -> Option<TimezonePreferences> {
        self.preferences.remove(user_id)
    }

    /// Lista todos os usuários com preferências
    pub fn list_users(&self) -> Vec<&String> {
        self.preferences.keys().collect()
    }

    /// Obtém estatísticas de preferências
    pub fn get_preferences_stats(&self) -> PreferencesStats {
        let total_users = self.preferences.len();
        let timezone_usage: HashMap<String, usize> =
            self.preferences.values().fold(HashMap::new(), |mut acc, prefs| {
                *acc.entry(prefs.default_timezone.clone()).or_insert(0) += 1;
                for tz in &prefs.preferred_timezones {
                    *acc.entry(tz.clone()).or_insert(0) += 1;
                }
                acc
            });

        let most_used_timezone = timezone_usage
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(timezone, _)| timezone.clone());

        let auto_detect_enabled = self.preferences.values().filter(|prefs| prefs.auto_detect).count();

        PreferencesStats {
            total_users,
            timezone_usage,
            most_used_timezone,
            auto_detect_enabled,
        }
    }

    /// Detecta preferências automaticamente
    pub fn auto_detect_preferences(&mut self, user_id: String) -> TimezoneResult<TimezonePreferences> {
        // Em um ambiente real, isso usaria bibliotecas do sistema
        // Por enquanto, retornamos UTC como fallback
        let preferences = TimezonePreferences::new(user_id.clone(), "UTC".to_string());
        self.preferences.insert(user_id, preferences.clone());
        Ok(preferences)
    }

    /// Sincroniza preferências entre usuários
    pub fn sync_preferences(&mut self, user_ids: Vec<&str>, target_timezone: &str) -> TimezoneResult<()> {
        for user_id in user_ids {
            if let Some(preferences) = self.preferences.get_mut(user_id) {
                preferences.default_timezone = target_timezone.to_string();
                preferences.add_preferred_timezone(target_timezone.to_string());
                preferences.updated_at = Utc::now();
            }
        }
        Ok(())
    }

    /// Exporta preferências para um formato específico
    pub fn export_preferences(&self, format: ExportFormat) -> TimezoneResult<String> {
        match format {
            ExportFormat::Json => serde_json::to_string(&self.preferences)
                .map_err(|e| TimezoneError::UnsupportedOperation(format!("JSON export failed: {}", e))),
            ExportFormat::Yaml => serde_yaml::to_string(&self.preferences)
                .map_err(|e| TimezoneError::UnsupportedOperation(format!("YAML export failed: {}", e))),
            ExportFormat::Csv => self.export_to_csv(),
        }
    }

    /// Exporta para CSV
    fn export_to_csv(&self) -> TimezoneResult<String> {
        let mut csv = String::from(
            "user_id,default_timezone,preferred_timezones,auto_detect,display_format,created_at,updated_at\n",
        );

        for (user_id, prefs) in &self.preferences {
            let preferred_tz = prefs.preferred_timezones.join(";");
            let display_format = match &prefs.display_format {
                TimeDisplayFormat::TwentyFourHour => "24h",
                TimeDisplayFormat::TwelveHour => "12h",
                TimeDisplayFormat::Iso8601 => "iso8601",
                TimeDisplayFormat::Custom(format) => format,
            };

            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                user_id,
                prefs.default_timezone,
                preferred_tz,
                prefs.auto_detect,
                display_format,
                prefs.created_at.to_rfc3339(),
                prefs.updated_at.to_rfc3339()
            ));
        }

        Ok(csv)
    }

    /// Importa preferências de um formato específico
    pub fn import_preferences(&mut self, data: &str, format: ExportFormat) -> TimezoneResult<()> {
        match format {
            ExportFormat::Json => {
                let prefs: HashMap<String, TimezonePreferences> = serde_json::from_str(data)
                    .map_err(|e| TimezoneError::UnsupportedOperation(format!("JSON import failed: {}", e)))?;
                self.preferences.extend(prefs);
            }
            ExportFormat::Yaml => {
                let prefs: HashMap<String, TimezonePreferences> = serde_yaml::from_str(data)
                    .map_err(|e| TimezoneError::UnsupportedOperation(format!("YAML import failed: {}", e)))?;
                self.preferences.extend(prefs);
            }
            ExportFormat::Csv => {
                self.import_from_csv(data)?;
            }
        }
        Ok(())
    }

    /// Importa de CSV
    fn import_from_csv(&mut self, data: &str) -> TimezoneResult<()> {
        let lines: Vec<&str> = data.lines().collect();
        if lines.is_empty() {
            return Ok(());
        }

        // Pular cabeçalho
        for line in lines.iter().skip(1) {
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 7 {
                continue;
            }

            let user_id = fields[0].to_string();
            let default_timezone = fields[1].to_string();
            let preferred_timezones: Vec<String> = fields[2].split(';').map(|s| s.to_string()).collect();
            let auto_detect = fields[3].parse().unwrap_or(false);
            let display_format = match fields[4] {
                "24h" => TimeDisplayFormat::TwentyFourHour,
                "12h" => TimeDisplayFormat::TwelveHour,
                "iso8601" => TimeDisplayFormat::Iso8601,
                _ => TimeDisplayFormat::Custom(fields[4].to_string()),
            };

            let preferences = TimezonePreferences {
                user_id: user_id.clone(),
                default_timezone,
                preferred_timezones,
                auto_detect,
                display_format,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.preferences.insert(user_id, preferences);
        }

        Ok(())
    }
}

/// Formato de exportação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Yaml,
    Csv,
}

/// Estatísticas de preferências
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreferencesStats {
    pub total_users: usize,
    pub timezone_usage: HashMap<String, usize>,
    pub most_used_timezone: Option<String>,
    pub auto_detect_enabled: usize,
}

impl Default for TimezonePreferencesManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_preferences_creation() {
        let manager = TimezonePreferencesManager::new();
        assert!(manager.preferences.is_empty());
    }

    #[test]
    fn test_set_user_preferences() {
        let mut manager = TimezonePreferencesManager::new();
        let preferences = TimezonePreferences::new("user1".to_string(), "America/New_York".to_string());

        manager.set_user_preferences("user1".to_string(), preferences);
        assert!(manager.preferences.contains_key("user1"));
    }

    #[test]
    fn test_get_user_preferences() {
        let mut manager = TimezonePreferencesManager::new();
        let preferences = TimezonePreferences::new("user1".to_string(), "America/New_York".to_string());

        manager.set_user_preferences("user1".to_string(), preferences);
        let retrieved = manager.get_user_preferences("user1");
        assert_eq!(retrieved.user_id, "user1");
    }

    #[test]
    fn test_get_nonexistent_user_preferences() {
        let manager = TimezonePreferencesManager::new();
        let retrieved = manager.get_user_preferences("nonexistent");
        assert_eq!(retrieved.user_id, "default");
    }

    #[test]
    fn test_export_preferences_json() {
        let mut manager = TimezonePreferencesManager::new();
        let preferences = TimezonePreferences::new("user1".to_string(), "UTC".to_string());
        manager.set_user_preferences("user1".to_string(), preferences);

        let result = manager.export_preferences(ExportFormat::Json);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("user1"));
    }

    #[test]
    fn test_get_preferences_stats() {
        let mut manager = TimezonePreferencesManager::new();
        let preferences = TimezonePreferences::new("user1".to_string(), "UTC".to_string());
        manager.set_user_preferences("user1".to_string(), preferences);

        let stats = manager.get_preferences_stats();
        assert_eq!(stats.total_users, 1);
    }
}
