use std::{env, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    application::{
        create_config::InitializeRepositoryUseCase, create_project::create_project,
        create_resource::create_resource,
    },
    infrastructure::persistence::config_repository::FileConfigRepository,
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

#[derive(Subcommand)]
enum CreateCommands {
    Project {
        name: String,
        #[clap(short, long)]
        description: Option<String>,
    },
    Resource {
        name: String,
        #[clap(short, long)]
        resource_type: String,
    },
    Task {
        description: String,
        #[clap(short, long)]
        project: Option<String>,
        #[clap(short, long)]
        resource: Option<String>,
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
            let config_repository = FileConfigRepository::new();
            let use_case = InitializeRepositoryUseCase::new(config_repository);
            let repo_path = path.clone().unwrap_or(std::env::current_dir()?);

            use_case.execute(repo_path, manager_name, manager_email)?;
        }
        Commands::Create { create_command } => {
            let config_path = std::env::current_dir()?;

            match create_command {
                CreateCommands::Project { name, description } => {
                    let _ = create_project(&config_path, name, description);
                }
                CreateCommands::Resource {
                    name,
                    resource_type,
                } => {
                    let _ = create_resource(name, resource_type);
                }
                &CreateCommands::Task { .. } => todo!(),
            }
        }
        Commands::Validate { validate_command } => match validate_command {
            ValidateCommands::Vacations => {
                println!("validando as férias")
            }
        },
    }

    Ok(())
}
