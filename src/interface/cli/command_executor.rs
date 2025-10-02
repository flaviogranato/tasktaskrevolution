use crate::interface::cli::commands;
use std::path::PathBuf;

pub fn execute_init(
    name: String,
    email: String,
    company_name: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::init::{InitManagerData, InitManagerUseCase};
    use crate::interface::cli::handlers::get_app_handler;

    let app = get_app_handler().get_app();
    let config_repo = &app.config_repository;

    let init_data = InitManagerData {
        name: name.clone(),
        email: email.clone(),
        timezone,
        work_hours_start,
        work_hours_end,
        work_days,
        company_name: company_name.clone(),
    };

    let init_use_case = InitManagerUseCase::new(Box::new(config_repo.clone()));

    match init_use_case.execute(init_data) {
        Ok(_config) => {
            if !crate::interface::cli::Cli::is_quiet() {
                println!("Manager/Consultant configured successfully");
                if crate::interface::cli::Cli::is_verbose() {
                    println!("Name: {}", name);
                    println!("Email: {}", email);
                    println!("Company: {}", company_name);
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to initialize system: {}", e);
            Err(Box::new(e))
        }
    }
}

pub fn execute_build(output: PathBuf, _base_url: String) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::build_use_case::BuildUseCase;

    if crate::interface::cli::Cli::is_verbose() {
        eprintln!("Building static site...");
    }

    let current_dir = std::env::current_dir()?;
    let build_use_case = BuildUseCase::new(current_dir, output.to_str().unwrap_or("dist"))?;

    match build_use_case.execute() {
        Ok(_) => {
            if !crate::interface::cli::Cli::is_quiet() {
                println!("Static site built successfully!");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to build static site: {}", e);
            Err(e)
        }
    }
}

pub fn execute_validate(command: commands::ValidateCommand) -> Result<(), Box<dyn std::error::Error>> {
    use crate::application::validate::{
        business_rules::ValidateBusinessRulesUseCase, data_integrity::ValidateDataIntegrityUseCase,
        entities::ValidateEntitiesUseCase, system::ValidateSystemUseCase,
    };
    use crate::infrastructure::persistence::{
        company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
    };

    let project_repository = FileProjectRepository::new();
    let resource_repository = FileResourceRepository::new(".");
    let company_repository = FileCompanyRepository::new(".");

    match command {
        commands::ValidateCommand::BusinessRules(args) => {
            if crate::interface::cli::Cli::is_verbose() {
                eprintln!("Running business rules validation...");
            }
            let validate_use_case =
                ValidateBusinessRulesUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(results) => {
                    if !crate::interface::cli::Cli::is_quiet() {
                        println!("Business rules validation completed");
                    }
                    print_validation_results(&results, &args);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Business rules validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::DataIntegrity(args) => {
            if crate::interface::cli::Cli::is_verbose() {
                eprintln!("Running data integrity validation...");
            }
            let validate_use_case =
                ValidateDataIntegrityUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(results) => {
                    if !crate::interface::cli::Cli::is_quiet() {
                        println!("Data integrity validation completed");
                    }
                    print_validation_results(&results, &args);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Data integrity validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::Entities(args) => {
            if crate::interface::cli::Cli::is_verbose() {
                eprintln!("Running entities validation...");
            }
            let validate_use_case =
                ValidateEntitiesUseCase::new(&project_repository, &resource_repository, &company_repository);
            match validate_use_case.execute() {
                Ok(results) => {
                    if !crate::interface::cli::Cli::is_quiet() {
                        println!("Entities validation completed");
                    }
                    print_validation_results(&results, &args);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Entities validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
        commands::ValidateCommand::System(args) => {
            if crate::interface::cli::Cli::is_verbose() {
                eprintln!("Running system validation...");
            }
            let validate_use_case =
                ValidateSystemUseCase::new(project_repository, resource_repository, company_repository);
            match validate_use_case.execute() {
                Ok(results) => {
                    if !crate::interface::cli::Cli::is_quiet() {
                        println!("System validation completed");
                    }
                    print_validation_results(&results, &args);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("System validation failed: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}

fn print_validation_results(results: &[crate::application::validate::types::ValidationResult], args: &crate::interface::cli::commands::validate::ValidateArgs) {
    use serde_json;
    use std::fs;

    // Filter results based on include_warnings flag
    let filtered_results: Vec<_> = if !args.include_warnings {
        results.iter()
            .filter(|r| matches!(r.level, crate::application::validate::types::ValidationSeverity::Error))
            .cloned()
            .collect()
    } else {
        results.to_vec()
    };

    // Check for errors in strict mode
    if args.strict {
        let has_errors = filtered_results.iter()
            .any(|r| matches!(r.level, crate::application::validate::types::ValidationSeverity::Error));
        if has_errors {
            eprintln!("Validation failed in strict mode");
            std::process::exit(1);
        }
    }

    // Output results based on format
    match args.format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&filtered_results).unwrap_or_else(|_| "[]".to_string());
            if let Some(output_path) = &args.output {
                if let Err(e) = fs::write(output_path, &json_output) {
                    eprintln!("Failed to write to file: {}", e);
                } else {
                    println!("Validation results written to file");
                }
            } else {
                println!("{}", json_output);
            }
        }
        "html" => {
            let html_output = generate_html_report(&filtered_results);
            if let Some(output_path) = &args.output {
                if let Err(e) = fs::write(output_path, &html_output) {
                    eprintln!("Failed to write to file: {}", e);
                } else {
                    println!("Validation report written to file");
                }
            } else {
                println!("{}", html_output);
            }
        }
        "table" => {
            print_table_output(&filtered_results);
        }
        _ => {
            eprintln!("Unknown format: {}", args.format);
            std::process::exit(1);
        }
    }
}

fn generate_html_report(results: &[crate::application::validate::types::ValidationResult]) -> String {
    let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Validation Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .error { color: #d32f2f; }
        .warning { color: #f57c00; }
        .info { color: #1976d2; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <h1>Validation Report</h1>
    <table>
        <tr>
            <th>Level</th>
            <th>Code</th>
            <th>Message</th>
            <th>Path</th>
            <th>Entity</th>
        </tr>
"#);

    for result in results {
        let level_class = match result.level {
            crate::application::validate::types::ValidationSeverity::Error => "error",
            crate::application::validate::types::ValidationSeverity::Warning => "warning",
            crate::application::validate::types::ValidationSeverity::Info => "info",
        };

        let entity_info = if let (Some(entity_type), Some(entity_code)) = (&result.entity_type, &result.entity_code) {
            format!("{}: {}", entity_type, entity_code)
        } else {
            String::from("-")
        };

        html.push_str(&format!(
            r#"        <tr>
            <td class="{}">{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
"#,
            level_class,
            result.level,
            result.code,
            result.message,
            result.path.as_deref().unwrap_or("-"),
            entity_info
        ));
    }

    html.push_str(r#"
    </table>
</body>
</html>
"#);

    html
}

fn print_table_output(results: &[crate::application::validate::types::ValidationResult]) {
    if results.is_empty() {
        println!("No validation issues found");
        return;
    }

    println!("\nValidation Results:");
    println!("{:<10} {:<15} {:<50} {:<30} {:<20}", "Level", "Code", "Message", "Path", "Entity");
    println!("{}", "-".repeat(125));

    for result in results {
        let entity_info = if let (Some(entity_type), Some(entity_code)) = (&result.entity_type, &result.entity_code) {
            format!("{}: {}", entity_type, entity_code)
        } else {
            String::from("-")
        };

        println!(
            "{:<10} {:<15} {:<50} {:<30} {:<20}",
            result.level,
            result.code,
            result.message,
            result.path.as_deref().unwrap_or("-"),
            entity_info
        );
    }
}
