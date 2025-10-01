use crate::application::errors::AppError;
use crate::application::test::data_validation::*;
use clap::{Args, Parser};
use serde_json;
use std::path::PathBuf;

/// Data validation command
#[derive(Parser, Debug)]
pub enum TestDataCommand {
    /// Run data validation
    Run(RunDataValidationArgs),
    /// Generate validation report
    Report(ReportDataValidationArgs),
    /// Validate specific entity type
    Validate(ValidateEntityArgs),
}

/// Arguments for running data validation
#[derive(Args, Debug)]
pub struct RunDataValidationArgs {
    /// Entity type to validate (company, project, task, resource, all)
    #[clap(short, long, default_value = "all")]
    pub entity: String,

    /// Output format (json, html, csv)
    #[clap(long, default_value = "json")]
    pub format: String,

    /// Output file for results
    #[clap(short, long, default_value = "./validation-results.json")]
    pub output: PathBuf,

    /// Verbose output
    #[clap(short, long)]
    pub verbose: bool,

    /// Generate detailed report
    #[clap(long)]
    pub report: bool,

    /// Include warnings in output
    #[clap(long)]
    pub include_warnings: bool,

    /// Fail on validation errors
    #[clap(long)]
    pub strict: bool,
}

/// Arguments for generating validation reports
#[derive(Args, Debug)]
pub struct ReportDataValidationArgs {
    /// Input file with validation results
    #[clap(short, long, default_value = "./validation-results.json")]
    pub input: PathBuf,

    /// Output file for the report
    #[clap(short, long, default_value = "./data-validation-report.html")]
    pub output: PathBuf,

    /// Report format (html, json, csv)
    #[clap(long, default_value = "html")]
    pub format: String,

    /// Include detailed validation results
    #[clap(long)]
    pub detailed: bool,
}

/// Arguments for validating specific entity
#[derive(Args, Debug)]
pub struct ValidateEntityArgs {
    /// Entity type to validate
    #[clap(short, long)]
    pub entity: String,

    /// Entity ID or code to validate
    #[clap(short, long)]
    pub id: Option<String>,

    /// Output format (json, html, csv)
    #[clap(long, default_value = "json")]
    pub format: String,

    /// Output file for results
    #[clap(short, long, default_value = "./entity-validation.json")]
    pub output: PathBuf,

    /// Verbose output
    #[clap(short, long)]
    pub verbose: bool,
}

impl TestDataCommand {
    /// Execute the data validation command
    pub async fn execute(&self, base_path: &str) -> Result<(), AppError> {
        match self {
            TestDataCommand::Run(args) => Self::run_validation(args, base_path).await,
            TestDataCommand::Report(args) => Self::generate_report(args, base_path).await,
            TestDataCommand::Validate(args) => Self::validate_entity(args, base_path).await,
        }
    }

    /// Run data validation
    async fn run_validation(args: &RunDataValidationArgs, base_path: &str) -> Result<(), AppError> {
        println!("Starting data validation...");
        if args.verbose {
            println!("Configuration:");
            println!("  Entity: {}", args.entity);
            println!("  Format: {}", args.format);
            println!("  Output: {:?}", args.output);
            println!("  Verbose: {}", args.verbose);
            println!("  Include Warnings: {}", args.include_warnings);
            println!("  Strict: {}", args.strict);
        }

        // Create validation service with repositories
        let validation_service = Self::create_validation_service(base_path)?;

        // Run validation based on entity type
        let results = match args.entity.as_str() {
            "company" => {
                println!("Validating companies...");
                validation_service.validate_companies().await?
            }
            "project" => {
                println!("Validating projects...");
                validation_service.validate_projects().await?
            }
            "task" => {
                println!("Validating tasks...");
                validation_service.validate_tasks().await?
            }
            "resource" => {
                println!("Validating resources...");
                validation_service.validate_resources().await?
            }
            "all" => {
                println!("Validating all entities...");
                validation_service.validate_all().await?
            }
            _ => {
                return Err(AppError::ValidationError {
                    field: "entity".to_string(),
                    message: format!("Unknown entity type: {}", args.entity),
                });
            }
        };

        // Dedupe results by (entity_type, entity_id)
        let results = Self::dedupe_results(results);
        // Calculate summary
        let summary = Self::calculate_summary(&results);

        // Create report
        let report = DataValidationReport { summary, results };

        // Save results based on format
        Self::save_results(&report, &args.output, &args.format, args.verbose).await?;

        // Print summary
        Self::print_summary(&report, args.verbose);

        // Check if we should fail on errors
        if args.strict && report.summary.total_errors > 0 {
            return Err(AppError::ValidationError {
                field: "validation".to_string(),
                message: format!("{} validation errors found", report.summary.total_errors),
            });
        }

        Ok(())
    }

    /// Generate validation report
    async fn generate_report(args: &ReportDataValidationArgs, _base_path: &str) -> Result<(), AppError> {
        println!("Generating data validation report...");
        if args.detailed {
            println!("  Detailed report requested");
        }

        // Load validation results
        let report = Self::load_validation_results(&args.input).await?;

        // Generate report based on format
        match args.format.as_str() {
            "html" => Self::generate_html_report(&report, &args.output, args.detailed).await?,
            "json" => Self::generate_json_report(&report, &args.output).await?,
            "csv" => Self::generate_csv_report(&report, &args.output).await?,
            _ => {
                return Err(AppError::ValidationError {
                    field: "format".to_string(),
                    message: "Unsupported report format".to_string(),
                });
            }
        }

        println!("Report generated successfully.");
        Ok(())
    }

    /// Validate specific entity
    async fn validate_entity(args: &ValidateEntityArgs, base_path: &str) -> Result<(), AppError> {
        println!("Validating {} entity...", args.entity);
        if args.verbose {
            println!("  ID: {:?}", args.id);
            println!("  Format: {}", args.format);
            println!("  Output: {:?}", args.output);
        }

        // Create validation service with repositories
        let validation_service = Self::create_validation_service(base_path)?;

        // Validate specific entity
        let results = match args.entity.as_str() {
            "company" => {
                if let Some(id) = &args.id {
                    validation_service.validate_company_by_id(id).await?
                } else {
                    validation_service.validate_companies().await?
                }
            }
            "project" => {
                if let Some(id) = &args.id {
                    validation_service.validate_project_by_id(id).await?
                } else {
                    validation_service.validate_projects().await?
                }
            }
            "task" => {
                if let Some(id) = &args.id {
                    validation_service.validate_task_by_id(id).await?
                } else {
                    validation_service.validate_tasks().await?
                }
            }
            "resource" => {
                if let Some(id) = &args.id {
                    validation_service.validate_resource_by_id(id).await?
                } else {
                    validation_service.validate_resources().await?
                }
            }
            _ => {
                return Err(AppError::ValidationError {
                    field: "entity".to_string(),
                    message: format!("Unknown entity type: {}", args.entity),
                });
            }
        };

        // Dedupe results by (entity_type, entity_id)
        let results = Self::dedupe_results(results);
        // Calculate summary
        let summary = Self::calculate_summary(&results);

        // Create report
        let report = DataValidationReport { summary, results };

        // Save results
        Self::save_results(&report, &args.output, &args.format, args.verbose).await?;

        // Print summary
        Self::print_summary(&report, args.verbose);

        Ok(())
    }

    /// Calculate validation summary
    fn calculate_summary(results: &[DataValidationResult]) -> DataValidationSummary {
        let total_entities = results.len();
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut valid_entities = 0;
        let mut entities_with_warnings = 0;

        for result in results {
            total_errors += result.errors.len();
            total_warnings += result.warnings.len();
            if result.errors.is_empty() {
                valid_entities += 1;
            }
            if !result.warnings.is_empty() {
                entities_with_warnings += 1;
            }
        }

        DataValidationSummary {
            total_entities,
            valid_entities,
            invalid_entities: total_entities - valid_entities,
            entities_with_errors: total_entities - valid_entities,
            entities_with_warnings,
            total_errors,
            total_warnings,
            success_rate: if total_entities > 0 {
                (valid_entities as f64 / total_entities as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Dedupe results by (entity_type, entity_id) and merge errors/warnings
    fn dedupe_results(results: Vec<DataValidationResult>) -> Vec<DataValidationResult> {
        use std::collections::{HashMap, HashSet};
        let mut map: HashMap<(String, String), DataValidationResult> = HashMap::new();
        for mut r in results.into_iter() {
            let key = (r.entity_type.clone(), r.entity_id.clone());
            map.entry(key)
                .and_modify(|existing| {
                    // merge errors
                    let mut seen_err: HashSet<(String, String)> = existing
                        .errors
                        .iter()
                        .map(|e| (e.field.clone(), e.message.clone()))
                        .collect();
                    for e in r.errors.drain(..) {
                        let sig = (e.field.clone(), e.message.clone());
                        if seen_err.insert(sig) {
                            existing.errors.push(e);
                        }
                    }
                    // merge warnings
                    let mut seen_warn: HashSet<(String, String)> = existing
                        .warnings
                        .iter()
                        .map(|w| (w.field.clone(), w.message.clone()))
                        .collect();
                    for w in r.warnings.drain(..) {
                        let sig = (w.field.clone(), w.message.clone());
                        if seen_warn.insert(sig) {
                            existing.warnings.push(w);
                        }
                    }
                })
                .or_insert(r);
        }
        map.into_values().collect()
    }

    /// Save validation results
    async fn save_results(
        report: &DataValidationReport,
        output_path: &PathBuf,
        format: &str,
        verbose: bool,
    ) -> Result<(), AppError> {
        if verbose {
            println!("Saving results to {:?} in {} format", output_path, format);
        }

        match format {
            "json" => {
                let json_content = serde_json::to_string_pretty(report)?;
                std::fs::write(output_path, json_content)?;
            }
            "html" => {
                Self::generate_html_report(report, output_path, true).await?;
            }
            "csv" => {
                Self::generate_csv_report(report, output_path).await?;
            }
            _ => {
                return Err(AppError::ValidationError {
                    field: "format".to_string(),
                    message: "Unsupported output format".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Load validation results from file
    async fn load_validation_results(input_path: &PathBuf) -> Result<DataValidationReport, AppError> {
        let content = std::fs::read_to_string(input_path)?;
        let report: DataValidationReport = serde_json::from_str(&content)?;
        Ok(report)
    }

    /// Generate HTML report
    async fn generate_html_report(
        report: &DataValidationReport,
        output_path: &PathBuf,
        detailed: bool,
    ) -> Result<(), AppError> {
        let mut html_content = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Data Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 20px; border-radius: 5px; margin-bottom: 20px; }}
        .entity-result {{ margin: 10px 0; padding: 15px; border-radius: 5px; border-left: 4px solid; }}
        .valid {{ border-left-color: #28a745; background: #d4edda; }}
        .invalid {{ border-left-color: #dc3545; background: #f8d7da; }}
        .warning {{ border-left-color: #ffc107; background: #fff3cd; }}
        .error {{ color: #dc3545; font-weight: bold; }}
        .warning-text {{ color: #856404; }}
        .stats {{ display: flex; gap: 20px; margin: 10px 0; }}
        .stat {{ text-align: center; }}
        .stat-value {{ font-size: 24px; font-weight: bold; }}
        .stat-label {{ font-size: 14px; color: #666; }}
    </style>
</head>
<body>
    <h1>Data Validation Report</h1>
    <div class="summary">
        <h2>Summary</h2>
        <div class="stats">
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Entities</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Valid Entities</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Invalid Entities</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Errors</div>
            </div>
            <div class="stat">
                <div class="stat-value">{}</div>
                <div class="stat-label">Total Warnings</div>
            </div>
            <div class="stat">
                <div class="stat-value">{:.1}%</div>
                <div class="stat-label">Success Rate</div>
            </div>
        </div>
    </div>
    <h2>Validation Results</h2>
"#,
            report.summary.total_entities,
            report.summary.valid_entities,
            report.summary.invalid_entities,
            report.summary.total_errors,
            report.summary.total_warnings,
            report.summary.success_rate,
        );

        // Add entity results
        for result in &report.results {
            let status_class = if result.errors.is_empty() { "valid" } else { "invalid" };

            html_content.push_str(&format!(
                r#"<div class="entity-result {}">
                    <h3>{} - {}</h3>
"#,
                status_class, result.entity_type, result.entity_id
            ));

            if detailed {
                if !result.errors.is_empty() {
                    html_content.push_str("<h4>Errors:</h4><ul>");
                    for error in &result.errors {
                        html_content.push_str(&format!("<li class=\"error\">{}: {}</li>", error.field, error.message));
                    }
                    html_content.push_str("</ul>");
                }

                if !result.warnings.is_empty() {
                    html_content.push_str("<h4>Warnings:</h4><ul>");
                    for warning in &result.warnings {
                        html_content.push_str(&format!(
                            "<li class=\"warning-text\">{}: {}</li>",
                            warning.field, warning.message
                        ));
                    }
                    html_content.push_str("</ul>");
                }
            }

            html_content.push_str("</div>");
        }

        html_content.push_str("</body></html>");

        std::fs::write(output_path, html_content)?;
        Ok(())
    }

    /// Generate JSON report
    async fn generate_json_report(report: &DataValidationReport, output_path: &PathBuf) -> Result<(), AppError> {
        let json_content = serde_json::to_string_pretty(report)?;
        std::fs::write(output_path, json_content)?;
        Ok(())
    }

    /// Generate CSV report
    async fn generate_csv_report(report: &DataValidationReport, output_path: &PathBuf) -> Result<(), AppError> {
        let mut csv_content =
            String::from("Entity Type,Entity ID,Status,Errors,Warnings,Error Details,Warning Details\n");

        for result in &report.results {
            let status = if result.errors.is_empty() { "Valid" } else { "Invalid" };
            let error_count = result.errors.len();
            let warning_count = result.warnings.len();
            let error_details = result
                .errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; ");
            let warning_details = result
                .warnings
                .iter()
                .map(|w| format!("{}: {}", w.field, w.message))
                .collect::<Vec<_>>()
                .join("; ");

            csv_content.push_str(&format!(
                "{},{},{},{},{},\"{}\",\"{}\"\n",
                result.entity_type,
                result.entity_id,
                status,
                error_count,
                warning_count,
                error_details,
                warning_details
            ));
        }

        std::fs::write(output_path, csv_content)?;
        Ok(())
    }

    /// Print validation summary
    fn print_summary(report: &DataValidationReport, verbose: bool) {
        println!("\nValidation Summary:");
        println!("  Total Entities: {}", report.summary.total_entities);
        println!("  Valid: {}", report.summary.valid_entities);
        println!("  Invalid: {}", report.summary.invalid_entities);
        println!("  Errors: {}", report.summary.total_errors);
        println!("  Warnings: {}", report.summary.total_warnings);
        println!("  Success Rate: {:.1}%", report.summary.success_rate);

        if verbose && report.summary.total_errors > 0 {
            println!("\nValidation Errors:");
            for result in &report.results {
                if !result.errors.is_empty() {
                    println!("  {} - {}:", result.entity_type, result.entity_id);
                    for error in &result.errors {
                        println!("    - {}: {}", error.field, error.message);
                    }
                }
            }
        }

        if verbose && report.summary.total_warnings > 0 {
            println!("\nValidation Warnings:");
            for result in &report.results {
                if !result.warnings.is_empty() {
                    println!("  {} - {}:", result.entity_type, result.entity_id);
                    for warning in &result.warnings {
                        println!("    - {}: {}", warning.field, warning.message);
                    }
                }
            }
        }
    }

    /// Create validation service with repositories
    fn create_validation_service(base_path: &str) -> Result<DataValidationService, AppError> {
        use crate::infrastructure::persistence::{
            company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
            resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
        };
        use std::sync::Arc;

        // Create repositories
        let company_repo = Arc::new(FileCompanyRepository::new(base_path));
        let project_repo = Arc::new(FileProjectRepository::with_base_path(base_path.into()));
        let resource_repo = Arc::new(FileResourceRepository::new(base_path));
        let task_repo = Arc::new(FileTaskRepository::new(base_path));

        // Create code resolver for code->ID validation using the same base path
        let code_resolver = crate::application::shared::code_resolver::CodeResolver::new(base_path);

        // Create validation service
        Ok(DataValidationService::new(
            company_repo,
            project_repo,
            resource_repo,
            task_repo,
            Some(code_resolver),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_summary() {
        let results = vec![
            DataValidationResult {
                entity_type: "Company".to_string(),
                entity_id: "C1".to_string(),
                errors: vec![],
                warnings: vec![],
            },
            DataValidationResult {
                entity_type: "Project".to_string(),
                entity_id: "P1".to_string(),
                errors: vec![ValidationError {
                    field: "code".to_string(),
                    expected: "non-empty".to_string(),
                    actual: "".to_string(),
                    message: "Project code cannot be empty".to_string(),
                }],
                warnings: vec![],
            },
        ];

        let summary = TestDataCommand::calculate_summary(&results);
        assert_eq!(summary.total_entities, 2);
        assert_eq!(summary.valid_entities, 1);
        assert_eq!(summary.invalid_entities, 1);
        assert_eq!(summary.entities_with_errors, 1);
        assert_eq!(summary.total_errors, 1);
        assert_eq!(summary.total_warnings, 0);
        assert!((summary.success_rate - 50.0).abs() < f64::EPSILON);
    }
}
