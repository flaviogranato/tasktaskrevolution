use std::path::Path;
use std::{env, fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_yml::to_string;

use crate::domain::config::ConfigManifest;
use crate::domain::project::ProjectManifest;
use crate::domain::resource::ResourceManifest;

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"), 
    version = env!("CARGO_PKG_VERSION"), 
    about = env!("CARGO_PKG_DESCRIPTION"), 
    long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
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
        #[clap(short, long)]
        project: Option<String>,
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
            let repo_path = path.clone().unwrap_or(std::env::current_dir()?);

            if !repo_path.exists() {
                match fs::create_dir(&repo_path) {
                    Ok(_) => println!("Criado o repositório de configurações"),
                    Err(e) => println!("Erro ao criar diretório de resources: {}", e),
                }
            }

            let config_path = repo_path.join("config.yaml");
            let config = ConfigManifest::basic(manager_name, manager_email);

            let config_yaml = to_string(&config)?;

            if let Err(e) = fs::write(config_path, config_yaml) {
                eprintln!("Erro ao criar o arquivo config.yaml: {}", e);
                return Ok(());
            }

            println!("Repositório inicializado em: {}", repo_path.display());
        }
        Commands::Create { create_command } => {
            let config_path = std::env::current_dir()?;

            match create_command {
                CreateCommands::Project { name, description } => {
                    let project_path = config_path.join(name);
                    let project_file_path = project_path.join("project.yaml");

                    let project = ProjectManifest::new(
                        None,
                        name.to_string(),
                        description.clone(),
                        None,
                        None,
                    );

                    let project_yaml = to_string(&project)?;
                    fs::create_dir_all(project_path.clone())?;
                    fs::write(project_file_path, project_yaml)?;
                    println!("Projeto criado em: {}", project_path.display());
                }
                CreateCommands::Resource {
                    name,
                    resource_type,
                    project,
                } => {
                    let resource_name = format!("{}.yaml", name);
                    let resources_path = Path::new("resources");

                    if !resources_path.exists() {
                        match fs::create_dir(resources_path) {
                            Ok(_) => println!("Criado o diretório de resources"),
                            Err(e) => println!("Erro ao criar diretório de resources: {}", e),
                        }
                    }

                    let resource_path = resources_path.join(resource_name);
                    let resource = ResourceManifest::basic(
                        name.to_string(),
                        resource_type.to_string(),
                        project.clone(),
                    );

                    let resource_yaml = to_string(&resource)?;

                    if let Err(e) = fs::write(resource_path, resource_yaml) {
                        eprintln!("Erro ao criar o arquivo config.yaml: {}", e);
                        return Ok(());
                    }
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
