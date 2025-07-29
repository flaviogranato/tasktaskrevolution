use crate::{
    application::{
        build_use_case::BuildUseCase,
        create_project_use_case::CreateProjectUseCase,
        create_resource_use_case::CreateResourceUseCase,
        create_task_use_case::{CreateTaskArgs, CreateTaskUseCase},
        create_time_off_use_case::CreateTimeOffUseCase,
        create_vacation_use_case::CreateVacationUseCase,
        initialize_repository_use_case::InitializeRepositoryUseCase,
        task_report_use_case::TaskReportUseCase,
        vacation_report_use_case::VacationReportUseCase,
        validate_vacations_use_case::ValidateVacationsUseCase,
    },
    infrastructure::persistence::{
        config_repository::FileConfigRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
    },
};
use clap::{Parser, Subcommand};
use csv::Writer;
use std::{env, path::PathBuf};

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
    Build {
        /// Opcional: Caminho para o diretório do projeto.
        /// Se não for fornecido, usa o diretório atual.
        path: Option<PathBuf>,
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
        project_code: String,
        #[arg(long, short)]
        code: String,
        #[arg(long, short)]
        name: String,
        #[arg(long, short)]
        description: Option<String>,
        #[arg(long, short)]
        start_date: String,
        #[arg(long, short)]
        due_date: String,
        #[arg(long, short, value_delimiter = ',')]
        assignees: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ValidateCommands {
    Vacations,
}

#[derive(Subcommand)]
enum ReportCommands {
    Vacation,
    Task,
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
            Ok(())
        }
        Commands::Build { path } => {
            let project_path = path.clone().unwrap_or_else(|| PathBuf::from("."));
            let output_dir = project_path.join("public");

            match BuildUseCase::new(project_path, output_dir.to_str().unwrap()) {
                Ok(use_case) => {
                    if let Err(e) = use_case.execute() {
                        println!("❌ Erro ao construir o site: {e}");
                    }
                }
                Err(e) => {
                    println!("❌ Erro ao inicializar o builder: {e}");
                }
            }
            Ok(())
        }
        Commands::Create { create_command } => match create_command {
            CreateCommands::Project { name, description } => {
                let repository = FileProjectRepository::new();
                let use_case = CreateProjectUseCase::new(repository);

                use_case.execute(name.clone(), description.clone())?;
                Ok(())
            }
            CreateCommands::Resource { name, resource_type } => {
                let repository = FileResourceRepository::new();
                let use_case = CreateResourceUseCase::new(repository);

                let _ = use_case.execute(name.clone(), resource_type.clone());
                Ok(())
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
                    Ok(result) => {
                        if result.success {
                            println!("✅ {}", result.message);
                        } else {
                            println!("❌ {}", result.message);
                        }
                    }
                    Err(e) => println!("❌ Erro inesperado: {e}"),
                };
                Ok(())
            }
            CreateCommands::TimeOff {
                resource,
                hours,
                date,
                description,
            } => {
                let repository = FileResourceRepository::new();
                let use_case = CreateTimeOffUseCase::new(repository);

                match use_case.execute(resource.clone(), *hours, date.clone(), description.clone()) {
                    Ok(result) => {
                        if result.success {
                            println!("✅ {}", result.message);
                            println!("📊 Novo saldo: {} horas", result.time_off_balance);
                            if let Some(desc) = &result.description {
                                println!("📝 Descrição: {desc}");
                            }
                            println!("📅 Data: {}", result.date);
                        } else {
                            println!("❌ {}", result.message);
                        }
                    }
                    Err(e) => println!("❌ Erro inesperado: {e}"),
                };
                Ok(())
            }
            CreateCommands::Task {
                project_code,
                code,
                name,
                description,
                start_date,
                due_date,
                assignees,
            } => {
                use chrono::NaiveDate;

                let start = match NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("❌ Erro: Data de início inválida. Use o formato YYYY-MM-DD");
                        return Ok(());
                    }
                };

                let due = match NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("❌ Erro: Data de vencimento inválida. Use o formato YYYY-MM-DD");
                        return Ok(());
                    }
                };

                let repository = FileTaskRepository::new();
                let use_case = CreateTaskUseCase::new(repository);

                let args = CreateTaskArgs {
                    project_code: project_code.clone(),
                    code: code.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    start_date: start,
                    due_date: due,
                    assigned_resources: assignees.clone(),
                };

                match use_case.execute(args) {
                    Ok(_) => {
                        println!("✅ Task '{name}' criada com sucesso!");
                        println!("📋 Código: {code}");
                        if let Some(desc) = description {
                            println!("📝 Descrição: {desc}");
                        }
                        println!("📅 Período: {start_date} até {due_date}");
                        if !assignees.is_empty() {
                            println!("👥 Responsáveis: {}", assignees.join(", "));
                        }
                    }
                    Err(e) => {
                        println!("❌ Erro ao criar task: {e}");
                    }
                };
                Ok(())
            }
        },
        Commands::Validate { validate_command } => {
            match validate_command {
                ValidateCommands::Vacations => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new();
                    let use_case = ValidateVacationsUseCase::new(project_repository, resource_repository);

                    match use_case.execute() {
                        Ok(mensagens) => {
                            println!("\nResultado da validação de férias:");
                            println!("--------------------------------");
                            for mensagem in mensagens {
                                println!("{mensagem}");
                            }
                        }
                        Err(e) => println!("Erro ao validar férias: {e}"),
                    }
                }
            }
            Ok(())
        }
        Commands::Report { report_command } => {
            match report_command {
                ReportCommands::Vacation => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new();
                    let use_case = VacationReportUseCase::new(project_repository, resource_repository);

                    let file_path = "vacation_report.csv";
                    match Writer::from_path(file_path) {
                        Ok(mut writer) => {
                            if let Err(e) = use_case.execute(&mut writer) {
                                println!("❌ Erro ao gerar relatório: {e}");
                            } else {
                                println!("✅ Relatório de férias gerado com sucesso em: {file_path}");
                            }
                        }
                        Err(e) => {
                            println!("❌ Erro ao criar arquivo de relatório: {e}");
                        }
                    }
                }
                ReportCommands::Task => {
                    let task_repo = FileTaskRepository::new();
                    let use_case = TaskReportUseCase::new(task_repo);

                    let file_path = "tasks_report.csv";
                    match Writer::from_path(file_path) {
                        Ok(mut writer) => {
                            if let Err(e) = use_case.execute(&mut writer) {
                                println!("❌ Erro ao gerar relatório de tarefas: {e}");
                            } else {
                                println!("✅ Relatório de tarefas gerado com sucesso em: {file_path}");
                            }
                        }
                        Err(e) => {
                            println!("❌ Erro ao criar arquivo de relatório de tarefas: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
