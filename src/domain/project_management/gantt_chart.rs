//! Sistema de Geração de Gráficos Gantt
//!
//! Este módulo implementa a geração de gráficos Gantt para visualização
//! de cronogramas de projeto com dependências e recursos.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::error::Error;

use super::advanced_dependencies::{AdvancedDependencyGraph, DependencyType};
use crate::application::errors::AppError;

// ============================================================================
// ENUMS
// ============================================================================

/// Status de uma tarefa no gráfico Gantt
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    // Performance optimizations
    pub enable_pagination: bool,
    pub tasks_per_page: usize,
    pub enable_virtualization: bool,
    pub cache_size: usize,
    pub enable_lazy_loading: bool,
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
            // Performance defaults
            enable_pagination: false,
            tasks_per_page: 50,
            enable_virtualization: false,
            cache_size: 1000,
            enable_lazy_loading: false,
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

    /// Habilita paginação para grandes datasets
    pub fn with_pagination(mut self, tasks_per_page: usize) -> Self {
        self.enable_pagination = true;
        self.tasks_per_page = tasks_per_page;
        self
    }

    /// Habilita virtualização para melhor performance
    pub fn with_virtualization(mut self, cache_size: usize) -> Self {
        self.enable_virtualization = true;
        self.cache_size = cache_size;
        self
    }

    /// Habilita carregamento lazy para datasets muito grandes
    pub fn with_lazy_loading(mut self) -> Self {
        self.enable_lazy_loading = true;
        self
    }

    /// Configura otimizações para datasets grandes (>1000 tarefas)
    pub fn for_large_dataset(mut self) -> Self {
        self.enable_pagination = true;
        self.tasks_per_page = 100;
        self.enable_virtualization = true;
        self.cache_size = 500;
        self.enable_lazy_loading = true;
        self
    }

    /// Configura otimizações para datasets muito grandes (>10000 tarefas)
    pub fn for_very_large_dataset(mut self) -> Self {
        self.enable_pagination = true;
        self.tasks_per_page = 50;
        self.enable_virtualization = true;
        self.cache_size = 200;
        self.enable_lazy_loading = true;
        self
    }
}

/// Informações de paginação para o Gantt Chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttPagination {
    pub current_page: usize,
    pub total_pages: usize,
    pub tasks_per_page: usize,
    pub total_tasks: usize,
    pub has_next_page: bool,
    pub has_previous_page: bool,
}

impl GanttPagination {
    pub fn new(current_page: usize, total_tasks: usize, tasks_per_page: usize) -> Self {
        let total_pages = total_tasks.div_ceil(tasks_per_page);
        Self {
            current_page,
            total_pages,
            tasks_per_page,
            total_tasks,
            has_next_page: current_page < total_pages - 1,
            has_previous_page: current_page > 0,
        }
    }

    pub fn get_page_range(&self) -> (usize, usize) {
        let start = self.current_page * self.tasks_per_page;
        let end = std::cmp::min(start + self.tasks_per_page, self.total_tasks);
        (start, end)
    }
}

/// Cache de virtualização para performance
#[derive(Debug, Clone)]
pub struct GanttCache {
    pub visible_tasks: Vec<GanttTask>,
    pub visible_dependencies: Vec<GanttDependency>,
    pub cache_size: usize,
    pub last_update: std::time::Instant,
}

impl GanttCache {
    pub fn new(cache_size: usize) -> Self {
        Self {
            visible_tasks: Vec::new(),
            visible_dependencies: Vec::new(),
            cache_size,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.last_update.elapsed() > max_age
    }

    pub fn update(&mut self, tasks: Vec<GanttTask>, dependencies: Vec<GanttDependency>) {
        self.visible_tasks = tasks;
        self.visible_dependencies = dependencies;
        self.last_update = std::time::Instant::now();
    }
}

/// Gráfico Gantt completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttChart {
    pub config: GanttConfig,
    pub tasks: Vec<GanttTask>,
    pub dependencies: Vec<GanttDependency>,
    // Performance optimizations
    pub pagination: Option<GanttPagination>,
    pub total_tasks: usize,
    pub is_optimized: bool,
    // Advanced features
    pub filters: GanttFilters,
    pub advanced_features: GanttAdvancedFeatures,
}

impl GanttChart {
    /// Cria um novo gráfico Gantt
    pub fn new(config: GanttConfig) -> Self {
        Self {
            config,
            tasks: Vec::new(),
            dependencies: Vec::new(),
            pagination: None,
            total_tasks: 0,
            is_optimized: false,
            filters: GanttFilters::new(),
            advanced_features: GanttAdvancedFeatures::new(),
        }
    }

    /// Cria um gráfico Gantt otimizado para grandes datasets
    pub fn new_optimized(config: GanttConfig, total_tasks: usize) -> Self {
        let mut chart = Self {
            config,
            tasks: Vec::new(),
            dependencies: Vec::new(),
            pagination: None,
            total_tasks,
            is_optimized: true,
            filters: GanttFilters::new(),
            advanced_features: GanttAdvancedFeatures::new(),
        };

        // Configurar paginação se habilitada
        if chart.config.enable_pagination {
            chart.pagination = Some(GanttPagination::new(0, total_tasks, chart.config.tasks_per_page));
        }

        chart
    }

    /// Adiciona uma tarefa ao gráfico
    pub fn add_task(&mut self, task: GanttTask) {
        self.tasks.push(task);
    }

    /// Adiciona uma dependência ao gráfico
    pub fn add_dependency(&mut self, dependency: GanttDependency) {
        self.dependencies.push(dependency);
    }

    /// Adiciona múltiplas tarefas de forma otimizada
    pub fn add_tasks_batch(&mut self, tasks: Vec<GanttTask>) {
        if self.is_optimized && self.config.enable_pagination {
            // Para gráficos otimizados, limitar o número de tarefas carregadas
            let max_tasks = if let Some(pagination) = &self.pagination {
                pagination.tasks_per_page
            } else {
                self.config.tasks_per_page
            };

            self.tasks.extend(tasks.into_iter().take(max_tasks));
        } else {
            self.tasks.extend(tasks);
        }
    }

    /// Obtém tarefas para a página atual (se paginação estiver habilitada)
    pub fn get_current_page_tasks(&self) -> &[GanttTask] {
        if let Some(pagination) = &self.pagination {
            let (start, end) = pagination.get_page_range();
            &self.tasks[start..end]
        } else {
            &self.tasks
        }
    }

    /// Obtém dependências para a página atual
    pub fn get_current_page_dependencies(&self) -> Vec<&GanttDependency> {
        if let Some(pagination) = &self.pagination {
            let (start, end) = pagination.get_page_range();
            let current_task_ids: std::collections::HashSet<String> =
                self.tasks[start..end].iter().map(|t| t.id.clone()).collect();

            self.dependencies
                .iter()
                .filter(|dep| current_task_ids.contains(&dep.from_task) || current_task_ids.contains(&dep.to_task))
                .collect()
        } else {
            self.dependencies.iter().collect()
        }
    }

    /// Navega para a próxima página
    pub fn next_page(&mut self) -> bool {
        if let Some(pagination) = &mut self.pagination
            && pagination.has_next_page
        {
            pagination.current_page += 1;
            pagination.has_next_page = pagination.current_page < pagination.total_pages - 1;
            pagination.has_previous_page = pagination.current_page > 0;
            return true;
        }
        false
    }

    /// Navega para a página anterior
    pub fn previous_page(&mut self) -> bool {
        if let Some(pagination) = &mut self.pagination
            && pagination.has_previous_page
        {
            pagination.current_page -= 1;
            pagination.has_next_page = pagination.current_page < pagination.total_pages - 1;
            pagination.has_previous_page = pagination.current_page > 0;
            return true;
        }
        false
    }

    /// Vai para uma página específica
    pub fn go_to_page(&mut self, page: usize) -> bool {
        if let Some(pagination) = &mut self.pagination
            && page < pagination.total_pages
        {
            pagination.current_page = page;
            pagination.has_next_page = pagination.current_page < pagination.total_pages - 1;
            pagination.has_previous_page = pagination.current_page > 0;
            return true;
        }
        false
    }

    /// Obtém estatísticas de performance
    pub fn get_performance_stats(&self) -> GanttPerformanceStats {
        GanttPerformanceStats {
            total_tasks: self.total_tasks,
            loaded_tasks: self.tasks.len(),
            total_dependencies: self.dependencies.len(),
            is_paginated: self.pagination.is_some(),
            is_optimized: self.is_optimized,
            memory_usage_estimate: self.estimate_memory_usage(),
        }
    }

    /// Estima o uso de memória em bytes
    fn estimate_memory_usage(&self) -> usize {
        let task_size = std::mem::size_of::<GanttTask>();
        let dependency_size = std::mem::size_of::<GanttDependency>();

        (self.tasks.len() * task_size) + (self.dependencies.len() * dependency_size)
    }

    /// Aplica filtros às tarefas e retorna apenas as que correspondem
    pub fn get_filtered_tasks(&self) -> Vec<&GanttTask> {
        self.filters.apply_to_tasks(&self.tasks)
    }

    /// Aplica filtros às dependências e retorna apenas as que correspondem
    pub fn get_filtered_dependencies(&self) -> Vec<&GanttDependency> {
        if !self.filters.show_dependencies {
            return Vec::new();
        }

        let filtered_tasks: std::collections::HashSet<String> =
            self.get_filtered_tasks().iter().map(|t| t.id.clone()).collect();

        self.dependencies
            .iter()
            .filter(|dep| filtered_tasks.contains(&dep.from_task) || filtered_tasks.contains(&dep.to_task))
            .collect()
    }

    /// Atualiza os filtros do gráfico
    pub fn set_filters(&mut self, filters: GanttFilters) {
        self.filters = filters;
    }

    /// Atualiza os recursos avançados do gráfico
    pub fn set_advanced_features(&mut self, features: GanttAdvancedFeatures) {
        self.advanced_features = features;
    }

    /// Obtém estatísticas dos filtros aplicados
    pub fn get_filter_stats(&self) -> GanttFilterStats {
        let total_tasks = self.tasks.len();
        let filtered_tasks = self.get_filtered_tasks().len();
        let total_dependencies = self.dependencies.len();
        let filtered_dependencies = self.get_filtered_dependencies().len();

        GanttFilterStats {
            total_tasks,
            filtered_tasks,
            total_dependencies,
            filtered_dependencies,
            filter_active: self.is_filter_active(),
        }
    }

    /// Verifica se algum filtro está ativo
    pub fn is_filter_active(&self) -> bool {
        self.filters.status_filter.is_some()
            || self.filters.resource_filter.is_some()
            || self.filters.date_range_filter.is_some()
            || self.filters.progress_filter.is_some()
            || self.filters.search_text.is_some()
            || !self.filters.show_dependencies
            || !self.filters.show_milestones
    }

    /// Limpa todos os filtros
    pub fn clear_filters(&mut self) {
        self.filters = GanttFilters::new();
    }

    /// Obtém recursos únicos das tarefas
    pub fn get_unique_resources(&self) -> Vec<String> {
        let resources: std::collections::HashSet<String> =
            self.tasks.iter().filter_map(|t| t.resource.clone()).collect();

        resources.into_iter().collect()
    }

    /// Obtém status únicos das tarefas
    pub fn get_unique_statuses(&self) -> Vec<TaskStatus> {
        let statuses: std::collections::HashSet<TaskStatus> = self.tasks.iter().map(|t| t.status.clone()).collect();

        statuses.into_iter().collect()
    }

    /// Calcula o caminho crítico do projeto
    pub fn calculate_critical_path(&self) -> Vec<String> {
        if !self.advanced_features.enable_critical_path {
            return Vec::new();
        }

        // Implementação simples do caminho crítico
        // Em uma implementação real, isso seria mais complexo
        let mut critical_path = Vec::new();
        let mut max_duration = 0;
        let mut longest_task = None;

        for task in &self.tasks {
            let duration = task.duration.num_days() as i32;
            if duration > max_duration {
                max_duration = duration;
                longest_task = Some(task.id.clone());
            }
        }

        if let Some(task_id) = longest_task {
            critical_path.push(task_id);
        }

        critical_path
    }

    /// Exporta o gráfico para diferentes formatos
    pub fn export(&self, format: GanttExportFormat) -> Result<String, Box<dyn Error>> {
        if !self.advanced_features.enable_export {
            return Err("Export is disabled".into());
        }

        match format {
            GanttExportFormat::Json => serde_json::to_string(self).map_err(|e| e.into()),
            GanttExportFormat::Csv => self.export_to_csv(),
            GanttExportFormat::Html => Ok(self.generate_html()),
        }
    }

    /// Exporta para CSV
    fn export_to_csv(&self) -> Result<String, Box<dyn Error>> {
        let mut csv = String::new();
        csv.push_str("Task ID,Task Name,Start Date,End Date,Status,Progress,Resource\n");

        for task in &self.tasks {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                task.id,
                task.name,
                task.start_date,
                task.end_date,
                task.status.description(),
                task.progress,
                task.resource.as_deref().unwrap_or("")
            ));
        }

        Ok(csv)
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
    pub fn from_dependency_graph(graph: &AdvancedDependencyGraph, config: GanttConfig) -> Result<Self, AppError> {
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

/// Filtros para o Gantt Chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttFilters {
    pub status_filter: Option<Vec<TaskStatus>>,
    pub resource_filter: Option<Vec<String>>,
    pub date_range_filter: Option<(NaiveDate, NaiveDate)>,
    pub progress_filter: Option<(f64, f64)>,
    pub search_text: Option<String>,
    pub show_dependencies: bool,
    pub show_milestones: bool,
}

impl GanttFilters {
    pub fn new() -> Self {
        Self {
            status_filter: None,
            resource_filter: None,
            date_range_filter: None,
            progress_filter: None,
            search_text: None,
            show_dependencies: true,
            show_milestones: true,
        }
    }
}

impl Default for GanttFilters {
    fn default() -> Self {
        Self::new()
    }
}

impl GanttFilters {
    pub fn with_status_filter(mut self, statuses: Vec<TaskStatus>) -> Self {
        self.status_filter = Some(statuses);
        self
    }

    pub fn with_resource_filter(mut self, resources: Vec<String>) -> Self {
        self.resource_filter = Some(resources);
        self
    }

    pub fn with_date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.date_range_filter = Some((start, end));
        self
    }

    pub fn with_progress_range(mut self, min: f64, max: f64) -> Self {
        self.progress_filter = Some((min, max));
        self
    }

    pub fn with_search_text(mut self, text: String) -> Self {
        self.search_text = Some(text);
        self
    }

    pub fn apply_to_tasks<'a>(&self, tasks: &'a [GanttTask]) -> Vec<&'a GanttTask> {
        tasks.iter().filter(|task| self.matches_task(task)).collect()
    }

    fn matches_task(&self, task: &GanttTask) -> bool {
        // Status filter
        if let Some(ref statuses) = self.status_filter
            && !statuses.contains(&task.status)
        {
            return false;
        }

        // Resource filter
        if let Some(ref resources) = self.resource_filter {
            if let Some(ref resource) = task.resource {
                if !resources.contains(resource) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Date range filter
        if let Some((start, end)) = self.date_range_filter
            && (task.start_date < start || task.end_date > end)
        {
            return false;
        }

        // Progress filter
        if let Some((min, max)) = self.progress_filter
            && (task.progress < min || task.progress > max)
        {
            return false;
        }

        // Search text filter
        if let Some(ref text) = self.search_text
            && !task.name.to_lowercase().contains(&text.to_lowercase())
        {
            return false;
        }

        true
    }
}

/// Recursos avançados do Gantt Chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttAdvancedFeatures {
    pub enable_zoom: bool,
    pub enable_pan: bool,
    pub enable_export: bool,
    pub enable_print: bool,
    pub enable_fullscreen: bool,
    pub enable_tooltips: bool,
    pub enable_drag_drop: bool,
    pub enable_critical_path: bool,
    pub enable_baseline: bool,
    pub enable_actual_vs_planned: bool,
    pub enable_resource_loading: bool,
    pub enable_milestone_tracking: bool,
}

impl GanttAdvancedFeatures {
    pub fn new() -> Self {
        Self {
            enable_zoom: true,
            enable_pan: true,
            enable_export: true,
            enable_print: true,
            enable_fullscreen: true,
            enable_tooltips: true,
            enable_drag_drop: false,
            enable_critical_path: true,
            enable_baseline: false,
            enable_actual_vs_planned: false,
            enable_resource_loading: true,
            enable_milestone_tracking: true,
        }
    }
}

impl Default for GanttAdvancedFeatures {
    fn default() -> Self {
        Self::new()
    }
}

impl GanttAdvancedFeatures {
    pub fn with_drag_drop(mut self) -> Self {
        self.enable_drag_drop = true;
        self
    }

    pub fn with_baseline(mut self) -> Self {
        self.enable_baseline = true;
        self
    }

    pub fn with_actual_vs_planned(mut self) -> Self {
        self.enable_actual_vs_planned = true;
        self
    }
}

/// Estatísticas de performance do Gantt Chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttPerformanceStats {
    pub total_tasks: usize,
    pub loaded_tasks: usize,
    pub total_dependencies: usize,
    pub is_paginated: bool,
    pub is_optimized: bool,
    pub memory_usage_estimate: usize,
}

impl GanttPerformanceStats {
    pub fn get_memory_usage_mb(&self) -> f64 {
        self.memory_usage_estimate as f64 / (1024.0 * 1024.0)
    }

    pub fn get_load_percentage(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.loaded_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }

    pub fn is_efficient(&self) -> bool {
        self.is_optimized && self.get_load_percentage() < 100.0
    }
}

/// Estatísticas dos filtros aplicados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GanttFilterStats {
    pub total_tasks: usize,
    pub filtered_tasks: usize,
    pub total_dependencies: usize,
    pub filtered_dependencies: usize,
    pub filter_active: bool,
}

impl GanttFilterStats {
    pub fn get_filter_percentage(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.filtered_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }
}

/// Formatos de exportação do Gantt Chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GanttExportFormat {
    Json,
    Csv,
    Html,
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

    #[test]
    fn test_gantt_filters() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Adicionar tarefas de teste
        let task1 = GanttTask::new(
            "task1".to_string(),
            "Task 1".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );
        chart.add_task(task1);

        let task2 = GanttTask::new(
            "task2".to_string(),
            "Task 2".to_string(),
            start_date,
            end_date,
            TaskStatus::Completed,
            1.0,
        );
        chart.add_task(task2);

        // Test status filter
        let filters = GanttFilters::new().with_status_filter(vec![TaskStatus::InProgress]);
        chart.set_filters(filters);

        let filtered_tasks = chart.get_filtered_tasks();
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].id, "task1");

        // Test search filter
        let filters = GanttFilters::new().with_search_text("Task 2".to_string());
        chart.set_filters(filters);

        let filtered_tasks = chart.get_filtered_tasks();
        assert_eq!(filtered_tasks.len(), 1);
        assert_eq!(filtered_tasks[0].id, "task2");
    }

    #[test]
    fn test_gantt_advanced_features() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Test advanced features
        let features = GanttAdvancedFeatures::new().with_drag_drop().with_baseline();
        chart.set_advanced_features(features);

        assert!(chart.advanced_features.enable_drag_drop);
        assert!(chart.advanced_features.enable_baseline);
    }

    #[test]
    fn test_gantt_export() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Adicionar tarefa de teste
        let task = GanttTask::new(
            "task1".to_string(),
            "Test Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );
        chart.add_task(task);

        // Test CSV export
        let csv = chart.export(GanttExportFormat::Csv).unwrap();
        assert!(csv.contains("Task ID,Task Name"));
        assert!(csv.contains("task1,Test Task"));

        // Test HTML export
        let html = chart.export(GanttExportFormat::Html).unwrap();
        assert!(html.contains("Test Project"));
        assert!(html.contains("gantt-container"));
    }

    #[test]
    fn test_gantt_filter_stats() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Adicionar tarefas de teste
        let task1 = GanttTask::new(
            "task1".to_string(),
            "Task 1".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );
        chart.add_task(task1);

        let task2 = GanttTask::new(
            "task2".to_string(),
            "Task 2".to_string(),
            start_date,
            end_date,
            TaskStatus::Completed,
            1.0,
        );
        chart.add_task(task2);

        // Test filter stats
        let stats = chart.get_filter_stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.filtered_tasks, 2);
        assert!(!stats.filter_active);

        // Apply filter
        let filters = GanttFilters::new().with_status_filter(vec![TaskStatus::InProgress]);
        chart.set_filters(filters);

        let stats = chart.get_filter_stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.filtered_tasks, 1);
        assert!(stats.filter_active);
    }

    #[test]
    fn test_gantt_unique_resources() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Adicionar tarefas com recursos
        let mut task1 = GanttTask::new(
            "task1".to_string(),
            "Task 1".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );
        task1.set_resource("Resource 1".to_string());
        chart.add_task(task1);

        let mut task2 = GanttTask::new(
            "task2".to_string(),
            "Task 2".to_string(),
            start_date,
            end_date,
            TaskStatus::Completed,
            1.0,
        );
        task2.set_resource("Resource 2".to_string());
        chart.add_task(task2);

        let resources = chart.get_unique_resources();
        assert_eq!(resources.len(), 2);
        assert!(resources.contains(&"Resource 1".to_string()));
        assert!(resources.contains(&"Resource 2".to_string()));
    }

    #[test]
    fn test_gantt_critical_path() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();

        let config = GanttConfig::new("Test Project".to_string(), start_date, end_date);
        let mut chart = GanttChart::new(config);

        // Adicionar tarefas com diferentes durações
        let task1 = GanttTask::new(
            "task1".to_string(),
            "Short Task".to_string(),
            start_date,
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
            TaskStatus::InProgress,
            0.5,
        );
        chart.add_task(task1);

        let task2 = GanttTask::new(
            "task2".to_string(),
            "Long Task".to_string(),
            start_date,
            end_date,
            TaskStatus::InProgress,
            0.5,
        );
        chart.add_task(task2);

        let critical_path = chart.calculate_critical_path();
        assert_eq!(critical_path.len(), 1);
        assert_eq!(critical_path[0], "task2");
    }
}
