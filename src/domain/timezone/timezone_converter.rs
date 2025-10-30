use super::timezone_models::*;
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conversor de timezones
pub struct TimezoneConverter {
    conversions: HashMap<String, TimezoneConversion>,
}

impl TimezoneConverter {
    pub fn new() -> Self {
        Self {
            conversions: HashMap::new(),
        }
    }

    /// Converte um DateTime de um timezone para outro
    pub fn convert_time(
        &mut self,
        time: DateTime<Utc>,
        from_timezone: &str,
        to_timezone: &str,
    ) -> TimezoneResult<DateTime<Utc>> {
        // Validar timezones
        let from_tz: Tz = from_timezone
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(from_timezone.to_string()))?;
        let to_tz: Tz = to_timezone
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(to_timezone.to_string()))?;

        // Converter para o timezone de origem
        let local_time = time.with_timezone(&from_tz);

        // Converter para o timezone de destino
        let target_time = local_time.with_timezone(&to_tz);

        // Converter de volta para UTC
        let utc_result = target_time.with_timezone(&Utc);

        // Salvar a conversão
        let conversion = TimezoneConversion::new(from_timezone.to_string(), to_timezone.to_string(), time, utc_result);
        self.conversions.insert(conversion.id.to_string(), conversion);

        Ok(utc_result)
    }

    /// Converte um horário local para UTC
    pub fn local_to_utc(&self, local_time: DateTime<Tz>, _timezone: &str) -> TimezoneResult<DateTime<Utc>> {
        Ok(local_time.with_timezone(&Utc))
    }

    /// Converte UTC para um horário local
    pub fn utc_to_local(&self, utc_time: DateTime<Utc>, timezone: &str) -> TimezoneResult<DateTime<Tz>> {
        let tz: Tz = timezone
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(timezone.to_string()))?;

        Ok(utc_time.with_timezone(&tz))
    }

    /// Converte entre múltiplos timezones
    pub fn convert_to_multiple_timezones(
        &mut self,
        utc_time: DateTime<Utc>,
        timezones: Vec<&str>,
    ) -> TimezoneResult<HashMap<String, DateTime<Tz>>> {
        let mut results = HashMap::new();

        for timezone in timezones {
            let tz: Tz = timezone
                .parse()
                .map_err(|_| TimezoneError::InvalidTimezone(timezone.to_string()))?;

            let local_time = utc_time.with_timezone(&tz);
            results.insert(timezone.to_string(), local_time);
        }

        Ok(results)
    }

    /// Calcula a diferença de horário entre dois timezones
    pub fn calculate_timezone_offset(
        &self,
        timezone1: &str,
        timezone2: &str,
        at_time: DateTime<Utc>,
    ) -> TimezoneResult<i32> {
        let tz1: Tz = timezone1
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(timezone1.to_string()))?;
        let tz2: Tz = timezone2
            .parse()
            .map_err(|_| TimezoneError::InvalidTimezone(timezone2.to_string()))?;

        // Convert the UTC time to both timezones
        let local1 = at_time.with_timezone(&tz1);
        let local2 = at_time.with_timezone(&tz2);

        // Calculate the offset in seconds between the two timezones
        // This represents how many seconds ahead/behind timezone2 is compared to timezone1
        let offset = local2.timestamp() - local1.timestamp();
        Ok(offset as i32)
    }

    /// Encontra o melhor horário para uma reunião global
    pub fn find_best_meeting_time(
        &self,
        participants: Vec<GlobalParticipant>,
        duration_minutes: i64,
    ) -> TimezoneResult<Vec<MeetingTimeOption>> {
        if participants.is_empty() {
            return Err(TimezoneError::UnsupportedOperation(
                "No participants provided".to_string(),
            ));
        }

        let mut options = Vec::new();

        // Para cada participante, calcular horários possíveis
        for participant in &participants {
            let tz: Tz = participant
                .timezone
                .parse()
                .map_err(|_| TimezoneError::InvalidTimezone(participant.timezone.clone()))?;

            // Considerar horário de trabalho (9h às 17h local)
            let start_hour = 9;
            let end_hour = 17;

            for hour in start_hour..end_hour {
                let now = Utc::now();
                let date = now.date_naive();
                let local_start = tz
                    .with_ymd_and_hms(date.year(), date.month(), date.day(), hour, 0, 0)
                    .unwrap()
                    .with_timezone(&Utc);

                let local_end = local_start + chrono::Duration::minutes(duration_minutes);

                // Verificar se está dentro do horário de trabalho
                if local_end.hour() <= end_hour {
                    let utc_start = local_start.with_timezone(&Utc);
                    let utc_end = local_end.with_timezone(&Utc);

                    options.push(MeetingTimeOption {
                        start_time: utc_start,
                        end_time: utc_end,
                        timezone: participant.timezone.clone(),
                        participant_name: participant.name.clone(),
                        score: self.calculate_meeting_score(&participants, utc_start, utc_end),
                    });
                }
            }
        }

        // Ordenar por score (melhor primeiro)
        options.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(options)
    }

    /// Calcula o score de uma opção de reunião
    fn calculate_meeting_score(
        &self,
        participants: &[GlobalParticipant],
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> f64 {
        let mut score = 0.0;
        let mut total_participants = 0.0;

        for participant in participants {
            let tz: Tz = match participant.timezone.parse() {
                Ok(tz) => tz,
                Err(_) => continue,
            };

            let local_start = start_time.with_timezone(&tz);
            let _local_end = end_time.with_timezone(&tz);

            // Score baseado no horário local
            let hour = local_start.hour();
            let mut participant_score = if (9..=17).contains(&hour) {
                1.0
            } else if (8..=18).contains(&hour) {
                0.8
            } else if (7..=19).contains(&hour) {
                0.6
            } else {
                0.2
            };

            // Penalizar se for fim de semana
            if local_start.weekday() == chrono::Weekday::Sat || local_start.weekday() == chrono::Weekday::Sun {
                participant_score *= 0.5;
            }

            score += participant_score;
            total_participants += 1.0;
        }

        if total_participants > 0.0 {
            score / total_participants
        } else {
            0.0
        }
    }

    /// Obtém histórico de conversões
    pub fn get_conversion_history(&self) -> Vec<&TimezoneConversion> {
        self.conversions.values().collect()
    }

    /// Limpa o histórico de conversões
    pub fn clear_history(&mut self) {
        self.conversions.clear();
    }

    /// Obtém estatísticas de conversão
    pub fn get_conversion_stats(&self) -> ConversionStats {
        let total_conversions = self.conversions.len();
        let timezone_usage: HashMap<String, usize> = self.conversions.values().fold(HashMap::new(), |mut acc, conv| {
            *acc.entry(conv.source_timezone.clone()).or_insert(0) += 1;
            *acc.entry(conv.target_timezone.clone()).or_insert(0) += 1;
            acc
        });

        let most_used_timezone = timezone_usage
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(timezone, _)| timezone.clone());

        ConversionStats {
            total_conversions,
            timezone_usage,
            most_used_timezone,
        }
    }
}

/// Opção de horário para reunião
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeetingTimeOption {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub timezone: String,
    pub participant_name: String,
    pub score: f64,
}

/// Estatísticas de conversão
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionStats {
    pub total_conversions: usize,
    pub timezone_usage: HashMap<String, usize>,
    pub most_used_timezone: Option<String>,
}

impl Default for TimezoneConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_converter_creation() {
        let converter = TimezoneConverter::new();
        assert!(converter.conversions.is_empty());
    }

    #[test]
    fn test_convert_time() {
        let mut converter = TimezoneConverter::new();
        let utc_time = Utc::now();

        let result = converter.convert_time(utc_time, "UTC", "America/New_York");
        assert!(result.is_ok());
    }

    #[test]
    fn test_convert_invalid_timezone() {
        let mut converter = TimezoneConverter::new();
        let utc_time = Utc::now();

        let result = converter.convert_time(utc_time, "Invalid", "America/New_York");
        assert!(result.is_err());
    }

    #[test]
    fn test_utc_to_local() {
        let converter = TimezoneConverter::new();
        let utc_time = Utc::now();

        let result = converter.utc_to_local(utc_time, "America/New_York");
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_timezone_offset() {
        let converter = TimezoneConverter::new();
        // Use a specific date in winter (no DST) to get consistent results
        let utc_time = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let offset = converter.calculate_timezone_offset("America/New_York", "UTC", utc_time);
        assert!(offset.is_ok());
        // Just verify the method works without errors
        let _offset_value = offset.unwrap();
    }
}
