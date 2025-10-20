use std::collections::HashSet;

use super::dependency::{DependencyError, DependencyType, TaskDependency};
use super::dependency_graph::DependencyGraph;
use crate::domain::shared::errors::DomainResult;

/// Validates task dependencies and ensures graph integrity
pub struct DependencyValidator {
    /// Available task codes for validation
    available_tasks: HashSet<String>,
}

impl DependencyValidator {
    /// Create a new validator with a list of available tasks
    pub fn new(available_tasks: HashSet<String>) -> Self {
        Self { available_tasks }
    }

    /// Validate a single dependency
    pub fn validate_dependency(&self, dependency: &TaskDependency) -> DomainResult<()> {
        // Check if predecessor exists
        if !self.available_tasks.contains(&dependency.predecessor) {
            return Err(DependencyError::TaskNotFound {
                code: dependency.predecessor.clone(),
            }.into());
        }

        // Check if successor exists
        if !self.available_tasks.contains(&dependency.successor) {
            return Err(DependencyError::TaskNotFound {
                code: dependency.successor.clone(),
            }.into());
        }

        // Check for self-dependency
        if dependency.predecessor == dependency.successor {
            return Err(DependencyError::InvalidConfiguration {
                message: "A task cannot depend on itself".to_string(),
            }.into());
        }

        // Validate dependency type specific rules
        self.validate_dependency_type_rules(dependency)?;

        // Validate time constraints
        self.validate_time_constraints(dependency)?;

        Ok(())
    }

    /// Validate a dependency graph
    pub fn validate_graph(&self, graph: &DependencyGraph) -> DomainResult<()> {
        // Validate all individual dependencies
        for dependency in graph.dependencies() {
            self.validate_dependency(dependency)?;
        }

        // Check for cycles
        graph.validate_no_cycles()?;

        // Check for orphaned dependencies
        self.validate_no_orphaned_dependencies(graph)?;

        // Check for redundant dependencies
        self.validate_no_redundant_dependencies(graph)?;

        Ok(())
    }

    /// Validate that adding a dependency to a graph would be valid
    pub fn validate_dependency_addition(
        &self,
        graph: &DependencyGraph,
        new_dependency: &TaskDependency,
    ) -> DomainResult<()> {
        // First validate the dependency itself
        self.validate_dependency(new_dependency)?;

        // Check if dependency already exists
        if graph.dependency_exists(&new_dependency.predecessor, &new_dependency.successor) {
            return Err(DependencyError::DependencyExists {
                predecessor: new_dependency.predecessor.clone(),
                successor: new_dependency.successor.clone(),
            }.into());
        }

        // Create a temporary graph with the new dependency to check for cycles
        let mut temp_graph = graph.clone();
        temp_graph.add_dependency(new_dependency.clone())?;

        Ok(())
    }

    /// Validate dependency type specific rules
    fn validate_dependency_type_rules(&self, dependency: &TaskDependency) -> DomainResult<()> {
        match dependency.dependency_type {
            DependencyType::FinishToStart => {
                // FS: Successor cannot start until predecessor finishes
                // This is the most common and straightforward dependency
                Ok(())
            }
            DependencyType::StartToStart => {
                // SS: Successor cannot start until predecessor starts
                // Both tasks can start at the same time
                Ok(())
            }
            DependencyType::FinishToFinish => {
                // FF: Successor cannot finish until predecessor finishes
                // Both tasks can finish at the same time
                Ok(())
            }
            DependencyType::StartToFinish => {
                // SF: Successor cannot finish until predecessor starts
                // This is the least common dependency type
                Ok(())
            }
        }
    }

    /// Validate time constraints (lag/lead time)
    fn validate_time_constraints(&self, dependency: &TaskDependency) -> DomainResult<()> {
        // Check that both lag and lead time are not specified
        if dependency.lag_time.is_some() && dependency.lead_time.is_some() {
            return Err(DependencyError::InvalidConfiguration {
                message: "Cannot specify both lag time and lead time for the same dependency".to_string(),
            }.into());
        }

        // Validate lag time is positive
        if let Some(lag) = dependency.lag_time {
            if lag.num_seconds() < 0 {
                return Err(DependencyError::InvalidConfiguration {
                    message: "Lag time must be positive".to_string(),
                }.into());
            }
        }

        // Validate lead time is positive
        if let Some(lead) = dependency.lead_time {
            if lead.num_seconds() < 0 {
                return Err(DependencyError::InvalidConfiguration {
                    message: "Lead time must be positive".to_string(),
                }.into());
            }
        }

        Ok(())
    }

    /// Validate that there are no orphaned dependencies
    fn validate_no_orphaned_dependencies(&self, graph: &DependencyGraph) -> DomainResult<()> {
        for dependency in graph.dependencies() {
            if !self.available_tasks.contains(&dependency.predecessor) {
                return Err(DependencyError::InvalidConfiguration {
                    message: format!(
                        "Dependency references non-existent predecessor: {}",
                        dependency.predecessor
                    ),
                }.into());
            }

            if !self.available_tasks.contains(&dependency.successor) {
                return Err(DependencyError::InvalidConfiguration {
                    message: format!(
                        "Dependency references non-existent successor: {}",
                        dependency.successor
                    ),
                }.into());
            }
        }

        Ok(())
    }

    /// Validate that there are no redundant dependencies
    fn validate_no_redundant_dependencies(&self, graph: &DependencyGraph) -> DomainResult<()> {
        let mut seen = HashSet::new();

        for dependency in graph.dependencies() {
            let key = format!("{}->{}", dependency.predecessor, dependency.successor);
            
            if seen.contains(&key) {
                return Err(DependencyError::InvalidConfiguration {
                    message: format!(
                        "Redundant dependency found: {} -> {}",
                        dependency.predecessor, dependency.successor
                    ),
                }.into());
            }

            seen.insert(key);
        }

        Ok(())
    }

    /// Get validation summary for a dependency graph
    pub fn get_validation_summary(&self, graph: &DependencyGraph) -> ValidationSummary {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check for cycles
        if let Err(e) = graph.validate_no_cycles() {
            issues.push(format!("Cycle detected: {}", e));
        }

        // Check for orphaned dependencies
        for dependency in graph.dependencies() {
            if !self.available_tasks.contains(&dependency.predecessor) {
                issues.push(format!(
                    "Orphaned dependency: predecessor '{}' not found",
                    dependency.predecessor
                ));
            }

            if !self.available_tasks.contains(&dependency.successor) {
                issues.push(format!(
                    "Orphaned dependency: successor '{}' not found",
                    dependency.successor
                ));
            }
        }

        // Check for potential issues
        let ready_tasks = graph.get_ready_tasks();
        if ready_tasks.is_empty() && !graph.dependencies().is_empty() {
            warnings.push("No tasks are ready to start - all tasks have dependencies".to_string());
        }

        // Check for tasks with many dependencies
        for task in self.available_tasks.iter() {
            let predecessor_count = graph.get_predecessors(task).len();
            if predecessor_count > 5 {
                warnings.push(format!(
                    "Task '{}' has {} dependencies - consider simplifying",
                    task, predecessor_count
                ));
            }
        }

        ValidationSummary {
            is_valid: issues.is_empty(),
            issues,
            warnings,
            total_dependencies: graph.dependencies().len(),
            ready_tasks: ready_tasks.len(),
        }
    }
}

/// Summary of dependency validation results
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSummary {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub total_dependencies: usize,
    pub ready_tasks: usize,
}

impl std::fmt::Display for ValidationSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dependency Validation Summary:")?;
        writeln!(f, "Valid: {}", self.is_valid)?;
        writeln!(f, "Total Dependencies: {}", self.total_dependencies)?;
        writeln!(f, "Ready Tasks: {}", self.ready_tasks)?;

        if !self.issues.is_empty() {
            writeln!(f, "\nIssues:")?;
            for issue in &self.issues {
                writeln!(f, "  ❌ {}", issue)?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  ⚠️  {}", warning)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::dependency::{DependencyType, DependencyStatus};
    use chrono::Duration;

    fn create_test_dependency(predecessor: &str, successor: &str) -> TaskDependency {
        TaskDependency::new(
            predecessor.to_string(),
            successor.to_string(),
            DependencyType::FinishToStart,
            "test".to_string(),
        ).unwrap()
    }

    #[test]
    fn test_validate_valid_dependency() {
        let tasks = HashSet::from(["TASK-1".to_string(), "TASK-2".to_string()]);
        let validator = DependencyValidator::new(tasks);
        let dependency = create_test_dependency("TASK-1", "TASK-2");

        assert!(validator.validate_dependency(&dependency).is_ok());
    }

    #[test]
    fn test_validate_missing_predecessor() {
        let tasks = HashSet::from(["TASK-2".to_string()]);
        let validator = DependencyValidator::new(tasks);
        let dependency = create_test_dependency("TASK-1", "TASK-2");

        assert!(validator.validate_dependency(&dependency).is_err());
    }

    #[test]
    fn test_validate_self_dependency() {
        let tasks = HashSet::from(["TASK-1".to_string()]);
        let validator = DependencyValidator::new(tasks);
        
        // Create a dependency that would be self-referencing
        let dependency_result = TaskDependency::new(
            "TASK-1".to_string(),
            "TASK-1".to_string(),
            DependencyType::FinishToStart,
            "test".to_string(),
        );

        // The dependency creation itself should fail
        assert!(dependency_result.is_err());
    }

    #[test]
    fn test_validate_duplicate_dependency() {
        let tasks = HashSet::from(["TASK-1".to_string(), "TASK-2".to_string()]);
        let validator = DependencyValidator::new(tasks);
        
        let mut graph = DependencyGraph::new();
        graph.add_dependency(create_test_dependency("TASK-1", "TASK-2")).unwrap();
        
        let duplicate = create_test_dependency("TASK-1", "TASK-2");
        assert!(validator.validate_dependency_addition(&graph, &duplicate).is_err());
    }

    #[test]
    fn test_validation_summary() {
        let tasks = HashSet::from(["TASK-1".to_string(), "TASK-2".to_string()]);
        let validator = DependencyValidator::new(tasks);
        
        let mut graph = DependencyGraph::new();
        graph.add_dependency(create_test_dependency("TASK-1", "TASK-2")).unwrap();
        
        let summary = validator.get_validation_summary(&graph);
        assert!(summary.is_valid);
        assert_eq!(summary.total_dependencies, 1);
        assert_eq!(summary.ready_tasks, 1); // TASK-1 is ready
    }
}
