use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    application::{
        create_project_use_case::CreateProjectUseCase,
        create_resource_use_case::CreateResourceUseCase,
        initialize_repository_use_case::InitializeRepositoryUseCase,
        validate_vacations_use_case::ValidateVacationsUseCase,
        create_vacation_use_case::CreateVacationUseCase,
    },
    infrastructure::persistence::{
        config_repository::FileConfigRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
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
        resource: String, // Pode ser código ou nome
        #[arg(long, short)]
        start_date: String,
        #[arg(long, short)]
        end_date: String,
        #[arg(long, short, default_value = "false")]
        is_time_off_compensation: bool,
        #[arg(long, short)]
        compensated_hours: Option<u32>,
    },
}

#[derive(Subcommand)]
enum ValidateCommands {
    Vacations,
}

pub fn run(cli: Cli) -> Result<()> {
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
                    Ok(resource) => println!("✅ Período de férias adicionado com sucesso para {}", resource.name),
                    Err(e) => println!("❌ Erro ao adicionar período de férias: {}", e),
                }
            }
        },
        Commands::Validate { validate_command } => match validate_command {
            ValidateCommands::Vacations => {
                let project_repository = FileProjectRepository::new();
                let resource_repository = FileResourceRepository::new();
                let use_case = ValidateVacationsUseCase::new(
                    project_repository,
                    resource_repository,
                );

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
    }

    Ok(())
}
