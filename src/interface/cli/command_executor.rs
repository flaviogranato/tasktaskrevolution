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
    use crate::application::validate::serializer::ValidationSerializer;
    use crate::application::validate::types::OutputFormat;
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

    // Parse output format
    let format = match args.format.parse::<OutputFormat>() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Serialize results using the new serializer
    let output = match ValidationSerializer::serialize(&filtered_results, format) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Failed to serialize results: {}", e);
            std::process::exit(1);
        }
    };

    // Output results
    if let Some(output_path) = &args.output {
        if let Err(e) = fs::write(output_path, &output) {
            eprintln!("Failed to write to file: {}", e);
        } else {
            if !crate::interface::cli::Cli::is_quiet() {
                println!("Validation results written to file");
            }
        }
    } else {
        println!("{}", output);
    }
}

