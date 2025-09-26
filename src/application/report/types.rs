use chrono::{DateTime, Local};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tipos de relatório suportados
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum ReportType {
    Task,
    Resource,
    Project,
    Company,
}

/// Formatos de exportação suportados
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
    Yaml,
    Pdf,
}

/// Configuração de filtro para relatórios
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

/// Operadores de filtro
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
}

/// Configuração de agrupamento
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupBy {
    pub field: String,
    pub sort_order: SortOrder,
}

/// Ordem de classificação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Configuração de relatório
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportConfig {
    pub report_type: ReportType,
    pub format: ExportFormat,
    pub filters: Vec<ReportFilter>,
    pub group_by: Option<GroupBy>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
    pub include_summary: bool,
    pub template: Option<String>,
}

/// Dados de relatório genérico
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportData {
    pub title: String,
    pub generated_at: DateTime<Local>,
    pub total_records: usize,
    pub data: Vec<HashMap<String, serde_json::Value>>,
    pub summary: Option<ReportSummary>,
}

/// Resumo de relatório
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_count: usize,
    pub field_stats: HashMap<String, FieldStats>,
}

/// Estatísticas de campo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldStats {
    pub count: usize,
    pub unique_count: usize,
    pub min_value: Option<serde_json::Value>,
    pub max_value: Option<serde_json::Value>,
    pub avg_value: Option<f64>,
}

/// Resultado de geração de relatório
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResult {
    pub success: bool,
    pub data: Option<ReportData>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl ReportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportType::Task => "task",
            ReportType::Resource => "resource",
            ReportType::Project => "project",
            ReportType::Company => "company",
        }
    }
}

impl ExportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Yaml => "yaml",
            ExportFormat::Pdf => "pdf",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Yaml => "yaml",
            ExportFormat::Pdf => "pdf",
        }
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            report_type: ReportType::Task,
            format: ExportFormat::Json,
            filters: Vec::new(),
            group_by: None,
            sort_by: None,
            sort_order: None,
            include_summary: false,
            template: None,
        }
    }
}
