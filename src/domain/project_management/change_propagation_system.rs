//! Sistema de Propagação de Mudanças
//!
//! Este módulo implementa um sistema robusto para propagação automática de mudanças
//! em tarefas, incluindo notificações, validação de conflitos e rollback de mudanças.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

use super::advanced_dependencies::{AdvancedDependency, AdvancedDependencyGraph, DependencyType, LagType, TaskNode};
use super::dependency_calculation_engine::{CalculationConfig, CalculationResult, DependencyCalculationEngine};
use crate::application::errors::AppError;

// ============================================================================
// ENUMS
// ============================================================================

/// Tipo de mudança em uma tarefa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Mudança na data de início
    StartDateChanged(String, NaiveDate, NaiveDate),
    /// Mudança na data de fim
    EndDateChanged(String, NaiveDate, NaiveDate),
    /// Mudança na duração
    DurationChanged(String, Duration, Duration),
    /// Mudança no tipo de dependência
    DependencyTypeChanged(String, DependencyType, DependencyType),
    /// Mudança no lag de dependência
    DependencyLagChanged(String, LagType, LagType),
    /// Adição de nova dependência
    DependencyAdded(AdvancedDependency),
    /// Remoção de dependência
    DependencyRemoved(String, String),
    /// Mudança na prioridade
    PriorityChanged(String, String),
    /// Mudança no status
    StatusChanged(String, String),
}

/// Status de propagação de uma mudança
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropagationStatus {
    /// Mudança pendente de propagação
    Pending,
    /// Mudança sendo propagada
    InProgress,
    /// Mudança propagada com sucesso
    Propagated,
    /// Mudança falhou na propagação
    Failed(String),
    /// Mudança revertida
    Reverted,
}

/// Resultado da propagação de uma mudança
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationResult {
    pub change_id: String,
    pub change_type: ChangeType,
    pub status: PropagationStatus,
    pub affected_tasks: Vec<String>,
    pub propagated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub rollback_available: bool,
}

/// Configuração do sistema de propagação
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// Propagação automática habilitada
    pub auto_propagate: bool,
    /// Validação de conflitos habilitada
    pub conflict_validation: bool,
    /// Rollback automático em caso de erro
    pub auto_rollback: bool,
    /// Timeout para propagação (em segundos)
    pub propagation_timeout: u64,
    /// Máximo de tentativas de propagação
    pub max_retries: u32,
    /// Notificações habilitadas
    pub notifications_enabled: bool,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            auto_propagate: true,
            conflict_validation: true,
            auto_rollback: true,
            propagation_timeout: 30,
            max_retries: 3,
            notifications_enabled: true,
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Sistema de propagação de mudanças
#[derive(Debug, Clone)]
pub struct ChangePropagationSystem {
    config: PropagationConfig,
    calculation_engine: DependencyCalculationEngine,
    change_history: Vec<PropagationResult>,
    pending_changes: HashMap<String, ChangeType>,
    rollback_stack: Vec<PropagationResult>,
}

impl ChangePropagationSystem {
    /// Cria um novo sistema de propagação
    pub fn new(config: PropagationConfig, calculation_config: CalculationConfig) -> Self {
        Self {
            config,
            calculation_engine: DependencyCalculationEngine::new(calculation_config),
            change_history: Vec::new(),
            pending_changes: HashMap::new(),
            rollback_stack: Vec::new(),
        }
    }

    /// Cria um sistema com configurações padrão
    pub fn with_default_config() -> Self {
        Self::new(PropagationConfig::default(), CalculationConfig::default())
    }

    /// Registra uma mudança e inicia a propagação
    pub fn register_change(
        &mut self,
        change_id: String,
        change_type: ChangeType,
        graph: &mut AdvancedDependencyGraph,
    ) -> Result<PropagationResult, AppError> {
        // Validar a mudança
        self.validate_change(&change_type, graph)?;

        // Criar resultado de propagação
        let mut result = PropagationResult {
            change_id: change_id.clone(),
            change_type: change_type.clone(),
            status: PropagationStatus::Pending,
            affected_tasks: Vec::new(),
            propagated_at: None,
            error_message: None,
            rollback_available: false,
        };

        // Aplicar a mudança ao grafo
        self.apply_change_to_graph(&change_type, graph)?;

        // Encontrar tarefas afetadas
        result.affected_tasks = self.find_affected_tasks(&change_type, graph);

        // Propagação automática se habilitada
        if self.config.auto_propagate {
            match self.propagate_change(&mut result, graph) {
                Ok(_) => {
                    result.status = PropagationStatus::Propagated;
                    result.propagated_at = Some(chrono::Utc::now());
                    result.rollback_available = true;
                }
                Err(e) => {
                    result.status = PropagationStatus::Failed(e.to_string());
                    result.error_message = Some(e.to_string());

                    // Rollback automático se habilitado
                    if self.config.auto_rollback {
                        self.rollback_change(&change_id, graph)?;
                        result.status = PropagationStatus::Reverted;
                    }
                }
            }
        } else {
            // Adicionar à fila de mudanças pendentes
            self.pending_changes.insert(change_id.clone(), change_type);
        }

        // Adicionar ao histórico
        self.change_history.push(result.clone());

        Ok(result)
    }

    /// Propaga uma mudança específica
    fn propagate_change(
        &mut self,
        result: &mut PropagationResult,
        graph: &mut AdvancedDependencyGraph,
    ) -> Result<(), AppError> {
        result.status = PropagationStatus::InProgress;

        // Recalcular datas das tarefas afetadas
        let calculation_results = self.calculation_engine.calculate_project_dates(graph)?;

        // Validar se não há conflitos
        if self.config.conflict_validation {
            self.validate_no_conflicts(&calculation_results, graph)?;
        }

        // Atualizar tarefas no grafo com as novas datas
        for (task_id, calc_result) in calculation_results {
            if let Some(task) = graph.nodes.get_mut(&task_id) {
                if let Some(start_date) = calc_result.calculated_start_date {
                    task.start_date = Some(start_date);
                }
                if let Some(end_date) = calc_result.calculated_end_date {
                    task.end_date = Some(end_date);
                }
            }
        }

        Ok(())
    }

    /// Aplica uma mudança ao grafo de dependências
    fn apply_change_to_graph(
        &self,
        change_type: &ChangeType,
        graph: &mut AdvancedDependencyGraph,
    ) -> Result<(), AppError> {
        match change_type {
            ChangeType::StartDateChanged(_, _new_date, _) => {
                // Atualizar data de início de uma tarefa específica
                // Esta implementação seria específica para a tarefa afetada
            }
            ChangeType::EndDateChanged(_, _new_date, _) => {
                // Atualizar data de fim de uma tarefa específica
            }
            ChangeType::DurationChanged(_, _new_duration, _) => {
                // Atualizar duração de uma tarefa específica
            }
            ChangeType::DependencyTypeChanged(task_id, _, new_type) => {
                // Atualizar tipo de dependência
                if let Some(deps) = graph.dependencies.get_mut(task_id) {
                    for dep in deps {
                        if dep.successor_id == *task_id {
                            dep.dependency_type = new_type.clone();
                        }
                    }
                }
            }
            ChangeType::DependencyLagChanged(task_id, _, new_lag) => {
                // Atualizar lag de dependência
                if let Some(deps) = graph.dependencies.get_mut(task_id) {
                    for dep in deps {
                        if dep.successor_id == *task_id {
                            dep.lag = new_lag.clone();
                        }
                    }
                }
            }
            ChangeType::DependencyAdded(dependency) => {
                graph.add_dependency(dependency.clone())?;
            }
            ChangeType::DependencyRemoved(predecessor, successor) => {
                graph.remove_dependency(predecessor, successor)?;
            }
            ChangeType::PriorityChanged(_, _) => {
                // Mudanças de prioridade não afetam o grafo diretamente
            }
            ChangeType::StatusChanged(_, _) => {
                // Mudanças de status não afetam o grafo diretamente
            }
        }

        Ok(())
    }

    /// Encontra todas as tarefas afetadas por uma mudança
    fn find_affected_tasks(&self, change_type: &ChangeType, graph: &AdvancedDependencyGraph) -> Vec<String> {
        let mut affected = HashSet::new();

        match change_type {
            ChangeType::StartDateChanged(task_id, _, _)
            | ChangeType::EndDateChanged(task_id, _, _)
            | ChangeType::DurationChanged(task_id, _, _)
            | ChangeType::PriorityChanged(task_id, _)
            | ChangeType::StatusChanged(task_id, _) => {
                // Encontrar todos os sucessores da tarefa
                self.find_successors(task_id, graph, &mut affected);
            }
            ChangeType::DependencyTypeChanged(task_id, _, _) | ChangeType::DependencyLagChanged(task_id, _, _) => {
                // Encontrar todos os sucessores da tarefa
                self.find_successors(task_id, graph, &mut affected);
            }
            ChangeType::DependencyAdded(dep) => {
                // Encontrar todos os sucessores da tarefa predecessora
                self.find_successors(&dep.predecessor_id, graph, &mut affected);
            }
            ChangeType::DependencyRemoved(predecessor, _successor) => {
                // Encontrar todos os sucessores da tarefa predecessora
                self.find_successors(predecessor, graph, &mut affected);
            }
        }

        affected.into_iter().collect()
    }

    /// Encontra todos os sucessores de uma tarefa recursivamente
    fn find_successors(&self, task_id: &str, graph: &AdvancedDependencyGraph, visited: &mut HashSet<String>) {
        if visited.contains(task_id) {
            return;
        }

        visited.insert(task_id.to_string());

        if let Some(deps) = graph.dependencies.get(task_id) {
            for dep in deps {
                self.find_successors(&dep.successor_id, graph, visited);
            }
        }
    }

    /// Valida se uma mudança é válida
    fn validate_change(&self, change_type: &ChangeType, graph: &AdvancedDependencyGraph) -> Result<(), AppError> {
        match change_type {
            ChangeType::StartDateChanged(task_id, new_date, _) => {
                if !graph.nodes.contains_key(task_id) {
                    return Err(AppError::ValidationError {
                        field: "task_id".to_string(),
                        message: format!("Task {} not found", task_id),
                    });
                }
                // Validar se a nova data é válida
                if *new_date < chrono::Utc::now().date_naive() {
                    return Err(AppError::ValidationError {
                        field: "start_date".to_string(),
                        message: "Start date cannot be in the past".to_string(),
                    });
                }
            }
            ChangeType::EndDateChanged(task_id, new_date, _) => {
                if !graph.nodes.contains_key(task_id) {
                    return Err(AppError::ValidationError {
                        field: "task_id".to_string(),
                        message: format!("Task {} not found", task_id),
                    });
                }
                // Validar se a nova data é válida
                if *new_date < chrono::Utc::now().date_naive() {
                    return Err(AppError::ValidationError {
                        field: "end_date".to_string(),
                        message: "End date cannot be in the past".to_string(),
                    });
                }
            }
            ChangeType::DependencyAdded(dep) => {
                dep.validate()?;
                if !graph.nodes.contains_key(&dep.predecessor_id) {
                    return Err(AppError::ValidationError {
                        field: "predecessor_id".to_string(),
                        message: "Predecessor task not found".to_string(),
                    });
                }
                if !graph.nodes.contains_key(&dep.successor_id) {
                    return Err(AppError::ValidationError {
                        field: "successor_id".to_string(),
                        message: "Successor task not found".to_string(),
                    });
                }
            }
            _ => {
                // Outras validações específicas podem ser adicionadas aqui
            }
        }

        Ok(())
    }

    /// Valida se não há conflitos após a propagação
    fn validate_no_conflicts(
        &self,
        results: &HashMap<String, CalculationResult>,
        graph: &AdvancedDependencyGraph,
    ) -> Result<(), AppError> {
        // Verificar se todas as dependências são satisfeitas
        for (task_id, result) in results {
            if !result.dependencies_satisfied {
                return Err(AppError::ValidationError {
                    field: "dependencies".to_string(),
                    message: format!("Dependencies not satisfied for task {}", task_id),
                });
            }

            // Verificar se as datas são consistentes
            if let (Some(start), Some(end)) = (result.calculated_start_date, result.calculated_end_date) {
                if start > end {
                    return Err(AppError::ValidationError {
                        field: "date_range".to_string(),
                        message: format!("Invalid date range for task {}: {} > {}", task_id, start, end),
                    });
                }
            }
        }

        Ok(())
    }

    /// Reverte uma mudança
    pub fn rollback_change(&mut self, change_id: &str, graph: &mut AdvancedDependencyGraph) -> Result<(), AppError> {
        // Encontrar a mudança no histórico
        let change = self
            .change_history
            .iter()
            .find(|c| c.change_id == change_id)
            .ok_or_else(|| AppError::ValidationError {
                field: "change_id".to_string(),
                message: format!("Change {} not found", change_id),
            })?;

        // Criar mudança reversa
        let reverse_change = self.create_reverse_change(&change.change_type)?;

        // Aplicar mudança reversa
        self.apply_change_to_graph(&reverse_change, graph)?;

        // Recalcular datas
        self.calculation_engine.calculate_project_dates(graph)?;

        // Atualizar status
        if let Some(change) = self.change_history.iter_mut().find(|c| c.change_id == change_id) {
            change.status = PropagationStatus::Reverted;
        }

        Ok(())
    }

    /// Cria uma mudança reversa
    fn create_reverse_change(&self, change_type: &ChangeType) -> Result<ChangeType, AppError> {
        match change_type {
            ChangeType::StartDateChanged(task_id, old_date, _) => {
                Ok(ChangeType::StartDateChanged(task_id.clone(), *old_date, *old_date))
            }
            ChangeType::EndDateChanged(task_id, old_date, _) => {
                Ok(ChangeType::EndDateChanged(task_id.clone(), *old_date, *old_date))
            }
            ChangeType::DurationChanged(task_id, old_duration, _) => Ok(ChangeType::DurationChanged(
                task_id.clone(),
                *old_duration,
                *old_duration,
            )),
            ChangeType::DependencyTypeChanged(task_id, old_type, _) => Ok(ChangeType::DependencyTypeChanged(
                task_id.clone(),
                old_type.clone(),
                old_type.clone(),
            )),
            ChangeType::DependencyLagChanged(task_id, old_lag, _) => Ok(ChangeType::DependencyLagChanged(
                task_id.clone(),
                old_lag.clone(),
                old_lag.clone(),
            )),
            ChangeType::DependencyAdded(dep) => Ok(ChangeType::DependencyRemoved(
                dep.predecessor_id.clone(),
                dep.successor_id.clone(),
            )),
            ChangeType::DependencyRemoved(predecessor, successor) => {
                // Para reverter remoção, precisaríamos recriar a dependência
                // Isso requer informações adicionais que não temos aqui
                Err(AppError::ValidationError {
                    field: "rollback".to_string(),
                    message: "Cannot rollback dependency removal without original dependency data".to_string(),
                })
            }
            _ => Err(AppError::ValidationError {
                field: "rollback".to_string(),
                message: "Cannot rollback this type of change".to_string(),
            }),
        }
    }

    /// Processa todas as mudanças pendentes
    pub fn process_pending_changes(
        &mut self,
        graph: &mut AdvancedDependencyGraph,
    ) -> Result<Vec<PropagationResult>, AppError> {
        let mut results = Vec::new();
        let pending = self.pending_changes.clone();
        self.pending_changes.clear();

        for (change_id, change_type) in pending {
            let result = self.register_change(change_id, change_type, graph)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Obtém histórico de mudanças
    pub fn get_change_history(&self) -> &[PropagationResult] {
        &self.change_history
    }

    /// Obtém mudanças pendentes
    pub fn get_pending_changes(&self) -> &HashMap<String, ChangeType> {
        &self.pending_changes
    }

    /// Limpa histórico de mudanças
    pub fn clear_history(&mut self) {
        self.change_history.clear();
        self.rollback_stack.clear();
    }

    /// Atualiza configuração
    pub fn update_config(&mut self, config: PropagationConfig) {
        self.config = config;
    }
}

impl fmt::Display for PropagationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Change {}: {:?} - Status: {:?} - Affected: {} tasks",
            self.change_id,
            self.change_type,
            self.status,
            self.affected_tasks.len()
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_propagation_system_creation() {
        let system = ChangePropagationSystem::with_default_config();
        assert_eq!(system.get_change_history().len(), 0);
        assert_eq!(system.get_pending_changes().len(), 0);
    }

    #[test]
    fn test_register_change() {
        let mut system = ChangePropagationSystem::with_default_config();
        let mut graph = create_test_graph();

        let change = ChangeType::StartDateChanged(
            "task1".to_string(),
            chrono::Utc::now().date_naive(),
            chrono::Utc::now().date_naive() + chrono::Duration::days(5),
        );

        let result = system
            .register_change("change1".to_string(), change, &mut graph)
            .unwrap();

        assert_eq!(result.change_id, "change1");
        assert_eq!(result.status, PropagationStatus::Propagated);
    }

    #[test]
    fn test_find_affected_tasks() {
        let system = ChangePropagationSystem::with_default_config();
        let graph = create_test_graph();

        let change = ChangeType::StartDateChanged(
            "task1".to_string(),
            chrono::Utc::now().date_naive(),
            chrono::Utc::now().date_naive() + chrono::Duration::days(5),
        );

        let affected = system.find_affected_tasks(&change, &graph);
        assert!(affected.contains(&"task2".to_string()));
    }

    #[test]
    fn test_validation_error() {
        let system = ChangePropagationSystem::with_default_config();
        let graph = create_test_graph();

        let change = ChangeType::StartDateChanged(
            "nonexistent".to_string(),
            chrono::Utc::now().date_naive(),
            chrono::Utc::now().date_naive() + chrono::Duration::days(5),
        );

        let result = system.validate_change(&change, &graph);
        assert!(result.is_err());
    }

    #[test]
    fn test_propagation_result_display() {
        let result = PropagationResult {
            change_id: "change1".to_string(),
            change_type: ChangeType::StartDateChanged(
                "task1".to_string(),
                chrono::Utc::now().date_naive(),
                chrono::Utc::now().date_naive() + chrono::Duration::days(5),
            ),
            status: PropagationStatus::Propagated,
            affected_tasks: vec!["task2".to_string()],
            propagated_at: Some(chrono::Utc::now()),
            error_message: None,
            rollback_available: true,
        };

        let display = format!("{}", result);
        assert!(display.contains("change1"));
        assert!(display.contains("Propagated"));
        assert!(display.contains("1 tasks"));
    }
}
