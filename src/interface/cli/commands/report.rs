use crate::application::errors::AppError;
use crate::application::report::engine::ReportEngine;
use crate::application::report::types::{
    ExportFormat, FilterOperator, GroupBy, ReportConfig, ReportFilter, ReportType, SortOrder,
};
use crate::application::shared::code_resolver::CodeResolver;
use crate::infrastructure::persistence::company_repository::FileCompanyRepository;
use crate::infrastructure::persistence::project_repository::FileProjectRepository;
use crate::infrastructure::persistence::resource_repository::FileResourceRepository;
use clap::{Args, Subcommand};
use std::path::Path;

#[derive(Subcommand)]
pub enum ReportCommand {
    /// Generate a report
    Generate(ReportArgs),
}

#[derive(Args)]
pub struct ReportArgs {
    /// Type of report to generate
    #[arg(short = 't', long = "type", value_enum)]
    pub report_type: ReportType,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum, default_value = "json")]
    pub format: ExportFormat,

    /// Output file path
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Filter by field:value
    #[arg(long = "filter", value_name = "FIELD:VALUE")]
    pub filters: Vec<String>,

    /// Group by field
    #[arg(long = "group-by")]
    pub group_by: Option<String>,

    /// Sort by field
    #[arg(long = "sort-by")]
    pub sort_by: Option<String>,

    /// Sort order (asc/desc)
    #[arg(long = "sort-order", value_enum, default_value = "ascending")]
    pub sort_order: SortOrder,

    /// Include summary statistics
    #[arg(long = "summary")]
    pub include_summary: bool,

    /// Template name
    #[arg(long = "template")]
    pub template: Option<String>,

    /// Project code filter
    #[arg(long = "project")]
    pub project: Option<String>,

    /// Company code filter
    #[arg(long = "company")]
    pub company: Option<String>,

    /// Resource code filter
    #[arg(long = "resource")]
    pub resource: Option<String>,
}

pub fn execute_report(args: ReportArgs) -> Result<(), AppError> {
    let base_path = Path::new(".");
    let code_resolver = CodeResolver::new(base_path);
    let project_repository = FileProjectRepository::new();
    let resource_repository = FileResourceRepository::new(base_path);
    let company_repository = FileCompanyRepository::new(base_path);

    let report_engine = ReportEngine::new(
        code_resolver,
        project_repository,
        resource_repository,
        company_repository,
    );

    let mut config = ReportConfig {
        report_type: args.report_type,
        format: args.format,
        filters: parse_filters(&args.filters)?,
        group_by: args.group_by.map(|field| GroupBy {
            field,
            sort_order: args.sort_order.clone(),
        }),
        sort_by: args.sort_by,
        sort_order: Some(args.sort_order),
        include_summary: args.include_summary,
        template: args.template,
    };

    // Adicionar filtros específicos baseados nos argumentos
    if let Some(project) = args.project {
        config.filters.push(ReportFilter {
            field: "project_code".to_string(),
            operator: FilterOperator::Equal,
            value: project,
        });
    }

    if let Some(company) = args.company {
        config.filters.push(ReportFilter {
            field: "company_code".to_string(),
            operator: FilterOperator::Equal,
            value: company,
        });
    }

    if let Some(resource) = args.resource {
        config.filters.push(ReportFilter {
            field: "code".to_string(),
            operator: FilterOperator::Equal,
            value: resource,
        });
    }

    let result = if let Some(ref output_path) = args.output {
        report_engine.export_report(&config, output_path)?
    } else {
        report_engine.generate_report(&config)?
    };

    if result.success {
        if let Some(data) = result.data {
            println!("✅ Report generated successfully!");
            println!("📊 Title: {}", data.title);
            println!("📅 Generated at: {}", data.generated_at.format("%Y-%m-%d %H:%M:%S"));
            println!("📈 Total records: {}", data.total_records);
            println!("⏱️  Execution time: {}ms", result.execution_time_ms);

            if let Some(summary) = data.summary {
                println!("\n📋 Summary:");
                println!("  Total count: {}", summary.total_count);
                for (field, stats) in summary.field_stats {
                    println!("  {}: {} records, {} unique", field, stats.count, stats.unique_count);
                }
            }

            if args.output.is_none() {
                println!("\n💡 Use --output to save report to file");
            }
        }
    } else {
        eprintln!("❌ Failed to generate report: {}", result.error.unwrap_or_default());
        return Err(AppError::validation_error("report", "Report generation failed"));
    }

    Ok(())
}

fn parse_filters(filter_strings: &[String]) -> Result<Vec<ReportFilter>, AppError> {
    let mut filters = Vec::new();

    for filter_str in filter_strings {
        let parts: Vec<&str> = filter_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AppError::validation_error(
                "filter",
                format!("Invalid filter format: {}. Expected field:value", filter_str),
            ));
        }

        let field = parts[0].to_string();
        let value = parts[1].to_string();
        let operator = determine_operator(&value);

        filters.push(ReportFilter {
            field,
            operator,
            value: clean_value(value),
        });
    }

    Ok(filters)
}

fn determine_operator(value: &str) -> FilterOperator {
    if value.starts_with('>') {
        FilterOperator::GreaterThan
    } else if value.starts_with('<') {
        FilterOperator::LessThan
    } else if value.starts_with(">=") {
        FilterOperator::GreaterThanOrEqual
    } else if value.starts_with("<=") {
        FilterOperator::LessThanOrEqual
    } else if value.starts_with('!') {
        FilterOperator::NotEqual
    } else if value.contains("~") {
        FilterOperator::Contains
    } else {
        FilterOperator::Equal
    }
}

fn clean_value(value: String) -> String {
    value
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('!')
        .trim_start_matches('~')
        .to_string()
}
