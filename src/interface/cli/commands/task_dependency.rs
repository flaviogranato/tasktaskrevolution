use clap::{Args, Subcommand};
use std::collections::HashSet;

use crate::domain::shared::errors::DomainResult;
use crate::domain::task_management::{DependencyGraph, DependencyType, DependencyValidator, TaskDependency};

/// Manage task dependencies
#[derive(Args, Debug)]
pub struct TaskDependencyArgs {
    #[clap(subcommand)]
    pub command: TaskDependencyCommand,
}

#[derive(Subcommand, Debug)]
pub enum TaskDependencyCommand {
    /// Add a dependency between two tasks
    Add {
        /// Code of the predecessor task
        #[clap(short, long)]
        predecessor: String,
        /// Code of the successor task
        #[clap(short, long)]
        successor: String,
        /// Type of dependency (FS, SS, FF, SF)
        #[clap(short, long, default_value = "FS")]
        dependency_type: String,
        /// Lag time in days (optional)
        #[clap(long)]
        lag_days: Option<u32>,
        /// Lead time in days (optional)
        #[clap(long)]
        lead_days: Option<u32>,
        /// Description of the dependency
        #[clap(long)]
        description: Option<String>,
    },
    /// Remove a dependency between two tasks
    Remove {
        /// Code of the predecessor task
        #[clap(short, long)]
        predecessor: String,
        /// Code of the successor task
        #[clap(short, long)]
        successor: String,
    },
    /// List dependencies for a task or project
    List {
        /// Task code to list dependencies for
        #[clap(short, long)]
        task: Option<String>,
        /// Project code to list dependencies for
        #[clap(short, long)]
        project: Option<String>,
    },
    /// Validate the dependency graph
    Validate {
        /// Project code to validate
        #[clap(short, long)]
        project: Option<String>,
    },
    /// Show dependency graph summary
    Summary {
        /// Project code to summarize
        #[clap(short, long)]
        project: Option<String>,
    },
}

impl TaskDependencyCommand {
    /// Execute the dependency command
    pub fn execute(&self, base_path: &str) -> DomainResult<()> {
        match self {
            TaskDependencyCommand::Add {
                predecessor,
                successor,
                dependency_type,
                lag_days,
                lead_days,
                description,
            } => self.add_dependency(
                base_path,
                predecessor,
                successor,
                dependency_type,
                *lag_days,
                *lead_days,
                description.as_deref(),
            ),
            TaskDependencyCommand::Remove { predecessor, successor } => {
                self.remove_dependency(base_path, predecessor, successor)
            }
            TaskDependencyCommand::List { task, project } => {
                self.list_dependencies(base_path, task.as_deref(), project.as_deref())
            }
            TaskDependencyCommand::Validate { project } => self.validate_dependencies(base_path, project.as_deref()),
            TaskDependencyCommand::Summary { project } => self.show_summary(base_path, project.as_deref()),
        }
    }

    fn add_dependency(
        &self,
        base_path: &str,
        predecessor: &str,
        successor: &str,
        dependency_type: &str,
        lag_days: Option<u32>,
        lead_days: Option<u32>,
        description: Option<&str>,
    ) -> DomainResult<()> {
        // Parse dependency type
        let dep_type = DependencyType::from_str(dependency_type)?;

        // Create dependency
        let mut dependency = TaskDependency::new(
            predecessor.to_string(),
            successor.to_string(),
            dep_type,
            "cli".to_string(),
        )?;

        // Add time constraints
        if let Some(lag) = lag_days {
            dependency = dependency.with_lag_time(chrono::Duration::days(lag as i64));
        }

        if let Some(lead) = lead_days {
            dependency = dependency.with_lead_time(chrono::Duration::days(lead as i64));
        }

        // Add description
        if let Some(desc) = description {
            dependency = dependency.with_description(desc.to_string());
        }

        // Load existing dependencies
        let mut graph = self.load_dependency_graph(base_path)?;

        // Validate the new dependency
        let available_tasks = self.get_available_tasks(base_path)?;
        let validator = DependencyValidator::new(available_tasks);
        validator.validate_dependency_addition(&graph, &dependency)?;

        // Add the dependency
        graph.add_dependency(dependency)?;

        // Save the updated graph
        self.save_dependency_graph(base_path, &graph)?;

        println!("✅ Dependency added successfully: {} -> {}", predecessor, successor);
        Ok(())
    }

    fn remove_dependency(&self, base_path: &str, predecessor: &str, successor: &str) -> DomainResult<()> {
        // Load existing dependencies
        let mut graph = self.load_dependency_graph(base_path)?;

        // Remove the dependency
        graph.remove_dependency(predecessor, successor)?;

        // Save the updated graph
        self.save_dependency_graph(base_path, &graph)?;

        println!("✅ Dependency removed successfully: {} -> {}", predecessor, successor);
        Ok(())
    }

    fn list_dependencies(&self, base_path: &str, task: Option<&str>, project: Option<&str>) -> DomainResult<()> {
        let graph = self.load_dependency_graph(base_path)?;

        if let Some(task_code) = task {
            // List dependencies for a specific task
            println!("Dependencies for task: {}", task_code);
            println!("{}", "=".repeat(50));

            let predecessors = graph.get_predecessors(task_code);
            let successors = graph.get_successors(task_code);

            if !predecessors.is_empty() {
                println!("\nPredecessors:");
                for dep in &predecessors {
                    println!("  • {} ({})", dep.human_description(), dep.status);
                }
            }

            if !successors.is_empty() {
                println!("\nSuccessors:");
                for dep in &successors {
                    println!("  • {} ({})", dep.human_description(), dep.status);
                }
            }

            if predecessors.is_empty() && successors.is_empty() {
                println!("No dependencies found for this task.");
            }
        } else if let Some(_project_code) = project {
            // List all dependencies in a project
            println!("All dependencies in project");
            println!("{}", "=".repeat(50));

            let dependencies = graph.dependencies();
            if dependencies.is_empty() {
                println!("No dependencies found in this project.");
            } else {
                for dep in dependencies {
                    println!("• {}", dep.human_description());
                }
            }
        } else {
            // List all dependencies
            println!("All dependencies");
            println!("{}", "=".repeat(50));

            let dependencies = graph.dependencies();
            if dependencies.is_empty() {
                println!("No dependencies found.");
            } else {
                for dep in dependencies {
                    println!("• {}", dep.human_description());
                }
            }
        }

        Ok(())
    }

    fn validate_dependencies(&self, base_path: &str, _project: Option<&str>) -> DomainResult<()> {
        let graph = self.load_dependency_graph(base_path)?;
        let available_tasks = self.get_available_tasks(base_path)?;
        let validator = DependencyValidator::new(available_tasks);

        // Validate the graph
        match validator.validate_graph(&graph) {
            Ok(()) => {
                println!("✅ Dependency graph is valid!");
                let summary = validator.get_validation_summary(&graph);
                println!("{}", summary);
            }
            Err(e) => {
                println!("❌ Dependency graph validation failed:");
                println!("{}", e);
                let summary = validator.get_validation_summary(&graph);
                println!("{}", summary);
                return Err(e);
            }
        }

        Ok(())
    }

    fn show_summary(&self, base_path: &str, _project: Option<&str>) -> DomainResult<()> {
        let graph = self.load_dependency_graph(base_path)?;
        let summary = graph.get_summary();

        println!("{}", summary);

        // Show ready tasks
        let ready_tasks = graph.get_ready_tasks();
        if !ready_tasks.is_empty() {
            println!("\nReady to start:");
            for task in ready_tasks {
                println!("  • {}", task);
            }
        }

        // Show critical path
        if let Ok(critical_path) = graph.get_critical_path() {
            println!("\nCritical path:");
            println!("  {}", critical_path.join(" → "));
        }

        Ok(())
    }

    fn load_dependency_graph(&self, _base_path: &str) -> DomainResult<DependencyGraph> {
        // For now, return an empty graph
        // In a full implementation, this would load from the repository
        Ok(DependencyGraph::new())
    }

    fn save_dependency_graph(&self, _base_path: &str, _graph: &DependencyGraph) -> DomainResult<()> {
        // For now, do nothing
        // In a full implementation, this would save to the repository
        Ok(())
    }

    fn get_available_tasks(&self, _base_path: &str) -> DomainResult<HashSet<String>> {
        // For now, return a mock set of tasks
        // In a full implementation, this would load from the repository
        Ok(HashSet::from([
            "TASK-1".to_string(),
            "TASK-2".to_string(),
            "TASK-3".to_string(),
        ]))
    }
}
