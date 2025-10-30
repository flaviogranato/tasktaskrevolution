use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Métricas de performance de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneMetrics {
    pub timezone_id: String,
    pub total_meetings: usize,
    pub total_duration_minutes: i64,
    pub average_meeting_duration: f64,
    pub peak_usage_hours: Vec<u8>,
    pub utilization_percentage: f64,
    pub efficiency_score: f64,
    pub timezone_conflicts: usize,
    pub last_updated: DateTime<Utc>,
}

impl TimezoneMetrics {
    pub fn new(timezone_id: String) -> Self {
        Self {
            timezone_id,
            total_meetings: 0,
            total_duration_minutes: 0,
            average_meeting_duration: 0.0,
            peak_usage_hours: Vec::new(),
            utilization_percentage: 0.0,
            efficiency_score: 0.0,
            timezone_conflicts: 0,
            last_updated: Utc::now(),
        }
    }

    pub fn add_meeting(&mut self, duration_minutes: i64) {
        self.total_meetings += 1;
        self.total_duration_minutes += duration_minutes;
        self.average_meeting_duration = self.total_duration_minutes as f64 / self.total_meetings as f64;
        self.last_updated = Utc::now();
    }

    pub fn calculate_efficiency_score(&mut self) {
        // Score baseado em vários fatores
        let mut score = 0.0;

        // Fator 1: Utilização (0-40 pontos)
        score += (self.utilization_percentage / 100.0) * 40.0;

        // Fator 2: Duração média (0-30 pontos)
        // Duração ideal: 60-90 minutos
        let ideal_duration = 75.0;
        let duration_diff = (self.average_meeting_duration - ideal_duration).abs();
        let duration_score = if duration_diff <= 15.0 {
            30.0
        } else if duration_diff <= 30.0 {
            20.0
        } else {
            10.0
        };
        score += duration_score;

        // Fator 3: Conflitos (0-30 pontos)
        let conflict_penalty = (self.timezone_conflicts as f64 * 5.0).min(30.0);
        score += 30.0 - conflict_penalty;

        self.efficiency_score = score.clamp(0.0, 100.0);
    }

    pub fn update_peak_usage(&mut self, hour: u8) {
        if !self.peak_usage_hours.contains(&hour) {
            self.peak_usage_hours.push(hour);
            self.peak_usage_hours.sort();
        }
    }

    pub fn add_timezone_conflict(&mut self) {
        self.timezone_conflicts += 1;
        self.last_updated = Utc::now();
    }
}

/// Agregador de métricas globais
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneMetricsAggregator {
    pub metrics: HashMap<String, TimezoneMetrics>,
    pub global_metrics: GlobalTimezoneMetrics,
    pub last_aggregation: DateTime<Utc>,
}

impl TimezoneMetricsAggregator {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            global_metrics: GlobalTimezoneMetrics::new(),
            last_aggregation: Utc::now(),
        }
    }

    pub fn add_meeting(&mut self, timezone: String, duration_minutes: i64, hour: u8) {
        let metrics = self
            .metrics
            .entry(timezone.clone())
            .or_insert_with(|| TimezoneMetrics::new(timezone.clone()));

        metrics.add_meeting(duration_minutes);
        metrics.update_peak_usage(hour);

        // Atualizar métricas globais
        self.global_metrics.total_meetings += 1;
        self.global_metrics.total_duration_minutes += duration_minutes;
        self.global_metrics.last_updated = Utc::now();
    }

    pub fn add_timezone_conflict(&mut self, timezone: String) {
        let metrics = self
            .metrics
            .entry(timezone.clone())
            .or_insert_with(|| TimezoneMetrics::new(timezone.clone()));

        metrics.add_timezone_conflict();
        self.global_metrics.total_conflicts += 1;
    }

    pub fn aggregate_metrics(&mut self) {
        self.last_aggregation = Utc::now();

        // Calcular métricas globais
        self.global_metrics.total_timezones = self.metrics.len();

        if self.global_metrics.total_meetings > 0 {
            self.global_metrics.average_meeting_duration =
                self.global_metrics.total_duration_minutes as f64 / self.global_metrics.total_meetings as f64;
        }

        // Calcular eficiência de cada timezone
        for metrics in self.metrics.values_mut() {
            metrics.calculate_efficiency_score();
        }

        // Encontrar timezone mais eficiente
        self.global_metrics.most_efficient_timezone = self
            .metrics
            .iter()
            .max_by(|a, b| a.1.efficiency_score.partial_cmp(&b.1.efficiency_score).unwrap())
            .map(|(timezone, _)| timezone.clone());

        // Calcular distribuição de eficiência
        self.calculate_efficiency_distribution();
    }

    fn calculate_efficiency_distribution(&mut self) {
        let mut efficiency_ranges = HashMap::new();

        for metrics in self.metrics.values() {
            let range = match metrics.efficiency_score {
                score if score >= 90.0 => "Excellent",
                score if score >= 80.0 => "Good",
                score if score >= 70.0 => "Average",
                score if score >= 60.0 => "Below Average",
                _ => "Poor",
            };

            *efficiency_ranges.entry(range.to_string()).or_insert(0) += 1;
        }

        self.global_metrics.efficiency_distribution = efficiency_ranges;
    }

    pub fn get_timezone_metrics(&self, timezone: &str) -> Option<&TimezoneMetrics> {
        self.metrics.get(timezone)
    }

    pub fn get_top_performing_timezones(&self, limit: usize) -> Vec<&TimezoneMetrics> {
        let mut timezones: Vec<&TimezoneMetrics> = self.metrics.values().collect();
        timezones.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap());
        timezones.truncate(limit);
        timezones
    }

    pub fn get_underperforming_timezones(&self, threshold: f64) -> Vec<&TimezoneMetrics> {
        self.metrics
            .values()
            .filter(|metrics| metrics.efficiency_score < threshold)
            .collect()
    }

    pub fn export_metrics(&self, format: MetricsFormat) -> Result<String, String> {
        match format {
            MetricsFormat::Json => {
                serde_json::to_string_pretty(self).map_err(|e| format!("Erro ao exportar JSON: {}", e))
            }
            MetricsFormat::Csv => self.export_to_csv(),
        }
    }

    fn export_to_csv(&self) -> Result<String, String> {
        let mut csv = String::from(
            "timezone,total_meetings,total_duration_minutes,average_duration,efficiency_score,conflicts\n",
        );

        for metrics in self.metrics.values() {
            csv.push_str(&format!(
                "{},{},{},{:.2},{:.2},{}\n",
                metrics.timezone_id,
                metrics.total_meetings,
                metrics.total_duration_minutes,
                metrics.average_meeting_duration,
                metrics.efficiency_score,
                metrics.timezone_conflicts
            ));
        }

        Ok(csv)
    }
}

/// Métricas globais de timezone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalTimezoneMetrics {
    pub total_timezones: usize,
    pub total_meetings: usize,
    pub total_duration_minutes: i64,
    pub average_meeting_duration: f64,
    pub total_conflicts: usize,
    pub most_efficient_timezone: Option<String>,
    pub efficiency_distribution: HashMap<String, usize>,
    pub last_updated: DateTime<Utc>,
}

impl GlobalTimezoneMetrics {
    pub fn new() -> Self {
        Self {
            total_timezones: 0,
            total_meetings: 0,
            total_duration_minutes: 0,
            average_meeting_duration: 0.0,
            total_conflicts: 0,
            most_efficient_timezone: None,
            efficiency_distribution: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for GlobalTimezoneMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Formato de exportação de métricas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricsFormat {
    Json,
    Csv,
}

impl Default for TimezoneMetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_metrics_creation() {
        let metrics = TimezoneMetrics::new("UTC".to_string());
        assert_eq!(metrics.timezone_id, "UTC");
        assert_eq!(metrics.total_meetings, 0);
    }

    #[test]
    fn test_add_meeting() {
        let mut metrics = TimezoneMetrics::new("UTC".to_string());
        metrics.add_meeting(60);
        assert_eq!(metrics.total_meetings, 1);
        assert_eq!(metrics.total_duration_minutes, 60);
        assert_eq!(metrics.average_meeting_duration, 60.0);
    }

    #[test]
    fn test_calculate_efficiency_score() {
        let mut metrics = TimezoneMetrics::new("UTC".to_string());
        metrics.utilization_percentage = 80.0;
        metrics.average_meeting_duration = 75.0;
        metrics.timezone_conflicts = 0;
        metrics.calculate_efficiency_score();
        assert!(metrics.efficiency_score > 0.0);
    }

    #[test]
    fn test_metrics_aggregator() {
        let mut aggregator = TimezoneMetricsAggregator::new();
        aggregator.add_meeting("UTC".to_string(), 60, 9);
        assert_eq!(aggregator.global_metrics.total_meetings, 1);
    }

    #[test]
    fn test_export_metrics() {
        let aggregator = TimezoneMetricsAggregator::new();
        let result = aggregator.export_metrics(MetricsFormat::Json);
        assert!(result.is_ok());
    }
}
