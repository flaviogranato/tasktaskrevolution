//! Testes de Integração do Sistema de Cálculo de Dependências
//!
//! Este módulo contém testes abrangentes para todo o sistema de cálculo
//! automático de datas, incluindo engine de cálculo, propagação de mudanças,
//! validação de conflitos e sistema de cache.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use std::fs;

use task_task_revolution::domain::project_management::{
    AdvancedDependency, AdvancedDependencyGraph, DependencyType, LagType, TaskNode,
    DependencyCalculationEngine, CalculationConfig, CalculationResult,
    ChangePropagationSystem, ChangeType, PropagationConfig,
    ConflictValidationSystem, ValidationConfig,
    CalculationCacheSystem, CacheConfig,
};

#[test]
fn test_complete_dependency_calculation_workflow() {
    // Criar grafo de dependências
    let mut graph = AdvancedDependencyGraph::new();
    
    // Adicionar tarefas
    let task1 = TaskNode::new(
        "task1".to_string(),
        "Design Phase".to_string(),
        None,
        None,
        Some(chrono::Duration::days(5)),
    );
    let task2 = TaskNode::new(
        "task2".to_string(),
        "Development Phase".to_string(),
        None,
        None,
        Some(chrono::Duration::days(10)),
    );
    let task3 = TaskNode::new(
        "task3".to_string(),
        "Testing Phase".to_string(),
        None,
        None,
        Some(chrono::Duration::days(3)),
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
        Some("Design must be completed before development".to_string()),
    );
    let dep2 = AdvancedDependency::new(
        "task2".to_string(),
        "task3".to_string(),
        DependencyType::FinishToStart,
        LagType::zero(),
        "user1".to_string(),
        Some("Development must be completed before testing".to_string()),
    );

    graph.add_dependency(dep1).unwrap();
    graph.add_dependency(dep2).unwrap();

    // Configurar engine de cálculo
    let config = CalculationConfig {
        project_start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        default_task_duration: chrono::Duration::days(1),
        working_days_only: false,
        working_hours_per_day: 8,
        cache_enabled: true,
    };

    let mut engine = DependencyCalculationEngine::new(config);

    // Calcular datas do projeto
    let results = engine.calculate_project_dates(&graph).unwrap();

    // Verificar resultados
    assert_eq!(results.len(), 3);
    
    // Verificar que task1 começa na data do projeto
    let task1_result = results.get("task1").unwrap();
    assert_eq!(task1_result.calculated_start_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
    assert_eq!(task1_result.calculated_end_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()));

    // Verificar que task2 começa após task1 terminar
    let task2_result = results.get("task2").unwrap();
    assert_eq!(task2_result.calculated_start_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()));
    assert_eq!(task2_result.calculated_end_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));

    // Verificar que task3 começa após task2 terminar
    let task3_result = results.get("task3").unwrap();
    assert_eq!(task3_result.calculated_start_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 16).unwrap()));
    assert_eq!(task3_result.calculated_end_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 18).unwrap()));
}

#[test]
fn test_circular_dependency_detection() {
    let mut graph = AdvancedDependencyGraph::new();
    
    let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
    let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);
    let task3 = TaskNode::new("task3".to_string(), "Task 3".to_string(), None, None, None);

    graph.add_task(task1);
    graph.add_task(task2);
    graph.add_task(task3);

    // Criar dependência circular: task1 -> task2 -> task3 -> task1
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
    let dep3 = AdvancedDependency::new(
        "task3".to_string(),
        "task1".to_string(),
        DependencyType::FinishToStart,
        LagType::zero(),
        "user1".to_string(),
        None,
    );

    graph.add_dependency(dep1).unwrap();
    graph.add_dependency(dep2).unwrap();
    graph.add_dependency(dep3).unwrap();

    // Tentar calcular datas - deve falhar
    let config = CalculationConfig::default();
    let mut engine = DependencyCalculationEngine::new(config);
    let result = engine.calculate_project_dates(&graph);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Circular dependency"));
}

#[test]
fn test_change_propagation_system() {
    let mut graph = AdvancedDependencyGraph::new();
    
    let task1 = TaskNode::new(
        "task1".to_string(),
        "Task 1".to_string(),
        None,
        None,
        Some(chrono::Duration::days(5)),
    );
    let task2 = TaskNode::new(
        "task2".to_string(),
        "Task 2".to_string(),
        None,
        None,
        Some(chrono::Duration::days(3)),
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

    // Configurar sistema de propagação
    let propagation_config = PropagationConfig::default();
    let calculation_config = CalculationConfig::default();
    let mut propagation_system = ChangePropagationSystem::new(propagation_config, calculation_config);

    // Registrar mudança na data de início da task1
    let change = ChangeType::StartDateChanged(
        "task1".to_string(),
        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
    );

    let result = propagation_system.register_change(
        "change1".to_string(),
        change,
        &mut graph,
    ).unwrap();

    // Verificar que a mudança foi propagada
    assert_eq!(result.status, task_task_revolution::domain::project_management::PropagationStatus::Propagated);
    assert!(result.affected_tasks.contains(&"task2".to_string()));
}

#[test]
fn test_conflict_validation_system() {
    let mut graph = AdvancedDependencyGraph::new();
    
    let task1 = TaskNode::new(
        "task1".to_string(),
        "Task 1".to_string(),
        None,
        None,
        Some(chrono::Duration::days(5)),
    );
    let task2 = TaskNode::new(
        "task2".to_string(),
        "Task 2".to_string(),
        None,
        None,
        Some(chrono::Duration::days(3)),
    );

    graph.add_task(task1);
    graph.add_task(task2);

    // Configurar sistema de validação
    let validation_config = ValidationConfig::default();
    let mut validation_system = ConflictValidationSystem::new(validation_config);

    // Criar resultados de cálculo com sobreposição de datas
    let mut calculation_results = std::collections::HashMap::new();
    
    calculation_results.insert("task1".to_string(), CalculationResult {
        task_id: "task1".to_string(),
        calculated_start_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        calculated_end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 10).unwrap()),
        is_critical: false,
        total_float: None,
        free_float: None,
        dependencies_satisfied: true,
        calculation_order: 0,
    });
    
    calculation_results.insert("task2".to_string(), CalculationResult {
        task_id: "task2".to_string(),
        calculated_start_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
        calculated_end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        is_critical: false,
        total_float: None,
        free_float: None,
        dependencies_satisfied: true,
        calculation_order: 1,
    });

    // Validar grafo
    let status = validation_system.validate_graph(&graph, &calculation_results);
    
    match status {
        task_task_revolution::domain::project_management::ValidationStatus::Invalid(conflicts) => {
            assert!(!conflicts.is_empty());
            assert!(conflicts.iter().any(|c| matches!(c.conflict_type, task_task_revolution::domain::project_management::ConflictType::DateOverlap(_, _, _, _))));
        }
        _ => panic!("Expected validation to fail with conflicts"),
    }
}

#[test]
fn test_calculation_cache_system() {
    let mut cache = CalculationCacheSystem::with_default_config();
    
    let dependency = AdvancedDependency::new(
        "task1".to_string(),
        "task2".to_string(),
        DependencyType::FinishToStart,
        LagType::zero(),
        "user1".to_string(),
        None,
    );

    let result = CalculationResult {
        task_id: "task1".to_string(),
        calculated_start_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        calculated_end_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()),
        is_critical: false,
        total_float: Some(chrono::Duration::days(0)),
        free_float: None,
        dependencies_satisfied: true,
        calculation_order: 0,
    };

    let config = CalculationConfig::default();

    // Armazenar no cache
    cache.put("task1", &[dependency.clone()], &config, result.clone()).unwrap();

    // Recuperar do cache
    let retrieved = cache.get("task1", &[dependency.clone()], &config);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().task_id, "task1");

    // Verificar estatísticas
    let stats = cache.get_stats();
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.miss_count, 0);
    assert_eq!(stats.hit_rate, 1.0);
}

#[test]
fn test_dependency_types_and_lags() {
    let mut graph = AdvancedDependencyGraph::new();
    
    let task1 = TaskNode::new(
        "task1".to_string(),
        "Task 1".to_string(),
        None,
        None,
        Some(chrono::Duration::days(5)),
    );
    let task2 = TaskNode::new(
        "task2".to_string(),
        "Task 2".to_string(),
        None,
        None,
        Some(chrono::Duration::days(3)),
    );

    graph.add_task(task1);
    graph.add_task(task2);

    // Testar diferentes tipos de dependência
    let dep_fs = AdvancedDependency::new(
        "task1".to_string(),
        "task2".to_string(),
        DependencyType::FinishToStart,
        LagType::positive_days(2),
        "user1".to_string(),
        Some("Finish to Start with 2 days lag".to_string()),
    );

    graph.add_dependency(dep_fs).unwrap();

    // Configurar engine de cálculo
    let config = CalculationConfig {
        project_start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        default_task_duration: chrono::Duration::days(1),
        working_days_only: false,
        working_hours_per_day: 8,
        cache_enabled: true,
    };

    let mut engine = DependencyCalculationEngine::new(config);
    let results = engine.calculate_project_dates(&graph).unwrap();

    // Verificar que o lag foi aplicado
    let task1_result = results.get("task1").unwrap();
    let task2_result = results.get("task2").unwrap();
    
    assert_eq!(task1_result.calculated_end_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap()));
    assert_eq!(task2_result.calculated_start_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 7).unwrap())); // 5 + 2 dias de lag
}

#[test]
fn test_complex_project_scenario() {
    // Cenário complexo: projeto com múltiplas tarefas e dependências
    let mut graph = AdvancedDependencyGraph::new();
    
    // Adicionar tarefas do projeto
    let tasks = vec![
        ("analysis", "Requirements Analysis", 3),
        ("design", "System Design", 5),
        ("frontend", "Frontend Development", 8),
        ("backend", "Backend Development", 10),
        ("database", "Database Setup", 2),
        ("testing", "Integration Testing", 4),
        ("deployment", "Deployment", 1),
    ];

    for (id, name, days) in tasks {
        let task = TaskNode::new(
            id.to_string(),
            name.to_string(),
            None,
            None,
            Some(chrono::Duration::days(days)),
        );
        graph.add_task(task);
    }

    // Adicionar dependências complexas
    let dependencies = vec![
        ("analysis", "design", DependencyType::FinishToStart, 0),
        ("design", "frontend", DependencyType::FinishToStart, 0),
        ("design", "backend", DependencyType::FinishToStart, 0),
        ("design", "database", DependencyType::FinishToStart, 0),
        ("database", "backend", DependencyType::FinishToStart, 0),
        ("frontend", "testing", DependencyType::FinishToStart, 0),
        ("backend", "testing", DependencyType::FinishToStart, 0),
        ("testing", "deployment", DependencyType::FinishToStart, 0),
    ];

    for (pred, succ, dep_type, lag_days) in dependencies {
        let dep = AdvancedDependency::new(
            pred.to_string(),
            succ.to_string(),
            dep_type,
            if lag_days == 0 { LagType::zero() } else { LagType::positive_days(lag_days) },
            "user1".to_string(),
            None,
        );
        graph.add_dependency(dep).unwrap();
    }

    // Calcular datas do projeto
    let config = CalculationConfig {
        project_start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        default_task_duration: chrono::Duration::days(1),
        working_days_only: false,
        working_hours_per_day: 8,
        cache_enabled: true,
    };

    let mut engine = DependencyCalculationEngine::new(config);
    let results = engine.calculate_project_dates(&graph).unwrap();

    // Verificar que todas as tarefas foram calculadas
    assert_eq!(results.len(), 7);

    // Verificar que a análise começa na data do projeto
    let analysis_result = results.get("analysis").unwrap();
    assert_eq!(analysis_result.calculated_start_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));

    // Verificar que o deployment é a última tarefa
    let deployment_result = results.get("deployment").unwrap();
    let deployment_end = deployment_result.calculated_end_date.unwrap();
    
    // Verificar que todas as outras tarefas terminam antes do deployment
    for (task_id, result) in &results {
        if *task_id != "deployment" {
            if let Some(end_date) = result.calculated_end_date {
                assert!(end_date <= deployment_end, "Task {} ends after deployment", task_id);
            }
        }
    }
}

#[test]
fn test_error_handling_and_recovery() {
    let mut graph = AdvancedDependencyGraph::new();
    
    // Tentar adicionar dependência para tarefa inexistente
    let dep = AdvancedDependency::new(
        "nonexistent".to_string(),
        "task2".to_string(),
        DependencyType::FinishToStart,
        LagType::zero(),
        "user1".to_string(),
        None,
    );

    let result = graph.add_dependency(dep);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Predecessor task does not exist"));

    // Tentar adicionar dependência circular
    let task1 = TaskNode::new("task1".to_string(), "Task 1".to_string(), None, None, None);
    let task2 = TaskNode::new("task2".to_string(), "Task 2".to_string(), None, None, None);
    
    graph.add_task(task1);
    graph.add_task(task2);

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
    let result = graph.add_dependency(dep2);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("would create a cycle"));
}

#[test]
fn test_performance_with_large_project() {
    // Teste de performance com projeto grande
    let mut graph = AdvancedDependencyGraph::new();
    
    // Criar 100 tarefas
    for i in 0..100 {
        let task = TaskNode::new(
            format!("task_{}", i),
            format!("Task {}", i),
            None,
            None,
            Some(chrono::Duration::days(1)),
        );
        graph.add_task(task);
    }

    // Criar dependências em cadeia
    for i in 0..99 {
        let dep = AdvancedDependency::new(
            format!("task_{}", i),
            format!("task_{}", i + 1),
            DependencyType::FinishToStart,
            LagType::zero(),
            "user1".to_string(),
            None,
        );
        graph.add_dependency(dep).unwrap();
    }

    // Medir tempo de cálculo
    let start = std::time::Instant::now();
    
    let config = CalculationConfig::default();
    let mut engine = DependencyCalculationEngine::new(config);
    let results = engine.calculate_project_dates(&graph).unwrap();
    
    let duration = start.elapsed();
    
    // Verificar que o cálculo foi rápido (menos de 1 segundo)
    assert!(duration.as_secs() < 1);
    
    // Verificar que todas as tarefas foram calculadas
    assert_eq!(results.len(), 100);
    
    // Verificar que as tarefas estão na ordem correta
    for i in 0..99 {
        let current_result = results.get(&format!("task_{}", i)).unwrap();
        let next_result = results.get(&format!("task_{}", i + 1)).unwrap();
        
        if let (Some(current_end), Some(next_start)) = (current_result.calculated_end_date, next_result.calculated_start_date) {
            assert!(current_end <= next_start, "Task {} ends after task {} starts", i, i + 1);
        }
    }
}
