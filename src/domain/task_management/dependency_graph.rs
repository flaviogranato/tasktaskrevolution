use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use super::dependency::{DependencyError, DependencyStatus, DependencyType, TaskDependency};
use crate::domain::shared::errors::DomainResult;

/// Represents a graph of task dependencies for cycle detection and topological sorting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraph {
    /// All dependencies in the graph
    dependencies: Vec<TaskDependency>,
    /// Adjacency list for efficient graph traversal
    adjacency_list: HashMap<String, Vec<String>>,
    /// Reverse adjacency list for finding predecessors
    reverse_adjacency_list: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
        }
    }

    /// Create a dependency graph from a list of dependencies
    pub fn from_dependencies(dependencies: Vec<TaskDependency>) -> DomainResult<Self> {
        let mut graph = Self::new();

        for dependency in dependencies {
            graph.add_dependency(dependency)?;
        }

        Ok(graph)
    }

    /// Add a dependency to the graph
    pub fn add_dependency(&mut self, dependency: TaskDependency) -> DomainResult<()> {
        // Check if dependency already exists
        if self.dependency_exists(&dependency.predecessor, &dependency.successor) {
            return Err(DependencyError::DependencyExists {
                predecessor: dependency.predecessor.clone(),
                successor: dependency.successor.clone(),
            }
            .into());
        }

        // Create a temporary graph to validate no cycles would be introduced
        let mut temp_graph = self.clone();
        temp_graph.dependencies.push(dependency.clone());

        // Update adjacency lists in temp graph
        temp_graph
            .adjacency_list
            .entry(dependency.predecessor.clone())
            .or_default()
            .push(dependency.successor.clone());

        temp_graph
            .reverse_adjacency_list
            .entry(dependency.successor.clone())
            .or_default()
            .push(dependency.predecessor.clone());

        // Validate no cycles would be introduced
        temp_graph.validate_no_cycles()?;

        // If validation passes, add to the real graph
        self.dependencies.push(dependency.clone());

        // Update adjacency lists
        self.adjacency_list
            .entry(dependency.predecessor.clone())
            .or_default()
            .push(dependency.successor.clone());

        self.reverse_adjacency_list
            .entry(dependency.successor.clone())
            .or_default()
            .push(dependency.predecessor.clone());

        Ok(())
    }

    /// Remove a dependency from the graph
    pub fn remove_dependency(&mut self, predecessor: &str, successor: &str) -> DomainResult<()> {
        // Find and remove the dependency
        if let Some(pos) = self
            .dependencies
            .iter()
            .position(|dep| dep.predecessor == predecessor && dep.successor == successor)
        {
            self.dependencies.remove(pos);
        } else {
            return Err(DependencyError::DependencyNotFound {
                id: format!("{} -> {}", predecessor, successor),
            }
            .into());
        }

        // Update adjacency lists
        if let Some(successors) = self.adjacency_list.get_mut(predecessor) {
            successors.retain(|s| s != successor);
            if successors.is_empty() {
                self.adjacency_list.remove(predecessor);
            }
        }

        if let Some(predecessors) = self.reverse_adjacency_list.get_mut(successor) {
            predecessors.retain(|p| p != predecessor);
            if predecessors.is_empty() {
                self.reverse_adjacency_list.remove(successor);
            }
        }

        Ok(())
    }

    /// Check if a dependency exists between two tasks
    pub fn dependency_exists(&self, predecessor: &str, successor: &str) -> bool {
        self.dependencies
            .iter()
            .any(|dep| dep.predecessor == predecessor && dep.successor == successor)
    }

    /// Get all dependencies
    pub fn dependencies(&self) -> &[TaskDependency] {
        &self.dependencies
    }

    /// Get dependencies where the given task is a predecessor
    pub fn get_successors(&self, task_code: &str) -> Vec<&TaskDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.predecessor == task_code)
            .collect()
    }

    /// Get dependencies where the given task is a successor
    pub fn get_predecessors(&self, task_code: &str) -> Vec<&TaskDependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.successor == task_code)
            .collect()
    }

    /// Get all tasks that are involved in dependencies
    pub fn get_all_tasks(&self) -> HashSet<String> {
        let mut tasks = HashSet::new();
        for dep in &self.dependencies {
            tasks.insert(dep.predecessor.clone());
            tasks.insert(dep.successor.clone());
        }
        tasks
    }

    /// Validate that the graph has no cycles
    pub fn validate_no_cycles(&self) -> DomainResult<()> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for task in self.get_all_tasks() {
            if !visited.contains(&task)
                && let Some(cycle) = self.dfs_cycle_detection(&task, &mut visited, &mut recursion_stack)
            {
                return Err(DependencyError::CycleDetected { path: cycle }.into());
            }
        }

        Ok(())
    }

    /// Perform topological sort of tasks based on dependencies
    pub fn topological_sort(&self) -> DomainResult<Vec<String>> {
        // First validate no cycles
        self.validate_no_cycles()?;

        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        for task in self.get_all_tasks() {
            in_degree.insert(task.clone(), 0);
        }

        for dep in &self.dependencies {
            if dep.status == DependencyStatus::Active {
                *in_degree.get_mut(&dep.successor).unwrap() += 1;
            }
        }

        // Find tasks with no incoming dependencies
        for (task, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(task.clone());
            }
        }

        // Process tasks
        while let Some(task) = queue.pop_front() {
            result.push(task.clone());

            // Reduce in-degree for successors
            if let Some(successors) = self.adjacency_list.get(&task) {
                for successor in successors {
                    if let Some(degree) = in_degree.get_mut(successor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(successor.clone());
                        }
                    }
                }
            }
        }

        // Check if all tasks were processed
        if result.len() != self.get_all_tasks().len() {
            return Err(DependencyError::InvalidConfiguration {
                message: "Graph contains cycles or disconnected components".to_string(),
            }
            .into());
        }

        Ok(result)
    }

    /// Get the critical path through the dependency graph
    pub fn get_critical_path(&self) -> DomainResult<Vec<String>> {
        // For now, return topological sort as critical path
        // In a full implementation, this would calculate the longest path
        self.topological_sort()
    }

    /// Get all tasks that can be started (no active dependencies)
    pub fn get_ready_tasks(&self) -> Vec<String> {
        let mut ready_tasks = Vec::new();

        for task in self.get_all_tasks() {
            let has_active_predecessors = self
                .get_predecessors(&task)
                .iter()
                .any(|dep| dep.status == DependencyStatus::Active);

            if !has_active_predecessors {
                ready_tasks.push(task);
            }
        }

        ready_tasks
    }

    /// Get tasks that are blocked by the given task
    pub fn get_blocked_tasks(&self, task_code: &str) -> Vec<String> {
        self.get_successors(task_code)
            .iter()
            .map(|dep| dep.successor.clone())
            .collect()
    }

    /// Find all cycles in the graph (for debugging)
    pub fn find_all_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for task in self.get_all_tasks() {
            if !visited.contains(&task)
                && let Some(cycle) = self.dfs_cycle_detection(&task, &mut visited, &mut recursion_stack)
            {
                cycles.push(cycle);
            }
        }

        cycles
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &self,
        task: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        visited.insert(task.to_string());
        recursion_stack.insert(task.to_string());

        if let Some(successors) = self.adjacency_list.get(task) {
            for successor in successors {
                if !visited.contains(successor) {
                    if let Some(cycle) = self.dfs_cycle_detection(successor, visited, recursion_stack) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(successor) {
                    // Cycle detected
                    let mut cycle = vec![successor.clone()];
                    let mut current = task;
                    while current != successor {
                        cycle.push(current.to_string());
                        // Find the predecessor in the current path
                        if let Some(pred) = self
                            .reverse_adjacency_list
                            .get(current)
                            .and_then(|preds| preds.iter().find(|p| recursion_stack.contains(*p)))
                        {
                            current = pred;
                        } else {
                            break;
                        }
                    }
                    cycle.push(successor.clone());
                    cycle.reverse();
                    return Some(cycle);
                }
            }
        }

        recursion_stack.remove(task);
        None
    }

    /// Get a summary of the dependency graph
    pub fn get_summary(&self) -> DependencyGraphSummary {
        let total_dependencies = self.dependencies.len();
        let active_dependencies = self
            .dependencies
            .iter()
            .filter(|dep| dep.status == DependencyStatus::Active)
            .count();

        let total_tasks = self.get_all_tasks().len();
        let ready_tasks = self.get_ready_tasks().len();

        DependencyGraphSummary {
            total_tasks,
            total_dependencies,
            active_dependencies,
            ready_tasks,
            has_cycles: self.validate_no_cycles().is_err(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary information about the dependency graph
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGraphSummary {
    pub total_tasks: usize,
    pub total_dependencies: usize,
    pub active_dependencies: usize,
    pub ready_tasks: usize,
    pub has_cycles: bool,
}

impl fmt::Display for DependencyGraphSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dependency Graph Summary:\n\
             - Total Tasks: {}\n\
             - Total Dependencies: {}\n\
             - Active Dependencies: {}\n\
             - Ready Tasks: {}\n\
             - Has Cycles: {}",
            self.total_tasks, self.total_dependencies, self.active_dependencies, self.ready_tasks, self.has_cycles
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::dependency::{DependencyStatus, DependencyType};
    use super::*;

    fn create_test_dependency(predecessor: &str, successor: &str) -> TaskDependency {
        TaskDependency::new(
            predecessor.to_string(),
            successor.to_string(),
            DependencyType::FinishToStart,
            "test".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.dependencies().len(), 0);
        assert!(graph.validate_no_cycles().is_ok());
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let dep = create_test_dependency("TASK-1", "TASK-2");

        assert!(graph.add_dependency(dep).is_ok());
        assert_eq!(graph.dependencies().len(), 1);
        assert!(graph.validate_no_cycles().is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // Add dependencies that create a cycle
        graph
            .add_dependency(create_test_dependency("TASK-1", "TASK-2"))
            .unwrap();
        graph
            .add_dependency(create_test_dependency("TASK-2", "TASK-3"))
            .unwrap();

        // This should be rejected because it creates a cycle
        let result = graph.add_dependency(create_test_dependency("TASK-3", "TASK-1"));
        assert!(result.is_err());

        // Verify the graph still has only 2 dependencies (the cycle was rejected)
        assert_eq!(graph.dependencies().len(), 2);
        assert!(graph.validate_no_cycles().is_ok());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();

        // A -> B -> C
        graph.add_dependency(create_test_dependency("A", "B")).unwrap();
        graph.add_dependency(create_test_dependency("B", "C")).unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_ready_tasks() {
        let mut graph = DependencyGraph::new();

        // A -> B, C -> D (no dependencies)
        graph.add_dependency(create_test_dependency("A", "B")).unwrap();
        graph.add_dependency(create_test_dependency("C", "D")).unwrap();

        let ready = graph.get_ready_tasks();
        assert!(ready.contains(&"A".to_string()));
        assert!(ready.contains(&"C".to_string()));
        assert!(!ready.contains(&"B".to_string()));
        assert!(!ready.contains(&"D".to_string()));
    }

    #[test]
    fn test_duplicate_dependency() {
        let mut graph = DependencyGraph::new();

        graph
            .add_dependency(create_test_dependency("TASK-1", "TASK-2"))
            .unwrap();

        // Try to add the same dependency again
        let result = graph.add_dependency(create_test_dependency("TASK-1", "TASK-2"));
        assert!(result.is_err());
    }
}
