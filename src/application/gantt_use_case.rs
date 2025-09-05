//! Use Case para Geração de Gráficos Gantt
//!
//! Este módulo implementa a lógica de negócio para gerar gráficos Gantt
//! a partir dos dados de projetos e tarefas.

use chrono::NaiveDate;
use std::error::Error;
use std::path::PathBuf;

use crate::domain::project_management::{
    DependencyType, GanttChart, GanttConfig, GanttTask, GanttViewType, TaskStatus,
};

/// Use Case para geração de gráficos Gantt
pub struct GanttUseCase;

impl GanttUseCase {
    /// Cria uma nova instância do use case
    pub fn new(_base_path: PathBuf) -> Self {
        Self
    }

    /// Gera gráfico Gantt simples para demonstração
    pub fn generate_demo_gantt(&self) -> Result<GanttChart, Box<dyn Error>> {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = GanttConfig::new("Demo Project - Gantt Chart".to_string(), start_date, end_date)
            .with_view_type(GanttViewType::Days)
            .with_dependencies(true)
            .with_resources(true)
            .with_progress(true)
            .with_dimensions(1200, 600);

        let mut gantt = GanttChart::new(config);

        // Adicionar tarefas de exemplo
        let task1 = GanttTask::new(
            "task1".to_string(),
            "Análise de Requisitos".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            TaskStatus::Completed,
            1.0,
        );
        gantt.add_task(task1);

        let task2 = GanttTask::new(
            "task2".to_string(),
            "Desenvolvimento".to_string(),
            NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            TaskStatus::InProgress,
            0.6,
        );
        gantt.add_task(task2);

        let task3 = GanttTask::new(
            "task3".to_string(),
            "Testes".to_string(),
            NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            TaskStatus::NotStarted,
            0.0,
        );
        gantt.add_task(task3);

        let task4 = GanttTask::new(
            "task4".to_string(),
            "Deploy".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
            NaiveDate::from_ymd_opt(2024, 4, 30).unwrap(),
            TaskStatus::NotStarted,
            0.0,
        );
        gantt.add_task(task4);

        // Adicionar dependências
        let dep1 = crate::domain::project_management::GanttDependency {
            from_task: "task1".to_string(),
            to_task: "task2".to_string(),
            dependency_type: DependencyType::FinishToStart,
        };
        gantt.add_dependency(dep1);

        let dep2 = crate::domain::project_management::GanttDependency {
            from_task: "task2".to_string(),
            to_task: "task3".to_string(),
            dependency_type: DependencyType::FinishToStart,
        };
        gantt.add_dependency(dep2);

        let dep3 = crate::domain::project_management::GanttDependency {
            from_task: "task3".to_string(),
            to_task: "task4".to_string(),
            dependency_type: DependencyType::FinishToStart,
        };
        gantt.add_dependency(dep3);

        Ok(gantt)
    }

    /// Gera HTML do gráfico Gantt e salva em arquivo
    pub fn generate_and_save_demo_gantt_html(&self, output_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let gantt = self.generate_demo_gantt()?;
        let html = gantt.generate_html();

        // Criar diretório se não existir
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, html)?;
        println!("✅ Gráfico Gantt demo gerado: {}", output_path.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_gantt_use_case_creation() {
        let temp_dir = tempdir().unwrap();
        let use_case = GanttUseCase::new(temp_dir.path().to_path_buf());
        // Teste que a criação funciona sem erros
        assert!(true);
    }

    #[test]
    fn test_generate_demo_gantt() {
        let temp_dir = tempdir().unwrap();
        let use_case = GanttUseCase::new(temp_dir.path().to_path_buf());

        let gantt = use_case.generate_demo_gantt().unwrap();
        assert_eq!(gantt.tasks.len(), 4);
        assert_eq!(gantt.dependencies.len(), 3);
    }
}
