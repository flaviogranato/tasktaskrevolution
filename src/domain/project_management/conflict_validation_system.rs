//! Sistema de Validação de Conflitos e Dependências Circulares
//!
//! Este módulo implementa um sistema robusto para validação de conflitos
//! e detecção de dependências circulares em projetos.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use super::advanced_dependencies::{AdvancedDependency, AdvancedDependencyGraph, DependencyType};
use super::dependency_calculation_engine::CalculationResult;
use crate::domain::shared::errors::{DomainError, DomainResult};

// ============================================================================
// ENUMS
// ============================================================================

/// Tipo de conflito detectado
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// Dependência circular detectada
    CircularDependency(Vec<String>),
    /// Conflito de datas (sobreposição)
    DateOverlap(String, String, NaiveDate, NaiveDate),
    /// Conflito de recursos (mesmo recurso em tarefas sobrepostas)
    ResourceConflict(String, String, String, NaiveDate, NaiveDate),
    /// Dependência impossível (data de fim antes de início)
    ImpossibleDependency(String, String, NaiveDate, NaiveDate),
    /// Conflito de prioridades
    PriorityConflict(String, String, String, String),
    /// Conflito de capacidade de recursos
    ResourceCapacityExceeded(String, NaiveDate, NaiveDate, u8, u8),
    /// Conflito de restrições de tempo
    TimeConstraintViolation(String, String, String),
}

/// Severidade do conflito
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictSeverity {
    /// Erro crítico que impede a execução
    Critical,
    /// Aviso que pode causar problemas
    Warning,
    /// Informação que pode ser útil
    Info,
}

/// Status da validação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Validação passou sem problemas
    Valid,
    /// Validação falhou com conflitos
    Invalid(Vec<ConflictReport>),
    /// Validação em andamento
    InProgress,
    /// Validação falhou por erro interno
    Error(String),
}

/// Relatório de conflito
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictReport {
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub message: String,
    pub affected_tasks: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

/// Configuração do sistema de validação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validação de dependências circulares habilitada
    pub circular_dependency_check: bool,
    /// Validação de sobreposição de datas habilitada
    pub date_overlap_check: bool,
    /// Validação de conflitos de recursos habilitada
    pub resource_conflict_check: bool,
    /// Validação de capacidade de recursos habilitada
    pub resource_capacity_check: bool,
    /// Validação de restrições de tempo habilitada
    pub time_constraint_check: bool,
    /// Tolerância para conflitos de datas (em dias)
    pub date_tolerance_days: i64,
    /// Capacidade máxima de recursos por dia
    pub max_resource_capacity: u8,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            circular_dependency_check: true,
            date_overlap_check: true,
            resource_conflict_check: true,
            resource_capacity_check: true,
            time_constraint_check: true,
            date_tolerance_days: 0,
            max_resource_capacity: 100,
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Sistema de validação de conflitos
#[derive(Debug, Clone)]
pub struct ConflictValidationSystem {
    config: ValidationConfig,
    validation_cache: HashMap<String, ValidationStatus>,
    conflict_history: Vec<ConflictReport>,
}

impl ConflictValidationSystem {
    /// Cria um novo sistema de validação
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            config,
            validation_cache: HashMap::new(),
            conflict_history: Vec::new(),
        }
    }

    /// Cria um sistema com configuração padrão
    pub fn with_default_config() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Valida um grafo de dependências completo
    pub fn validate_graph(
        &mut self,
        graph: &AdvancedDependencyGraph,
        calculation_results: &HashMap<String, CalculationResult>,
    ) -> ValidationStatus {
        let mut conflicts = Vec::new();

        // Validação de dependências circulares
        if self.config.circular_dependency_check
            && let Some(circular_conflicts) = self.detect_circular_dependencies(graph)
        {
            conflicts.extend(circular_conflicts);
        }

        // Validação de sobreposição de datas
        if self.config.date_overlap_check
            && let Some(date_conflicts) = self.detect_date_overlaps(calculation_results)
        {
            conflicts.extend(date_conflicts);
        }

        // Validação de conflitos de recursos
        if self.config.resource_conflict_check
            && let Some(resource_conflicts) = self.detect_resource_conflicts(graph, calculation_results)
        {
            conflicts.extend(resource_conflicts);
        }

        // Validação de capacidade de recursos
        if self.config.resource_capacity_check
            && let Some(capacity_conflicts) = self.detect_capacity_conflicts(graph, calculation_results)
        {
            conflicts.extend(capacity_conflicts);
        }

        // Validação de restrições de tempo
        if self.config.time_constraint_check
            && let Some(time_conflicts) = self.detect_time_constraint_violations(graph, calculation_results)
        {
            conflicts.extend(time_conflicts);
        }

        // Validação de dependências impossíveis
        if let Some(impossible_conflicts) = self.detect_impossible_dependencies(graph, calculation_results) {
            conflicts.extend(impossible_conflicts);
        }

        if conflicts.is_empty() {
            ValidationStatus::Valid
        } else {
            // Adicionar conflitos ao histórico
            self.conflict_history.extend(conflicts.clone());
            ValidationStatus::Invalid(conflicts)
        }
    }

    /// Detecta dependências circulares
    fn detect_circular_dependencies(&self, graph: &AdvancedDependencyGraph) -> Option<Vec<ConflictReport>> {
        let mut conflicts = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for task_id in graph.nodes.keys() {
            if !visited.contains(task_id)
                && let Some(cycle) = self.detect_cycle_from_task(task_id, graph, &mut visited, &mut recursion_stack)
            {
                let conflict = ConflictReport {
                    conflict_type: ConflictType::CircularDependency(cycle.clone()),
                    severity: ConflictSeverity::Critical,
                    message: format!("Circular dependency detected: {}", cycle.join(" -> ")),
                    affected_tasks: cycle.clone(),
                    suggested_fixes: vec![
                        "Remove one of the dependencies in the cycle".to_string(),
                        "Restructure the task dependencies".to_string(),
                    ],
                    detected_at: chrono::Utc::now(),
                };
                conflicts.push(conflict);
            }
        }

        if conflicts.is_empty() { None } else { Some(conflicts) }
    }

    /// Detecta ciclo a partir de uma tarefa específica
    #[allow(clippy::only_used_in_recursion)]
    fn detect_cycle_from_task(
        &self,
        task_id: &str,
        graph: &AdvancedDependencyGraph,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        visited.insert(task_id.to_string());
        recursion_stack.insert(task_id.to_string());

        if let Some(deps) = graph.dependencies.get(task_id) {
            for dep in deps {
                if !visited.contains(&dep.successor_id) {
                    if let Some(cycle) = self.detect_cycle_from_task(&dep.successor_id, graph, visited, recursion_stack)
                    {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(&dep.successor_id) {
                    // Ciclo detectado
                    let mut cycle = vec![dep.successor_id.clone()];
                    let mut current = task_id.to_string();
                    while current != dep.successor_id {
                        cycle.push(current.clone());
                        // Encontrar predecessora na pilha de recursão
                        for (pred_id, deps) in &graph.dependencies {
                            if deps.iter().any(|d| d.successor_id == current) {
                                current = pred_id.clone();
                                break;
                            }
                        }
                    }
                    cycle.reverse();
                    return Some(cycle);
                }
            }
        }

        recursion_stack.remove(task_id);
        None
    }

    /// Detecta sobreposições de datas
    fn detect_date_overlaps(
        &self,
        calculation_results: &HashMap<String, CalculationResult>,
    ) -> Option<Vec<ConflictReport>> {
        let mut conflicts = Vec::new();
        let mut task_ranges: Vec<(String, NaiveDate, NaiveDate)> = Vec::new();

        // Coletar intervalos de datas
        for (task_id, result) in calculation_results {
            if let (Some(start), Some(end)) = (result.calculated_start_date, result.calculated_end_date) {
                task_ranges.push((task_id.clone(), start, end));
            }
        }

        // Verificar sobreposições
        for i in 0..task_ranges.len() {
            for j in i + 1..task_ranges.len() {
                let (task1, start1, end1) = &task_ranges[i];
                let (task2, start2, end2) = &task_ranges[j];

                if self.dates_overlap(*start1, *end1, *start2, *end2) {
                    let conflict = ConflictReport {
                        conflict_type: ConflictType::DateOverlap(task1.clone(), task2.clone(), *start1, *end1),
                        severity: ConflictSeverity::Warning,
                        message: format!(
                            "Date overlap between tasks {} and {}: {} - {} overlaps with {} - {}",
                            task1, task2, start1, end1, start2, end2
                        ),
                        affected_tasks: vec![task1.clone(), task2.clone()],
                        suggested_fixes: vec![
                            "Adjust task dates to avoid overlap".to_string(),
                            "Add dependency between tasks".to_string(),
                            "Modify task durations".to_string(),
                        ],
                        detected_at: chrono::Utc::now(),
                    };
                    conflicts.push(conflict);
                }
            }
        }

        if conflicts.is_empty() { None } else { Some(conflicts) }
    }

    /// Verifica se duas datas se sobrepõem
    fn dates_overlap(&self, start1: NaiveDate, end1: NaiveDate, start2: NaiveDate, end2: NaiveDate) -> bool {
        start1 <= end2 && start2 <= end1
    }

    /// Detecta conflitos de recursos
    fn detect_resource_conflicts(
        &self,
        _graph: &AdvancedDependencyGraph,
        _calculation_results: &HashMap<String, CalculationResult>,
    ) -> Option<Vec<ConflictReport>> {
        // Esta implementação seria específica para o sistema de recursos
        // Por enquanto, retorna None pois não temos informações de recursos no grafo
        None
    }

    /// Detecta conflitos de capacidade de recursos
    fn detect_capacity_conflicts(
        &self,
        _graph: &AdvancedDependencyGraph,
        _calculation_results: &HashMap<String, CalculationResult>,
    ) -> Option<Vec<ConflictReport>> {
        // Esta implementação seria específica para o sistema de recursos
        // Por enquanto, retorna None pois não temos informações de recursos no grafo
        None
    }

    /// Detecta violações de restrições de tempo
    fn detect_time_constraint_violations(
        &self,
        _graph: &AdvancedDependencyGraph,
        calculation_results: &HashMap<String, CalculationResult>,
    ) -> Option<Vec<ConflictReport>> {
        let mut conflicts = Vec::new();

        // Verificar se todas as dependências são satisfeitas
        for (task_id, result) in calculation_results {
            if !result.dependencies_satisfied {
                let conflict = ConflictReport {
                    conflict_type: ConflictType::TimeConstraintViolation(
                        task_id.clone(),
                        "dependencies".to_string(),
                        "not_satisfied".to_string(),
                    ),
                    severity: ConflictSeverity::Critical,
                    message: format!("Dependencies not satisfied for task {}", task_id),
                    affected_tasks: vec![task_id.clone()],
                    suggested_fixes: vec![
                        "Check predecessor task completion".to_string(),
                        "Verify dependency relationships".to_string(),
                    ],
                    detected_at: chrono::Utc::now(),
                };
                conflicts.push(conflict);
            }
        }

        if conflicts.is_empty() { None } else { Some(conflicts) }
    }

    /// Detecta dependências impossíveis
    fn detect_impossible_dependencies(
        &self,
        graph: &AdvancedDependencyGraph,
        calculation_results: &HashMap<String, CalculationResult>,
    ) -> Option<Vec<ConflictReport>> {
        let mut conflicts = Vec::new();

        for deps in graph.dependencies.values() {
            for dep in deps {
                if let (Some(pred_result), Some(succ_result)) = (
                    calculation_results.get(&dep.predecessor_id),
                    calculation_results.get(&dep.successor_id),
                ) && let (Some(pred_end), Some(succ_start)) =
                    (pred_result.calculated_end_date, succ_result.calculated_start_date)
                {
                    match dep.dependency_type {
                        DependencyType::FinishToStart => {
                            if pred_end > succ_start {
                                let conflict = ConflictReport {
                                    conflict_type: ConflictType::ImpossibleDependency(
                                        dep.predecessor_id.clone(),
                                        dep.successor_id.clone(),
                                        pred_end,
                                        succ_start,
                                    ),
                                    severity: ConflictSeverity::Critical,
                                    message: format!(
                                        "Impossible dependency: {} finishes after {} starts ({} > {})",
                                        dep.predecessor_id, dep.successor_id, pred_end, succ_start
                                    ),
                                    affected_tasks: vec![dep.predecessor_id.clone(), dep.successor_id.clone()],
                                    suggested_fixes: vec![
                                        "Adjust task dates".to_string(),
                                        "Change dependency type".to_string(),
                                        "Add lag to dependency".to_string(),
                                    ],
                                    detected_at: chrono::Utc::now(),
                                };
                                conflicts.push(conflict);
                            }
                        }
                        DependencyType::StartToStart => {
                            if let (Some(pred_start), Some(succ_start)) =
                                (pred_result.calculated_start_date, succ_result.calculated_start_date)
                                && pred_start > succ_start
                            {
                                let conflict = ConflictReport {
                                    conflict_type: ConflictType::ImpossibleDependency(
                                        dep.predecessor_id.clone(),
                                        dep.successor_id.clone(),
                                        pred_start,
                                        succ_start,
                                    ),
                                    severity: ConflictSeverity::Critical,
                                    message: format!(
                                        "Impossible dependency: {} starts after {} starts ({} > {})",
                                        dep.predecessor_id, dep.successor_id, pred_start, succ_start
                                    ),
                                    affected_tasks: vec![dep.predecessor_id.clone(), dep.successor_id.clone()],
                                    suggested_fixes: vec![
                                        "Adjust task dates".to_string(),
                                        "Change dependency type".to_string(),
                                        "Add lag to dependency".to_string(),
                                    ],
                                    detected_at: chrono::Utc::now(),
                                };
                                conflicts.push(conflict);
                            }
                        }
                        _ => {
                            // Outros tipos de dependência podem ser validados aqui
                        }
                    }
                }
            }
        }

        if conflicts.is_empty() { None } else { Some(conflicts) }
    }

    /// Valida uma dependência específica antes de adicioná-la
    pub fn validate_dependency(
        &self,
        dependency: &AdvancedDependency,
        graph: &AdvancedDependencyGraph,
    ) -> DomainResult<()> {
        // Verificar se criaria ciclo
        if self.would_create_cycle(dependency, graph) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Adding this dependency would create a circular dependency".to_string(),
            });
        }

        // Verificar se as tarefas existem
        if !graph.nodes.contains_key(&dependency.predecessor_id) {
            return Err(DomainError::ValidationError {
                field: "predecessor_id".to_string(),
                message: "Predecessor task does not exist".to_string(),
            });
        }

        if !graph.nodes.contains_key(&dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "successor_id".to_string(),
                message: "Successor task does not exist".to_string(),
            });
        }

        // Verificar se a dependência já existe
        if graph.has_dependency(&dependency.predecessor_id, &dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Dependency already exists".to_string(),
            });
        }

        Ok(())
    }

    /// Verifica se adicionar uma dependência criaria um ciclo
    fn would_create_cycle(&self, dependency: &AdvancedDependency, graph: &AdvancedDependencyGraph) -> bool {
        // Se o successor já é predecessora do predecessor, criaria ciclo
        self.is_predecessor(&dependency.successor_id, &dependency.predecessor_id, graph)
    }

    /// Verifica se uma tarefa é predecessora de outra
    fn is_predecessor(&self, task_id: &str, target_id: &str, graph: &AdvancedDependencyGraph) -> bool {
        if task_id == target_id {
            return false; // Uma tarefa não é predecessora de si mesma
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(task_id.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            if current == target_id {
                return true;
            }

            if let Some(deps) = graph.dependencies.get(&current) {
                for dep in deps {
                    queue.push_back(dep.successor_id.clone());
                }
            }
        }

        false
    }

    /// Obtém histórico de conflitos
    pub fn get_conflict_history(&self) -> &[ConflictReport] {
        &self.conflict_history
    }

    /// Limpa histórico de conflitos
    pub fn clear_history(&mut self) {
        self.conflict_history.clear();
        self.validation_cache.clear();
    }

    /// Atualiza configuração
    pub fn update_config(&mut self, config: ValidationConfig) {
        self.config = config;
        self.validation_cache.clear();
    }

    /// Obtém estatísticas de validação
    pub fn get_validation_stats(&self) -> (usize, usize, usize) {
        let total_conflicts = self.conflict_history.len();
        let critical_conflicts = self
            .conflict_history
            .iter()
            .filter(|c| c.severity == ConflictSeverity::Critical)
            .count();
        let warning_conflicts = self
            .conflict_history
            .iter()
            .filter(|c| c.severity == ConflictSeverity::Warning)
            .count();

        (total_conflicts, critical_conflicts, warning_conflicts)
    }
}

impl fmt::Display for ConflictReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:?}] {} - {:?} (Affected: {})",
            self.severity,
            self.message,
            self.conflict_type,
            self.affected_tasks.join(", ")
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::{LagType, TaskNode};
    use chrono::Duration;
    use chrono::NaiveDate;

    fn create_test_graph() -> AdvancedDependencyGraph {
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new(
            "task1".to_string(),
            "Task 1".to_string(),
            None,
            None,
            Some(Duration::days(5)),
        );
        let task2 = TaskNode::new(
            "task2".to_string(),
            "Task 2".to_string(),
            None,
            None,
            Some(Duration::days(3)),
        );

        graph.add_task(task1);
        graph.add_task(task2);

        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        graph.add_dependency(dep).unwrap();
        graph
    }

    #[test]
    fn test_validation_system_creation() {
        let system = ConflictValidationSystem::with_default_config();
        assert_eq!(system.get_conflict_history().len(), 0);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Criar dependência circular
        let dep1 = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        let dep2 = AdvancedDependency::new(
            "task2".to_string(),
            "task1".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        // Add dependencies - the second one should fail due to circular dependency
        graph.add_dependency(dep1).unwrap();
        let dep2_result = graph.add_dependency(dep2);
        assert!(dep2_result.is_err()); // This should fail due to circular dependency

        let mut system = ConflictValidationSystem::with_default_config();
        let calculation_results = HashMap::new();

        let status = system.validate_graph(&graph, &calculation_results);
        // Since the circular dependency was prevented, the graph should be valid
        assert!(matches!(status, ValidationStatus::Valid));
    }

    #[test]
    fn test_date_overlap_detection() {
        let mut system = ConflictValidationSystem::with_default_config();
        let mut calculation_results = HashMap::new();

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        calculation_results.insert(
            "task1".to_string(),
            CalculationResult {
                task_id: "task1".to_string(),
                calculated_start_date: Some(start_date),
                calculated_end_date: Some(end_date),
                is_critical: false,
                total_float: None,
                free_float: None,
                dependencies_satisfied: true,
                calculation_order: 0,
            },
        );

        calculation_results.insert(
            "task2".to_string(),
            CalculationResult {
                task_id: "task2".to_string(),
                calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
                calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
                is_critical: false,
                total_float: None,
                free_float: None,
                dependencies_satisfied: true,
                calculation_order: 1,
            },
        );

        let graph = AdvancedDependencyGraph::new();
        let status = system.validate_graph(&graph, &calculation_results);
        assert!(matches!(status, ValidationStatus::Invalid(_)));
    }

    #[test]
    fn test_dependency_validation() {
        let system = ConflictValidationSystem::with_default_config();
        let graph = create_test_graph();

        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        // Dependência já existe
        let result = system.validate_dependency(&dep, &graph);
        assert!(result.is_err());
    }

    #[test]
    fn test_conflict_report_display() {
        let report = ConflictReport {
            conflict_type: ConflictType::CircularDependency(vec!["task1".to_string(), "task2".to_string()]),
            severity: ConflictSeverity::Critical,
            message: "Circular dependency detected".to_string(),
            affected_tasks: vec!["task1".to_string(), "task2".to_string()],
            suggested_fixes: vec!["Remove dependency".to_string()],
            detected_at: chrono::Utc::now(),
        };

        let display = format!("{}", report);
        assert!(display.contains("Critical"));
        assert!(display.contains("Circular dependency detected"));
        assert!(display.contains("task1, task2"));
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert!(config.circular_dependency_check);
        assert!(config.date_overlap_check);
        assert!(config.resource_conflict_check);
        assert!(config.resource_capacity_check);
        assert!(config.time_constraint_check);
        assert_eq!(config.date_tolerance_days, 0);
        assert_eq!(config.max_resource_capacity, 100);
    }

    #[test]
    fn test_validation_config_custom() {
        let config = ValidationConfig {
            circular_dependency_check: false,
            date_overlap_check: true,
            resource_conflict_check: false,
            resource_capacity_check: true,
            time_constraint_check: false,
            date_tolerance_days: 5,
            max_resource_capacity: 50,
        };

        assert!(!config.circular_dependency_check);
        assert!(config.date_overlap_check);
        assert!(!config.resource_conflict_check);
        assert!(config.resource_capacity_check);
        assert!(!config.time_constraint_check);
        assert_eq!(config.date_tolerance_days, 5);
        assert_eq!(config.max_resource_capacity, 50);
    }

    #[test]
    fn test_conflict_validation_system_new() {
        let config = ValidationConfig::default();
        let system = ConflictValidationSystem::new(config);
        assert_eq!(system.get_conflict_history().len(), 0);
    }

    #[test]
    fn test_conflict_validation_system_with_default_config() {
        let system = ConflictValidationSystem::with_default_config();
        assert_eq!(system.get_conflict_history().len(), 0);
    }

    #[test]
    fn test_conflict_type_date_overlap() {
        let start1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end1 = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let _start2 = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let _end2 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let conflict = ConflictType::DateOverlap("task1".to_string(), "task2".to_string(), start1, end1);

        match conflict {
            ConflictType::DateOverlap(task1, task2, start, end) => {
                assert_eq!(task1, "task1");
                assert_eq!(task2, "task2");
                assert_eq!(start, start1);
                assert_eq!(end, end1);
            }
            _ => panic!("Expected DateOverlap conflict type"),
        }
    }

    #[test]
    fn test_conflict_type_resource_conflict() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let conflict = ConflictType::ResourceConflict(
            "task1".to_string(),
            "task2".to_string(),
            "resource1".to_string(),
            start,
            end,
        );

        match conflict {
            ConflictType::ResourceConflict(task1, task2, resource, start_date, end_date) => {
                assert_eq!(task1, "task1");
                assert_eq!(task2, "task2");
                assert_eq!(resource, "resource1");
                assert_eq!(start_date, start);
                assert_eq!(end_date, end);
            }
            _ => panic!("Expected ResourceConflict conflict type"),
        }
    }

    #[test]
    fn test_conflict_type_impossible_dependency() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let conflict = ConflictType::ImpossibleDependency("task1".to_string(), "task2".to_string(), start, end);

        match conflict {
            ConflictType::ImpossibleDependency(task1, task2, start_date, end_date) => {
                assert_eq!(task1, "task1");
                assert_eq!(task2, "task2");
                assert_eq!(start_date, start);
                assert_eq!(end_date, end);
            }
            _ => panic!("Expected ImpossibleDependency conflict type"),
        }
    }

    #[test]
    fn test_conflict_type_priority_conflict() {
        let conflict = ConflictType::PriorityConflict(
            "task1".to_string(),
            "task2".to_string(),
            "High".to_string(),
            "Low".to_string(),
        );

        match conflict {
            ConflictType::PriorityConflict(task1, task2, priority1, priority2) => {
                assert_eq!(task1, "task1");
                assert_eq!(task2, "task2");
                assert_eq!(priority1, "High");
                assert_eq!(priority2, "Low");
            }
            _ => panic!("Expected PriorityConflict conflict type"),
        }
    }

    #[test]
    fn test_conflict_type_resource_capacity_exceeded() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let conflict = ConflictType::ResourceCapacityExceeded("resource1".to_string(), start, end, 150, 100);

        match conflict {
            ConflictType::ResourceCapacityExceeded(resource, start_date, end_date, required, available) => {
                assert_eq!(resource, "resource1");
                assert_eq!(start_date, start);
                assert_eq!(end_date, end);
                assert_eq!(required, 150);
                assert_eq!(available, 100);
            }
            _ => panic!("Expected ResourceCapacityExceeded conflict type"),
        }
    }

    #[test]
    fn test_conflict_type_time_constraint_violation() {
        let conflict = ConflictType::TimeConstraintViolation(
            "task1".to_string(),
            "constraint1".to_string(),
            "Must finish by deadline".to_string(),
        );

        match conflict {
            ConflictType::TimeConstraintViolation(task, constraint, message) => {
                assert_eq!(task, "task1");
                assert_eq!(constraint, "constraint1");
                assert_eq!(message, "Must finish by deadline");
            }
            _ => panic!("Expected TimeConstraintViolation conflict type"),
        }
    }

    #[test]
    fn test_conflict_severity_variants() {
        assert_eq!(ConflictSeverity::Critical, ConflictSeverity::Critical);
        assert_eq!(ConflictSeverity::Warning, ConflictSeverity::Warning);
        assert_eq!(ConflictSeverity::Info, ConflictSeverity::Info);
    }

    #[test]
    fn test_validation_status_variants() {
        let valid_status = ValidationStatus::Valid;
        let invalid_status = ValidationStatus::Invalid(vec![]);
        let in_progress_status = ValidationStatus::InProgress;
        let error_status = ValidationStatus::Error("Test error".to_string());

        assert_eq!(valid_status, ValidationStatus::Valid);
        assert!(matches!(invalid_status, ValidationStatus::Invalid(_)));
        assert_eq!(in_progress_status, ValidationStatus::InProgress);
        assert!(matches!(error_status, ValidationStatus::Error(_)));
    }

    #[test]
    fn test_conflict_report_creation() {
        let report = ConflictReport {
            conflict_type: ConflictType::CircularDependency(vec!["task1".to_string()]),
            severity: ConflictSeverity::Warning,
            message: "Test message".to_string(),
            affected_tasks: vec!["task1".to_string()],
            suggested_fixes: vec!["Fix 1".to_string()],
            detected_at: chrono::Utc::now(),
        };

        assert!(matches!(report.conflict_type, ConflictType::CircularDependency(_)));
        assert_eq!(report.severity, ConflictSeverity::Warning);
        assert_eq!(report.message, "Test message");
        assert_eq!(report.affected_tasks.len(), 1);
        assert_eq!(report.suggested_fixes.len(), 1);
    }

    #[test]
    fn test_validate_graph_with_disabled_checks() {
        let config = ValidationConfig {
            circular_dependency_check: false,
            date_overlap_check: false,
            resource_conflict_check: false,
            resource_capacity_check: false,
            time_constraint_check: false,
            ..Default::default()
        };

        let mut system = ConflictValidationSystem::new(config);
        let graph = AdvancedDependencyGraph::new();
        let calculation_results = HashMap::new();

        let status = system.validate_graph(&graph, &calculation_results);
        assert!(matches!(status, ValidationStatus::Valid));
    }

    #[test]
    fn test_validate_graph_with_empty_calculation_results() {
        let mut system = ConflictValidationSystem::with_default_config();
        let graph = AdvancedDependencyGraph::new();
        let calculation_results = HashMap::new();

        let status = system.validate_graph(&graph, &calculation_results);
        assert!(matches!(status, ValidationStatus::Valid));
    }

    #[test]
    fn test_validate_dependency_with_valid_dependency() {
        let system = ConflictValidationSystem::with_default_config();
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        let result = system.validate_dependency(&dep, &graph);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependency_with_nonexistent_task() {
        let system = ConflictValidationSystem::with_default_config();
        let graph = AdvancedDependencyGraph::new();

        let dep = AdvancedDependency::new(
            "nonexistent".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        let result = system.validate_dependency(&dep, &graph);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_conflict_history() {
        let system = ConflictValidationSystem::with_default_config();
        let history = system.get_conflict_history();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_clear_history() {
        let mut system = ConflictValidationSystem::with_default_config();
        system.clear_history();
        // Should not panic and should work without issues
    }

    #[test]
    fn test_get_validation_stats() {
        let config = ValidationConfig::default();
        let system = ConflictValidationSystem::new(config);
        let (total_conflicts, critical_conflicts, warnings) = system.get_validation_stats();
        assert_eq!(total_conflicts, 0);
        assert_eq!(critical_conflicts, 0);
        assert_eq!(warnings, 0);
    }

    #[test]
    fn test_update_config() {
        let mut system = ConflictValidationSystem::with_default_config();
        let new_config = ValidationConfig {
            circular_dependency_check: false,
            date_tolerance_days: 5,
            ..Default::default()
        };

        system.update_config(new_config.clone());
        // Config is updated internally, we can't directly access it
        // but we can verify the system still works
        let graph = AdvancedDependencyGraph::new();
        let calculation_results = HashMap::new();
        let status = system.validate_graph(&graph, &calculation_results);
        assert!(matches!(status, ValidationStatus::Valid));
    }
}
