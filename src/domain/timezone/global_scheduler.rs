use super::timezone_config::*;
use super::timezone_metrics::*;
use super::timezone_models::*;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Agendador global para coordenação entre timezones
pub struct GlobalScheduler {
    schedules: HashMap<String, GlobalSchedule>,
    timezone_converter: super::TimezoneConverter,
    coordination_rules: Vec<CoordinationRule>,
    conflict_resolver: ConflictResolver,
    #[allow(dead_code)]
    metrics_aggregator: TimezoneMetricsAggregator,
    working_hours_cache: HashMap<String, WorkingHours>,
}

impl GlobalScheduler {
    pub fn new() -> Self {
        Self {
            schedules: HashMap::new(),
            timezone_converter: super::TimezoneConverter::new(),
            coordination_rules: Vec::new(),
            conflict_resolver: ConflictResolver::new(),
            metrics_aggregator: TimezoneMetricsAggregator::new(),
            working_hours_cache: HashMap::new(),
        }
    }

    /// Cria um novo cronograma global
    pub fn create_schedule(
        &mut self,
        project_id: String,
        timezone: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> TimezoneResult<GlobalSchedule> {
        if start_time >= end_time {
            return Err(TimezoneError::InvalidTimeFormat(
                "Start time must be before end time".to_string(),
            ));
        }

        let schedule = GlobalSchedule::new(project_id.clone(), timezone, start_time, end_time);
        self.schedules.insert(project_id, schedule.clone());
        Ok(schedule)
    }

    /// Adiciona um participante a um cronograma
    pub fn add_participant(&mut self, schedule_id: &str, participant: GlobalParticipant) -> TimezoneResult<()> {
        if let Some(schedule) = self.schedules.get_mut(schedule_id) {
            schedule.add_participant(participant);
            Ok(())
        } else {
            Err(TimezoneError::UnsupportedOperation(format!(
                "Schedule {} not found",
                schedule_id
            )))
        }
    }

    /// Encontra horários disponíveis para todos os participantes
    pub fn find_available_times(
        &self,
        participants: Vec<GlobalParticipant>,
        duration_minutes: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> TimezoneResult<Vec<AvailableTimeSlot>> {
        let mut available_slots = Vec::new();
        let duration = Duration::minutes(duration_minutes);

        // Gerar slots de tempo em intervalos de 30 minutos
        let mut current_time = start_date;
        while current_time + duration <= end_date {
            let slot_end = current_time + duration;

            // Verificar se todos os participantes estão disponíveis
            let mut all_available = true;
            let mut slot_participants = Vec::new();

            for participant in &participants {
                let tz: Tz = participant
                    .timezone
                    .parse()
                    .map_err(|_| TimezoneError::InvalidTimezone(participant.timezone.clone()))?;

                let local_start = current_time.with_timezone(&tz);
                let local_end = slot_end.with_timezone(&tz);

                // Verificar se está dentro do horário de trabalho (9h-17h)
                let is_working_hours = local_start.hour() >= 9 && local_end.hour() <= 17;
                let is_weekday =
                    local_start.weekday() != chrono::Weekday::Sat && local_start.weekday() != chrono::Weekday::Sun;

                if !is_working_hours || !is_weekday {
                    all_available = false;
                    break;
                }

                slot_participants.push(SlotParticipant {
                    participant: participant.clone(),
                    local_start_time: local_start.with_timezone(&Utc),
                    local_end_time: local_end.with_timezone(&Utc),
                    is_available: true,
                });
            }

            if all_available {
                let score = self.calculate_slot_score(&slot_participants);
                available_slots.push(AvailableTimeSlot {
                    start_time: current_time,
                    end_time: slot_end,
                    participants: slot_participants,
                    score,
                });
            }

            current_time += Duration::minutes(30);
        }

        // Ordenar por score (melhor primeiro)
        available_slots.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(available_slots)
    }

    /// Calcula o score de um slot de tempo
    fn calculate_slot_score(&self, participants: &[SlotParticipant]) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        for participant in participants {
            let hour = participant.local_start_time.hour();
            #[allow(unused_assignments)]
            let mut score = 0.0;

            // Score baseado no horário local
            if (9..=11).contains(&hour) {
                score = 1.0; // Manhã ideal
            } else if (14..=16).contains(&hour) {
                score = 0.9; // Tarde ideal
            } else if (12..=13).contains(&hour) {
                score = 0.7; // Hora do almoço
            } else if (8..=17).contains(&hour) {
                score = 0.8; // Horário de trabalho
            } else {
                score = 0.3; // Fora do horário ideal
            }

            total_score += score;
            count += 1;
        }

        if count > 0 { total_score / count as f64 } else { 0.0 }
    }

    /// Sincroniza cronogramas entre diferentes timezones
    pub fn sync_schedules(
        &mut self,
        schedule_ids: Vec<&str>,
        target_timezone: &str,
    ) -> TimezoneResult<Vec<SynchronizedSchedule>> {
        let mut synchronized = Vec::new();

        for schedule_id in schedule_ids {
            if let Some(schedule) = self.schedules.get(schedule_id) {
                let tz: Tz = target_timezone
                    .parse()
                    .map_err(|_| TimezoneError::InvalidTimezone(target_timezone.to_string()))?;

                let local_start = schedule.start_time.with_timezone(&tz);
                let local_end = schedule.end_time.with_timezone(&tz);

                let mut synced_participants = Vec::new();
                for participant in &schedule.participants {
                    let participant_tz: Tz = participant
                        .timezone
                        .parse()
                        .map_err(|_| TimezoneError::InvalidTimezone(participant.timezone.clone()))?;

                    let participant_local_start = schedule.start_time.with_timezone(&participant_tz);
                    let participant_local_end = schedule.end_time.with_timezone(&participant_tz);

                    synced_participants.push(SynchronizedParticipant {
                        participant: participant.clone(),
                        local_start_time: participant_local_start.with_timezone(&Utc),
                        local_end_time: participant_local_end.with_timezone(&Utc),
                        timezone_offset: self.calculate_offset(&tz, &participant_tz, schedule.start_time),
                    });
                }

                synchronized.push(SynchronizedSchedule {
                    original_schedule: schedule.clone(),
                    target_timezone: target_timezone.to_string(),
                    local_start_time: local_start.with_timezone(&Utc),
                    local_end_time: local_end.with_timezone(&Utc),
                    participants: synced_participants,
                });
            }
        }

        Ok(synchronized)
    }

    /// Calcula o offset entre dois timezones
    fn calculate_offset(&self, tz1: &Tz, tz2: &Tz, reference_time: DateTime<Utc>) -> i32 {
        let time1 = reference_time.with_timezone(tz1);
        let time2 = reference_time.with_timezone(tz2);
        (time2.timestamp() - time1.timestamp()) as i32
    }

    /// Obtém estatísticas de agendamento
    pub fn get_scheduling_stats(&self) -> SchedulingStats {
        let total_schedules = self.schedules.len();
        let total_participants: usize = self.schedules.values().map(|s| s.participants.len()).sum();

        let timezone_distribution: HashMap<String, usize> =
            self.schedules.values().fold(HashMap::new(), |mut acc, schedule| {
                *acc.entry(schedule.timezone.clone()).or_insert(0) += 1;
                for participant in &schedule.participants {
                    *acc.entry(participant.timezone.clone()).or_insert(0) += 1;
                }
                acc
            });

        let most_used_timezone = timezone_distribution
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(timezone, _)| timezone.clone());

        SchedulingStats {
            total_schedules,
            total_participants,
            timezone_distribution,
            most_used_timezone,
        }
    }

    /// Obtém um cronograma por ID
    pub fn get_schedule(&self, schedule_id: &str) -> Option<&GlobalSchedule> {
        self.schedules.get(schedule_id)
    }

    /// Lista todos os cronogramas
    pub fn list_schedules(&self) -> Vec<&GlobalSchedule> {
        self.schedules.values().collect()
    }

    /// Remove um cronograma
    pub fn remove_schedule(&mut self, schedule_id: &str) -> Option<GlobalSchedule> {
        self.schedules.remove(schedule_id)
    }

    /// Adiciona regra de coordenação
    pub fn add_coordination_rule(&mut self, rule: CoordinationRule) {
        self.coordination_rules.push(rule);
    }

    /// Remove regra de coordenação
    pub fn remove_coordination_rule(&mut self, rule_id: &str) {
        self.coordination_rules.retain(|r| r.id != rule_id);
    }

    /// Lista regras de coordenação
    pub fn list_coordination_rules(&self) -> &[CoordinationRule] {
        &self.coordination_rules
    }

    /// Coordena múltiplos cronogramas
    pub fn coordinate_schedules(&mut self, schedule_ids: Vec<String>) -> TimezoneResult<CoordinationResult> {
        let mut conflicts = Vec::new();
        let mut suggestions = Vec::new();
        let mut optimized_schedules = HashMap::new();

        // Verificar conflitos entre cronogramas
        for i in 0..schedule_ids.len() {
            for j in (i + 1)..schedule_ids.len() {
                if let Some(conflict) = self.detect_schedule_conflict(&schedule_ids[i], &schedule_ids[j]) {
                    conflicts.push(conflict);
                }
            }
        }

        // Resolver conflitos automaticamente
        for conflict in &conflicts {
            if let Some(resolution) = self.conflict_resolver.resolve_conflict(conflict) {
                suggestions.push(resolution);
            }
        }

        // Aplicar otimizações
        for schedule_id in &schedule_ids {
            if let Some(schedule) = self.schedules.get(schedule_id) {
                let optimized = self.optimize_schedule(schedule)?;
                optimized_schedules.insert(schedule_id.clone(), optimized);
            }
        }

        Ok(CoordinationResult {
            conflicts,
            suggestions,
            optimized_schedules,
            coordination_score: self.calculate_coordination_score(&schedule_ids),
        })
    }

    /// Detecta conflito entre dois cronogramas
    fn detect_schedule_conflict(&self, schedule_id1: &str, schedule_id2: &str) -> Option<ScheduleConflict> {
        let schedule1 = self.schedules.get(schedule_id1)?;
        let schedule2 = self.schedules.get(schedule_id2)?;

        // Verificar sobreposição de horários
        if schedule1.start_time < schedule2.end_time && schedule2.start_time < schedule1.end_time {
            // Verificar conflitos de participantes
            let participants1: HashSet<String> = schedule1.participants.iter().map(|p| p.user_id.clone()).collect();
            let participants2: HashSet<String> = schedule2.participants.iter().map(|p| p.user_id.clone()).collect();

            let common_participants: Vec<String> = participants1.intersection(&participants2).cloned().collect();

            if !common_participants.is_empty() {
                return Some(ScheduleConflict {
                    schedule_id1: schedule_id1.to_string(),
                    schedule_id2: schedule_id2.to_string(),
                    conflict_type: ConflictType::ParticipantOverlap,
                    common_participants,
                    severity: ConflictSeverity::High,
                    suggested_resolution: self.generate_conflict_resolution(schedule1, schedule2),
                });
            }
        }

        None
    }

    /// Gera resolução de conflito
    fn generate_conflict_resolution(
        &self,
        schedule1: &GlobalSchedule,
        schedule2: &GlobalSchedule,
    ) -> ConflictResolution {
        let time_diff = (schedule1.start_time - schedule2.start_time).num_minutes().abs();

        if time_diff < 30 {
            // Conflito próximo - sugerir reagendamento
            ConflictResolution::Reschedule {
                suggested_time: schedule1.start_time + Duration::hours(1),
                reason: "Time conflict detected".to_string(),
            }
        } else {
            // Conflito distante - sugerir ajuste menor
            ConflictResolution::AdjustTime {
                adjustment_minutes: 15,
                reason: "Minor time adjustment needed".to_string(),
            }
        }
    }

    /// Otimiza um cronograma
    fn optimize_schedule(&self, schedule: &GlobalSchedule) -> TimezoneResult<OptimizedSchedule> {
        let mut optimized_participants = Vec::new();
        let mut total_score = 0.0;

        for participant in &schedule.participants {
            let score = self.calculate_participant_score(participant, schedule);
            total_score += score;

            optimized_participants.push(OptimizedParticipant {
                participant: participant.clone(),
                score,
                timezone_efficiency: self.calculate_timezone_efficiency(&participant.timezone),
                working_hours_compatibility: self.check_working_hours_compatibility(participant, schedule),
            });
        }

        let average_score = if optimized_participants.is_empty() {
            0.0
        } else {
            total_score / optimized_participants.len() as f64
        };

        Ok(OptimizedSchedule {
            original_schedule: schedule.clone(),
            optimized_participants,
            optimization_score: average_score,
            timezone_distribution: self.analyze_timezone_distribution(schedule),
            recommendations: self.generate_optimization_recommendations(schedule),
        })
    }

    /// Calcula score de um participante
    fn calculate_participant_score(&self, participant: &GlobalParticipant, schedule: &GlobalSchedule) -> f64 {
        let mut score = 0.0;

        // Score baseado em timezone
        let timezone_score = self.calculate_timezone_efficiency(&participant.timezone);
        score += timezone_score * 0.4;

        // Score baseado em horário de trabalho
        let working_hours_score = self.check_working_hours_compatibility(participant, schedule);
        score += working_hours_score * 0.3;

        // Score baseado em disponibilidade
        let availability_score = self.calculate_availability_score(participant, schedule);
        score += availability_score * 0.3;

        score
    }

    /// Calcula eficiência do timezone
    fn calculate_timezone_efficiency(&self, timezone: &str) -> f64 {
        // Implementar lógica de eficiência baseada em timezone
        // Por enquanto, retornar score baseado em offset UTC
        match timezone {
            "UTC" => 1.0,
            "America/New_York" => 0.8,
            "Europe/London" => 0.9,
            "Asia/Tokyo" => 0.7,
            _ => 0.6,
        }
    }

    /// Verifica compatibilidade com horários de trabalho
    fn check_working_hours_compatibility(&self, participant: &GlobalParticipant, schedule: &GlobalSchedule) -> f64 {
        // Verificar se o horário está dentro dos horários de trabalho do participante
        if let Some(working_hours) = self.working_hours_cache.get(&participant.timezone) {
            if working_hours.is_working_time(schedule.start_time) {
                1.0
            } else {
                0.5
            }
        } else {
            0.7 // Score neutro se não houver informações de horário de trabalho
        }
    }

    /// Calcula score de disponibilidade
    fn calculate_availability_score(&self, participant: &GlobalParticipant, schedule: &GlobalSchedule) -> f64 {
        // Verificar se o participante está disponível no horário do cronograma
        if participant.local_start_time <= schedule.start_time && participant.local_end_time >= schedule.end_time {
            1.0
        } else {
            0.3
        }
    }

    /// Analisa distribuição de timezones
    fn analyze_timezone_distribution(&self, schedule: &GlobalSchedule) -> TimezoneDistribution {
        let mut timezone_count: HashMap<String, usize> = HashMap::new();

        for participant in &schedule.participants {
            *timezone_count.entry(participant.timezone.clone()).or_insert(0) += 1;
        }

        let total_participants = schedule.participants.len();
        let timezone_diversity = if total_participants > 0 {
            timezone_count.len() as f64 / total_participants as f64
        } else {
            0.0
        };

        let most_common_timezone = timezone_count
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(tz, _)| tz.clone());

        TimezoneDistribution {
            timezone_count,
            diversity_score: timezone_diversity,
            most_common_timezone,
        }
    }

    /// Gera recomendações de otimização
    fn generate_optimization_recommendations(&self, schedule: &GlobalSchedule) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();

        // Recomendação baseada em distribuição de timezones
        let timezone_distribution = self.analyze_timezone_distribution(schedule);
        if timezone_distribution.diversity_score < 0.3 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::TimezoneDiversity,
                title: "Melhorar Diversidade de Timezones".to_string(),
                description: "Considere incluir participantes de diferentes timezones para melhor coordenação global"
                    .to_string(),
                priority: RecommendationPriority::Medium,
            });
        }

        // Recomendação baseada em horários de trabalho
        let working_hours_conflicts = self.count_working_hours_conflicts(schedule);
        if working_hours_conflicts > 0 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::WorkingHours,
                title: "Ajustar Horários de Trabalho".to_string(),
                description: format!(
                    "{} participantes têm conflitos com horários de trabalho",
                    working_hours_conflicts
                ),
                priority: RecommendationPriority::High,
            });
        }

        recommendations
    }

    /// Conta conflitos de horários de trabalho
    fn count_working_hours_conflicts(&self, schedule: &GlobalSchedule) -> usize {
        schedule
            .participants
            .iter()
            .filter(|p| {
                if let Some(working_hours) = self.working_hours_cache.get(&p.timezone) {
                    !working_hours.is_working_time(schedule.start_time)
                } else {
                    false
                }
            })
            .count()
    }

    /// Calcula score de coordenação
    fn calculate_coordination_score(&self, schedule_ids: &[String]) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        for schedule_id in schedule_ids {
            if let Some(schedule) = self.schedules.get(schedule_id) {
                let score = self.calculate_schedule_coordination_score(schedule);
                total_score += score;
                count += 1;
            }
        }

        if count > 0 { total_score / count as f64 } else { 0.0 }
    }

    /// Calcula score de coordenação de um cronograma
    fn calculate_schedule_coordination_score(&self, schedule: &GlobalSchedule) -> f64 {
        let timezone_distribution = self.analyze_timezone_distribution(schedule);
        let conflict_ratio = if schedule.participants.is_empty() {
            0.0
        } else {
            self.count_working_hours_conflicts(schedule) as f64 / schedule.participants.len() as f64
        };
        let working_hours_score = (1.0 - conflict_ratio).max(0.0);

        (timezone_distribution.diversity_score + working_hours_score) / 2.0
    }

    /// Adiciona horários de trabalho para um timezone
    pub fn set_working_hours(&mut self, timezone: String, working_hours: WorkingHours) {
        self.working_hours_cache.insert(timezone, working_hours);
    }

    /// Obtém horários de trabalho para um timezone
    pub fn get_working_hours(&self, timezone: &str) -> Option<&WorkingHours> {
        self.working_hours_cache.get(timezone)
    }

    /// Sincroniza cronogramas com um timezone de referência
    pub fn sync_with_reference_timezone(&mut self, reference_timezone: &str) -> TimezoneResult<SyncResult> {
        let mut synced_schedules = Vec::new();
        let mut conflicts_resolved = 0;

        for (schedule_id, schedule) in &mut self.schedules {
            let original_start = schedule.start_time;
            let original_end = schedule.end_time;

            // Converter para timezone de referência
            let reference_start =
                self.timezone_converter
                    .convert_time(original_start, &schedule.timezone, reference_timezone)?;

            let reference_end =
                self.timezone_converter
                    .convert_time(original_end, &schedule.timezone, reference_timezone)?;

            // Atualizar cronograma
            schedule.start_time = reference_start;
            schedule.end_time = reference_end;
            schedule.timezone = reference_timezone.to_string();

            synced_schedules.push(schedule_id.clone());
            conflicts_resolved += 1;
        }

        Ok(SyncResult {
            reference_timezone: reference_timezone.to_string(),
            synced_schedules,
            conflicts_resolved,
            sync_timestamp: Utc::now(),
        })
    }
}

/// Slot de tempo disponível
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AvailableTimeSlot {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub participants: Vec<SlotParticipant>,
    pub score: f64,
}

/// Participante em um slot
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotParticipant {
    pub participant: GlobalParticipant,
    pub local_start_time: DateTime<Utc>,
    pub local_end_time: DateTime<Utc>,
    pub is_available: bool,
}

/// Cronograma sincronizado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynchronizedSchedule {
    pub original_schedule: GlobalSchedule,
    pub target_timezone: String,
    pub local_start_time: DateTime<Utc>,
    pub local_end_time: DateTime<Utc>,
    pub participants: Vec<SynchronizedParticipant>,
}

/// Participante sincronizado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynchronizedParticipant {
    pub participant: GlobalParticipant,
    pub local_start_time: DateTime<Utc>,
    pub local_end_time: DateTime<Utc>,
    pub timezone_offset: i32,
}

/// Estatísticas de agendamento
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulingStats {
    pub total_schedules: usize,
    pub total_participants: usize,
    pub timezone_distribution: HashMap<String, usize>,
    pub most_used_timezone: Option<String>,
}

impl Default for GlobalScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Regra de coordenação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: CoordinationRuleType,
    pub parameters: HashMap<String, String>,
    pub is_active: bool,
}

/// Tipo de regra de coordenação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordinationRuleType {
    TimezoneDiversity,
    WorkingHoursAlignment,
    ConflictResolution,
    Optimization,
}

/// Resolvedor de conflitos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictResolver {
    pub resolution_strategies: Vec<ResolutionStrategy>,
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            resolution_strategies: vec![
                ResolutionStrategy::TimeAdjustment,
                ResolutionStrategy::ParticipantReplacement,
                ResolutionStrategy::ScheduleRescheduling,
            ],
        }
    }

    pub fn resolve_conflict(&self, conflict: &ScheduleConflict) -> Option<ConflictResolution> {
        match conflict.severity {
            ConflictSeverity::Low => Some(ConflictResolution::AdjustTime {
                adjustment_minutes: 15,
                reason: "Minor time adjustment".to_string(),
            }),
            ConflictSeverity::Medium => Some(ConflictResolution::Reschedule {
                suggested_time: Utc::now() + Duration::hours(1),
                reason: "Schedule conflict detected".to_string(),
            }),
            ConflictSeverity::High => Some(ConflictResolution::ParticipantChange {
                suggested_participants: conflict.common_participants.clone(),
                reason: "Participant overlap detected".to_string(),
            }),
            ConflictSeverity::Critical => Some(ConflictResolution::Reschedule {
                suggested_time: Utc::now() + Duration::hours(2),
                reason: "Critical conflict - immediate rescheduling required".to_string(),
            }),
        }
    }
}

/// Estratégia de resolução
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    TimeAdjustment,
    ParticipantReplacement,
    ScheduleRescheduling,
}

/// Resultado de coordenação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoordinationResult {
    pub conflicts: Vec<ScheduleConflict>,
    pub suggestions: Vec<ConflictResolution>,
    pub optimized_schedules: HashMap<String, OptimizedSchedule>,
    pub coordination_score: f64,
}

/// Conflito de cronograma
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScheduleConflict {
    pub schedule_id1: String,
    pub schedule_id2: String,
    pub conflict_type: ConflictType,
    pub common_participants: Vec<String>,
    pub severity: ConflictSeverity,
    pub suggested_resolution: ConflictResolution,
}

/// Tipo de conflito
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    ParticipantOverlap,
    TimeOverlap,
    TimezoneConflict,
    WorkingHoursConflict,
}

/// Severidade do conflito
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Resolução de conflito
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    AdjustTime {
        adjustment_minutes: i64,
        reason: String,
    },
    Reschedule {
        suggested_time: DateTime<Utc>,
        reason: String,
    },
    ParticipantChange {
        suggested_participants: Vec<String>,
        reason: String,
    },
}

/// Cronograma otimizado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizedSchedule {
    pub original_schedule: GlobalSchedule,
    pub optimized_participants: Vec<OptimizedParticipant>,
    pub optimization_score: f64,
    pub timezone_distribution: TimezoneDistribution,
    pub recommendations: Vec<OptimizationRecommendation>,
}

/// Participante otimizado
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizedParticipant {
    pub participant: GlobalParticipant,
    pub score: f64,
    pub timezone_efficiency: f64,
    pub working_hours_compatibility: f64,
}

/// Distribuição de timezones
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimezoneDistribution {
    pub timezone_count: HashMap<String, usize>,
    pub diversity_score: f64,
    pub most_common_timezone: Option<String>,
}

/// Recomendação de otimização
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
}

/// Categoria da recomendação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationCategory {
    TimezoneDiversity,
    WorkingHours,
    ParticipantOptimization,
    ScheduleOptimization,
}

/// Prioridade da recomendação
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Resultado de sincronização
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyncResult {
    pub reference_timezone: String,
    pub synced_schedules: Vec<String>,
    pub conflicts_resolved: usize,
    pub sync_timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_scheduler_creation() {
        let scheduler = GlobalScheduler::new();
        assert!(scheduler.schedules.is_empty());
    }

    #[test]
    fn test_create_schedule() {
        let mut scheduler = GlobalScheduler::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::hours(2);

        let result = scheduler.create_schedule("project1".to_string(), "UTC".to_string(), start_time, end_time);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert_eq!(schedule.project_id, "project1");
    }

    #[test]
    fn test_create_invalid_schedule() {
        let mut scheduler = GlobalScheduler::new();
        let start_time = Utc::now();
        let end_time = start_time - Duration::hours(1); // End before start

        let result = scheduler.create_schedule("project1".to_string(), "UTC".to_string(), start_time, end_time);

        assert!(result.is_err());
    }

    #[test]
    fn test_find_available_times() {
        let scheduler = GlobalScheduler::new();
        let participants = vec![GlobalParticipant::new(
            "user1".to_string(),
            "User 1".to_string(),
            "UTC".to_string(),
            Utc::now(),
            Utc::now() + Duration::hours(1),
        )];

        let start_date = Utc::now();
        let end_date = start_date + Duration::days(1);

        let result = scheduler.find_available_times(participants, 60, start_date, end_date);

        assert!(result.is_ok());
    }

    #[test]
    fn test_get_scheduling_stats() {
        let mut scheduler = GlobalScheduler::new();
        let start_time = Utc::now();
        let end_time = start_time + Duration::hours(2);

        scheduler
            .create_schedule("project1".to_string(), "UTC".to_string(), start_time, end_time)
            .unwrap();

        let stats = scheduler.get_scheduling_stats();
        assert_eq!(stats.total_schedules, 1);
    }

    #[test]
    fn test_complex_multi_timezone_coordination() {
        let mut scheduler = GlobalScheduler::new();
        
        // Create schedules in different timezones
        let utc_time = Utc::now();
        let schedules = vec![
            ("project1".to_string(), "UTC".to_string(), utc_time, utc_time + Duration::hours(2)),
            ("project2".to_string(), "America/New_York".to_string(), utc_time, utc_time + Duration::hours(2)),
            ("project3".to_string(), "Europe/London".to_string(), utc_time, utc_time + Duration::hours(2)),
        ];

        for (project_id, timezone, start, end) in schedules {
            scheduler.create_schedule(project_id, timezone, start, end).unwrap();
        }

        // Test coordination
        let schedule_ids = vec!["project1".to_string(), "project2".to_string(), "project3".to_string()];
        let result = scheduler.coordinate_schedules(schedule_ids);
        assert!(result.is_ok());
        
        let coordination_result = result.unwrap();
        assert!(coordination_result.coordination_score >= 0.0);
    }

    #[test]
    fn test_conflict_detection_and_resolution() {
        let mut scheduler = GlobalScheduler::new();
        
        // Create overlapping schedules with participants
        let base_time = Utc::now();
        let _schedule1 = scheduler.create_schedule(
            "project1".to_string(),
            "UTC".to_string(),
            base_time,
            base_time + Duration::hours(2)
        ).unwrap();
        
        let _schedule2 = scheduler.create_schedule(
            "project2".to_string(),
            "UTC".to_string(),
            base_time + Duration::minutes(30), // Overlapping
            base_time + Duration::hours(3)
        ).unwrap();

        // Add participants to both schedules
        let participant1 = GlobalParticipant {
            user_id: "user1".to_string(),
            timezone: "UTC".to_string(),
            name: "Alice".to_string(),
            is_required: true,
            local_start_time: base_time,
            local_end_time: base_time + Duration::hours(2),
        };
        let participant2 = GlobalParticipant {
            user_id: "user2".to_string(),
            timezone: "UTC".to_string(),
            name: "Bob".to_string(),
            is_required: true,
            local_start_time: base_time,
            local_end_time: base_time + Duration::hours(2),
        };
        let participant3 = GlobalParticipant {
            user_id: "user1".to_string(),
            timezone: "UTC".to_string(),
            name: "Alice".to_string(),
            is_required: true,
            local_start_time: base_time + Duration::minutes(30),
            local_end_time: base_time + Duration::hours(3),
        };
        let participant4 = GlobalParticipant {
            user_id: "user3".to_string(),
            timezone: "UTC".to_string(),
            name: "Charlie".to_string(),
            is_required: true,
            local_start_time: base_time + Duration::minutes(30),
            local_end_time: base_time + Duration::hours(3),
        };
        
        scheduler.add_participant("project1", participant1).unwrap();
        scheduler.add_participant("project1", participant2).unwrap();
        scheduler.add_participant("project2", participant3).unwrap();
        scheduler.add_participant("project2", participant4).unwrap();

        // Detect conflicts
        let conflicts = scheduler.detect_schedule_conflict("project1", "project2");
        assert!(conflicts.is_some());
        
        if let Some(conflict) = conflicts {
            assert_eq!(conflict.conflict_type, ConflictType::ParticipantOverlap);
        }
    }

    #[test]
    fn test_optimization_recommendations() {
        let mut scheduler = GlobalScheduler::new();
        
        // Create multiple schedules
        let base_time = Utc::now();
        for i in 0..5 {
            scheduler.create_schedule(
                format!("project{}", i),
                "UTC".to_string(),
                base_time + Duration::hours(i as i64),
                base_time + Duration::hours((i + 1) as i64)
            ).unwrap();
        }

        // Test optimization
        let schedule = scheduler.get_schedule("project0").unwrap();
        let optimized = scheduler.optimize_schedule(schedule).unwrap();
        assert!(optimized.optimization_score >= 0.0);
        
        // Test recommendations
        let recommendations = scheduler.generate_optimization_recommendations(schedule);
        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_timezone_distribution_analysis() {
        let mut scheduler = GlobalScheduler::new();
        
        // Create schedules with different timezones
        let timezones = ["UTC", "America/New_York", "Europe/London", "Asia/Tokyo"];
        let base_time = Utc::now();
        
        for (i, timezone) in timezones.iter().enumerate() {
            scheduler.create_schedule(
                format!("project{}", i),
                timezone.to_string(),
                base_time + Duration::hours(i as i64),
                base_time + Duration::hours((i + 1) as i64)
            ).unwrap();
        }

        // Add participants to the first schedule
        let participant = GlobalParticipant {
            user_id: "user1".to_string(),
            timezone: "UTC".to_string(),
            name: "Alice".to_string(),
            is_required: true,
            local_start_time: base_time,
            local_end_time: base_time + Duration::hours(1),
        };
        scheduler.add_participant("project0", participant).unwrap();

        // Test timezone distribution analysis
        let schedule = scheduler.get_schedule("project0").unwrap();
        let distribution = scheduler.analyze_timezone_distribution(schedule);
        assert_eq!(distribution.timezone_count.len(), 1);
        assert!(distribution.most_common_timezone.is_some());
    }

    #[test]
    fn test_working_hours_management() {
        let mut scheduler = GlobalScheduler::new();
        
        // Set working hours for different timezones
        let utc_hours = WorkingHours::new(9, 17, "UTC".to_string());
        let ny_hours = WorkingHours::new(9, 17, "America/New_York".to_string());
        scheduler.set_working_hours("UTC".to_string(), utc_hours);
        scheduler.set_working_hours("America/New_York".to_string(), ny_hours);
        
        // Test working hours retrieval
        let utc_hours = scheduler.get_working_hours("UTC");
        assert!(utc_hours.is_some());
        
        if let Some(hours) = utc_hours {
            assert_eq!(hours.start_hour, 9);
            assert_eq!(hours.end_hour, 17);
        }
    }

    #[test]
    fn test_sync_with_reference_timezone() {
        let mut scheduler = GlobalScheduler::new();
        
        // Create schedules in different timezones
        let base_time = Utc::now();
        scheduler.create_schedule("project1".to_string(), "UTC".to_string(), base_time, base_time + Duration::hours(2)).unwrap();
        scheduler.create_schedule("project2".to_string(), "America/New_York".to_string(), base_time, base_time + Duration::hours(2)).unwrap();
        
        // Test synchronization
        let sync_result = scheduler.sync_with_reference_timezone("UTC");
        assert!(sync_result.is_ok());
        
        let result = sync_result.unwrap();
        assert!(!result.synced_schedules.is_empty());
    }
}
