//! Sistema de Geração de Gráficos Gantt
//!
//! Este módulo implementa a geração de gráficos Gantt para visualização
//! de cronogramas de projeto com dependências e recursos.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

use super::advanced_dependencies::{AdvancedDependencyGraph, DependencyType};
use crate::domain::shared::errors::DomainError;

// ============================================================================
// ENUMS
// ============================================================================

/// Status de uma tarefa no gráfico Gantt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Delayed,
    OnHold,
}

impl TaskStatus {
    /// Retorna uma descrição legível do status
    pub fn description(&self) -> &'static str {
        match self {
            TaskStatus::NotStarted => "Não Iniciada",
            TaskStatus::InProgress => "Em Progresso",
            TaskStatus::Completed => "Concluída",
            TaskStatus::Delayed => "Atrasada",
            TaskStatus::OnHold => "Em Espera",
        }
    }

    /// Retorna a cor CSS para o status
    pub fn color(&self) -> &'static str {
        match self {
            TaskStatus::NotStarted => "#e5e7eb", // cinza claro
            TaskStatus::InProgress => "#3b82f6", // azul
            TaskStatus::Completed => "#10b981",  // verde
            TaskStatus::Delayed => "#ef4444",    // vermelho
            TaskStatus::OnHold => "#f59e0b",     // amarelo
        }
    }
}

/// Tipo de visualização do gráfico Gantt
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GanttViewType {
    Days,
    Weeks,
    Months,
    Quarters,
}

impl GanttViewType {
    /// Retorna uma descrição legível do tipo de visualização
    pub fn description(&self) -> &'static str {
        match self {
            GanttViewType::Days => "Dias",
            GanttViewType::Weeks => "Semanas",
            GanttViewType::Months => "Meses",
            GanttViewType::Quarters => "Trimestres",
        }
    }

    /// Calcula o número de dias entre duas datas para o tipo de visualização
    pub fn days_between(&self, start: NaiveDate, end: NaiveDate) -> i64 {
        match self {
            GanttViewType::Days => end.signed_duration_since(start).num_days(),
            GanttViewType::Weeks => end.signed_duration_since(start).num_weeks(),
            GanttViewType::Months => {
                let months = (end.year() - start.year()) * 12 + (end.month() as i32 - start.month() as i32);
                months as i64
            }
            GanttViewType::Quarters => {
                let quarters =
                    (end.year() - start.year()) * 4 + ((end.month() as i32 - 1) / 3) - ((start.month() as i32 - 1) / 3);
                quarters as i64
            }
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

/// Item de tarefa no gráfico Gantt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttTask {
    pub id: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub duration: Duration,
    pub status: TaskStatus,
    pub progress: f64, // 0.0 a 1.0
    pub dependencies: Vec<String>,
    pub resource: Option<String>,
    pub description: Option<String>,
}

impl GanttTask {
    /// Cria uma nova tarefa Gantt
    pub fn new(
        id: String,
        name: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        status: TaskStatus,
        progress: f64,
    ) -> Self {
        let duration = end_date.signed_duration_since(start_date);
        Self {
            id,
            name,
            start_date,
            end_date,
            duration,
            status,
            progress: progress.clamp(0.0, 1.0),
            dependencies: Vec::new(),
            resource: None,
            description: None,
        }
    }

    /// Adiciona uma dependência à tarefa
    pub fn add_dependency(&mut self, dependency_id: String) {
        if !self.dependencies.contains(&dependency_id) {
            self.dependencies.push(dependency_id);
        }
    }

    /// Define o recurso responsável pela tarefa
    pub fn set_resource(&mut self, resource: String) {
        self.resource = Some(resource);
    }

    /// Define a descrição da tarefa
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// Calcula a porcentagem de progresso baseada nas datas
    pub fn calculate_progress(&self, current_date: NaiveDate) -> f64 {
        if self.status == TaskStatus::Completed {
            return 1.0;
        }

        if current_date < self.start_date {
            return 0.0;
        }

        if current_date >= self.end_date {
            return 1.0;
        }

        let total_duration = self.end_date.signed_duration_since(self.start_date).num_days() as f64;
        let elapsed_duration = current_date.signed_duration_since(self.start_date).num_days() as f64;

        (elapsed_duration / total_duration).clamp(0.0, 1.0)
    }

    /// Verifica se a tarefa está atrasada
    pub fn is_delayed(&self, current_date: NaiveDate) -> bool {
        self.status != TaskStatus::Completed && current_date > self.end_date
    }
}

/// Configuração do gráfico Gantt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttConfig {
    pub title: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub view_type: GanttViewType,
    pub show_dependencies: bool,
    pub show_resources: bool,
    pub show_progress: bool,
    pub width: u32,
    pub height: u32,
}

impl GanttConfig {
    /// Cria uma nova configuração padrão
    pub fn new(title: String, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            title,
            start_date,
            end_date,
            view_type: GanttViewType::Days,
            show_dependencies: true,
            show_resources: true,
            show_progress: true,
            width: 1200,
            height: 600,
        }
    }

    /// Define o tipo de visualização
    pub fn with_view_type(mut self, view_type: GanttViewType) -> Self {
        self.view_type = view_type;
        self
    }

    /// Define se deve mostrar dependências
    pub fn with_dependencies(mut self, show: bool) -> Self {
        self.show_dependencies = show;
        self
    }

    /// Define se deve mostrar recursos
    pub fn with_resources(mut self, show: bool) -> Self {
        self.show_resources = show;
        self
    }

    /// Define se deve mostrar progresso
    pub fn with_progress(mut self, show: bool) -> Self {
        self.show_progress = show;
        self
    }

    /// Define as dimensões do gráfico
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

/// Gráfico Gantt completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttChart {
    pub config: GanttConfig,
    pub tasks: Vec<GanttTask>,
    pub dependencies: Vec<GanttDependency>,
}

impl GanttChart {
    /// Cria um novo gráfico Gantt
    pub fn new(config: GanttConfig) -> Self {
        Self {
            config,
            tasks: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Adiciona uma tarefa ao gráfico
    pub fn add_task(&mut self, task: GanttTask) {
        self.tasks.push(task);
    }

    /// Adiciona uma dependência ao gráfico
    pub fn add_dependency(&mut self, dependency: GanttDependency) {
        self.dependencies.push(dependency);
    }

    /// Gera o HTML do gráfico Gantt
    pub fn generate_html(&self) -> String {
        let mut html = String::new();

        // CSS do gráfico
        html.push_str(&self.generate_css());

        // HTML do gráfico
        html.push_str(&self.generate_chart_html());

        // JavaScript do gráfico
        html.push_str(&self.generate_javascript());

        html
    }

    /// Gera o CSS do gráfico
    fn generate_css(&self) -> String {
        format!(
            r#"
<style>
.gantt-container {{
    width: {}px;
    height: {}px;
    border: 1px solid #ddd;
    border-radius: 4px;
    overflow: hidden;
    font-family: Arial, sans-serif;
}}

.gantt-header {{
    background: #f8f9fa;
    padding: 10px;
    border-bottom: 1px solid #ddd;
    text-align: center;
    font-weight: bold;
    font-size: 18px;
}}

.gantt-timeline {{
    display: flex;
    background: #e9ecef;
    border-bottom: 1px solid #ddd;
}}

.gantt-timeline-item {{
    flex: 1;
    padding: 8px;
    text-align: center;
    border-right: 1px solid #ddd;
    font-size: 12px;
    font-weight: bold;
}}

.gantt-tasks {{
    overflow-y: auto;
    max-height: calc(100% - 100px);
}}

.gantt-task {{
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border-bottom: 1px solid #eee;
    min-height: 30px;
}}

.gantt-task-name {{
    flex: 1;
    font-size: 14px;
    font-weight: 500;
}}

.gantt-task-bar {{
    flex: 2;
    height: 20px;
    background: #e5e7eb;
    border-radius: 10px;
    position: relative;
    margin: 0 10px;
}}

.gantt-task-progress {{
    height: 100%;
    border-radius: 10px;
    transition: width 0.3s ease;
}}

.gantt-task-status-not-started {{
    background: #e5e7eb;
}}

.gantt-task-status-in-progress {{
    background: #3b82f6;
}}

.gantt-task-status-completed {{
    background: #10b981;
}}

.gantt-task-status-delayed {{
    background: #ef4444;
}}

.gantt-task-status-on-hold {{
    background: #f59e0b;
}}

.gantt-task-resource {{
    flex: 1;
    font-size: 12px;
    color: #6b7280;
    text-align: right;
}}

.gantt-dependencies {{
    margin-top: 20px;
}}

.gantt-dependency {{
    display: flex;
    align-items: center;
    padding: 8px;
    background: #f8f9fa;
    border-radius: 4px;
    margin-bottom: 4px;
}}

.gantt-dependency-arrow {{
    margin: 0 10px;
    color: #6b7280;
}}

.gantt-legend {{
    display: flex;
    justify-content: center;
    gap: 20px;
    margin-top: 20px;
    flex-wrap: wrap;
}}

.gantt-legend-item {{
    display: flex;
    align-items: center;
    gap: 8px;
}}

.gantt-legend-color {{
    width: 16px;
    height: 16px;
    border-radius: 2px;
}}
</style>
"#,
            self.config.width, self.config.height
        )
    }

    /// Gera o HTML do gráfico
    fn generate_chart_html(&self) -> String {
        let mut html = format!(
            r#"
<div class="gantt-container">
    <div class="gantt-header">{}</div>
    <div class="gantt-timeline">
"#,
            self.config.title
        );

        // Timeline
        let mut current_date = self.config.start_date;
        while current_date <= self.config.end_date {
            html.push_str(&format!(
                r#"<div class="gantt-timeline-item">{}</div>"#,
                current_date.format("%d/%m")
            ));
            current_date += Duration::days(1);
        }

        html.push_str("</div><div class=\"gantt-tasks\">");

        // Tarefas
        for task in &self.tasks {
            let status_class = match task.status {
                TaskStatus::NotStarted => "not-started",
                TaskStatus::InProgress => "in-progress",
                TaskStatus::Completed => "completed",
                TaskStatus::Delayed => "delayed",
                TaskStatus::OnHold => "on-hold",
            };

            let progress_width = (task.progress * 100.0) as u32;
            let color = task.status.color();

            html.push_str(&format!(
                r#"
                <div class="gantt-task">
                    <div class="gantt-task-name">{}</div>
                    <div class="gantt-task-bar">
                        <div class="gantt-task-progress gantt-task-status-{}" 
                             style="width: {}%; background-color: {};"></div>
                    </div>
                    <div class="gantt-task-resource">{}</div>
                </div>
            "#,
                task.name,
                status_class,
                progress_width,
                color,
                task.resource.as_deref().unwrap_or("")
            ));
        }

        html.push_str("</div></div>");

        // Dependências
        if self.config.show_dependencies && !self.dependencies.is_empty() {
            html.push_str("<div class=\"gantt-dependencies\"><h3>Dependências</h3>");
            for dep in &self.dependencies {
                html.push_str(&format!(
                    r#"
                    <div class="gantt-dependency">
                        <span>{}</span>
                        <span class="gantt-dependency-arrow">→</span>
                        <span>{}</span>
                    </div>
                "#,
                    dep.from_task, dep.to_task
                ));
            }
            html.push_str("</div>");
        }

        // Legenda
        html.push_str("<div class=\"gantt-legend\">");
        for status in [
            TaskStatus::NotStarted,
            TaskStatus::InProgress,
            TaskStatus::Completed,
            TaskStatus::Delayed,
            TaskStatus::OnHold,
        ] {
            html.push_str(&format!(
                r#"
                <div class="gantt-legend-item">
                    <div class="gantt-legend-color" style="background-color: {};"></div>
                    <span>{}</span>
                </div>
            "#,
                status.color(),
                status.description()
            ));
        }
        html.push_str("</div>");

        html
    }

    /// Gera o JavaScript do gráfico
    fn generate_javascript(&self) -> String {
        r#"
<script>
// Funcionalidade básica do gráfico Gantt
document.addEventListener('DOMContentLoaded', function() {
    // Adicionar interatividade às barras de tarefa
    const taskBars = document.querySelectorAll('.gantt-task-bar');
    taskBars.forEach(bar => {
        bar.addEventListener('click', function() {
            // Toggle de detalhes da tarefa
            const task = this.closest('.gantt-task');
            const details = task.querySelector('.gantt-task-details');
            if (details) {
                details.style.display = details.style.display === 'none' ? 'block' : 'none';
            }
        });
    });

    // Adicionar tooltip com informações da tarefa
    const tasks = document.querySelectorAll('.gantt-task');
    tasks.forEach(task => {
        task.addEventListener('mouseenter', function() {
            const name = this.querySelector('.gantt-task-name').textContent;
            const resource = this.querySelector('.gantt-task-resource').textContent;
            const progress = this.querySelector('.gantt-task-progress').style.width;
            
            const tooltip = document.createElement('div');
            tooltip.className = 'gantt-tooltip';
            tooltip.innerHTML = `
                <strong>${name}</strong><br>
                Recurso: ${resource}<br>
                Progresso: ${progress}
            `;
            tooltip.style.cssText = `
                position: absolute;
                background: #333;
                color: white;
                padding: 8px;
                border-radius: 4px;
                font-size: 12px;
                z-index: 1000;
                pointer-events: none;
            `;
            document.body.appendChild(tooltip);
            
            const rect = this.getBoundingClientRect();
            tooltip.style.left = rect.left + 'px';
            tooltip.style.top = (rect.top - tooltip.offsetHeight - 5) + 'px';
        });
        
        task.addEventListener('mouseleave', function() {
            const tooltip = document.querySelector('.gantt-tooltip');
            if (tooltip) {
                tooltip.remove();
            }
        });
    });
});
</script>
"#
        .to_string()
    }

    /// Converte um AdvancedDependencyGraph em GanttChart
    pub fn from_dependency_graph(graph: &AdvancedDependencyGraph, config: GanttConfig) -> Result<Self, DomainError> {
        let start_date = config.start_date;
        let end_date = config.end_date;
        let mut gantt = GanttChart::new(config);

        // Converter nós em tarefas
        for (task_id, node) in &graph.nodes {
            let status = if node.start_date.is_some() && node.end_date.is_some() {
                TaskStatus::InProgress
            } else {
                TaskStatus::NotStarted
            };

            let start_date = node.start_date.unwrap_or(start_date);
            let end_date = node.end_date.unwrap_or(end_date);
            let progress = 0.0; // TODO: Calcular progresso baseado em dados reais

            let mut task = GanttTask::new(
                task_id.clone(),
                node.name.clone(),
                start_date,
                end_date,
                status,
                progress,
            );

            // Adicionar dependências
            for dep in graph.get_dependencies(task_id) {
                task.add_dependency(dep.successor_id.clone());
            }

            gantt.add_task(task);
        }

        // Converter dependências
        for (predecessor_id, deps) in &graph.dependencies {
            for dep in deps {
                let gantt_dep = GanttDependency {
                    from_task: predecessor_id.clone(),
                    to_task: dep.successor_id.clone(),
                    dependency_type: dep.dependency_type.clone(),
                };
                gantt.add_dependency(gantt_dep);
            }
        }

        Ok(gantt)
    }
}

/// Dependência no gráfico Gantt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttDependency {
    pub from_task: String,
    pub to_task: String,
    pub dependency_type: DependencyType,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_task_status_descriptions() {
        assert_eq!(TaskStatus::NotStarted.description(), "Não Iniciada");
        assert_eq!(TaskStatus::InProgress.description(), "Em Progresso");
        assert_eq!(TaskStatus::Completed.description(), "Concluída");
        assert_eq!(TaskStatus::Delayed.description(), "Atrasada");
        assert_eq!(TaskStatus::OnHold.description(), "Em Espera");
    }

    #[test]
    fn test_task_status_colors() {
        assert_eq!(TaskStatus::NotStarted.color(), "#e5e7eb");
        assert_eq!(TaskStatus::InProgress.color(), "#3b82f6");
        assert_eq!(TaskStatus::Completed.color(), "#10b981");
        assert_eq!(TaskStatus::Delayed.color(), "#ef4444");
        assert_eq!(TaskStatus::OnHold.color(), "#f59e0b");
    }

    #[test]
    fn test_gantt_view_type_descriptions() {
        assert_eq!(GanttViewType::Days.description(), "Dias");
        assert_eq!(GanttViewType::Weeks.description(), "Semanas");
        assert_eq!(GanttViewType::Months.description(), "Meses");
        assert_eq!(GanttViewType::Quarters.description(), "Trimestres");
    }

    #[test]
    fn test_gantt_task_creation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );

        assert_eq!(task.id, "task1");
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.start_date, start_date);
        assert_eq!(task.end_date, end_date);
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.progress, 0.5);
    }

    #[test]
    fn test_gantt_task_progress_calculation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.0,
        );

        // Teste antes do início
        let before_start = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        assert_eq!(task.calculate_progress(before_start), 0.0);

        // Teste no meio (dia 5 de 10 dias = 4 dias completos de 9 dias totais)
        let middle = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let expected_progress = 4.0 / 9.0; // 4 dias completos de 9 dias totais
        assert!((task.calculate_progress(middle) - expected_progress).abs() < 0.01);

        // Teste após o fim
        let after_end = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(task.calculate_progress(after_end), 1.0);
    }

    #[test]
    fn test_gantt_task_delay_detection() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();

        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.0,
        );

        // Teste antes do prazo
        let before_due = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        assert!(!task.is_delayed(before_due));

        // Teste após o prazo
        let after_due = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert!(task.is_delayed(after_due));
    }

    #[test]
    fn test_gantt_config_creation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);

        assert_eq!(config.title, "Test Project");
        assert_eq!(config.start_date, start_date);
        assert_eq!(config.end_date, end_date);
        assert_eq!(config.view_type, GanttViewType::Days);
        assert!(config.show_dependencies);
        assert!(config.show_resources);
        assert!(config.show_progress);
    }

    #[test]
    fn test_gantt_config_builder() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date)
            .with_view_type(GanttViewType::Weeks)
            .with_dependencies(false)
            .with_resources(false)
            .with_progress(false)
            .with_dimensions(800, 400);

        assert_eq!(config.view_type, GanttViewType::Weeks);
        assert!(!config.show_dependencies);
        assert!(!config.show_resources);
        assert!(!config.show_progress);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 400);
    }

    #[test]
    fn test_gantt_chart_creation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);

        let chart = GanttChart::new(config);

        assert_eq!(chart.tasks.len(), 0);
        assert_eq!(chart.dependencies.len(), 0);
    }

    #[test]
    fn test_gantt_chart_add_task() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);

        let mut chart = GanttChart::new(config);

        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );

        chart.add_task(task);

        assert_eq!(chart.tasks.len(), 1);
        assert_eq!(chart.tasks[0].id, "task1");
    }

    #[test]
    fn test_gantt_chart_html_generation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);

        let mut chart = GanttChart::new(config);

        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );

        chart.add_task(task);

        let html = chart.generate_html();

        assert!(html.contains("Test Project"));
        assert!(html.contains("Test Task"));
        assert!(html.contains("gantt-container"));
        assert!(html.contains("gantt-task"));
    }
}
