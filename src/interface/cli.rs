use crate::domain::company_settings::repository::ConfigRepository;
use crate::{
    application::{
        build_use_case::BuildUseCase,
        create::{
            project::CreateProjectUseCase, resource::CreateResourceUseCase, task::CreateTaskArgs,
            task::CreateTaskUseCase, time_off::CreateTimeOffUseCase, vacation::CreateVacationUseCase,
        },
        initialize_repository_use_case::InitializeRepositoryUseCase,
        list::{projects::ListProjectsUseCase, resources::ListResourcesUseCase, tasks::ListTasksUseCase},
        project::{
            cancel_project::CancelProjectUseCase,
            describe_project::DescribeProjectUseCase,
            update_project::{UpdateProjectArgs, UpdateProjectUseCase},
        },
        report::{task::TaskReportUseCase, vacation::VacationReportUseCase},
        resource::{
            deactivate_resource::DeactivateResourceUseCase,
            describe_resource::DescribeResourceUseCase,
            update_resource::{UpdateResourceArgs, UpdateResourceUseCase},
        },
        task::{
            delete_task::CancelTaskUseCase,
            describe_task::DescribeTaskUseCase,
            link_task::LinkTaskUseCase,
            update_task::{UpdateTaskArgs, UpdateTaskUseCase},
        },
        project::assign_resource_to_task::AssignResourceToTaskUseCase,
        validate::vacations::ValidateVacationsUseCase,
    },
    infrastructure::persistence::{
        config_repository::FileConfigRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
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
    /// Update an existing entity (project, resource, task).
    Update {
        #[clap(subcommand)]
        update_command: UpdateCommands,
    },
    /// Delete an entity (soft delete).
    Delete {
        #[clap(subcommand)]
        delete_command: DeleteCommands,
    },
    /// Describe a resource to see its details.
    Describe {
        #[clap(subcommand)]
        describe_command: DescribeCommands,
    },
    /// Manage tasks within a project
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

#[derive(Debug, Subcommand)]
pub enum UpdateCommands {
    /// Update an existing project's details.
    Project {
        /// The new name of the project.
        #[clap(long)]
        name: Option<String>,
        /// The new description of the project.
        #[clap(long)]
        description: Option<String>,
    },
    /// Update an existing resource's details.
    Resource {
        /// The code of the resource to update.
        code: String,
        /// The new name for the resource.
        #[clap(long)]
        name: Option<String>,
        /// The new email for the resource.
        #[clap(long)]
        email: Option<String>,
        /// The new type for the resource (e.g., Developer, QA).
        #[clap(long)]
        resource_type: Option<String>,
    },
    /// Update an existing task's details.
    Task {
        /// The code of the task to update.
        code: String,
        /// The new name for the task.
        #[clap(long)]
        name: Option<String>,
        /// The new description for the task.
        #[clap(long)]
        description: Option<String>,
        /// The new start date for the task (YYYY-MM-DD).
        #[clap(long)]
        start_date: Option<String>,
        /// The new due date for the task (YYYY-MM-DD).
        #[clap(long)]
        due_date: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum DeleteCommands {
    /// Deletes (cancels) the current project.
    #[clap(alias = "proj")]
    Project {},
    /// Deletes (deactivates) a resource.
    #[clap(alias = "res")]
    Resource {
        /// The code of the resource to delete.
        code: String,
    },
    /// Deletes (cancels) a task.
    Task {
        /// The code of the task to delete.
        code: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum DescribeCommands {
    /// Describe the current project's details.
    #[clap(alias = "proj")]
    Project {},
    /// Describe a resource to see its details.
    #[clap(alias = "res")]
    Resource {
        /// The code of the resource to describe.
        code: String,
    },
    /// Describe a task to see its details.
    Task {
        /// The code of the task to describe.
        code: String,
    },
    /// Describe the global configuration.
    Config {},
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
    /// Link one task as a dependency of another
    Link {
        /// The task that will have a new dependency
        #[arg(value_name = "TASK_CODE")]
        task: String,
        /// The task that must be completed first
        #[arg(long = "waits-for", value_name = "DEPENDENCY_CODE")]
        dependency: String,
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

                use_case.execute(name, description.as_deref())?;
                Ok(())
            }
            CreateCommands::Resource { name, resource_type } => {
                let repository = FileResourceRepository::new(".");
                let use_case = CreateResourceUseCase::new(repository);

                let _ = use_case.execute(name, resource_type);
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
                    resource,
                    start_date,
                    end_date,
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

                match use_case.execute(resource, *hours, date, description.as_deref()) {
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
                code: _,
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

                let repository = FileProjectRepository::new();

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
                        // The generated task code is now an internal detail of the project aggregate,
                        // and the main success message is printed by the use case.
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
                let repository = FileProjectRepository::new();
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
        Commands::Update { update_command } => match update_command {
            UpdateCommands::Task {
                code,
                name,
                description,
                start_date,
                due_date,
            } => {
                use chrono::NaiveDate;
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let parsed_start_date = match start_date
                    .as_ref()
                    .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                    .transpose()
                {
                    Ok(date) => date,
                    Err(_) => {
                        println!("❌ Error: Invalid start date format. Use YYYY-MM-DD.");
                        return Ok(());
                    }
                };

                let parsed_due_date = match due_date
                    .as_ref()
                    .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                    .transpose()
                {
                    Ok(date) => date,
                    Err(_) => {
                        println!("❌ Error: Invalid due date format. Use YYYY-MM-DD.");
                        return Ok(());
                    }
                };

                let project_repo = FileProjectRepository::new();
                let use_case = UpdateTaskUseCase::new(project_repo);

                let args = UpdateTaskArgs {
                    name: name.clone(),
                    description: description.clone(),
                    start_date: parsed_start_date,
                    due_date: parsed_due_date,
                };

                match use_case.execute(&project_code, code, args) {
                    Ok(updated_task) => {
                        println!("✅ Successfully updated task '{}'.", updated_task.code());
                        println!("   Name: {}", updated_task.name());
                        println!("   Description: {}", updated_task.description().map_or("N/A", |d| d));
                        println!("   Start Date: {}", updated_task.start_date());
                        println!("   Due Date: {}", updated_task.due_date());
                    }
                    Err(e) => {
                        println!("❌ Error updating task: {e}");
                    }
                }
                Ok(())
            }
            UpdateCommands::Resource {
                code,
                name,
                email,
                resource_type,
            } => {
                let resource_repo = FileResourceRepository::new(".");
                let use_case = UpdateResourceUseCase::new(resource_repo);

                let args = UpdateResourceArgs {
                    name: name.clone(),
                    email: email.clone(),
                    resource_type: resource_type.clone(),
                };

                match use_case.execute(code, args) {
                    Ok(updated_resource) => {
                        println!("✅ Successfully updated resource '{}'.", updated_resource.code());
                        println!("   Name: {}", updated_resource.name());
                        println!("   Email: {}", updated_resource.email().map_or("N/A", |e| e));
                        println!("   Type: {}", updated_resource.resource_type());
                    }
                    Err(e) => {
                        println!("❌ Error updating resource: {e}");
                    }
                }
                Ok(())
            }
            UpdateCommands::Project { name, description } => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let project_repo = FileProjectRepository::new();
                let use_case = UpdateProjectUseCase::new(project_repo);

                let args = UpdateProjectArgs {
                    name: name.clone(),
                    description: description.clone(),
                };

                match use_case.execute(&project_code, args) {
                    Ok(updated_project) => {
                        println!("✅ Successfully updated project '{}'.", updated_project.code());
                        println!("   Name: {}", updated_project.name());
                        println!("   Description: {}", updated_project.description().map_or("N/A", |d| d));
                    }
                    Err(e) => {
                        println!("❌ Error updating project: {e}");
                    }
                }
                Ok(())
            }
        },
        Commands::Delete { delete_command } => match delete_command {
            DeleteCommands::Task { code } => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let project_repo = FileProjectRepository::new();
                let use_case = CancelTaskUseCase::new(project_repo);

                match use_case.execute(&project_code, code) {
                    Ok(cancelled_task) => {
                        println!(
                            "✅ Successfully cancelled task '{}' (status is now '{}').",
                            cancelled_task.code(),
                            cancelled_task.status()
                        );
                    }
                    Err(e) => {
                        println!("❌ Error deleting task: {e}");
                    }
                }
                Ok(())
            }
            DeleteCommands::Project {} => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let project_repo = FileProjectRepository::new();
                let use_case = CancelProjectUseCase::new(project_repo);

                match use_case.execute(&project_code) {
                    Ok(cancelled_project) => {
                        println!(
                            "✅ Successfully cancelled project '{}'. Its status is now '{}'.",
                            cancelled_project.code(),
                            cancelled_project.status()
                        );
                    }
                    Err(e) => {
                        println!("❌ Error deleting project: {e}");
                    }
                }
                Ok(())
            }
            DeleteCommands::Resource { code } => {
                let resource_repo = FileResourceRepository::new(".");
                let use_case = DeactivateResourceUseCase::new(resource_repo);

                match use_case.execute(code) {
                    Ok(deactivated_resource) => {
                        println!(
                            "✅ Successfully deactivated resource '{}'. Status is now Inactive.",
                            deactivated_resource.code(),
                        );
                    }
                    Err(e) => {
                        println!("❌ Error deleting resource: {e}");
                    }
                }
                Ok(())
            }
        },
        Commands::Describe { describe_command } => match describe_command {
            DescribeCommands::Resource { code } => {
                let repo = FileResourceRepository::new(".");
                let use_case = DescribeResourceUseCase::new(repo);

                match use_case.execute(code) {
                    Ok(resource) => {
                        println!("{:<20} {}", "Name:", resource.name());
                        println!("{:<20} {}", "Code:", resource.code());
                        println!("{:<20} {}", "Type:", resource.resource_type());
                        println!("{:<20} {}", "Status:", resource.status());
                        println!("{:<20} {}", "Email:", resource.email().map_or("N/A", |e| e));
                        println!("{:<20} {} hours", "TimeOff Balance:", resource.time_off_balance());

                        println!("{:<20}", "\nVacations:");
                        if let Some(vacations) = resource.vacations() {
                            if vacations.is_empty() {
                                println!("  No vacations scheduled.");
                            } else {
                                for v in vacations {
                                    println!(
                                        "  - From {} to {} ({})",
                                        v.start_date.format("%Y-%m-%d"),
                                        v.end_date.format("%Y-%m-%d"),
                                        v.period_type
                                    );
                                }
                            }
                        } else {
                            println!("  No vacations scheduled.");
                        }
                    }
                    Err(e) => {
                        println!("❌ Error describing resource: {e}");
                    }
                }
                Ok(())
            }
            DescribeCommands::Project {} => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let repo = FileProjectRepository::new();
                let use_case = DescribeProjectUseCase::new(repo);

                match use_case.execute(&project_code) {
                    Ok(project) => {
                        println!("{:<20} {}", "Name:", project.name());
                        println!("{:<20} {}", "Code:", project.code());
                        println!("{:<20} {}", "Status:", project.status());
                        println!("{:<20} {}", "Description:", project.description().map_or("N/A", |d| d));

                        println!("{:<20}", "\nTasks:");
                        let tasks = project.tasks();
                        if tasks.is_empty() {
                            println!("  No tasks in this project.");
                        } else {
                            println!("  {:<15} {:<40} {:<15}", "CODE", "NAME", "STATUS");
                            println!("  {:-<15} {:-<40} {:-<15}", "", "", "");
                            for task in tasks.values() {
                                println!("  {:<15} {:<40} {:<15}", task.code(), task.name(), task.status());
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Error describing project: {e}");
                    }
                }
                Ok(())
            }
            DescribeCommands::Task { code } => {
                use serde::Deserialize;

                #[derive(Deserialize)]
                struct ProjMetadata {
                    code: String,
                }

                #[derive(Deserialize)]
                struct ProjManifest {
                    metadata: ProjMetadata,
                }

                let project_manifest_path = PathBuf::from("project.yaml");
                let project_code = if project_manifest_path.exists() {
                    let content = std::fs::read_to_string(project_manifest_path)?;
                    let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                    manifest.metadata.code
                } else {
                    println!("❌ Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let repo = FileProjectRepository::new();
                let use_case = DescribeTaskUseCase::new(repo);

                match use_case.execute(&project_code, code) {
                    Ok(task) => {
                        println!("{:<20} {}", "Name:", task.name());
                        println!("{:<20} {}", "Code:", task.code());
                        println!("{:<20} {}", "Project Code:", task.project_code());
                        println!("{:<20} {}", "Status:", task.status());
                        println!("{:<20} {}", "Description:", task.description().map_or("N/A", |d| d));
                        println!("{:<20} {}", "Start Date:", task.start_date());
                        println!("{:<20} {}", "Due Date:", task.due_date());

                        println!("{:<20}", "\nAssigned Resources:");
                        let assignees = task.assigned_resources();
                        if assignees.is_empty() {
                            println!("  No resources assigned.");
                        } else {
                            for res_code in assignees {
                                println!("  - {res_code}");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Error describing task: {e}");
                    }
                }
                Ok(())
            }
            DescribeCommands::Config {} => {
                let repo = FileConfigRepository::new();
                match repo.load() {
                    Ok((config, _)) => {
                        println!("{:<20} {}", "Manager Name:", config.manager_name);
                        println!("{:<20} {}", "Manager Email:", config.manager_email);
                    }
                    Err(e) => {
                        println!("❌ Error describing configuration: {e}");
                    }
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
                    let project_repo = FileProjectRepository::new();
                    let use_case = TaskReportUseCase::new(project_repo);

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
                    // Since a task is part of a project, we need to find the project first.
                    // We infer the project code from the current directory's `project.yaml`.
                    let project_manifest_path = PathBuf::from("project.yaml");
                    let project_code = if project_manifest_path.exists() {
                        let content = std::fs::read_to_string(project_manifest_path)?;
                        #[derive(Deserialize)]
                        struct ProjMetadata {
                            code: String,
                        }
                        #[derive(Deserialize)]
                        struct ProjManifest {
                            metadata: ProjMetadata,
                        }
                        let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                        manifest.metadata.code
                    } else {
                        println!("❌ Error: This command must be run from within a project directory.");
                        return Ok(());
                    };

                    let project_repo = FileProjectRepository::new();
                    let resource_repo = FileResourceRepository::new(".");
                    let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);
                    let resource_refs: Vec<&str> = resources.iter().map(|s| s.as_str()).collect();
                    match use_case.execute(&project_code, task, &resource_refs) {
                        Ok(updated_project) => {
                            if let Some(updated_task) = updated_project.tasks().get(task) {
                                println!("✅ Successfully assigned resources to task '{}'.", updated_task.code());
                                println!("   New assignees: {}", updated_task.assigned_resources().join(", "));
                            } else {
                                println!("❌ Error: Task '{}' not found in updated project", task);
                            }
                        }
                        Err(e) => {
                            println!("❌ Error assigning resources: {e}");
                        }
                    }
                }
                TaskCommands::Link { task, dependency } => {
                    use serde::Deserialize;
                    use std::path::PathBuf;

                    let project_manifest_path = PathBuf::from("project.yaml");
                    let project_code = if project_manifest_path.exists() {
                        let content = std::fs::read_to_string(project_manifest_path)?;
                        #[derive(Deserialize)]
                        struct ProjMetadata {
                            code: String,
                        }
                        #[derive(Deserialize)]
                        struct ProjManifest {
                            metadata: ProjMetadata,
                        }
                        let manifest: ProjManifest = serde_yaml::from_str(&content)?;
                        manifest.metadata.code
                    } else {
                        println!("❌ Error: This command must be run from within a project directory.");
                        return Ok(());
                    };

                    let project_repo = FileProjectRepository::new();
                    let use_case = LinkTaskUseCase::new(project_repo);

                    match use_case.execute(&project_code, task, dependency) {
                        Ok(_) => {
                            println!("✅ Successfully linked task '{task}' to wait for '{dependency}'.");
                        }
                        Err(e) => {
                            println!("❌ Error linking task: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}
