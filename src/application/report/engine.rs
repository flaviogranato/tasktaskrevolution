use crate::application::errors::AppError;
use crate::application::report::formatters::FormatterFactory;
use crate::application::report::types::*;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::company_management::CompanyRepository;
use crate::domain::company_management::company::Company;
use crate::domain::project_management::any_project::AnyProject;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::any_resource::AnyResource;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::any_task::AnyTask;
use chrono::Local;
use std::collections::HashMap;

/// Engine principal para geração de relatórios
pub struct ReportEngine<CR, PR, RR, CoR>
where
    CR: CodeResolverTrait,
    PR: ProjectRepository,
    RR: ResourceRepository,
    CoR: CompanyRepository,
{
    _code_resolver: CR,
    project_repository: PR,
    resource_repository: RR,
    company_repository: CoR,
}

impl<CR, PR, RR, CoR> ReportEngine<CR, PR, RR, CoR>
where
    CR: CodeResolverTrait,
    PR: ProjectRepository,
    RR: ResourceRepository,
    CoR: CompanyRepository,
{
    pub fn new(code_resolver: CR, project_repository: PR, resource_repository: RR, company_repository: CoR) -> Self {
        Self {
            _code_resolver: code_resolver,
            project_repository,
            resource_repository,
            company_repository,
        }
    }

    /// Gera um relatório baseado na configuração fornecida
    pub fn generate_report(&self, config: &ReportConfig) -> Result<ReportResult, AppError> {
        let start_time = std::time::Instant::now();

        let result = match config.report_type {
            ReportType::Task => self.generate_task_report(config),
            ReportType::Resource => self.generate_resource_report(config),
            ReportType::Project => self.generate_project_report(config),
            ReportType::Company => self.generate_company_report(config),
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(data) => Ok(ReportResult {
                success: true,
                data: Some(data),
                error: None,
                execution_time_ms: execution_time,
            }),
            Err(e) => Ok(ReportResult {
                success: false,
                data: None,
                error: Some(e.to_string()),
                execution_time_ms: execution_time,
            }),
        }
    }

    /// Exporta um relatório para um arquivo
    pub fn export_report(&self, config: &ReportConfig, file_path: &str) -> Result<ReportResult, AppError> {
        let report_result = self.generate_report(config)?;

        if !report_result.success {
            return Ok(report_result);
        }

        let data = report_result.data.unwrap();
        let formatter = FormatterFactory::create_formatter(&config.format);

        formatter
            .format_to_file(&data, file_path)
            .map_err(|e| AppError::validation_error("export", format!("Failed to export report: {}", e)))?;

        Ok(ReportResult {
            success: true,
            data: Some(data),
            error: None,
            execution_time_ms: report_result.execution_time_ms,
        })
    }

    /// Gera relatório de tarefas
    fn generate_task_report(&self, config: &ReportConfig) -> Result<ReportData, AppError> {
        let projects = self.project_repository.find_all()?;
        let mut all_tasks = Vec::new();

        for project in projects {
            let p = project;
            for task in p.tasks().values() {
                let task_data = self.task_to_json(task)?;
                all_tasks.push(task_data);
            }
        }

        let filtered_tasks = self.apply_filters(all_tasks, &config.filters)?;
        let sorted_tasks = self.apply_sorting(filtered_tasks, &config.sort_by, &config.sort_order)?;
        let grouped_tasks = self.apply_grouping(sorted_tasks, &config.group_by)?;

        let summary = if config.include_summary {
            Some(self.generate_summary(&grouped_tasks))
        } else {
            None
        };

        Ok(ReportData {
            title: "Task Report".to_string(),
            generated_at: Local::now(),
            total_records: grouped_tasks.len(),
            data: grouped_tasks,
            summary,
        })
    }

    /// Gera relatório de recursos
    fn generate_resource_report(&self, config: &ReportConfig) -> Result<ReportData, AppError> {
        let resources = self.resource_repository.find_all()?;
        let mut all_resources = Vec::new();

        for resource in resources {
            let resource_data = self.resource_to_json(&resource)?;
            all_resources.push(resource_data);
        }

        let filtered_resources = self.apply_filters(all_resources, &config.filters)?;
        let sorted_resources = self.apply_sorting(filtered_resources, &config.sort_by, &config.sort_order)?;
        let grouped_resources = self.apply_grouping(sorted_resources, &config.group_by)?;

        let summary = if config.include_summary {
            Some(self.generate_summary(&grouped_resources))
        } else {
            None
        };

        Ok(ReportData {
            title: "Resource Report".to_string(),
            generated_at: Local::now(),
            total_records: grouped_resources.len(),
            data: grouped_resources,
            summary,
        })
    }

    /// Gera relatório de projetos
    fn generate_project_report(&self, config: &ReportConfig) -> Result<ReportData, AppError> {
        let projects = self.project_repository.find_all()?;
        let mut all_projects = Vec::new();

        for project in projects {
            let project_data = self.project_to_json(&project)?;
            all_projects.push(project_data);
        }

        let filtered_projects = self.apply_filters(all_projects, &config.filters)?;
        let sorted_projects = self.apply_sorting(filtered_projects, &config.sort_by, &config.sort_order)?;
        let grouped_projects = self.apply_grouping(sorted_projects, &config.group_by)?;

        let summary = if config.include_summary {
            Some(self.generate_summary(&grouped_projects))
        } else {
            None
        };

        Ok(ReportData {
            title: "Project Report".to_string(),
            generated_at: Local::now(),
            total_records: grouped_projects.len(),
            data: grouped_projects,
            summary,
        })
    }

    /// Gera relatório de empresas
    fn generate_company_report(&self, config: &ReportConfig) -> Result<ReportData, AppError> {
        let companies = self.company_repository.find_all()?;
        let mut all_companies = Vec::new();

        for company in companies {
            let company_data = self.company_to_json(&company)?;
            all_companies.push(company_data);
        }

        let filtered_companies = self.apply_filters(all_companies, &config.filters)?;
        let sorted_companies = self.apply_sorting(filtered_companies, &config.sort_by, &config.sort_order)?;
        let grouped_companies = self.apply_grouping(sorted_companies, &config.group_by)?;

        let summary = if config.include_summary {
            Some(self.generate_summary(&grouped_companies))
        } else {
            None
        };

        Ok(ReportData {
            title: "Company Report".to_string(),
            generated_at: Local::now(),
            total_records: grouped_companies.len(),
            data: grouped_companies,
            summary,
        })
    }

    /// Converte uma tarefa para JSON
    fn task_to_json(&self, task: &AnyTask) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::Value::String(task.id().to_string()));
        data.insert("code".to_string(), serde_json::Value::String(task.code().to_string()));
        data.insert("name".to_string(), serde_json::Value::String(task.name().to_string()));
        data.insert(
            "description".to_string(),
            serde_json::Value::String(task.description().unwrap_or_default().to_string()),
        );
        data.insert(
            "project_code".to_string(),
            serde_json::Value::String(task.project_code().to_string()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String(task.status().to_string()),
        );
        data.insert("priority".to_string(), serde_json::Value::String("medium".to_string()));
        data.insert(
            "start_date".to_string(),
            serde_json::Value::String(task.start_date().format("%Y-%m-%d").to_string()),
        );
        data.insert(
            "due_date".to_string(),
            serde_json::Value::String(task.due_date().format("%Y-%m-%d").to_string()),
        );
        data.insert(
            "assigned_resources".to_string(),
            serde_json::Value::Number(serde_json::Number::from(task.assigned_resources().len())),
        );
        Ok(data)
    }

    /// Converte um recurso para JSON
    fn resource_to_json(&self, resource: &AnyResource) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::Value::String(resource.id().to_string()));
        data.insert(
            "code".to_string(),
            serde_json::Value::String(resource.code().to_string()),
        );
        data.insert(
            "name".to_string(),
            serde_json::Value::String(resource.name().to_string()),
        );
        data.insert(
            "email".to_string(),
            serde_json::Value::String(resource.email().unwrap_or_default().to_string()),
        );
        data.insert(
            "resource_type".to_string(),
            serde_json::Value::String(resource.resource_type().to_string()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String(resource.status().to_string()),
        );
        data.insert(
            "capacity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(100)),
        );
        data.insert(
            "cost_per_hour".to_string(),
            serde_json::Value::Number(serde_json::Number::from(0)),
        );
        Ok(data)
    }

    /// Converte um projeto para JSON
    fn project_to_json(&self, project: &AnyProject) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::Value::String(project.id().to_string()));
        data.insert(
            "code".to_string(),
            serde_json::Value::String(project.code().to_string()),
        );
        data.insert(
            "name".to_string(),
            serde_json::Value::String(project.name().to_string()),
        );
        data.insert(
            "description".to_string(),
            serde_json::Value::String(project.description().unwrap_or(&String::new()).to_string()),
        );
        data.insert(
            "company_code".to_string(),
            serde_json::Value::String(project.company_code().to_string()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String(project.status().to_string()),
        );
        data.insert("priority".to_string(), serde_json::Value::String("medium".to_string()));
        data.insert(
            "start_date".to_string(),
            serde_json::Value::String(project.start_date().unwrap_or_default().format("%Y-%m-%d").to_string()),
        );
        data.insert(
            "end_date".to_string(),
            serde_json::Value::String(project.end_date().unwrap_or_default().format("%Y-%m-%d").to_string()),
        );
        data.insert(
            "task_count".to_string(),
            serde_json::Value::Number(serde_json::Number::from(project.tasks().len())),
        );
        Ok(data)
    }

    /// Converte uma empresa para JSON
    fn company_to_json(&self, company: &Company) -> Result<HashMap<String, serde_json::Value>, AppError> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), serde_json::Value::String(company.id().to_string()));
        data.insert(
            "code".to_string(),
            serde_json::Value::String(company.code().to_string()),
        );
        data.insert(
            "name".to_string(),
            serde_json::Value::String(company.name().to_string()),
        );
        data.insert(
            "email".to_string(),
            serde_json::Value::String(company.email.clone().unwrap_or_default()),
        );
        data.insert(
            "status".to_string(),
            serde_json::Value::String(company.status.to_string()),
        );
        data.insert(
            "created_at".to_string(),
            serde_json::Value::String(company.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        );
        data.insert(
            "updated_at".to_string(),
            serde_json::Value::String(company.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        );
        Ok(data)
    }

    /// Aplica filtros aos dados
    fn apply_filters(
        &self,
        data: Vec<HashMap<String, serde_json::Value>>,
        filters: &[ReportFilter],
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, AppError> {
        let mut filtered_data = data;

        for filter in filters {
            filtered_data.retain(|record| self.matches_filter(record, filter));
        }

        Ok(filtered_data)
    }

    /// Verifica se um registro corresponde a um filtro
    fn matches_filter(&self, record: &HashMap<String, serde_json::Value>, filter: &ReportFilter) -> bool {
        let field_value = record.get(&filter.field);
        if field_value.is_none() {
            return false;
        }

        let field_value = field_value.unwrap();
        let filter_value = &filter.value;

        match filter.operator {
            FilterOperator::Equal => field_value.as_str().is_some_and(|s| s == filter_value),
            FilterOperator::NotEqual => field_value.as_str().is_none_or(|s| s != filter_value),
            FilterOperator::Contains => field_value.to_string().contains(filter_value),
            FilterOperator::NotContains => !field_value.to_string().contains(filter_value),
            FilterOperator::GreaterThan => {
                if let (Some(field_num), Ok(filter_num)) = (field_value.as_f64(), filter_value.parse::<f64>()) {
                    field_num > filter_num
                } else {
                    false
                }
            }
            FilterOperator::LessThan => {
                if let (Some(field_num), Ok(filter_num)) = (field_value.as_f64(), filter_value.parse::<f64>()) {
                    field_num < filter_num
                } else {
                    false
                }
            }
            FilterOperator::GreaterThanOrEqual => {
                if let (Some(field_num), Ok(filter_num)) = (field_value.as_f64(), filter_value.parse::<f64>()) {
                    field_num >= filter_num
                } else {
                    false
                }
            }
            FilterOperator::LessThanOrEqual => {
                if let (Some(field_num), Ok(filter_num)) = (field_value.as_f64(), filter_value.parse::<f64>()) {
                    field_num <= filter_num
                } else {
                    false
                }
            }
        }
    }

    /// Aplica ordenação aos dados
    fn apply_sorting(
        &self,
        mut data: Vec<HashMap<String, serde_json::Value>>,
        sort_by: &Option<String>,
        sort_order: &Option<SortOrder>,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, AppError> {
        if let Some(field) = sort_by {
            let order = sort_order.clone().unwrap_or(SortOrder::Ascending);
            data.sort_by(|a, b| {
                let a_val = a.get(field).map(|v| v.to_string()).unwrap_or_default();
                let b_val = b.get(field).map(|v| v.to_string()).unwrap_or_default();

                match order {
                    SortOrder::Ascending => a_val.cmp(&b_val),
                    SortOrder::Descending => b_val.cmp(&a_val),
                }
            });
        }
        Ok(data)
    }

    /// Aplica agrupamento aos dados
    fn apply_grouping(
        &self,
        data: Vec<HashMap<String, serde_json::Value>>,
        _group_by: &Option<GroupBy>,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, AppError> {
        // Implementação básica - retorna os dados sem agrupamento
        // TODO: Implementar agrupamento real
        Ok(data)
    }

    /// Gera resumo dos dados
    fn generate_summary(&self, data: &[HashMap<String, serde_json::Value>]) -> ReportSummary {
        let mut field_stats = HashMap::new();

        for record in data {
            for (field, value) in record {
                let stats = field_stats.entry(field.clone()).or_insert(FieldStats {
                    count: 0,
                    unique_count: 0,
                    min_value: None,
                    max_value: None,
                    avg_value: None,
                });

                stats.count += 1;

                if let Some(num) = value.as_f64() {
                    let num_value = serde_json::Value::Number(serde_json::Number::from_f64(num).unwrap());
                    if stats.min_value.is_none() {
                        stats.min_value = Some(num_value.clone());
                    }
                    if stats.max_value.is_none() {
                        stats.max_value = Some(num_value.clone());
                    }
                }
            }
        }

        // Calcular contagem única e média
        for stats in field_stats.values_mut() {
            stats.unique_count = stats.count; // Simplificado - assumir todos únicos
            if let (Some(min), Some(max)) = (&stats.min_value, &stats.max_value)
                && let (Some(min_num), Some(max_num)) = (min.as_f64(), max.as_f64())
            {
                stats.avg_value = Some((min_num + max_num) / 2.0);
            }
        }

        ReportSummary {
            total_count: data.len(),
            field_stats,
        }
    }
}
