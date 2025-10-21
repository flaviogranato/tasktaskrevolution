use std::collections::HashSet;
use task_task_revolution::domain::task_management::{
    DependencyGraph, DependencyType, DependencyValidator, TaskDependency,
};

#[test]
fn test_dependency_creation() {
    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap();

    assert_eq!(dependency.predecessor, "TASK-1");
    assert_eq!(dependency.successor, "TASK-2");
    assert_eq!(dependency.dependency_type, DependencyType::FinishToStart);
}

#[test]
fn test_dependency_self_reference_fails() {
    let result = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-1".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    );

    assert!(result.is_err());
}

#[test]
fn test_dependency_with_lag_time() {
    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap()
    .with_lag_time(chrono::Duration::days(2));

    assert_eq!(dependency.lag_time, Some(chrono::Duration::days(2)));
}

#[test]
fn test_dependency_graph_creation() {
    let graph = DependencyGraph::new();
    assert_eq!(graph.dependencies().len(), 0);
}

#[test]
fn test_dependency_graph_add_dependency() {
    let mut graph = DependencyGraph::new();
    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap();

    assert!(graph.add_dependency(dependency).is_ok());
    assert_eq!(graph.dependencies().len(), 1);
}

#[test]
fn test_dependency_graph_cycle_detection() {
    let mut graph = DependencyGraph::new();

    // Add dependencies that create a cycle: A -> B -> C -> A
    graph
        .add_dependency(
            TaskDependency::new(
                "A".to_string(),
                "B".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    graph
        .add_dependency(
            TaskDependency::new(
                "B".to_string(),
                "C".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    // This should create a cycle
    let result = graph.add_dependency(
        TaskDependency::new(
            "C".to_string(),
            "A".to_string(),
            DependencyType::FinishToStart,
            "test".to_string(),
        )
        .unwrap(),
    );

    assert!(result.is_err());
}

#[test]
fn test_dependency_graph_topological_sort() {
    let mut graph = DependencyGraph::new();

    // A -> B -> C
    graph
        .add_dependency(
            TaskDependency::new(
                "A".to_string(),
                "B".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    graph
        .add_dependency(
            TaskDependency::new(
                "B".to_string(),
                "C".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted, vec!["A", "B", "C"]);
}

#[test]
fn test_dependency_graph_ready_tasks() {
    let mut graph = DependencyGraph::new();

    // A -> B, C (no dependencies)
    graph
        .add_dependency(
            TaskDependency::new(
                "A".to_string(),
                "B".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    // Add C to the graph by creating a dependency from C to D
    graph
        .add_dependency(
            TaskDependency::new(
                "C".to_string(),
                "D".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    let ready = graph.get_ready_tasks();
    assert!(ready.contains(&"A".to_string()));
    assert!(ready.contains(&"C".to_string()));
    assert!(!ready.contains(&"B".to_string()));
}

#[test]
fn test_dependency_validator() {
    let tasks = HashSet::from(["TASK-1".to_string(), "TASK-2".to_string()]);
    let validator = DependencyValidator::new(tasks);

    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap();

    assert!(validator.validate_dependency(&dependency).is_ok());
}

#[test]
fn test_dependency_validator_missing_task() {
    let tasks = HashSet::from(["TASK-2".to_string()]);
    let validator = DependencyValidator::new(tasks);

    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap();

    assert!(validator.validate_dependency(&dependency).is_err());
}

#[test]
fn test_dependency_types() {
    assert_eq!(DependencyType::from_str("FS").unwrap(), DependencyType::FinishToStart);
    assert_eq!(DependencyType::from_str("SS").unwrap(), DependencyType::StartToStart);
    assert_eq!(DependencyType::from_str("FF").unwrap(), DependencyType::FinishToFinish);
    assert_eq!(DependencyType::from_str("SF").unwrap(), DependencyType::StartToFinish);

    assert!(DependencyType::from_str("INVALID").is_err());
}

#[test]
fn test_dependency_human_description() {
    let dependency = TaskDependency::new(
        "TASK-1".to_string(),
        "TASK-2".to_string(),
        DependencyType::FinishToStart,
        "test".to_string(),
    )
    .unwrap()
    .with_lag_time(chrono::Duration::days(2));

    let description = dependency.human_description();
    assert!(description.contains("TASK-1"));
    assert!(description.contains("TASK-2"));
    assert!(description.contains("2d"));
}

#[test]
fn test_dependency_graph_summary() {
    let mut graph = DependencyGraph::new();

    graph
        .add_dependency(
            TaskDependency::new(
                "A".to_string(),
                "B".to_string(),
                DependencyType::FinishToStart,
                "test".to_string(),
            )
            .unwrap(),
        )
        .unwrap();

    let summary = graph.get_summary();
    assert_eq!(summary.total_tasks, 2);
    assert_eq!(summary.total_dependencies, 1);
    assert_eq!(summary.active_dependencies, 1);
    assert_eq!(summary.ready_tasks, 1); // A is ready
    assert!(!summary.has_cycles);
}
