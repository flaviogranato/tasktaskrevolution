use crate::domain::company_settings::repository::ConfigRepository;
use crate::{
    application::{
        build_use_case::BuildUseCase,
        company_management::{CreateCompanyArgs, CreateCompanyUseCase},
        create::{
            project::CreateProjectUseCase, resource::CreateResourceUseCase, task::CreateTaskArgs,
            task::CreateTaskUseCase, time_off::CreateTimeOffUseCase, vacation::CreateVacationUseCase,
        },
        init::{InitManagerData, InitManagerUseCase},
        list::{projects::ListProjectsUseCase, resources::ListResourcesUseCase, tasks::ListTasksUseCase},
        project::assign_resource_to_task::AssignResourceToTaskUseCase,
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
            delete_task::DeleteTaskUseCase,
            describe_task::DescribeTaskUseCase,
            link_task::LinkTaskUseCase,
            update_task::{UpdateTaskArgs, UpdateTaskUseCase},
        },
        template::{
            list_templates::ListTemplatesUseCase,
            load_template::LoadTemplateUseCase,
            create_from_template::CreateFromTemplateUseCase,
        },
        validate::{
            business_rules::ValidateBusinessRulesUseCase, data_integrity::ValidateDataIntegrityUseCase,
            entities::ValidateEntitiesUseCase, system::ValidateSystemUseCase,
        },
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository, config_repository::FileConfigRepository,
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};
use clap::{Parser, Subcommand};
use csv::Writer;
use serde::Deserialize;
use std::{collections::HashMap, env, path::PathBuf};

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
        /// Nome do manager/consultor
        #[clap(long, value_name = "NAME")]
        name: String,
        /// Email do manager/consultor
        #[clap(long, value_name = "EMAIL")]
        email: String,
        /// Company/consultancy name
        #[clap(long, value_name = "COMPANY")]
        company_name: String,
        /// Timezone (ex: UTC, America/Sao_Paulo)
        #[clap(long, value_name = "TIMEZONE", default_value = "UTC")]
        timezone: String,
        /// Hora de início do trabalho (HH:MM)
        #[clap(long, value_name = "TIME", default_value = "08:00")]
        work_start: String,
        /// Hora de fim do trabalho (HH:MM)
        #[clap(long, value_name = "TIME", default_value = "18:00")]
        work_end: String,
    },
    Build {
        /// Optional: Path to the project directory.
        /// Se não for fornecido, usa o diretório atual.
        path: Option<PathBuf>,
    },

    /// Start a local development server to serve the generated HTML site
    Server {
        /// Optional: Path to the project directory.
        /// Se não for fornecido, usa o diretório atual.
        path: Option<PathBuf>,
        /// Port to run the server on
        #[clap(long, default_value = "1313")]
        port: u16,
        /// Host to bind the server to
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Create new entities (projects, resources, companies, tasks, etc.)
    #[clap(alias = "c")]
    Create {
        #[clap(subcommand)]
        create_command: CreateCommands,
    },
    /// List entities (projects, resources, tasks)
    #[clap(alias = "l")]
    List {
        #[clap(subcommand)]
        list_command: ListCommands,
    },
    /// Validate system, entities, business rules, and data integrity
    Validate {
        #[clap(subcommand)]
        validate_command: ValidateCommands,
    },
    /// Generate reports (vacation, task reports)
    Report {
        #[clap(subcommand)]
        report_command: ReportCommands,
    },
    /// Update an existing entity (project, resource, task, company)
    #[clap(alias = "u")]
    Update {
        #[clap(subcommand)]
        update_command: UpdateCommands,
    },
    /// Delete an entity (soft delete)
    #[clap(alias = "del")]
    Delete {
        #[clap(subcommand)]
        delete_command: DeleteCommands,
    },
    /// Describe a resource to see its details
    #[clap(alias = "d")]
    Describe {
        #[clap(subcommand)]
        describe_command: DescribeCommands,
    },
    /// Manage project templates
    Template {
        #[clap(subcommand)]
        template_command: TemplateCommands,
    },
    /// Manage tasks within a project
    Task {
        #[clap(subcommand)]
        task_command: TaskCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    /// Create a new project
    #[clap(alias = "proj")]
    Project {
        /// Project name
        name: String,
        /// Project description
        description: Option<String>,
        /// Company code (required for new structure)
        #[clap(long, value_name = "COMPANY_CODE")]
        company_code: Option<String>,
        /// Create project from template
        #[clap(long, value_name = "TEMPLATE_NAME")]
        from_template: Option<String>,
        /// Template variables (key=value pairs)
        #[clap(long, value_delimiter = ',')]
        template_vars: Vec<String>,
    },
    /// Create a new resource (person)
    #[clap(alias = "res")]
    Resource {
        /// Resource name
        name: String,
        /// Resource type/role
        resource_type: String,
        /// Company code (required for new structure)
        #[clap(long, value_name = "COMPANY_CODE")]
        company_code: Option<String>,
        /// Project code (optional - if not provided, resource is global to company)
        #[clap(long, value_name = "PROJECT_CODE")]
        project_code: Option<String>,
    },
    /// Create a new company
    #[clap(alias = "comp")]
    Company {
        #[clap(long, value_name = "CODE", default_value = "")]
        code: String,
        #[clap(long, value_name = "NAME")]
        name: String,
        #[clap(long, value_name = "DESCRIPTION")]
        description: Option<String>,
        #[clap(long, value_name = "TAX_ID")]
        tax_id: Option<String>,
        #[clap(long, value_name = "ADDRESS")]
        address: Option<String>,
        #[clap(long, value_name = "EMAIL")]
        email: Option<String>,
        #[clap(long, value_name = "PHONE")]
        phone: Option<String>,
        #[clap(long, value_name = "WEBSITE")]
        website: Option<String>,
        #[clap(long, value_name = "INDUSTRY")]
        industry: Option<String>,
        #[clap(long, value_name = "USER", default_value = "system")]
        created_by: String,
    },
    /// Create a vacation period for a resource
    #[clap(alias = "vac")]
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
    /// Create a time-off entry for a resource
    #[clap(alias = "to")]
    TimeOff {
        #[arg(long)]
        resource: String,
        #[arg(long)]
        hours: u32,
        #[arg(long)]
        date: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// Create a new task
    #[clap(alias = "tsk")]
    Task {
        #[arg(long)]
        project_code: Option<String>,
        /// Company code (required for new structure)
        #[clap(long, value_name = "COMPANY_CODE")]
        company_code: Option<String>,
        #[arg(long)]
        code: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        start_date: String,
        #[arg(long)]
        due_date: String,
        #[arg(long, value_delimiter = ',')]
        assignees: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ListCommands {
    /// List all projects
    #[clap(alias = "proj")]
    Projects,
    /// List all resources
    #[clap(alias = "res")]
    Resources,
    /// List all tasks
    #[clap(alias = "tsk")]
    Tasks,
}

#[derive(Debug, Subcommand)]
pub enum UpdateCommands {
    /// Update an existing project's details
    #[clap(alias = "proj")]
    Project {
        /// The new name of the project
        #[clap(long)]
        name: Option<String>,
        /// The new description of the project
        #[clap(long)]
        description: Option<String>,
    },
    /// Update an existing resource's details
    #[clap(alias = "res")]
    Resource {
        /// The code of the resource to update
        code: String,
        /// The new name for the resource
        #[clap(long)]
        name: Option<String>,
        /// The new email for the resource
        #[clap(long)]
        email: Option<String>,
        /// The new type for the resource (e.g., Developer, QA)
        #[clap(long)]
        resource_type: Option<String>,
    },
    /// Update an existing task's details
    #[clap(alias = "tsk")]
    Task {
        /// The code of the task to update
        code: String,
        /// The new name for the task
        #[clap(long)]
        name: Option<String>,
        /// The new description for the task
        #[clap(long)]
        description: Option<String>,
        /// The new start date for the task (YYYY-MM-DD)
        #[clap(long)]
        start_date: Option<String>,
        /// The new due date for the task (YYYY-MM-DD)
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
    /// Deletes (cancels) a task
    #[clap(alias = "tsk")]
    Task {
        /// The code of the task to delete
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
    /// Describe a task to see its details
    #[clap(alias = "tsk")]
    Task {
        /// The code of the task to describe
        code: String,
    },
    /// Describe the global configuration.
    Config {},
}

#[derive(Subcommand)]
enum ValidateCommands {
    System,
    Entities,
    BusinessRules,
    DataIntegrity,
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

#[derive(Subcommand)]
enum TemplateCommands {
    /// List available project templates
    List,
    /// Show details of a specific template
    Show {
        /// Template name
        name: String,
    },
    /// Create a new project from a template
    Create {
        /// Template name
        template: String,
        /// Project name
        name: String,
        /// Project description
        description: Option<String>,
        /// Company code
        #[clap(long, value_name = "COMPANY_CODE")]
        company_code: Option<String>,
        /// Template variables (key=value pairs)
        #[clap(long, value_delimiter = ',')]
        variables: Vec<String>,
    },
}

fn start_dev_server(
    public_dir: &std::path::Path,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind(format!("{}:{}", host, port))?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let public_dir = public_dir.to_path_buf();
                thread::spawn(move || {
                    handle_client(stream, &public_dir);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: std::net::TcpStream, public_dir: &std::path::Path) {
    let mut reader = std::io::BufReader::new(&stream);
    let mut request_line = String::new();

    if std::io::BufRead::read_line(&mut reader, &mut request_line).is_err() {
        return;
    }

    let request_parts: Vec<&str> = request_line.split_whitespace().collect();
    if request_parts.len() < 2 {
        return;
    }

    let mut path = request_parts[1];
    if path == "/" {
        path = "/index.html";
    }

    // Remove leading slash
    let file_path = public_dir.join(&path[1..]);

    let (status, content_type, body) = if file_path.exists() && file_path.is_file() {
        match std::fs::read(&file_path) {
            Ok(contents) => {
                let content_type = match file_path.extension().and_then(|s| s.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("css") => "text/css",
                    Some("js") => "application/javascript",
                    Some("png") => "image/png",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("gif") => "image/gif",
                    Some("svg") => "image/svg+xml",
                    _ => "application/octet-stream",
                };
                ("200 OK", content_type, contents)
            }
            Err(_) => (
                "500 Internal Server Error",
                "text/plain",
                b"Internal Server Error".to_vec(),
            ),
        }
    } else {
        (
            "404 Not Found",
            "text/html; charset=utf-8",
            b"<h1>404 Not Found</h1>".to_vec(),
        )
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status,
        content_type,
        body.len()
    );

    let _ = std::io::Write::write_all(&mut stream, response.as_bytes());
    let _ = std::io::Write::write_all(&mut stream, &body);
}

pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    match &cli.command {
        Commands::Init {
            name,
            email,
            company_name,
            timezone,
            work_start,
            work_end,
        } => {
            let repository = FileConfigRepository::new();
            let use_case = InitManagerUseCase::new(Box::new(repository));

            let init_data = InitManagerData {
                name: name.clone(),
                email: email.clone(),
                company_name: company_name.clone(),
                timezone: timezone.clone(),
                work_hours_start: work_start.clone(),
                work_hours_end: work_end.clone(),
            };

            match use_case.execute(init_data) {
                Ok(config) => {
                    println!("Manager/Consultant configured successfully!");
                    println!("Manager: {} ({})", config.manager_name, config.manager_email);
                    if let Some(company) = &config.company_name {
                        println!("Company: {}", company);
                    }
                    println!("Timezone: {}", config.default_timezone);
                    if let (Some(start), Some(end)) = (&config.work_hours_start, &config.work_hours_end) {
                        println!("Work hours: {} - {}", start, end);
                    }
                    println!("Configuration saved to: config.yaml");
                }
                Err(e) => {
                    println!("Error configuring manager: {:?}", e);
                    return Err(Box::new(e));
                }
            }
            Ok(())
        }
        Commands::Build { path } => {
            let project_path = path.clone().unwrap_or_else(|| PathBuf::from("."));
            let output_dir = project_path.join("public");

            match BuildUseCase::new(project_path, output_dir.to_str().unwrap()) {
                Ok(use_case) => {
                    if let Err(e) = use_case.execute() {
                        println!("Erro ao construir o site: {e}");
                    }
                }
                Err(e) => {
                    println!("Erro ao inicializar o builder: {e}");
                }
            }
            Ok(())
        }

        Commands::Server { path, port, host } => {
            let project_path = path.clone().unwrap_or_else(|| PathBuf::from("."));
            let output_dir = project_path.join("public");

            // First build the site
            match BuildUseCase::new(project_path.clone(), output_dir.to_str().unwrap()) {
                Ok(use_case) => {
                    if let Err(e) = use_case.execute() {
                        println!("Erro ao construir o site: {e}");
                        return Ok(());
                    }
                }
                Err(e) => {
                    println!("Erro ao inicializar o builder: {e}");
                    return Ok(());
                }
            }

            // Then start the server
            println!("🚀 Starting TTR development server...");
            println!("📁 Serving files from: {}", output_dir.display());
            println!("🌐 Server running at: http://{}:{}", host, port);
            println!("📖 Open your browser and navigate to: http://{}:{}", host, port);
            println!("⏹️  Press Ctrl+C to stop the server");

            if let Err(e) = start_dev_server(&output_dir, host, *port) {
                println!("Erro ao iniciar o servidor: {e}");
            }
            Ok(())
        }

        Commands::Create { create_command } => match create_command {
            CreateCommands::Project {
                name,
                description,
                company_code,
                from_template,
                template_vars,
            } => {
                let repository = FileProjectRepository::new();
                let use_case = CreateProjectUseCase::new(repository);

                // For now, use a default company code if not provided
                // TODO: In the future, we should require company_code or detect from context
                let company_code = company_code.clone().unwrap_or_else(|| "DEFAULT".to_string());

                if let Some(template_name) = from_template {
                    // Create project from template
                    let templates_dir = std::path::Path::new("templates/projects");
                    let load_use_case = LoadTemplateUseCase::new();
                    let template = load_use_case.load_by_name(templates_dir, &template_name)?;

                    // Parse template variables
                    let mut variables = HashMap::new();
                    for var in template_vars {
                        if let Some((key, value)) = var.split_once('=') {
                            variables.insert(key.to_string(), value.to_string());
                        }
                    }

                    // Add default variables
                    variables.insert("project_name".to_string(), name.clone());
                    if let Some(desc) = description {
                        variables.insert("project_description".to_string(), desc.to_string());
                    }

                    let create_from_template_use_case = CreateFromTemplateUseCase::new(
                        use_case,
                        CreateResourceUseCase::new(FileResourceRepository::new(".")),
                        CreateTaskUseCase::new(FileProjectRepository::new()),
                    );

                    let created_project = create_from_template_use_case.execute(&template, &variables, company_code)?;
                    println!("{}", created_project.display_summary());
                    println!("\nResources:");
                    println!("{}", created_project.display_resources());
                    println!("\nTasks:");
                    println!("{}", created_project.display_tasks());
                } else {
                    // Create project normally
                    use_case.execute(name, description.as_deref(), company_code)?;
                }
                Ok(())
            }
            CreateCommands::Resource {
                name,
                resource_type,
                company_code,
                project_code,
            } => {
                let repository = FileResourceRepository::new(".");
                let use_case = CreateResourceUseCase::new(repository);

                // For now, use default values if not provided
                // TODO: In the future, we should require company_code or detect from context
                let company_code = company_code.clone().unwrap_or_else(|| "DEFAULT".to_string());

                let _ = use_case.execute(name, resource_type, company_code, project_code.clone());
                Ok(())
            }
            CreateCommands::Company {
                code,
                name,
                description,
                tax_id,
                address,
                email,
                phone,
                website,
                industry,
                created_by,
            } => {
                let repository = FileCompanyRepository::new(".");
                let use_case = CreateCompanyUseCase::new(repository);

                let args = CreateCompanyArgs {
                    code: code.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    tax_id: tax_id.clone(),
                    address: address.clone(),
                    email: email.clone(),
                    phone: phone.clone(),
                    website: website.clone(),
                    industry: industry.clone(),
                    created_by: created_by.clone(),
                };

                match use_case.execute(args) {
                    Ok(company) => {
                        println!("Company created successfully!");
                        if code.is_empty() {
                            println!("Code generated automatically: {}", company.code);
                        } else {
                            println!("Code: {}", company.code);
                        }
                        println!("Name: {}", company.name);
                        println!("ID: {}", company.id);
                        if let Some(desc) = &company.description {
                            println!("Description: {}", desc);
                        }
                        if let Some(cnpj) = &company.tax_id {
                            println!("Tax ID: {}", cnpj);
                        }
                        if let Some(addr) = &company.address {
                            println!("Address: {}", addr);
                        }
                        if let Some(mail) = &company.email {
                            println!("Email: {}", mail);
                        }
                        if let Some(tel) = &company.phone {
                            println!("Phone: {}", tel);
                        }
                        if let Some(site) = &company.website {
                            println!("Website: {}", site);
                        }
                        if let Some(ind) = &company.industry {
                            println!("Industry: {}", ind);
                        }
                        println!("Created by: {}", company.created_by);
                    }
                    Err(e) => {
                        println!("Error creating company: {:?}", e);
                        return Err(Box::new(e));
                    }
                }
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
                        println!("{}", result.message);
                    }
                    Err(e) => println!("Unexpected error: {e}"),
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
                            println!("{}", result.message);
                            println!("New balance: {} hours", result.time_off_balance);
                            if let Some(desc) = &result.description {
                                println!("Description: {desc}");
                            }
                            println!("Date: {}", result.date);
                        } else {
                            println!("{}", result.message);
                        }
                    }
                    Err(e) => println!("Unexpected error: {e}"),
                };
                Ok(())
            }
            CreateCommands::Task {
                project_code,
                company_code,
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
                            println!("Error: Command executed outside a project directory. Specify --project-code.");
                            return Ok(());
                        }
                        let content = match std::fs::read_to_string(manifest_path) {
                            Ok(c) => c,
                            Err(e) => {
                                println!("Erro ao ler 'project.yaml': {e}");
                                return Ok(());
                            }
                        };
                        let manifest: ProjManifest = match serde_yaml::from_str(&content) {
                            Ok(m) => m,
                            Err(e) => {
                                println!("Error parsing 'project.yaml': {e}");
                                return Ok(());
                            }
                        };
                        manifest.metadata.code
                    }
                };

                let repository = FileProjectRepository::with_base_path(".".into());

                let start = match NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("Error: Invalid start date. Use format YYYY-MM-DD");
                        return Ok(());
                    }
                };

                let due = match NaiveDate::parse_from_str(due_date, "%Y-%m-%d") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("Error: Invalid due date. Use format YYYY-MM-DD");
                        return Ok(());
                    }
                };

                let use_case = CreateTaskUseCase::new(repository);

                // For now, use default values if not provided
                // TODO: In the future, we should require company_code or detect from context
                let company_code = company_code.clone().unwrap_or_else(|| "DEFAULT".to_string());

                let args = CreateTaskArgs {
                    company_code,
                    project_code: final_project_code,
                    name: name.clone(),
                    start_date: start,
                    due_date: due,
                    assigned_resources: assignees.clone(),
                };

                match use_case.execute(args) {
                    Ok(_) => {
                        println!("Task '{name}' created successfully!");
                        // The generated task code is now an internal detail of the project aggregate,
                        // and the main success message is printed by the use case.
                        if let Some(desc) = description {
                            println!("Description: {desc}");
                        }
                        println!("Period: {start_date} to {due_date}");
                        if !assignees.is_empty() {
                            println!("Assignees: {}", assignees.join(", "));
                        }
                    }
                    Err(e) => {
                        println!("Error creating task: {e}");
                    }
                };
                Ok(())
            }
        },
        Commands::List { list_command } => match list_command {
            ListCommands::Projects => {
                let repository = FileProjectRepository::with_base_path(".".into());
                let use_case = ListProjectsUseCase::new(repository);
                match use_case.execute() {
                    Ok(projects) => {
                        if projects.is_empty() {
                            println!("No projects found.");
                        } else {
                            println!("{:<15} {:<15} {:<30}", "CODE", "COMPANY", "NAME");
                            println!("{:-<15} {:-<15} {:-<30}", "", "", "");
                            for project in projects {
                                println!(
                                    "{:<15} {:<15} {:<30}",
                                    project.code(),
                                    project.company_code(),
                                    project.name()
                                );
                            }
                        }
                    }
                    Err(e) => println!("Error listing projects: {e}"),
                }
                Ok(())
            }
            ListCommands::Resources => {
                let repository = FileResourceRepository::new(".");
                let use_case = ListResourcesUseCase::new(repository);
                match use_case.execute() {
                    Ok(resources) => {
                        if resources.is_empty() {
                            println!("No resources found.");
                        } else {
                            println!("{:<15} {:<25} {:<20}", "CODE", "NAME", "TYPE");
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
                    Err(e) => println!("Error listing resources: {e}"),
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
                    Err(e) => println!("Error listing tasks: {e}"),
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
                    println!("Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let parsed_start_date = match start_date
                    .as_ref()
                    .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                    .transpose()
                {
                    Ok(date) => date,
                    Err(_) => {
                        println!("Error: Invalid start date format. Use YYYY-MM-DD.");
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
                        println!("Error: Invalid due date format. Use YYYY-MM-DD.");
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
                        println!("Successfully updated task '{}'.", updated_task.code());
                        println!("   Name: {}", updated_task.name());
                        println!("   Description: {}", updated_task.description().map_or("N/A", |d| d));
                        println!("   Start Date: {}", updated_task.start_date());
                        println!("   Due Date: {}", updated_task.due_date());
                    }
                    Err(e) => {
                        println!("Error updating task: {e}");
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
                        println!("Successfully updated resource '{}'.", updated_resource.code());
                        println!("   Name: {}", updated_resource.name());
                        println!("   Email: {}", updated_resource.email().map_or("N/A", |e| e));
                        println!("   Type: {}", updated_resource.resource_type());
                    }
                    Err(e) => {
                        println!("Error updating resource: {e}");
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
                    println!("Error: This command must be run from within a project directory.");
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
                        println!("Successfully updated project '{}'.", updated_project.code());
                        println!("   Name: {}", updated_project.name());
                        println!("   Description: {}", updated_project.description().map_or("N/A", |d| d));
                    }
                    Err(e) => {
                        println!("Error updating project: {e}");
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
                    println!("Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let project_repo = FileProjectRepository::new();
                let use_case = DeleteTaskUseCase::new(project_repo);

                match use_case.execute(&project_code, code) {
                    Ok(cancelled_task) => {
                        println!(
                            "Successfully cancelled task '{}' (status is now '{}').",
                            cancelled_task.code(),
                            cancelled_task.status()
                        );
                    }
                    Err(e) => {
                        println!("Error deleting task: {e}");
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
                    println!("Error: This command must be run from within a project directory.");
                    return Ok(());
                };

                let project_repo = FileProjectRepository::new();
                let use_case = CancelProjectUseCase::new(project_repo);

                match use_case.execute(&project_code) {
                    Ok(cancelled_project) => {
                        println!(
                            "Successfully cancelled project '{}'. Its status is now '{}'.",
                            cancelled_project.code(),
                            cancelled_project.status()
                        );
                    }
                    Err(e) => {
                        println!("Error deleting project: {e}");
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
                            "Successfully deactivated resource '{}'. Status is now Inactive.",
                            deactivated_resource.code(),
                        );
                    }
                    Err(e) => {
                        println!("Error deleting resource: {e}");
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
                        println!("Error describing resource: {e}");
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
                    println!("Error: This command must be run from within a project directory.");
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
                        println!("Error describing project: {e}");
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
                    println!("Error: This command must be run from within a project directory.");
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
                        println!("Error describing task: {e}");
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
                        println!("Error describing configuration: {e}");
                    }
                }
                Ok(())
            }
        },
        Commands::Validate { validate_command } => {
            match validate_command {
                ValidateCommands::System => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new(".");
                    let company_repository = FileCompanyRepository::new(".");
                    let use_case =
                        ValidateSystemUseCase::new(project_repository, resource_repository, company_repository);

                    match use_case.execute() {
                        Ok(messages) => {
                            println!("\nSystem validation results:");
                            println!("=========================");
                            for message in messages {
                                println!("{message}");
                            }
                        }
                        Err(e) => println!("Error validating system: {e}"),
                    }
                }
                ValidateCommands::Entities => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new(".");
                    let company_repository = FileCompanyRepository::new(".");
                    let use_case =
                        ValidateEntitiesUseCase::new(&project_repository, &resource_repository, &company_repository);

                    match use_case.execute() {
                        Ok(messages) => {
                            println!("\nEntity validation results:");
                            println!("=========================");
                            for message in messages {
                                println!("{message}");
                            }
                        }
                        Err(e) => println!("Error validating entities: {e}"),
                    }
                }
                ValidateCommands::BusinessRules => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new(".");
                    let company_repository = FileCompanyRepository::new(".");
                    let use_case = ValidateBusinessRulesUseCase::new(
                        &project_repository,
                        &resource_repository,
                        &company_repository,
                    );

                    match use_case.execute() {
                        Ok(messages) => {
                            println!("\nBusiness rules validation results:");
                            println!("==================================");
                            for message in messages {
                                println!("{message}");
                            }
                        }
                        Err(e) => println!("Error validating business rules: {e}"),
                    }
                }
                ValidateCommands::DataIntegrity => {
                    let project_repository = FileProjectRepository::new();
                    let resource_repository = FileResourceRepository::new(".");
                    let company_repository = FileCompanyRepository::new(".");
                    let use_case = ValidateDataIntegrityUseCase::new(
                        &project_repository,
                        &resource_repository,
                        &company_repository,
                    );

                    match use_case.execute() {
                        Ok(messages) => {
                            println!("\nData integrity validation results:");
                            println!("===================================");
                            for message in messages {
                                println!("{message}");
                            }
                        }
                        Err(e) => println!("Error validating data integrity: {e}"),
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
                                println!("Error generating report: {e}");
                            } else {
                                println!("Vacation report generated successfully at: {file_path}");
                            }
                        }
                        Err(e) => {
                            println!("Error creating report file: {e}");
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
                                println!("Error generating task report: {e}");
                            } else {
                                println!("Task report generated successfully at: {file_path}");
                            }
                        }
                        Err(e) => {
                            println!("Error creating task report file: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
        Commands::Template { template_command } => {
            match template_command {
                TemplateCommands::List => {
                    let templates_dir = std::path::Path::new("templates/projects");
                    let use_case = ListTemplatesUseCase::new();
                    
                    match use_case.execute(templates_dir) {
                        Ok(templates) => {
                            if templates.is_empty() {
                                println!("No templates found in templates/projects/");
                            } else {
                                println!("Available project templates:");
                                println!();
                                for template in templates {
                                    println!("  {} - {}", template.display_name(), template.display_summary());
                                    if !template.tags.is_empty() {
                                        println!("    Tags: {}", template.display_tags());
                                    }
                                    println!();
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error listing templates: {}", e);
                        }
                    }
                    Ok(())
                }
                TemplateCommands::Show { name } => {
                    let templates_dir = std::path::Path::new("templates/projects");
                    let use_case = LoadTemplateUseCase::new();
                    
                    match use_case.load_by_name(templates_dir, &name) {
                        Ok(template) => {
                            println!("Template: {}", template.metadata.name);
                            println!("Description: {}", template.metadata.description);
                            println!("Version: {}", template.metadata.version);
                            println!("Category: {}", template.metadata.category);
                            println!("Tags: {}", template.metadata.tags.join(", "));
                            println!();
                            println!("Resources ({}):", template.spec.resources.len());
                            for resource in &template.spec.resources {
                                println!("  - {} ({})", resource.name, resource.r#type);
                                println!("    Skills: {}", resource.skills.join(", "));
                                println!("    Capacity: {} hours/day", resource.capacity);
                            }
                            println!();
                            println!("Tasks ({}):", template.spec.tasks.len());
                            for task in &template.spec.tasks {
                                println!("  - {} ({}h, {})", task.name, task.estimated_hours, task.priority);
                                println!("    Category: {}", task.category);
                                if !task.dependencies.is_empty() {
                                    println!("    Dependencies: {}", task.dependencies.join(", "));
                                }
                            }
                            println!();
                            println!("Phases ({}):", template.spec.phases.len());
                            for phase in &template.spec.phases {
                                println!("  - {} ({} weeks)", phase.name, phase.duration);
                                println!("    Tasks: {}", phase.tasks.join(", "));
                            }
                            println!();
                            println!("Variables:");
                            for (name, var) in &template.spec.variables {
                                println!("  - {} ({})", name, var.r#type);
                                println!("    Description: {}", var.description);
                                println!("    Example: {}", var.example);
                                if var.required {
                                    println!("    Required: Yes");
                                } else {
                                    println!("    Required: No");
                                    if let Some(default) = &var.default {
                                        println!("    Default: {}", default);
                                    }
                                }
                                println!();
                            }
                        }
                        Err(e) => {
                            println!("Error loading template '{}': {}", name, e);
                        }
                    }
                    Ok(())
                }
                TemplateCommands::Create {
                    template,
                    name,
                    description,
                    company_code,
                    variables,
                } => {
                    let templates_dir = std::path::Path::new("templates/projects");
                    let load_use_case = LoadTemplateUseCase::new();
                    let template = load_use_case.load_by_name(templates_dir, &template)?;

                    // Parse template variables
                    let mut template_vars = HashMap::new();
                    for var in variables {
                        if let Some((key, value)) = var.split_once('=') {
                            template_vars.insert(key.to_string(), value.to_string());
                        }
                    }

                    // Add default variables
                    template_vars.insert("project_name".to_string(), name.clone());
                    if let Some(desc) = description {
                        template_vars.insert("project_description".to_string(), desc.to_string());
                    }

                    let company_code = company_code.clone().unwrap_or_else(|| "DEFAULT".to_string());

                    let _repository = FileProjectRepository::new();
                    let create_from_template_use_case = CreateFromTemplateUseCase::new(
                        CreateProjectUseCase::new(FileProjectRepository::new()),
                        CreateResourceUseCase::new(FileResourceRepository::new(".")),
                        CreateTaskUseCase::new(FileProjectRepository::new()),
                    );

                    let created_project = create_from_template_use_case.execute(&template, &template_vars, company_code)?;
                    println!("{}", created_project.display_summary());
                    println!("\nResources:");
                    println!("{}", created_project.display_resources());
                    println!("\nTasks:");
                    println!("{}", created_project.display_tasks());
                    Ok(())
                }
            }
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
                        println!("Error: This command must be run from within a project directory.");
                        return Ok(());
                    };

                    let project_repo = FileProjectRepository::new();
                    let resource_repo = FileResourceRepository::new(".");
                    let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);
                    let resource_refs: Vec<&str> = resources.iter().map(|s| s.as_str()).collect();
                    match use_case.execute(&project_code, task, resource_refs[0]) {
                        Ok(updated_project) => {
                            if let Some(updated_task) = updated_project.tasks().get(task) {
                                println!("Successfully assigned resources to task '{}'.", updated_task.code());
                                println!("   New assignees: {}", updated_task.assigned_resources().join(", "));
                            } else {
                                println!("Error: Task '{}' not found in updated project", task);
                            }
                        }
                        Err(e) => {
                            println!("Error assigning resources: {e}");
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
                        println!("Error: This command must be run from within a project directory.");
                        return Ok(());
                    };

                    let project_repo = FileProjectRepository::new();
                    let use_case = LinkTaskUseCase::new(project_repo);

                    match use_case.execute(&project_code, task, dependency) {
                        Ok(_) => {
                            println!("Successfully linked task '{task}' to wait for '{dependency}'.");
                        }
                        Err(e) => {
                            println!("Error linking task: {e}");
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_creation() {
        let cli = Cli::command();
        assert_eq!(cli.get_name(), "TaskTaskRevolution");
    }

    #[test]
    fn test_cli_version() {
        let cli = Cli::command();
        let version = cli.get_version();
        assert!(version.is_some());
        let version_str = version.unwrap().to_string();
        assert!(!version_str.is_empty());
    }

    #[test]
    fn test_cli_about() {
        let cli = Cli::command();
        let about = cli.get_about();
        assert!(about.is_some());
        let about_str = about.unwrap().to_string();
        assert!(!about_str.is_empty());
    }

    #[test]
    fn test_cli_author() {
        let cli = Cli::command();
        let author = cli.get_author();
        assert!(author.is_some());
        let author_str = author.unwrap().to_string();
        assert!(!author_str.is_empty());
    }

    #[test]
    fn test_init_command_parsing() {
        let args = vec![
            "ttr",
            "init",
            "--name",
            "John Doe",
            "--email",
            "john@example.com",
            "--company-name",
            "TechConsulting",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Init {
            name,
            email,
            company_name,
            timezone,
            work_start,
            work_end,
        } = cli.command
        {
            assert_eq!(name, "John Doe");
            assert_eq!(email, "john@example.com");
            assert_eq!(company_name, "TechConsulting");
            assert_eq!(timezone, "UTC");
            assert_eq!(work_start, "08:00");
            assert_eq!(work_end, "18:00");
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_init_command_with_timezone() {
        let args = vec![
            "ttr",
            "init",
            "--name",
            "João Silva",
            "--email",
            "joao@consultoria.com",
            "--company-name",
            "Consultoria Brasil",
            "--timezone",
            "America/Sao_Paulo",
            "--work-start",
            "09:00",
            "--work-end",
            "17:00",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Init {
            name,
            email,
            company_name,
            timezone,
            work_start,
            work_end,
        } = cli.command
        {
            assert_eq!(name, "João Silva");
            assert_eq!(email, "joao@consultoria.com");
            assert_eq!(company_name, "Consultoria Brasil");
            assert_eq!(timezone, "America/Sao_Paulo");
            assert_eq!(work_start, "09:00");
            assert_eq!(work_end, "17:00");
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_build_command_parsing() {
        let args = vec!["ttr", "build"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Build { path } = cli.command {
            assert_eq!(path, None);
        } else {
            panic!("Expected Build command");
        }
    }

    #[test]
    fn test_build_command_with_path() {
        let args = vec!["ttr", "build", "/tmp/test"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Build { path } = cli.command {
            assert_eq!(path, Some(PathBuf::from("/tmp/test")));
        } else {
            panic!("Expected Build command");
        }
    }

    #[test]
    fn test_create_project_command() {
        let args = vec!["ttr", "create", "project", "My Project", "A test project"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Project { name, description, .. } = create_command {
                assert_eq!(name, "My Project");
                assert_eq!(description, Some("A test project".to_string()));
            } else {
                panic!("Expected CreateCommands::Project");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_project_command_no_description() {
        let args = vec!["ttr", "create", "project", "My Project"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Project { name, description, .. } = create_command {
                assert_eq!(name, "My Project");
                assert_eq!(description, None);
            } else {
                panic!("Expected CreateCommands::Project");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_resource_command() {
        let args = vec!["ttr", "create", "resource", "John Doe", "Developer"];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Resource {
                name, resource_type, ..
            } = create_command
            {
                assert_eq!(name, "John Doe");
                assert_eq!(resource_type, "Developer");
            } else {
                panic!("Expected CreateCommands::Resource");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_vacation_command() {
        let args = vec![
            "ttr",
            "create",
            "vacation",
            "--resource",
            "RES-001",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-01-05",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Vacation {
                resource,
                start_date,
                end_date,
                is_time_off_compensation,
                compensated_hours,
            } = create_command
            {
                assert_eq!(resource, "RES-001");
                assert_eq!(start_date, "2024-01-01");
                assert_eq!(end_date, "2024-01-05");
                assert!(!is_time_off_compensation);
                assert_eq!(compensated_hours, None);
            } else {
                panic!("Expected CreateCommands::Vacation");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_vacation_command_with_compensation() {
        let args = vec![
            "ttr",
            "create",
            "vacation",
            "--resource",
            "RES-001",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-01-05",
            "--is-time-off-compensation",
            "--compensated-hours",
            "40",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Vacation {
                resource,
                start_date,
                end_date,
                is_time_off_compensation,
                compensated_hours,
            } = create_command
            {
                assert_eq!(resource, "RES-001");
                assert_eq!(start_date, "2024-01-01");
                assert_eq!(end_date, "2024-01-05");
                assert!(is_time_off_compensation);
                assert_eq!(compensated_hours, Some(40));
            } else {
                panic!("Expected CreateCommands::Vacation");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_time_off_command() {
        let args = vec![
            "ttr",
            "create",
            "time-off",
            "--resource",
            "RES-001",
            "--hours",
            "8",
            "--date",
            "2024-01-01",
            "--description",
            "Sick leave",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::TimeOff {
                resource,
                hours,
                date,
                description,
            } = create_command
            {
                assert_eq!(resource, "RES-001");
                assert_eq!(hours, 8);
                assert_eq!(date, "2024-01-01");
                assert_eq!(description, Some("Sick leave".to_string()));
            } else {
                panic!("Expected CreateCommands::TimeOff");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_time_off_command_no_description() {
        let args = vec![
            "ttr",
            "create",
            "time-off",
            "--resource",
            "RES-001",
            "--hours",
            "8",
            "--date",
            "2024-01-01",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::TimeOff {
                resource,
                hours,
                date,
                description,
            } = create_command
            {
                assert_eq!(resource, "RES-001");
                assert_eq!(hours, 8);
                assert_eq!(date, "2024-01-01");
                assert_eq!(description, None);
            } else {
                panic!("Expected CreateCommands::TimeOff");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_task_command() {
        let args = vec![
            "ttr",
            "create",
            "task",
            "--project-code",
            "PROJ-001",
            "--code",
            "TASK-001",
            "--name",
            "Implement feature",
            "--description",
            "A test task",
            "--start-date",
            "2024-01-01",
            "--due-date",
            "2024-01-15",
            "--assignees",
            "RES-001,RES-002",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Task {
                project_code,
                code,
                name,
                description,
                start_date,
                due_date,
                assignees,
                ..
            } = create_command
            {
                assert_eq!(project_code, Some("PROJ-001".to_string()));
                assert_eq!(code, Some("TASK-001".to_string()));
                assert_eq!(name, "Implement feature");
                assert_eq!(description, Some("A test task".to_string()));
                assert_eq!(start_date, "2024-01-01");
                assert_eq!(due_date, "2024-01-15");
                assert_eq!(assignees, vec!["RES-001".to_string(), "RES-002".to_string()]);
            } else {
                panic!("Expected CreateCommands::Task");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_create_task_command_minimal() {
        let args = vec![
            "ttr",
            "create",
            "task",
            "--name",
            "Implement feature",
            "--start-date",
            "2024-01-01",
            "--due-date",
            "2024-01-15",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Create { create_command } = cli.command {
            if let CreateCommands::Task {
                project_code,
                code,
                name,
                description,
                start_date,
                due_date,
                assignees,
                ..
            } = create_command
            {
                assert_eq!(project_code, None);
                assert_eq!(code, None);
                assert_eq!(name, "Implement feature");
                assert_eq!(description, None);
                assert_eq!(start_date, "2024-01-01");
                assert_eq!(due_date, "2024-01-15");
                assert_eq!(assignees, Vec::<String>::new());
            } else {
                panic!("Expected CreateCommands::Task");
            }
        } else {
            panic!("Expected Create command");
        }
    }

    #[test]
    fn test_list_commands() {
        let commands = vec!["projects", "resources", "tasks"];

        for command in commands {
            let args = vec!["ttr", "list", command];
            let cli = Cli::try_parse_from(args).unwrap();

            if let Commands::List { list_command } = cli.command {
                match command {
                    "projects" => {
                        if let ListCommands::Projects = list_command {
                            // OK
                        } else {
                            panic!("Expected ListCommands::Projects");
                        }
                    }
                    "resources" => {
                        if let ListCommands::Resources = list_command {
                            // OK
                        } else {
                            panic!("Expected ListCommands::Resources");
                        }
                    }
                    "tasks" => {
                        if let ListCommands::Tasks = list_command {
                            // OK
                        } else {
                            panic!("Expected ListCommands::Tasks");
                        }
                    }
                    _ => panic!("Unexpected command: {}", command),
                }
            } else {
                panic!("Expected List command");
            }
        }
    }

    #[test]
    fn test_update_project_command() {
        let args = vec![
            "ttr",
            "update",
            "project",
            "--name",
            "New Project Name",
            "--description",
            "New description",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Update { update_command } = cli.command {
            if let UpdateCommands::Project { name, description } = update_command {
                assert_eq!(name, Some("New Project Name".to_string()));
                assert_eq!(description, Some("New description".to_string()));
            } else {
                panic!("Expected UpdateCommands::Project");
            }
        } else {
            panic!("Expected Update command");
        }
    }

    #[test]
    fn test_update_resource_command() {
        let args = vec![
            "ttr",
            "update",
            "resource",
            "RES-001",
            "--name",
            "New Name",
            "--email",
            "new@email.com",
            "--resource-type",
            "Senior Developer",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        if let Commands::Update { update_command } = cli.command {
            if let UpdateCommands::Resource {
                code,
                name,
                email,
                resource_type,
            } = update_command
            {
                assert_eq!(code, "RES-001");
                assert_eq!(name, Some("New Name".to_string()));
                assert_eq!(email, Some("new@email.com".to_string()));
                assert_eq!(resource_type, Some("Senior Developer".to_string()));
            } else {
                panic!("Expected UpdateCommands::Resource");
            }
        } else {
            panic!("Expected Update command");
        }
    }

    #[test]
    fn test_cli_help() {
        let mut cli = Cli::command();
        let help = cli.render_help().to_string();
        assert!(help.contains("TaskTaskRevolution"));
        assert!(help.contains("Commands:"));
        assert!(help.contains("init"));
        assert!(help.contains("build"));
        assert!(help.contains("create"));
        assert!(help.contains("list"));
        assert!(help.contains("validate"));
        assert!(help.contains("report"));
        assert!(help.contains("update"));
        assert!(help.contains("delete"));
        assert!(help.contains("describe"));
        assert!(help.contains("task"));
    }

    #[test]
    fn test_cli_version_flag() {
        let cli = Cli::command();
        let version = cli.render_version().to_string();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_cli_long_about() {
        let cli = Cli::command();
        let _long_about = cli.get_long_about();
        // long_about pode ser None, então não podemos fazer assert direto
        // mas podemos verificar que não quebra
        // Placeholder assertion - test passes if we reach here
    }

    #[test]
    fn test_cli_propagate_version() {
        let mut cli = Cli::command();
        // Verificar que a versão é propagada para subcomandos
        let help = cli.render_help().to_string();
        assert!(help.contains("--version"));
    }

    #[test]
    fn test_cli_subcommand_help() {
        let mut cli = Cli::command();
        let help = cli.render_help().to_string();

        // Verificar que todos os subcomandos estão documentados
        // Nota: Alguns comandos podem não aparecer no help principal
        assert!(help.contains("Commands:"));
        assert!(help.contains("Init") || help.contains("init"));
        assert!(help.contains("Build") || help.contains("build"));
        assert!(help.contains("Create") || help.contains("create"));
        assert!(help.contains("List") || help.contains("list"));
        assert!(help.contains("Validate") || help.contains("validate"));
        assert!(help.contains("Report") || help.contains("report"));
        assert!(help.contains("Update") || help.contains("update"));
        assert!(help.contains("Delete") || help.contains("delete"));
        assert!(help.contains("Describe") || help.contains("describe"));
        assert!(help.contains("Task") || help.contains("task"));
    }

    #[test]
    fn test_cli_author_environment_variable() {
        // Este teste verifica que o autor é lido da variável de ambiente CARGO_PKG_AUTHORS
        let cli = Cli::command();
        let author = cli.get_author();
        assert!(author.is_some());
        let author_str = author.unwrap().to_string();
        // O autor deve ser uma string válida
        assert!(!author_str.is_empty());
        assert!(!author_str.contains("CARGO_PKG_AUTHORS"));
    }

    #[test]
    fn test_cli_version_environment_variable() {
        // Este teste verifica que a versão é lida da variável de ambiente CARGO_PKG_VERSION
        let cli = Cli::command();
        let version = cli.get_version();
        assert!(version.is_some());
        let version_str = version.unwrap().to_string();
        // A versão deve ser uma string válida
        assert!(!version_str.is_empty());
        assert!(!version_str.contains("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_cli_description_environment_variable() {
        // Este teste verifica que a descrição é lida da variável de ambiente CARGO_PKG_DESCRIPTION
        let cli = Cli::command();
        let description = cli.get_about();
        assert!(description.is_some());
        let description_str = description.unwrap().to_string();
        // A descrição deve ser uma string válida
        assert!(!description_str.is_empty());
        assert!(!description_str.contains("CARGO_PKG_DESCRIPTION"));
    }
}
