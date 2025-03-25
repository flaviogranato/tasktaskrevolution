use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};

use crate::{
    application::{
        create_project_use_case::CreateProjectUseCase,
        create_resource_use_case::CreateResourceUseCase, create_task_use_case::CreateTaskUseCase,
        create_time_off_use_case::CreateTimeOffUseCase,
        create_vacation_use_case::CreateVacationUseCase,
        initialize_repository_use_case::InitializeRepositoryUseCase,
        vacation_report_use_case::VacationReportUseCase,
        validate_vacations_use_case::ValidateVacationsUseCase,
    },
    infrastructure::persistence::{
        config_repository::FileConfigRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
    },
};

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        path: Option<PathBuf>,
        #[clap(long, value_name = "NAME")]
        manager_name: String,
        #[clap(long, value_name = "EMAIL")]
        manager_email: String,
    },
    Create {
        #[clap(subcommand)]
        create_command: CreateCommands,
    },
    Validate {
        #[clap(subcommand)]
        validate_command: ValidateCommands,
    },
    Report {
        #[clap(subcommand)]
        report_command: ReportCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    Project {
        name: String,
        description: Option<String>,
    },
    Resource {
        name: String,
        resource_type: String,
    },
    Vacation {
        #[arg(long, short)]
        resource: String,
        #[arg(long, short)]
        start_date: String,
        #[arg(long, short)]
        end_date: String,
        #[arg(long, short, default_value = "false")]
        is_time_off_compensation: bool,
        #[arg(long, short)]
        compensated_hours: Option<u32>,
    },
    TimeOff {
        #[arg(long, short)]
        resource: String,
        #[arg(long, short)]
        hours: u32,
        #[arg(long, short)]
        date: String,
        #[arg(long, short)]
        description: Option<String>,
    },
    Task {
        #[arg(long, short)]
        name: String,
        #[arg(long, short)]
        description: Option<String>,
        #[arg(long)]
        due_date: Option<String>,
    },
}

#[derive(Subcommand)]
enum ValidateCommands {
    Vacations,
}

#[derive(Subcommand)]
enum ReportCommands {
    Vacation,
}

pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    match &cli.command {
        Commands::Init {
            path,
            manager_name,
            manager_email,
        } => {
            let repository = FileConfigRepository::new();
            let use_case = InitializeRepositoryUseCase::new(repository);
            let repo_path = path.clone().unwrap_or(std::env::current_dir()?);

            use_case.execute(repo_path, manager_name.clone(), manager_email.clone())?;
        }
        Commands::Create { create_command } => match create_command {
            CreateCommands::Project { name, description } => {
                let repository = FileProjectRepository::new();
                let use_case = CreateProjectUseCase::new(repository);

                use_case.execute(name.clone(), description.clone())?;
            }
            CreateCommands::Resource {
                name,
                resource_type,
            } => {
                let repository = FileResourceRepository::new();
                let use_case = CreateResourceUseCase::new(repository);

                let _ = use_case.execute(name.clone(), resource_type.clone());
            }
            CreateCommands::Vacation {
                resource,
                start_date,
                end_date,
                is_time_off_compensation,
                compensated_hours,
            } => {
                let repository = FileResourceRepository::new();
                let use_case = CreateVacationUseCase::new(repository);

                match use_case.execute(
                    resource.clone(),
                    start_date.clone(),
                    end_date.clone(),
                    *is_time_off_compensation,
                    *compensated_hours,
                ) {
                    Ok(resource) => println!(
                        "✅ Período de férias adicionado com sucesso para {}",
                        resource.name
                    ),
                    Err(e) => println!("❌ Erro ao adicionar período de férias: {}", e),
                }
            }
            CreateCommands::TimeOff {
                resource,
                hours,
                date,
                description,
            } => {
                let repository = FileResourceRepository::new();
                let use_case = CreateTimeOffUseCase::new(repository);

                match use_case.execute(resource.clone(), *hours, date.clone(), description.clone())
                {
                    Ok(resource) => {
                        println!(
                            "✅ {} horas adicionadas com sucesso para {}",
                            hours, resource.name
                        );
                        println!("📊 Novo saldo: {} horas", resource.time_off_balance);
                        if let Some(desc) = description {
                            println!("📝 Descrição: {}", desc);
                        }
                        println!("📅 Data: {}", date);
                    }
                    Err(e) => println!("❌ Erro ao adicionar horas extras: {}", e),
                }
            }
            CreateCommands::Task {
                name,
                description,
                due_date,
            } => {
                let current_dir = std::env::current_dir()?;
                let repository = FileTaskRepository::new(current_dir);
                let use_case = CreateTaskUseCase::new(repository);

                let due_date = due_date.as_ref().map(|date| {
                    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                        .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                        .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
                        .unwrap()
                });

                match use_case.execute(name.clone(), description.clone(), due_date) {
                    Ok(task) => println!("✅ Tarefa '{}' criada com sucesso", task.name),
                    Err(e) => println!("❌ Erro ao criar tarefa: {}", e),
                }
            }
        },
        Commands::Validate { validate_command } => match validate_command {
            ValidateCommands::Vacations => {
                let project_repository = FileProjectRepository::new();
                let resource_repository = FileResourceRepository::new();
                let use_case =
                    ValidateVacationsUseCase::new(project_repository, resource_repository);

                match use_case.execute() {
                    Ok(mensagens) => {
                        println!("\nResultado da validação de férias:");
                        println!("--------------------------------");
                        for mensagem in mensagens {
                            println!("{}", mensagem);
                        }
                    }
                    Err(e) => println!("Erro ao validar férias: {}", e),
                }
            }
        },
        Commands::Report { report_command } => match report_command {
            ReportCommands::Vacation => {
                let use_case = VacationReportUseCase::new();
                use_case.execute()?;
            }
        },
    }

    Ok(())
}
