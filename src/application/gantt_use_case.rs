//! Use Case para Geração de Gráficos Gantt
//!
//! Este módulo implementa a lógica de negócio para gerar gráficos Gantt
//! a partir dos dados de projetos e tarefas.

use chrono::NaiveDate;
use std::error::Error;
use std::path::PathBuf;
use tera::{Context, Tera};

use crate::domain::project_management::{
    DependencyType, GanttChart, GanttConfig, GanttTask, GanttViewType, TaskStatus, repository::ProjectRepository,
};
use crate::infrastructure::persistence::project_repository::FileProjectRepository;

/// Use Case para geração de gráficos Gantt
pub struct GanttUseCase {
    project_repository: FileProjectRepository,
    tera: Tera,
}

impl GanttUseCase {
    /// Cria uma nova instância do use case
    pub fn new(base_path: PathBuf) -> Self {
        let tera = Tera::new("templates/**/*").unwrap_or_else(|e| {
            eprintln!("Template parsing error(s): {}", e);
            std::process::exit(1);
        });

        Self {
            project_repository: FileProjectRepository::with_base_path(base_path),
            tera,
        }
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

    /// Gera gráfico Gantt com dados reais de um projeto específico
    pub fn generate_project_gantt(&self, project_code: &str) -> Result<GanttChart, Box<dyn Error>> {
        // Carregar o projeto
        let project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| format!("Project '{}' not found", project_code))?;

        // Usar datas do projeto ou datas padrão
        let start_date = project
            .start_date()
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        let end_date = project
            .end_date()
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        let config = GanttConfig::new(format!("{} - Gantt Chart", project.name()), start_date, end_date)
            .with_view_type(GanttViewType::Days)
            .with_dependencies(true)
            .with_resources(true)
            .with_progress(true)
            .with_dimensions(1200, 600);

        let mut gantt = GanttChart::new(config);

        // Adicionar tarefas reais do projeto
        for task in project.tasks().values() {
            let task_status = match task.status().to_string().as_str() {
                "Completed" => TaskStatus::Completed,
                "In Progress" => TaskStatus::InProgress,
                "Not Started" => TaskStatus::NotStarted,
                _ => TaskStatus::NotStarted,
            };

            let progress = if task_status == TaskStatus::Completed {
                1.0
            } else if task_status == TaskStatus::InProgress {
                0.5
            } else {
                0.0
            };

            let gantt_task = GanttTask::new(
                task.code().to_string(),
                task.name().to_string(),
                *task.start_date(),
                *task.due_date(),
                task_status,
                progress,
            );
            gantt.add_task(gantt_task);
        }

        // Adicionar dependências reais das tarefas
        for task in project.tasks().values() {
            for dependency in task.dependencies() {
                if let Some(dependent_task) = project.tasks().get(dependency) {
                    let dep = crate::domain::project_management::GanttDependency {
                        from_task: task.code().to_string(),
                        to_task: dependent_task.code().to_string(),
                        dependency_type: DependencyType::FinishToStart,
                    };
                    gantt.add_dependency(dep);
                }
            }
        }

        Ok(gantt)
    }

    /// Gera gráfico Gantt com dados reais de todos os projetos de uma empresa
    pub fn generate_company_gantt(&self, company_code: &str) -> Result<GanttChart, Box<dyn Error>> {
        // Carregar todos os projetos da empresa
        let projects = self.project_repository.find_all()?;
        let company_projects: Vec<_> = projects
            .into_iter()
            .filter(|p| p.company_code() == company_code)
            .collect();

        if company_projects.is_empty() {
            return Err(format!("No projects found for company '{}'", company_code).into());
        }

        // Encontrar datas mínimas e máximas
        let mut min_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut max_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        for project in &company_projects {
            if let Some(start) = project.start_date() {
                min_date = min_date.min(start);
            }
            if let Some(end) = project.end_date() {
                max_date = max_date.max(end);
            }
        }

        let config = GanttConfig::new(format!("{} - Company Gantt Chart", company_code), min_date, max_date)
            .with_view_type(GanttViewType::Days)
            .with_dependencies(true)
            .with_resources(true)
            .with_progress(true)
            .with_dimensions(1200, 600);

        let mut gantt = GanttChart::new(config);

        // Adicionar todas as tarefas de todos os projetos
        for project in &company_projects {
            for task in project.tasks().values() {
                let task_status = match task.status().to_string().as_str() {
                    "Completed" => TaskStatus::Completed,
                    "In Progress" => TaskStatus::InProgress,
                    "Not Started" => TaskStatus::NotStarted,
                    _ => TaskStatus::NotStarted,
                };

                let progress = if task_status == TaskStatus::Completed {
                    1.0
                } else if task_status == TaskStatus::InProgress {
                    0.5
                } else {
                    0.0
                };

                let gantt_task = GanttTask::new(
                    format!("{}-{}", project.code(), task.code()),
                    format!("{} - {}", project.name(), task.name()),
                    *task.start_date(),
                    *task.due_date(),
                    task_status,
                    progress,
                );
                gantt.add_task(gantt_task);
            }
        }

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

    /// Gera HTML do gráfico Gantt de projeto e salva em arquivo
    pub fn generate_and_save_project_gantt_html(
        &self,
        project_code: &str,
        output_path: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        // Carregar o projeto
        let project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| format!("Project '{}' not found", project_code))?;

        // Criar contexto para o template
        let mut context = Context::new();
        context.insert("project", &project);
        context.insert("tasks", &project.tasks().values().collect::<Vec<_>>());
        context.insert("relative_path_prefix", "../../../");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

        // Renderizar o template
        let html = self.tera.render("gantt.html", &context)?;

        // Criar diretório se não existir
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, html)?;
        println!(
            "✅ Gráfico Gantt do projeto '{}' gerado: {}",
            project_code,
            output_path.display()
        );

        Ok(())
    }

    /// Gera HTML do gráfico Gantt de empresa e salva em arquivo
    pub fn generate_and_save_company_gantt_html(
        &self,
        company_code: &str,
        output_path: &PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        // Carregar todos os projetos da empresa
        let projects = self.project_repository.find_all()?;
        let company_projects: Vec<_> = projects
            .into_iter()
            .filter(|p| p.company_code() == company_code)
            .collect();

        if company_projects.is_empty() {
            return Err(format!("No projects found for company '{}'", company_code).into());
        }

        // Criar um projeto virtual que contém todas as tarefas
        let mut all_tasks = Vec::new();
        let mut project_name = format!("{} - All Projects", company_code);

        for project in &company_projects {
            for task in project.tasks().values() {
                all_tasks.push(task.clone());
            }
            if project_name == format!("{} - All Projects", company_code) {
                project_name = format!("{} - {}", company_code, project.name());
            }
        }

        // Criar contexto para o template
        let mut context = Context::new();
        context.insert(
            "project",
            &serde_json::json!({
                "name": project_name,
                "code": company_code,
                "start_date": "2024-01-01",
                "end_date": "2024-12-31"
            }),
        );
        context.insert("tasks", &all_tasks);
        context.insert("relative_path_prefix", "../../");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

        // Renderizar o template
        let html = self.tera.render("gantt.html", &context)?;

        // Criar diretório se não existir
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(output_path, html)?;
        println!(
            "✅ Gráfico Gantt da empresa '{}' gerado: {}",
            company_code,
            output_path.display()
        );

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
        let _use_case = GanttUseCase::new(temp_dir.path().to_path_buf());
        // Teste que a criação funciona sem erros
        // Test completed successfully
    }

    #[test]
    fn test_generate_demo_gantt() {
        let temp_dir = tempdir().unwrap();
        let _use_case = GanttUseCase::new(temp_dir.path().to_path_buf());

        let gantt = _use_case.generate_demo_gantt().unwrap();
        assert_eq!(gantt.tasks.len(), 4);
        assert_eq!(gantt.dependencies.len(), 3);
    }
}
