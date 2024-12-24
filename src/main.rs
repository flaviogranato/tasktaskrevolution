mod entities;

use std::env;
use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use serde_yml::to_string;

use crate::entities::config::{ConfigManifest, ConfigMetadata, ConfigSpec};
use crate::entities::project::ProjectManifest;

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"), 
    version = env!("CARGO_PKG_VERSION"), 
    about = env!("CARGO_PKG_DESCRIPTION"), 
    long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init {
            path,
            manager_name,
            manager_email,
        } => {
            let repo_path = path.clone().unwrap_or(std::env::current_dir()?);
            let _ = create_config_file(&repo_path, manager_name, manager_email);

            println!("Repositório inicializado em: {}", repo_path.display());
        }
        Commands::Create { create_command } => {
            let config_path = std::env::current_dir()?;

            match create_command {
                CreateCommands::Project { name, description } => {
                    println!(
                        "Criando projeto: {} no diretório: {}",
                        name,
                        config_path.display()
                    );
                    if let Some(desc) = description {
                        println!("Descrição: {}", desc);
                    }

                    let project_path = config_path.join(name);
                    fs::create_dir_all(project_path.clone())?;
                    let _ = create_project(name, description);
                    println!("Projeto criado em: {}", project_path.display());
                }
                &CreateCommands::Resource { .. } | &CreateCommands::Task { .. } => todo!(),
            }
        }
    }

    Ok(())
}

fn create_config_file(path: &PathBuf, name: &str, email: &str) -> Result<(), serde_yml::Error> {
    let config_path = path.join("config.yaml");
    let _config = ConfigManifest {
        api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
        kind: "Config".to_string(),
        metadata: ConfigMetadata {
            name: "config".to_string(),
            manager_name: name.to_string(),
            manager_email: email.to_string(),
        },
        spec: ConfigSpec {
            currency: "BRL".to_string(),
            work_hours_per_day: 8,
            work_days_per_week: vec![
                "segunda-feira".to_string(),
                "terça-feira".to_string(),
                "quarta-feira".to_string(),
                "quinta-feira".to_string(),
                "sexta-feira".to_string(),
            ],
            date_format: "yyyy-mm-dd".to_string(),
            default_task_duration: 8,
            locale: "pt_BR".to_string(),
        },
    };

    let config_yaml = to_string(&_config)?;

    if let Err(e) = fs::write(config_path, config_yaml) {
        eprintln!("Erro ao criar o arquivo config.yaml: {}", e);
        return Ok(());
    }

    Ok(())
}

fn create_project(name: &String, description: &Option<String>) -> Result<(), serde_yml::Error> {
    let project_path = PathBuf::from(name);
    let project_file_path = project_path.join("project.yaml");
    let project = ProjectManifest::new(name.to_string(), None, None);
    let project_yaml = to_string(&project)?;

    if let Err(e) = fs::write(project_file_path, project_yaml) {
        eprintln!("Erro ao criar o arquivo config.yaml: {}", e);
        return Ok(());
    }

    Ok(())
}
