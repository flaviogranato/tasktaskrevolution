use chrono::{NaiveDate, Duration};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid7::Uuid;

use crate::domain::shared::errors::{DomainError, DomainResult};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    FinishToStart,    // Predecessor must finish before successor starts
    StartToStart,     // Predecessor must start before successor starts
    FinishToFinish,   // Predecessor must finish before successor finishes
    StartToFinish,    // Predecessor must start before successor finishes
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LagType {
    Positive(Duration),  // Delay after dependency
    Negative(Duration),  // Start before dependency
    Zero,               // No lag
}

// ============================================================================
// STRUCTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    pub id: String,
    pub predecessor_id: String,
    pub successor_id: String,
    pub dependency_type: DependencyType,
    pub lag: LagType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, TaskNode>,
    pub edges: HashMap<String, Vec<TaskDependency>>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPathResult {
    pub critical_path: Vec<String>,
    pub total_duration: Duration,
    pub slack_times: HashMap<String, Duration>,
    pub early_start_dates: HashMap<String, NaiveDate>,
    pub late_start_dates: HashMap<String, NaiveDate>,
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl TaskDependency {
    pub fn new(
        predecessor_id: String,
        successor_id: String,
        dependency_type: DependencyType,
        lag: LagType,
        created_by: String,
    ) -> DomainResult<Self> {
        if predecessor_id == successor_id {
            return Err(DomainError::validation_error("dependency", "Task cannot depend on itself"));
        }

        Ok(Self {
            id: Uuid::new_v7().to_string(),
            predecessor_id,
            successor_id,
            dependency_type,
            lag,
            created_at: chrono::Utc::now(),
            created_by,
        })
    }

    pub fn calculate_start_date(
        &self,
        predecessor_start: NaiveDate,
        predecessor_end: NaiveDate,
        predecessor_duration: Duration,
    ) -> DomainResult<NaiveDate> {
        match self.dependency_type {
            DependencyType::FinishToStart => {
                let base_date = predecessor_end;
                self.apply_lag(base_date)
            }
            DependencyType::StartToStart => {
                let base_date = predecessor_start;
                self.apply_lag(base_date)
            }
            DependencyType::FinishToFinish => {
                let base_date = predecessor_end;
                let lag_date = self.apply_lag(base_date)?;
                Ok(lag_date - predecessor_duration)
            }
            DependencyType::StartToFinish => {
                let base_date = predecessor_start;
                let lag_date = self.apply_lag(base_date)?;
                Ok(lag_date - predecessor_duration)
            }
        }
    }

    fn apply_lag(&self, base_date: NaiveDate) -> DomainResult<NaiveDate> {
        match &self.lag {
            LagType::Positive(duration) => {
                Ok(base_date + duration.num_days())
            }
            LagType::Negative(duration) => {
                let days = duration.num_days();
                if days > base_date.signed_duration_since(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap()).num_days() {
                    return Err(DomainError::ValidationError {
                        field: "lag".to_string(),
                        message: "Negative lag would result in invalid date".to_string(),
                    }));
                }
                Ok(base_date - duration.num_days())
            }
            LagType::Zero => Ok(base_date),
        }
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_task(&mut self, task: TaskNode) {
        self.nodes.insert(task.id.clone(), task);
        self.edges.insert(task.id.clone(), Vec::new());
    }

    pub fn add_dependency(&mut self, dependency: TaskDependency) -> DomainResult<()> {
        // Validar se as tarefas existem
        if !self.nodes.contains_key(&dependency.predecessor_id) {
            return Err(DomainError::ValidationError {
                field: "predecessor_id".to_string(),
                message: "Predecessor task does not exist".to_string(),
            }));
        }

        if !self.nodes.contains_key(&dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "successor_id".to_string(),
                message: "Successor task does not exist".to_string(),
            }));
        }

        // Verificar se a dependência já existe
        if self.has_dependency(&dependency.predecessor_id, &dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Dependency already exists".to_string(),
            }));
        }

        // Verificar se criaria ciclo
        if self.would_create_cycle(&dependency.predecessor_id, &dependency.successor_id) {
            return Err(DomainError::ValidationError {
                field: "dependency".to_string(),
                message: "Dependency would create a cycle".to_string(),
            }));
        }

        // Adicionar dependência
        if let Some(edges) = self.edges.get_mut(&dependency.predecessor_id) {
            edges.push(dependency.clone());
        }

        // Atualizar nós
        if let Some(node) = self.nodes.get_mut(&dependency.predecessor_id) {
            node.successors.push(dependency.successor_id.clone());
        }

        if let Some(node) = self.nodes.get_mut(&dependency.successor_id) {
            node.predecessors.push(dependency.predecessor_id.clone());
        }

        Ok(())
    }

    pub fn remove_dependency(&mut self, predecessor_id: &str, successor_id: &str) -> DomainResult<()> {
        // Remover da lista de edges
        if let Some(edges) = self.edges.get_mut(predecessor_id) {
            edges.retain(|d| d.successor_id != successor_id);
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

    pub fn has_dependency(&self, predecessor_id: &str, successor_id: &str) -> bool {
        if let Some(edges) = self.edges.get(predecessor_id) {
            edges.iter().any(|d| d.successor_id == successor_id)
        } else {
            false
        }
    }

    pub fn would_create_cycle(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        self.dfs_cycle_detection(to, from, &mut visited, &mut rec_stack)
    }

    fn dfs_cycle_detection(
        &self,
        current: &str,
        target: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if current == target {
            return true;
        }

        if rec_stack.contains(current) {
            return false;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());
        rec_stack.insert(current.to_string());

        if let Some(edges) = self.edges.get(current) {
            for edge in edges {
                if self.dfs_cycle_detection(&edge.successor_id, target, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(current);
        false
    }

    pub fn calculate_critical_path(&self) -> DomainResult<CriticalPathResult> {
        if self.nodes.is_empty() {
            return Err(DomainError::ValidationError {
                field: "graph".to_string(),
                message: "Cannot calculate critical path for empty graph".to_string(),
            }));
        }

        // Forward pass - calcular early start dates
        let early_start_dates = self.forward_pass()?;

        // Backward pass - calcular late start dates
        let late_start_dates = self.backward_pass(&early_start_dates)?;

        // Calcular slack times
        let slack_times = self.calculate_slack_times(&early_start_dates, &late_start_dates);

        // Identificar critical path
        let critical_path = self.identify_critical_path(&slack_times);

        // Calcular duração total
        let total_duration = self.calculate_total_duration(&early_start_dates);

        Ok(CriticalPathResult {
            critical_path,
            total_duration,
            slack_times,
            early_start_dates,
            late_start_dates,
        })
    }

    fn forward_pass(&self) -> DomainResult<HashMap<String, NaiveDate>> {
        let mut early_start_dates = HashMap::new();
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();

        // Inicializar in-degree para cada nó
        for node_id in self.nodes.keys() {
            let degree = self.nodes.get(node_id)
                .map(|n| n.predecessors.len())
                .unwrap_or(0);
            in_degree.insert(node_id.clone(), degree);

            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // Processar nós em ordem topológica
        while let Some(node_id) = queue.pop_front() {
            let node = self.nodes.get(&node_id)
                .ok_or_else(|| DomainError::ValidationError {
                    field: "node".to_string(),
                    message: format!("Node {} not found", node_id),
                }))?;

            // Calcular early start date
            let early_start = if node.predecessors.is_empty() {
                node.start_date.unwrap_or_else(|| chrono::Utc::now().date_naive())
            } else {
                let mut max_date = None;
                for pred_id in &node.predecessors {
                    if let Some(pred_edges) = self.edges.get(pred_id) {
                        for edge in pred_edges {
                            if edge.successor_id == node_id {
                                let pred_early_start = early_start_dates.get(pred_id)
                                    .ok_or_else(|| DomainError::ValidationError {
                                        field: "predecessor".to_string(),
                                        message: format!("Early start date not found for predecessor {}", pred_id),
                                    }))?;

                                let pred_node = self.nodes.get(pred_id)
                                    .ok_or_else(|| DomainError::ValidationError {
                                        field: "predecessor".to_string(),
                                        message: format!("Predecessor node {} not found", pred_id),
                                    }))?;

                                let pred_duration = pred_node.duration
                                    .unwrap_or_else(|| Duration::days(1));

                                let dependency_start = edge.calculate_start_date(
                                    *pred_early_start,
                                    *pred_early_start + pred_duration,
                                    pred_duration,
                                )?;

                                max_date = Some(max_date.map_or(dependency_start, |d| d.max(dependency_start)));
                            }
                        }
                    }
                }
                max_date.unwrap_or_else(|| chrono::Utc::now().date_naive())
            };

            early_start_dates.insert(node_id.clone(), early_start);

            // Atualizar in-degree dos sucessores
            for edge in self.edges.get(&node_id).unwrap_or(&Vec::new()) {
                let successor_degree = in_degree.get_mut(&edge.successor_id)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "successor".to_string(),
                        message: format!("Successor {} not found in in-degree map", edge.successor_id),
                    }))?;

                *successor_degree -= 1;
                if *successor_degree == 0 {
                    queue.push_back(edge.successor_id.clone());
                }
            }
        }

        Ok(early_start_dates)
    }

    fn backward_pass(&self, early_start_dates: &HashMap<String, NaiveDate>) -> DomainResult<HashMap<String, NaiveDate>> {
        let mut late_start_dates = HashMap::new();
        let mut out_degree = HashMap::new();
        let mut queue = VecDeque::new();

        // Inicializar out-degree para cada nó
        for node_id in self.nodes.keys() {
            let degree = self.nodes.get(node_id)
                .map(|n| n.successors.len())
                .unwrap_or(0);
            out_degree.insert(node_id.clone(), degree);

            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        // Processar nós em ordem topológica reversa
        while let Some(node_id) = queue.pop_front() {
            let node = self.nodes.get(&node_id)
                .ok_or_else(|| DomainError::ValidationError {
                    field: "node".to_string(),
                    message: format!("Node {} not found", node_id),
                }))?;

            // Calcular late start date
            let late_start = if node.successors.is_empty() {
                early_start_dates.get(&node_id)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "early_start".to_string(),
                        message: format!("Early start date not found for node {}", node_id),
                    }))?
                    .clone()
            } else {
                let mut min_date = None;
                for succ_id in &node.successors {
                    if let Some(succ_edges) = self.edges.get(&node_id) {
                        for edge in succ_edges {
                            if edge.successor_id == *succ_id {
                                let succ_late_start = late_start_dates.get(succ_id)
                                    .ok_or_else(|| DomainError::ValidationError {
                                        field: "successor".to_string(),
                                        message: format!("Late start date not found for successor {}", succ_id),
                                    }))?;

                                let node_duration = node.duration
                                    .unwrap_or_else(|| Duration::days(1));

                                let dependency_start = edge.calculate_start_date(
                                    *succ_late_start,
                                    *succ_late_start + node_duration,
                                    node_duration,
                                )?;

                                min_date = Some(min_date.map_or(dependency_start, |d| d.min(dependency_start)));
                            }
                        }
                    }
                }
                min_date.unwrap_or_else(|| chrono::Utc::now().date_naive())
            };

            late_start_dates.insert(node_id.clone(), late_start);

            // Atualizar out-degree dos predecessores
            for pred_id in &node.predecessors {
                let pred_out_degree = out_degree.get_mut(pred_id)
                    .ok_or_else(|| DomainError::ValidationError {
                        field: "predecessor".to_string(),
                        message: format!("Predecessor {} not found in out-degree map", pred_id),
                    }))?;

                *pred_out_degree -= 1;
                if *pred_out_degree == 0 {
                    queue.push_back(pred_id.clone());
                }
            }
        }

        Ok(late_start_dates)
    }

    fn calculate_slack_times(
        &self,
        early_start_dates: &HashMap<String, NaiveDate>,
        late_start_dates: &HashMap<String, NaiveDate>,
    ) -> HashMap<String, Duration> {
        let mut slack_times = HashMap::new();

        for node_id in self.nodes.keys() {
            let early_start = early_start_dates.get(node_id).unwrap_or(&chrono::Utc::now().date_naive());
            let late_start = late_start_dates.get(node_id).unwrap_or(&chrono::Utc::now().date_naive());

            let slack = Duration::days(
                late_start.signed_duration_since(*early_start).num_days()
            );

            slack_times.insert(node_id.clone(), slack);
        }

        slack_times
    }

    fn identify_critical_path(&self, slack_times: &HashMap<String, Duration>) -> Vec<String> {
        let mut critical_path = Vec::new();

        for (node_id, slack) in slack_times {
            if slack.num_days() == 0 {
                critical_path.push(node_id.clone());
            }
        }

        // Ordenar por data de início
        critical_path.sort_by(|a, b| {
            let a_start = self.nodes.get(a).and_then(|n| n.start_date);
            let b_start = self.nodes.get(b).and_then(|n| n.start_date);
            a_start.cmp(&b_start)
        });

        critical_path
    }

    fn calculate_total_duration(&self, early_start_dates: &HashMap<String, NaiveDate>) -> Duration {
        let mut max_end_date = None;

        for (node_id, early_start) in early_start_dates {
            if let Some(node) = self.nodes.get(node_id) {
                if let Some(duration) = node.duration {
                    let end_date = *early_start + duration;
                    max_end_date = Some(max_end_date.map_or(end_date, |d| d.max(end_date)));
                }
            }
        }

        if let Some(max_end) = max_end_date {
            let min_start = early_start_dates.values().min().unwrap_or(&chrono::Utc::now().date_naive());
            Duration::days(max_end.signed_duration_since(*min_start).num_days())
        } else {
            Duration::days(0)
        }
    }

    pub fn get_topological_order(&self) -> DomainResult<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut order = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                if !self.dfs_topological(node_id, &mut visited, &mut rec_stack, &mut order) {
                    return Err(DomainError::ValidationError {
                        field: "graph".to_string(),
                        message: "Graph contains cycles, cannot determine topological order".to_string(),
                    }));
                }
            }
        }

        order.reverse();
        Ok(order)
    }

    fn dfs_topological(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> bool {
        if rec_stack.contains(node_id) {
            return false; // Cycle detected
        }

        if visited.contains(node_id) {
            return true;
        }

        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        if let Some(edges) = self.edges.get(node_id) {
            for edge in edges {
                if !self.dfs_topological(&edge.successor_id, visited, rec_stack, order) {
                    return false;
                }
            }
        }

        rec_stack.remove(node_id);
        order.push(node_id.to_string());
        true
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_dependency_creation() {
        let dependency = TaskDependency::new(
            "TASK-001".to_string(),
            "TASK-002".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        );

        assert!(dependency.is_ok());

        let dependency = dependency.unwrap();
        assert_eq!(dependency.predecessor_id, "TASK-001");
        assert_eq!(dependency.successor_id, "TASK-002");
        assert_eq!(dependency.dependency_type, DependencyType::FinishToStart);
        assert_eq!(dependency.lag, LagType::Zero);
    }

    #[test]
    fn test_task_dependency_self_reference() {
        let result = TaskDependency::new(
            "TASK-001".to_string(),
            "TASK-001".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_dependency_graph_creation() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_add_task_to_graph() {
        let mut graph = DependencyGraph::new();

        let task = TaskNode {
            id: "TASK-001".to_string(),
            name: "Test Task".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(5)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        graph.add_task(task);

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.nodes.contains_key("TASK-001"));
    }

    #[test]
    fn test_add_dependency_to_graph() {
        let mut graph = DependencyGraph::new();

        // Adicionar tarefas
        let task1 = TaskNode {
            id: "TASK-001".to_string(),
            name: "Task 1".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(3)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        let task2 = TaskNode {
            id: "TASK-002".to_string(),
            name: "Task 2".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(2)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência
        let dependency = TaskDependency::new(
            "TASK-001".to_string(),
            "TASK-002".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        ).unwrap();

        let result = graph.add_dependency(dependency);
        assert!(result.is_ok());

        // Verificar se a dependência foi adicionada
        assert!(graph.has_dependency("TASK-001", "TASK-002"));
        assert_eq!(graph.nodes["TASK-001"].successors.len(), 1);
        assert_eq!(graph.nodes["TASK-002"].predecessors.len(), 1);
    }

    #[test]
    fn test_dependency_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // Adicionar tarefas
        let task1 = TaskNode {
            id: "TASK-001".to_string(),
            name: "Task 1".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(3)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        let task2 = TaskNode {
            id: "TASK-002".to_string(),
            name: "Task 2".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(2)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        graph.add_task(task1);
        graph.add_task(task2);

        // Adicionar dependência que criaria ciclo
        let dependency = TaskDependency::new(
            "TASK-002".to_string(),
            "TASK-001".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        ).unwrap();

        // Primeiro adicionar dependência válida
        graph.add_dependency(TaskDependency::new(
            "TASK-001".to_string(),
            "TASK-002".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        ).unwrap()).unwrap();

        // Agora tentar adicionar dependência que criaria ciclo
        let result = graph.add_dependency(dependency);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();

        // Criar grafo simples: A -> B -> C
        let task_a = TaskNode {
            id: "A".to_string(),
            name: "Task A".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(1)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        let task_b = TaskNode {
            id: "B".to_string(),
            name: "Task B".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(1)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        let task_c = TaskNode {
            id: "C".to_string(),
            name: "Task C".to_string(),
            start_date: None,
            end_date: None,
            duration: Some(Duration::days(1)),
            predecessors: Vec::new(),
            successors: Vec::new(),
        };

        graph.add_task(task_a);
        graph.add_task(task_b);
        graph.add_task(task_c);

        // Adicionar dependências
        graph.add_dependency(TaskDependency::new(
            "A".to_string(),
            "B".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        ).unwrap()).unwrap();

        graph.add_dependency(TaskDependency::new(
            "B".to_string(),
            "C".to_string(),
            DependencyType::FinishToStart,
            LagType::Zero,
            "user-001".to_string(),
        ).unwrap()).unwrap();

        let order = graph.get_topological_order().unwrap();

        // Verificar ordem topológica
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "A");
        assert_eq!(order[1], "B");
        assert_eq!(order[2], "C");
    }
}
