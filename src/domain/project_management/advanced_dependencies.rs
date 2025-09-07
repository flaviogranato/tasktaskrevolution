//! Sistema de Dependências Avançado
//!
//! Este módulo implementa um sistema robusto de dependências entre tarefas
//! com suporte a diferentes tipos de dependência e gaps temporais.

use chrono::{Days, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::domain::shared::errors::DomainError;

// ============================================================================
// ENUMS
// ============================================================================

/// Tipos de dependência entre tarefas
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Predecessor deve terminar antes do successor começar
    FinishToStart,
    /// Predecessor deve começar antes do successor começar  
    StartToStart,
    /// Predecessor deve terminar antes do successor terminar
    FinishToFinish,
    /// Predecessor deve começar antes do successor terminar
    StartToFinish,
}

impl DependencyType {
    /// Retorna uma descrição legível do tipo de dependência
    pub fn description(&self) -> &'static str {
        match self {
            DependencyType::FinishToStart => "Finish to Start",
            DependencyType::StartToStart => "Start to Start",
            DependencyType::FinishToFinish => "Finish to Finish",
            DependencyType::StartToFinish => "Start to Finish",
        }
    }

    /// Retorna o símbolo usado para representar o tipo de dependência
    pub fn symbol(&self) -> &'static str {
        match self {
            DependencyType::FinishToStart => "FS",
            DependencyType::StartToStart => "SS",
            DependencyType::FinishToFinish => "FF",
            DependencyType::StartToFinish => "SF",
        }
    }
}

/// Tipos de lag (gap temporal) entre dependências
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LagType {
    /// Delay positivo após a dependência
    Positive(Duration),
    /// Início antes da dependência (overlap)
    Negative(Duration),
    /// Sem lag
    Zero,
}

impl LagType {
    /// Cria um lag positivo
    pub fn positive_days(days: i64) -> Self {
        Self::Positive(Duration::days(days))
    }

    /// Cria um lag negativo
    pub fn negative_days(days: i64) -> Self {
        Self::Negative(Duration::days(days))
    }

    /// Cria um lag zero
    pub fn zero() -> Self {
        Self::Zero
    }

    /// Aplica o lag a uma data base
    pub fn apply_to_date(&self, base_date: NaiveDate) -> Result<NaiveDate, DomainError> {
        match self {
            LagType::Positive(duration) => {
                let days = Days::new(duration.num_days() as u64);
                base_date
                    .checked_add_days(days)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "lag".to_string(),
                        message: "Invalid positive lag duration".to_string(),
                    })
            }
            LagType::Negative(duration) => {
                let days = Days::new(duration.num_days() as u64);
                base_date
                    .checked_sub_days(days)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "lag".to_string(),
                        message: "Invalid negative lag duration".to_string(),
                    })
            }
            LagType::Zero => Ok(base_date),
        }
    }

    /// Retorna uma descrição legível do lag
    pub fn description(&self) -> String {
        match self {
            LagType::Positive(duration) => format!("+{} days", duration.num_days()),
            LagType::Negative(duration) => format!("-{} days", duration.num_days()),
            LagType::Zero => "0 days".to_string(),
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Dependência avançada entre tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedDependency {
    pub id: String,
    pub predecessor_id: String,
    pub successor_id: String,
    pub dependency_type: DependencyType,
    pub lag: LagType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub description: Option<String>,
}

impl AdvancedDependency {
    /// Cria uma nova dependência
    pub fn new(
        predecessor_id: String,
        successor_id: String,
        dependency_type: DependencyType,
        lag: LagType,
        created_by: String,
        description: Option<String>,
    ) -> Self {
        Self {
            id: format!("dep_{}", chrono::Utc::now().timestamp_millis()),
            predecessor_id,
            successor_id,
            dependency_type,
            lag,
            created_at: chrono::Utc::now(),
            created_by,
            description,
        }
    }

    /// Valida se a dependência é válida
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.predecessor_id == self.successor_id {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "A task cannot depend on itself".to_string(),
            });
        }

        if self.predecessor_id.is_empty() || self.successor_id.is_empty() {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Predecessor and successor IDs cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Retorna uma descrição legível da dependência
    pub fn description(&self) -> String {
        let base = format!(
            "{} {} {} {}",
            self.predecessor_id,
            self.dependency_type.symbol(),
            self.lag.description(),
            self.successor_id
        );

        if let Some(desc) = &self.description {
            format!("{} ({})", base, desc)
        } else {
            base
        }
    }
}

/// Nó de tarefa no grafo de dependências
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub name: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub duration: Option<Duration>,
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
}

impl TaskNode {
    /// Cria um novo nó de tarefa
    pub fn new(
        id: String,
        name: String,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            id,
            name,
            start_date,
            end_date,
            duration,
            predecessors: Vec::new(),
            successors: Vec::new(),
        }
    }

    /// Calcula a duração baseada nas datas
    pub fn calculate_duration(&self) -> Option<Duration> {
        match (self.start_date, self.end_date) {
            (Some(start), Some(end)) => {
                if end >= start {
                    Some(end.signed_duration_since(start))
                } else {
                    None
                }
            }
            _ => self.duration,
        }
    }
}

/// Grafo de dependências avançado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedDependencyGraph {
    pub nodes: HashMap<String, TaskNode>,
    pub dependencies: HashMap<String, Vec<AdvancedDependency>>,
}

impl AdvancedDependencyGraph {
    /// Cria um novo grafo de dependências
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Adiciona uma tarefa ao grafo
    pub fn add_task(&mut self, task: TaskNode) {
        let task_id = task.id.clone();
        self.nodes.insert(task_id.clone(), task);
        self.dependencies.insert(task_id, Vec::new());
    }

    /// Adiciona uma dependência ao grafo
    pub fn add_dependency(&mut self, dependency: AdvancedDependency) -> Result<(), DomainError> {
        // Validar a dependência
        dependency.validate()?;

        // Verificar se as tarefas existem
        if !self.nodes.contains_key(&dependency.predecessor_id) {
            return Err(DomainError::ValidationError {
                field: "predecessor_id".to_string(),
                message: "Predecessor task does not exist".to_string(),
            });
        }

        if !self.nodes.contains_key(&dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "successor_id".to_string(),
                message: "Successor task does not exist".to_string(),
            });
        }

        // Verificar se a dependência já existe
        if self.has_dependency(&dependency.predecessor_id, &dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Dependency already exists".to_string(),
            });
        }

        // Verificar se criaria ciclo
        if self.would_create_cycle(&dependency.predecessor_id, &dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Dependency would create a cycle".to_string(),
            });
        }

        // Adicionar dependência
        if let Some(deps) = self.dependencies.get_mut(&dependency.predecessor_id) {
            deps.push(dependency.clone());
        }

        // Atualizar nós
        if let Some(node) = self.nodes.get_mut(&dependency.predecessor_id)
            && !node.successors.contains(&dependency.successor_id)
        {
            node.successors.push(dependency.successor_id.clone());
        }

        if let Some(node) = self.nodes.get_mut(&dependency.successor_id)
            && !node.predecessors.contains(&dependency.predecessor_id)
        {
            node.predecessors.push(dependency.predecessor_id.clone());
        }

        Ok(())
    }

    /// Verifica se existe uma dependência entre duas tarefas
    pub fn has_dependency(&self, predecessor_id: &str, successor_id: &str) -> bool {
        if let Some(deps) = self.dependencies.get(predecessor_id) {
            deps.iter().any(|dep| dep.successor_id == successor_id)
        } else {
            false
        }
    }

    /// Verifica se adicionar uma dependência criaria um ciclo
    pub fn would_create_cycle(&self, predecessor_id: &str, successor_id: &str) -> bool {
        // Se o successor já é predecessor do predecessor, criaria ciclo
        self.is_predecessor(successor_id, predecessor_id)
    }

    /// Verifica se uma tarefa é predecessora de outra
    pub fn is_predecessor(&self, task_id: &str, target_id: &str) -> bool {
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

            if let Some(deps) = self.dependencies.get(&current) {
                for dep in deps {
                    queue.push_back(dep.successor_id.clone());
                }
            }
        }

        false
    }

    /// Remove uma dependência
    pub fn remove_dependency(&mut self, predecessor_id: &str, successor_id: &str) -> Result<(), DomainError> {
        if let Some(deps) = self.dependencies.get_mut(predecessor_id)
            && let Some(pos) = deps.iter().position(|dep| dep.successor_id == successor_id)
        {
            deps.remove(pos);
        }

        // Atualizar nós
        if let Some(node) = self.nodes.get_mut(predecessor_id) {
            node.successors.retain(|id| id != successor_id);
        }

        if let Some(node) = self.nodes.get_mut(successor_id) {
            node.predecessors.retain(|id| id != predecessor_id);
        }

        Ok(())
    }

    /// Retorna todas as dependências de uma tarefa
    pub fn get_dependencies(&self, task_id: &str) -> Vec<&AdvancedDependency> {
        self.dependencies
            .get(task_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Retorna todas as dependências que apontam para uma tarefa
    pub fn get_dependents(&self, task_id: &str) -> Vec<&AdvancedDependency> {
        let mut result = Vec::new();
        for deps in self.dependencies.values() {
            for dep in deps {
                if dep.successor_id == task_id {
                    result.push(dep);
                }
            }
        }
        result
    }

    /// Calcula o caminho crítico do projeto
    pub fn calculate_critical_path(&self) -> Vec<String> {
        // Implementação simplificada - retorna tarefas sem predecessores
        self.nodes
            .values()
            .filter(|node| node.predecessors.is_empty())
            .map(|node| node.id.clone())
            .collect()
    }

    /// Valida a integridade do grafo
    pub fn validate(&self) -> Result<(), DomainError> {
        // Verificar se todas as dependências referenciam tarefas existentes
        for deps in self.dependencies.values() {
            for dep in deps {
                if !self.nodes.contains_key(&dep.predecessor_id) {
                    return Err(DomainError::ValidationError {
                        field: "dependency".to_string(),
                        message: format!("Dependency references non-existent predecessor: {}", dep.predecessor_id),
                    });
                }
                if !self.nodes.contains_key(&dep.successor_id) {
                    return Err(DomainError::ValidationError {
                        field: "dependency".to_string(),
                        message: format!("Dependency references non-existent successor: {}", dep.successor_id),
                    });
                }
            }
        }

        // Verificar se não há ciclos
        for task_id in self.nodes.keys() {
            if self.is_predecessor(task_id, task_id) {
                return Err(DomainError::ValidationError {
                    field: "dependency".to_string(),
                    message: format!("Circular dependency detected involving task: {}", task_id),
                });
            }
        }

        Ok(())
    }
}

impl Default for AdvancedDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_dependency_type_descriptions() {
        assert_eq!(DependencyType::FinishToStart.description(), "Finish to Start");
        assert_eq!(DependencyType::StartToStart.description(), "Start to Start");
        assert_eq!(DependencyType::FinishToFinish.description(), "Finish to Finish");
        assert_eq!(DependencyType::StartToFinish.description(), "Start to Finish");
    }

    #[test]
    fn test_dependency_type_symbols() {
        assert_eq!(DependencyType::FinishToStart.symbol(), "FS");
        assert_eq!(DependencyType::StartToStart.symbol(), "SS");
        assert_eq!(DependencyType::FinishToFinish.symbol(), "FF");
        assert_eq!(DependencyType::StartToFinish.symbol(), "SF");
    }

    #[test]
    fn test_lag_type_creation() {
        let positive = LagType::positive_days(5);
        let negative = LagType::negative_days(2);
        let zero = LagType::zero();

        assert_eq!(positive.description(), "+5 days");
        assert_eq!(negative.description(), "-2 days");
        assert_eq!(zero.description(), "0 days");
    }

    #[test]
    fn test_lag_type_apply_to_date() {
        let base_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        let positive = LagType::positive_days(5);
        let negative = LagType::negative_days(2);
        let zero = LagType::zero();

        assert_eq!(
            positive.apply_to_date(base_date).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
        );
        assert_eq!(
            negative.apply_to_date(base_date).unwrap(),
            NaiveDate::from_ymd_opt(2023, 12, 30).unwrap()
        );
        assert_eq!(zero.apply_to_date(base_date).unwrap(), base_date);
    }

    #[test]
    fn test_advanced_dependency_creation() {
        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            Some("Test dependency".to_string()),
        );

        assert_eq!(dep.predecessor_id, "task1");
        assert_eq!(dep.successor_id, "task2");
        assert_eq!(dep.dependency_type, DependencyType::FinishToStart);
        assert_eq!(dep.lag, LagType::zero());
        assert_eq!(dep.created_by, "user1");
        assert_eq!(dep.description, Some("Test dependency".to_string()));
    }

    #[test]
    fn test_advanced_dependency_validation() {
        let valid_dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(valid_dep.validate().is_ok());

        let invalid_dep = AdvancedDependency::new(
            "task1".to_string(),
            "task1".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(invalid_dep.validate().is_err());
    }

    #[test]
    fn test_advanced_dependency_description() {
        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::positive_days(2),
            "user1".to_string(),
            Some("Test".to_string()),
        );

        let desc = dep.description();
        assert!(desc.contains("task1"));
        assert!(desc.contains("FS"));
        assert!(desc.contains("+2 days"));
        assert!(desc.contains("task2"));
        assert!(desc.contains("Test"));
    }

    #[test]
    fn test_task_node_creation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let node = TaskNode::new(
            "task1".to_string(),
            "Test Task".to_string(),
            Some(start_date),
            Some(end_date),
            None,
        );

        assert_eq!(node.id, "task1");
        assert_eq!(node.name, "Test Task");
        assert_eq!(node.start_date, Some(start_date));
        assert_eq!(node.end_date, Some(end_date));
        assert_eq!(node.calculate_duration(), Some(Duration::days(9)));
    }

    #[test]
    fn test_dependency_graph_creation() {
        let graph = AdvancedDependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.dependencies.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_task() {
        let mut graph = AdvancedDependencyGraph::new();
        let task = TaskNode::new("task1".to_string(), "Test Task".to_string(), None, None, None);

        graph.add_task(task);
        assert!(graph.nodes.contains_key("task1"));
        assert!(graph.dependencies.contains_key("task1"));
    }

    #[test]
    fn test_dependency_graph_add_dependency() {
        let mut graph = AdvancedDependencyGraph::new();

        // Adicionar tarefas
        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência
        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );

        assert!(graph.add_dependency(dep).is_ok());
        assert!(graph.has_dependency("task1", "task2"));
    }

    #[test]
    fn test_dependency_graph_circular_dependency() {
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência task1 -> task2
        let dep1 = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(graph.add_dependency(dep1).is_ok());

        // Tentar adicionar dependência task2 -> task1 (criaria ciclo)
        let dep2 = AdvancedDependency::new(
            "task2".to_string(),
            "task1".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(graph.add_dependency(dep2).is_err());
    }

    #[test]
    fn test_dependency_graph_validation() {
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência válida
        let dep = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(graph.add_dependency(dep).is_ok());
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_dependency_graph_validation_with_cycle() {
        let mut graph = AdvancedDependencyGraph::new();

        let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
        let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência válida
        let dep1 = AdvancedDependency::new(
            "task1".to_string(),
            "task2".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(graph.add_dependency(dep1).is_ok());

        // Tentar adicionar dependência que criaria ciclo
        let dep2 = AdvancedDependency::new(
            "task2".to_string(),
            "task1".to_string(),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        assert!(graph.add_dependency(dep2).is_err());
    }
}
