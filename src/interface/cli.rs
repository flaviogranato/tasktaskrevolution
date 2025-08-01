use crate::{
    application::{
        assign_resource_to_task_use_case::AssignResourceToTaskUseCase,
        build_use_case::BuildUseCase,
        create_project_use_case::CreateProjectUseCase,
        create_resource_use_case::CreateResourceUseCase,
        create_task_use_case::{CreateTaskArgs, CreateTaskUseCase},
        create_time_off_use_case::CreateTimeOffUseCase,
        create_vacation_use_case::CreateVacationUseCase,
        initialize_repository_use_case::InitializeRepositoryUseCase,
        list_projects_use_case::ListProjectsUseCase,
        list_resources_use_case::ListResourcesUseCase,
        list_tasks_use_case::ListTasksUseCase,
        task_report_use_case::TaskReportUseCase,
        vacation_report_use_case::VacationReportUseCase,
        validate_vacations_use_case::ValidateVacationsUseCase,
    },
    domain::task_management::repository::TaskRepository,
    infrastructure::persistence::{
        config_repository::FileConfigRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
    },
};
use clap::{Parser, Subcommand};
use csv::Writer;
use serde::Deserialize;
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
    List {
        #[clap(subcommand)]
        list_command: ListCommands,
    },
    Validate {
        #[clap(subcommand)]
        validate_command: ValidateCommands,
    },
    Report {
        #[clap(subcommand)]
        report_command: ReportCommands,
    },
    Task {
        #[clap(subcommand)]
        task_command: TaskCommands,
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
        project_code: Option<String>,
        #[arg(long, short)]
        code: Option<String>,
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

#[derive(Subcommand, Debug)]
pub enum ListCommands {
    Projects,
    Resources,
    Tasks,
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

#[derive(Subcommand, Debug)]
enum TaskCommands {
    /// Assign one or more resources to a task
    Assign {
        /// The code of the task to assign resources to
        #[arg(long, short)]
        task: String,
        /// A comma-separated list of resource codes to assign
        #[arg(long, short, value_delimiter = ',')]
        resources: Vec<String>,
    },
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
                let repository = FileResourceRepository::new(".");
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
                let repository = FileResourceRepository::new(".");
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
                let repository = FileResourceRepository::new(".");
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

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let final_project_code = match project_code.as_ref() {
                    Some(code) => code.clone(),
                    None => {
                        let manifest_path = PathBuf::from("project.yaml");
                        if !manifest_path.exists() {
                            println!(
                                "❌ Erro: Comando executado fora de um diretório de projeto. Especifique --project-code."
                            );
                            return Ok(());
                        }
                        let content = match std::fs::read_to_string(manifest_path) {
                            Ok(c) => c,
                            Err(e) => {
                                println!("❌ Erro ao ler 'project.yaml': {e}");
                                return Ok(());
                            }
                        };
                        let manifest: ProjManifest = match serde_yaml::from_str(&content) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("❌ Erro ao analisar 'project.yaml': {e}");
                                return Ok(());
                            }
                        };
                        manifest.metadata.code
                    }
                };

                let repository = FileTaskRepository::new(".");
                let final_code = match code.as_ref() {
                    Some(c) => c.clone(),
                    None => repository.get_next_code()?,
                };

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

                let use_case = CreateTaskUseCase::new(repository);

                let args = CreateTaskArgs {
                    project_code: final_project_code,
                    name: name.clone(),
                    start_date: start,
                    due_date: due,
                    assigned_resources: assignees.clone(),
                };

                match use_case.execute(args) {
                    Ok(_) => {
                        println!("✅ Task '{name}' criada com sucesso!");
                        println!("📋 Código: {final_code}");
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
        Commands::List { list_command } => match list_command {
            ListCommands::Projects => {
                let repository = FileProjectRepository::new();
                let use_case = ListProjectsUseCase::new(repository);
                match use_case.execute() {
                    Ok(projects) => {
                        if projects.is_empty() {
                            println!("Nenhum projeto encontrado.");
                        } else {
                            println!("{:<15} {:<30}", "CÓDIGO", "NOME");
                            println!("{:-<15} {:-<30}", "", "");
                            for project in projects {
                                println!("{:<15} {:<30}", project.code(), project.name());
                            }
                        }
                    }
                    Err(e) => println!("❌ Erro ao listar projetos: {e}"),
                }
                Ok(())
            }
            ListCommands::Resources => {
                let repository = FileResourceRepository::new(".");
                let use_case = ListResourcesUseCase::new(repository);
                match use_case.execute() {
                    Ok(resources) => {
                        if resources.is_empty() {
                            println!("Nenhum recurso encontrado.");
                        } else {
                            println!("{:<15} {:<25} {:<20}", "CÓDIGO", "NOME", "TIPO");
                            println!("{:-<15} {:-<25} {:-<20}", "", "", "");
                            for resource in resources {
                                println!(
                                    "{:<15} {:<25} {:<20}",
                                    resource.code(),
                                    resource.name(),
                                    resource.resource_type()
                                );
                            }
                        }
                    }
                    Err(e) => println!("❌ Erro ao listar recursos: {e}"),
                }
                Ok(())
            }
            ListCommands::Tasks => {
                let repository = FileTaskRepository::new(".");
                let use_case = ListTasksUseCase::new(repository);
                match use_case.execute() {
                    Ok(tasks) => {
                        if tasks.is_empty() {
                            println!("Nenhuma tarefa encontrada.");
                        } else {
                            println!(
                                "{:<15} {:<40} {:<15} {:<20}",
                                "CÓDIGO", "NOME", "STATUS", "RESPONSÁVEIS"
                            );
                            println!("{:-<15} {:-<40} {:-<15} {:-<20}", "", "", "", "");
                            for task in tasks {
                                let assignees = task.assigned_resources().join(", ");
                                println!(
                                    "{:<15} {:<40} {:<15} {:<20}",
                                    task.code(),
                                    task.name(),
                                    task.status().to_string(),
                                    assignees
                                );
                            }
                        }
                    }
                    Err(e) => println!("❌ Erro ao listar tarefas: {e}"),
                }
                Ok(())
            }
        },
        Commands::Validate { validate_command } => {
            match validate_command {
                ValidateCommands::Vacations => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new(".");
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
                    let resource_repository = FileResourceRepository::new(".");
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
                    let task_repo = FileTaskRepository::new(".");
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
        Commands::Task { task_command } => {
            match task_command {
                TaskCommands::Assign { task, resources } => {
                    let task_repo = FileTaskRepository::new(".");
                    let resource_repo = FileResourceRepository::new(".");
                    let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);
                    match use_case.execute(task, resources) {
                        Ok(updated_task) => {
                            println!("✅ Successfully assigned resources to task '{}'.", updated_task.code());
                            println!("   New assignees: {}", updated_task.assigned_resources().join(", "));
                        }
                        Err(e) => {
                            println!("❌ Error assigning resources: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
