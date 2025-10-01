//! Engine de Cálculo de Dependências
//!
//! Este módulo implementa um engine robusto para cálculo automático de datas
//! baseado em dependências entre tarefas, incluindo suporte a diferentes tipos
//! de dependência, lags temporais e propagação de mudanças.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use super::advanced_dependencies::{AdvancedDependency, AdvancedDependencyGraph, DependencyType, TaskNode};
use crate::domain::shared::errors::{DomainError, DomainResult};

// ============================================================================
// ENUMS
// ============================================================================

/// Resultado do cálculo de dependências
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculationResult {
    pub task_id: String,
    pub calculated_start_date: Option<NaiveDate>,
    pub calculated_end_date: Option<NaiveDate>,
    pub is_critical: bool,
    pub total_float: Option<Duration>,
    pub free_float: Option<Duration>,
    pub dependencies_satisfied: bool,
    pub calculation_order: usize,
}

/// Status de uma tarefa no cálculo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskCalculationStatus {
    /// Tarefa pronta para cálculo (todas as dependências satisfeitas)
    Ready,
    /// Tarefa aguardando dependências
    Waiting,
    /// Tarefa calculada
    Calculated,
    /// Tarefa com erro no cálculo
    Error(String),
}

/// Configuração do engine de cálculo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationConfig {
    /// Data de início do projeto
    pub project_start_date: NaiveDate,
    /// Duração padrão para tarefas sem duração definida
    pub default_task_duration: Duration,
    /// Considerar apenas dias úteis (excluir fins de semana)
    pub working_days_only: bool,
    /// Horas de trabalho por dia
    pub working_hours_per_day: u8,
    /// Cache habilitado
    pub cache_enabled: bool,
}

impl Default for CalculationConfig {
    fn default() -> Self {
        Self {
            project_start_date: chrono::Utc::now().date_naive(),
            default_task_duration: Duration::days(1),
            working_days_only: false,
            working_hours_per_day: 8,
            cache_enabled: true,
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Engine principal de cálculo de dependências
#[derive(Debug, Clone)]
pub struct DependencyCalculationEngine {
    config: CalculationConfig,
    cache: HashMap<String, CalculationResult>,
    calculation_order: Vec<String>,
}

impl DependencyCalculationEngine {
    /// Cria um novo engine de cálculo
    pub fn new(config: CalculationConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            calculation_order: Vec::new(),
        }
    }

    /// Cria um engine com configuração padrão
    pub fn with_default_config() -> Self {
        Self::new(CalculationConfig::default())
    }

    /// Calcula todas as datas do projeto baseado nas dependências
    pub fn calculate_project_dates(
        &mut self,
        graph: &AdvancedDependencyGraph,
    ) -> DomainResult<HashMap<String, CalculationResult>> {
        // Limpar cache se necessário
        if !self.config.cache_enabled {
            self.cache.clear();
        }

        // Validar o grafo antes do cálculo
        graph.validate()?;

        // Ordenar tarefas topologicamente
        let sorted_tasks = self.topological_sort(graph)?;
        self.calculation_order = sorted_tasks.clone();

        let mut results = HashMap::new();
        let mut task_status = HashMap::new();

        // Inicializar status de todas as tarefas
        for task_id in graph.nodes.keys() {
            task_status.insert(task_id.clone(), TaskCalculationStatus::Waiting);
        }

        // Calcular datas para cada tarefa na ordem topológica
        for (order, task_id) in sorted_tasks.iter().enumerate() {
            if let Some(result) = self.calculate_task_dates(task_id, graph, &results, &mut task_status, order)? {
                results.insert(task_id.clone(), result);
                task_status.insert(task_id.clone(), TaskCalculationStatus::Calculated);
            }
        }

        // Calcular floats e caminho crítico
        self.calculate_floats_and_critical_path(&mut results, graph)?;

        Ok(results)
    }

    /// Calcula as datas de uma tarefa específica
    fn calculate_task_dates(
        &self,
        task_id: &str,
        graph: &AdvancedDependencyGraph,
        calculated_results: &HashMap<String, CalculationResult>,
        task_status: &mut HashMap<String, TaskCalculationStatus>,
        order: usize,
    ) -> DomainResult<Option<CalculationResult>> {
        // Verificar se já está no cache
        if self.config.cache_enabled
            && let Some(cached) = self.cache.get(task_id)
        {
            return Ok(Some(cached.clone()));
        }

        let task = graph.nodes.get(task_id).ok_or_else(|| DomainError::ValidationError {
            field: "task_id".to_string(),
            message: format!("Task {} not found", task_id),
        })?;

        // Verificar se todas as dependências foram calculadas
        let dependencies = graph.get_dependents(task_id);
        let mut all_dependencies_satisfied = true;

        for dep in &dependencies {
            if !calculated_results.contains_key(&dep.predecessor_id) {
                all_dependencies_satisfied = false;
                break;
            }
        }

        if !all_dependencies_satisfied {
            task_status.insert(task_id.to_string(), TaskCalculationStatus::Waiting);
            return Ok(None);
        }

        // Calcular datas baseadas nas dependências
        let (start_date, end_date) = if dependencies.is_empty() {
            // Tarefa sem dependências - usar data de início do projeto
            let start = self.config.project_start_date;
            let duration = task.calculate_duration().unwrap_or(self.config.default_task_duration);
            let end = start + duration;
            (Some(start), Some(end))
        } else {
            // Calcular baseado nas dependências
            self.calculate_dates_from_dependencies(task, &dependencies, calculated_results)?
        };

        let result = CalculationResult {
            task_id: task_id.to_string(),
            calculated_start_date: start_date,
            calculated_end_date: end_date,
            is_critical: false, // Será calculado depois
            total_float: None,  // Será calculado depois
            free_float: None,   // Será calculado depois
            dependencies_satisfied: all_dependencies_satisfied,
            calculation_order: order,
        };

        Ok(Some(result))
    }

    /// Calcula datas baseadas nas dependências
    fn calculate_dates_from_dependencies(
        &self,
        task: &TaskNode,
        dependencies: &[&AdvancedDependency],
        calculated_results: &HashMap<String, CalculationResult>,
    ) -> DomainResult<(Option<NaiveDate>, Option<NaiveDate>)> {
        let mut latest_start_date = None;
        let mut latest_end_date = None;

        for dep in dependencies {
            let predecessor_result =
                calculated_results
                    .get(&dep.predecessor_id)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "predecessor".to_string(),
                        message: format!("Predecessor {} not calculated", dep.predecessor_id),
                    })?;

            let (dep_start, dep_end) = match dep.dependency_type {
                DependencyType::FinishToStart => {
                    // Successor começa após predecessor terminar
                    let base_date =
                        predecessor_result
                            .calculated_end_date
                            .ok_or_else(|| DomainError::ValidationError {
                                field: "predecessor_end_date".to_string(),
                                message: "Predecessor end date not available".to_string(),
                            })?;
                    let adjusted_date = dep.lag.apply_to_date(base_date)?;
                    (Some(adjusted_date), None)
                }
                DependencyType::StartToStart => {
                    // Successor começa quando predecessor começa
                    let base_date =
                        predecessor_result
                            .calculated_start_date
                            .ok_or_else(|| DomainError::ValidationError {
                                field: "predecessor_start_date".to_string(),
                                message: "Predecessor start date not available".to_string(),
                            })?;
                    let adjusted_date = dep.lag.apply_to_date(base_date)?;
                    (Some(adjusted_date), None)
                }
                DependencyType::FinishToFinish => {
                    // Successor termina quando predecessor termina
                    let base_date =
                        predecessor_result
                            .calculated_end_date
                            .ok_or_else(|| DomainError::ValidationError {
                                field: "predecessor_end_date".to_string(),
                                message: "Predecessor end date not available".to_string(),
                            })?;
                    let adjusted_date = dep.lag.apply_to_date(base_date)?;
                    (None, Some(adjusted_date))
                }
                DependencyType::StartToFinish => {
                    // Successor termina quando predecessor começa
                    let base_date =
                        predecessor_result
                            .calculated_start_date
                            .ok_or_else(|| DomainError::ValidationError {
                                field: "predecessor_start_date".to_string(),
                                message: "Predecessor start date not available".to_string(),
                            })?;
                    let adjusted_date = dep.lag.apply_to_date(base_date)?;
                    (None, Some(adjusted_date))
                }
            };

            // Atualizar datas mais restritivas
            if let Some(start) = dep_start {
                latest_start_date = Some(match latest_start_date {
                    Some(current) => std::cmp::max(current, start),
                    None => start,
                });
            }

            if let Some(end) = dep_end {
                latest_end_date = Some(match latest_end_date {
                    Some(current) => std::cmp::max(current, end),
                    None => end,
                });
            }
        }

        // Se não temos data de início calculada, usar a data do projeto
        let start_date = latest_start_date.unwrap_or(self.config.project_start_date);

        // Calcular data de fim baseada na duração
        let duration = task.calculate_duration().unwrap_or(self.config.default_task_duration);
        let calculated_end_date = start_date + duration;

        // Se temos uma data de fim restritiva das dependências, usar a mais tardia
        let final_end_date = match latest_end_date {
            Some(dep_end) => std::cmp::max(calculated_end_date, dep_end),
            None => calculated_end_date,
        };

        Ok((Some(start_date), Some(final_end_date)))
    }

    /// Ordenação topológica das tarefas
    fn topological_sort(&self, graph: &AdvancedDependencyGraph) -> DomainResult<Vec<String>> {
        let mut in_degree = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // Inicializar graus de entrada
        for task_id in graph.nodes.keys() {
            in_degree.insert(task_id.clone(), 0);
        }

        // Calcular graus de entrada
        for deps in graph.dependencies.values() {
            for dep in deps {
                *in_degree.get_mut(&dep.successor_id).unwrap() += 1;
            }
        }

        // Adicionar tarefas sem dependências à fila
        for (task_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(task_id.clone());
            }
        }

        // Processar fila
        while let Some(task_id) = queue.pop_front() {
            result.push(task_id.clone());

            // Reduzir grau de entrada dos sucessores
            if let Some(deps) = graph.dependencies.get(&task_id) {
                for dep in deps {
                    let successor_degree = in_degree.get_mut(&dep.successor_id).unwrap();
                    *successor_degree -= 1;
                    if *successor_degree == 0 {
                        queue.push_back(dep.successor_id.clone());
                    }
                }
            }
        }

        // Verificar se todas as tarefas foram processadas (sem ciclos)
        if result.len() != graph.nodes.len() {
            return Err(DomainError::ValidationError {
                field: "dependency_graph".to_string(),
                message: "Circular dependency detected in project".to_string(),
            });
        }

        Ok(result)
    }

    /// Calcula floats e identifica caminho crítico
    fn calculate_floats_and_critical_path(
        &self,
        results: &mut HashMap<String, CalculationResult>,
        graph: &AdvancedDependencyGraph,
    ) -> DomainResult<()> {
        // Calcular data de fim do projeto
        let project_end_date = results
            .values()
            .filter_map(|r| r.calculated_end_date)
            .max()
            .unwrap_or(self.config.project_start_date);

        // Calcular total float (backward pass)
        for task_id in self.calculation_order.iter().rev() {
            let mut min_successor_start = None;

            // Encontrar menor data de início dos sucessores
            if let Some(deps) = graph.dependencies.get(task_id) {
                for dep in deps {
                    if let Some(successor_result) = results.get(&dep.successor_id)
                        && let Some(successor_start) = successor_result.calculated_start_date
                    {
                        min_successor_start = Some(match min_successor_start {
                            Some(current) => std::cmp::min(current, successor_start),
                            None => successor_start,
                        });
                    }
                }
            }

            // Atualizar resultado com float calculado
            if let Some(result) = results.get_mut(task_id) {
                // Calcular total float
                if let Some(successor_start) = min_successor_start {
                    if let Some(task_end) = result.calculated_end_date {
                        let total_float = successor_start.signed_duration_since(task_end);
                        result.total_float = Some(total_float);
                        result.is_critical = total_float.num_days() == 0;
                    }
                } else {
                    // Tarefa sem sucessores - float é até o fim do projeto
                    if let Some(task_end) = result.calculated_end_date {
                        let total_float = project_end_date.signed_duration_since(task_end);
                        result.total_float = Some(total_float);
                        result.is_critical = total_float.num_days() == 0;
                    }
                }
            }
        }

        Ok(())
    }

    /// Recalcula datas quando uma tarefa é modificada
    pub fn recalculate_affected_tasks(
        &mut self,
        modified_task_id: &str,
        graph: &AdvancedDependencyGraph,
    ) -> DomainResult<HashMap<String, CalculationResult>> {
        // Encontrar todas as tarefas afetadas (sucessores)
        let affected_tasks = self.find_affected_tasks(modified_task_id, graph);

        // Limpar cache das tarefas afetadas
        for task_id in &affected_tasks {
            self.cache.remove(task_id);
        }

        // Recalcular apenas as tarefas afetadas
        self.calculate_project_dates(graph)
    }

    /// Encontra todas as tarefas afetadas por uma mudança
    fn find_affected_tasks(&self, task_id: &str, graph: &AdvancedDependencyGraph) -> Vec<String> {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(task_id.to_string());

        while let Some(current) = queue.pop_front() {
            if affected.contains(&current) {
                continue;
            }
            affected.insert(current.clone());

            // Adicionar sucessores à fila
            if let Some(deps) = graph.dependencies.get(&current) {
                for dep in deps {
                    queue.push_back(dep.successor_id.clone());
                }
            }
        }

        affected.into_iter().collect()
    }

    /// Obtém configuração atual
    pub fn config(&self) -> &CalculationConfig {
        &self.config
    }

    /// Atualiza configuração
    pub fn update_config(&mut self, config: CalculationConfig) {
        self.config = config;
        if !self.config.cache_enabled {
            self.cache.clear();
        }
    }

    /// Limpa cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Obtém estatísticas do cache
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.calculation_order.len())
    }
}

impl fmt::Display for CalculationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task {}: {} -> {} (Critical: {}, Float: {} days)",
            self.task_id,
            self.calculated_start_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            self.calculated_end_date
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "N/A".to_string()),
            self.is_critical,
            self.total_float
                .map(|d| d.num_days().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::LagType;
    use chrono::NaiveDate;

    fn create_test_graph() -> AdvancedDependencyGraph {
        let mut graph = AdvancedDependencyGraph::new();

        // Adicionar tarefas
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
        let task3 = TaskNode::new(
            "task3".to_string(),
            "Task 3".to_string(),
            None,
            None,
            Some(Duration::days(2)),
        );

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        // Adicionar dependências
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
            "task3".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        graph.add_dependency(dep1).unwrap();
        graph.add_dependency(dep2).unwrap();

        graph
    }

    #[test]
    fn test_engine_creation() {
        let engine = DependencyCalculationEngine::with_default_config();
        assert_eq!(engine.cache_stats(), (0, 0));
    }

    #[test]
    fn test_calculation_config_default() {
        let config = CalculationConfig::default();
        assert_eq!(config.working_hours_per_day, 8);
        assert_eq!(config.default_task_duration, Duration::days(1));
        assert!(config.cache_enabled);
    }

    #[test]
    fn test_calculate_project_dates() {
        let mut engine = DependencyCalculationEngine::with_default_config();
        let graph = create_test_graph();

        let results = engine.calculate_project_dates(&graph).unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.contains_key("task1"));
        assert!(results.contains_key("task2"));
        assert!(results.contains_key("task3"));
    }

    #[test]
    fn test_topological_sort() {
        let engine = DependencyCalculationEngine::with_default_config();
        let graph = create_test_graph();

        let sorted = engine.topological_sort(&graph).unwrap();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], "task1"); // Sem dependências
        assert_eq!(sorted[1], "task2"); // Depende de task1
        assert_eq!(sorted[2], "task3"); // Depende de task2
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

        let engine = DependencyCalculationEngine::with_default_config();
        let result = engine.topological_sort(&graph);
        // Since the circular dependency was prevented, the sort should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculation_result_display() {
        let result = CalculationResult {
            task_id: "task1".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: true,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        };

        let display = format!("{}", result);
        assert!(display.contains("task1"));
        assert!(display.contains("2024-01-01"));
        assert!(display.contains("2024-01-05"));
        assert!(display.contains("Critical: true"));
    }

    #[test]
    fn test_calculation_config_custom() {
        let config = CalculationConfig {
            project_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            default_task_duration: Duration::days(2),
            working_days_only: true,
            working_hours_per_day: 6,
            cache_enabled: false,
        };

        assert_eq!(config.project_start_date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(config.default_task_duration, Duration::days(2));
        assert!(config.working_days_only);
        assert_eq!(config.working_hours_per_day, 6);
        assert!(!config.cache_enabled);
    }

    #[test]
    fn test_dependency_calculation_engine_new() {
        let config = CalculationConfig::default();
        let engine = DependencyCalculationEngine::new(config);
        assert_eq!(engine.cache_stats(), (0, 0));
    }

    #[test]
    fn test_dependency_calculation_engine_with_custom_config() {
        let config = CalculationConfig {
            project_start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            default_task_duration: Duration::days(3),
            working_days_only: false,
            working_hours_per_day: 6,
            cache_enabled: true,
        };
        let engine = DependencyCalculationEngine::new(config);
        assert_eq!(engine.cache_stats(), (0, 0));
    }

    #[test]
    fn test_task_calculation_status_variants() {
        let ready_status = TaskCalculationStatus::Ready;
        let waiting_status = TaskCalculationStatus::Waiting;
        let calculated_status = TaskCalculationStatus::Calculated;
        let error_status = TaskCalculationStatus::Error("Test error".to_string());

        assert_eq!(ready_status, TaskCalculationStatus::Ready);
        assert_eq!(waiting_status, TaskCalculationStatus::Waiting);
        assert_eq!(calculated_status, TaskCalculationStatus::Calculated);
        assert!(matches!(error_status, TaskCalculationStatus::Error(_)));
    }

    #[test]
    fn test_calculation_result_creation() {
        let result = CalculationResult {
            task_id: "test_task".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: false,
            total_float: Some(Duration::days(2)),
            free_float: Some(Duration::days(1)),
            dependencies_satisfied: true,
            calculation_order: 1,
        };

        assert_eq!(result.task_id, "test_task");
        assert!(result.calculated_start_date.is_some());
        assert!(result.calculated_end_date.is_some());
        assert!(!result.is_critical);
        assert!(result.total_float.is_some());
        assert!(result.free_float.is_some());
        assert!(result.dependencies_satisfied);
        assert_eq!(result.calculation_order, 1);
    }

    #[test]
    fn test_calculation_result_with_none_dates() {
        let result = CalculationResult {
            task_id: "test_task".to_string(),
            calculated_start_date: None,
            calculated_end_date: None,
            is_critical: false,
            total_float: None,
            free_float: None,
            dependencies_satisfied: false,
            calculation_order: 0,
        };

        assert_eq!(result.task_id, "test_task");
        assert!(result.calculated_start_date.is_none());
        assert!(result.calculated_end_date.is_none());
        assert!(!result.is_critical);
        assert!(result.total_float.is_none());
        assert!(result.free_float.is_none());
        assert!(!result.dependencies_satisfied);
        assert_eq!(result.calculation_order, 0);
    }

    #[test]
    fn test_calculate_project_dates_with_empty_graph() {
        let mut engine = DependencyCalculationEngine::with_default_config();
        let graph = AdvancedDependencyGraph::new();

        let results = engine.calculate_project_dates(&graph).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_calculate_project_dates_with_single_task() {
        let mut engine = DependencyCalculationEngine::with_default_config();
        let mut graph = AdvancedDependencyGraph::new();

        let task = TaskNode::new(
            "single_task".to_string(),
            "Single Task".to_string(),
            None,
            None,
            Some(Duration::days(3)),
        );
        graph.add_task(task);

        let results = engine.calculate_project_dates(&graph).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains_key("single_task"));
    }

    #[test]
    fn test_topological_sort_with_empty_graph() {
        let engine = DependencyCalculationEngine::with_default_config();
        let graph = AdvancedDependencyGraph::new();

        let sorted = engine.topological_sort(&graph).unwrap();
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_topological_sort_with_single_task() {
        let engine = DependencyCalculationEngine::with_default_config();
        let mut graph = AdvancedDependencyGraph::new();

        let task = TaskNode::new("single_task".to_string(), "Single Task".to_string(), None, None, None);
        graph.add_task(task);

        let sorted = engine.topological_sort(&graph).unwrap();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0], "single_task");
    }

    #[test]
    fn test_topological_sort_with_parallel_tasks() {
        let engine = DependencyCalculationEngine::with_default_config();
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);
        let task3 = TaskNode::new("task3".to_string(), "Task 3".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        // Sem dependências - todas as tarefas são independentes
        let sorted = engine.topological_sort(&graph).unwrap();
        assert_eq!(sorted.len(), 3);
        // A ordem pode variar, mas todas as tarefas devem estar presentes
        assert!(sorted.contains(&"task1".to_string()));
        assert!(sorted.contains(&"task2".to_string()));
        assert!(sorted.contains(&"task3".to_string()));
    }

    #[test]
    fn test_calculate_task_dates_with_valid_dependency() {
        let engine = DependencyCalculationEngine::with_default_config();
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

        let calculation_results = HashMap::new();
        let mut task_status = HashMap::new();
        let result = engine.calculate_task_dates("task2", &graph, &calculation_results, &mut task_status, 0);

        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_task_dates_with_nonexistent_task() {
        let engine = DependencyCalculationEngine::with_default_config();
        let graph = AdvancedDependencyGraph::new();
        let calculation_results = HashMap::new();
        let mut task_status = HashMap::new();

        let result = engine.calculate_task_dates("nonexistent", &graph, &calculation_results, &mut task_status, 0);

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_task_dates_with_circular_dependency() {
        let _engine = DependencyCalculationEngine::with_default_config();
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Tentar criar dependência circular (deve falhar no grafo)
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

        graph.add_dependency(dep1).unwrap();
        let dep2_result = graph.add_dependency(dep2);
        assert!(dep2_result.is_err()); // Deve falhar por dependência circular
    }

    #[test]
    fn test_cache_stats() {
        let engine = DependencyCalculationEngine::with_default_config();
        let (hits, misses) = engine.cache_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_clear_cache() {
        let mut engine = DependencyCalculationEngine::with_default_config();
        engine.clear_cache();
        let (hits, misses) = engine.cache_stats();
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
    }

    #[test]
    fn test_get_calculation_config() {
        let config = CalculationConfig::default();
        let engine = DependencyCalculationEngine::new(config.clone());
        let retrieved_config = engine.config();
        assert_eq!(retrieved_config.project_start_date, config.project_start_date);
        assert_eq!(retrieved_config.default_task_duration, config.default_task_duration);
        assert_eq!(retrieved_config.working_hours_per_day, config.working_hours_per_day);
        assert_eq!(retrieved_config.cache_enabled, config.cache_enabled);
        assert_eq!(retrieved_config.working_days_only, config.working_days_only);
    }

    #[test]
    fn test_update_calculation_config() {
        let mut engine = DependencyCalculationEngine::with_default_config();
        let new_config = CalculationConfig {
            working_hours_per_day: 6,
            cache_enabled: false,
            ..Default::default()
        };

        engine.update_config(new_config.clone());
        let retrieved_config = engine.config();
        assert_eq!(retrieved_config.working_hours_per_day, 6);
        assert!(!retrieved_config.cache_enabled);
    }

    #[test]
    fn test_calculation_result_equality() {
        let result1 = CalculationResult {
            task_id: "task1".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: true,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        };

        let result2 = CalculationResult {
            task_id: "task1".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: true,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        };

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_calculation_result_inequality() {
        let result1 = CalculationResult {
            task_id: "task1".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: true,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        };

        let result2 = CalculationResult {
            task_id: "task2".to_string(),
            calculated_start_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            calculated_end_date: Some(NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
            is_critical: true,
            total_float: Some(Duration::days(0)),
            free_float: None,
            dependencies_satisfied: true,
            calculation_order: 0,
        };

        assert_ne!(result1, result2);
    }
}
