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
            create_from_template::CreateFromTemplateUseCase, list_templates::ListTemplatesUseCase,
            load_template::LoadTemplateUseCase,
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

pub mod commands;
pub mod handlers;

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"),
       version = env!("CARGO_PKG_VERSION"),
       about = env!("CARGO_PKG_DESCRIPTION"),
       long_about = None,
       name = "ttr")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize the project management system
    Init {
        /// Manager name
        #[clap(short, long)]
        name: String,
        /// Manager email
        #[clap(short, long)]
        email: String,
        /// Default timezone
        #[clap(short, long, default_value = "UTC")]
        timezone: String,
        /// Work hours start (HH:MM format)
        #[clap(long, default_value = "09:00")]
        work_hours_start: String,
        /// Work hours end (HH:MM format)
        #[clap(long, default_value = "18:00")]
        work_hours_end: String,
        /// Work days (comma-separated: monday,tuesday,wednesday,thursday,friday)
        #[clap(long, default_value = "monday,tuesday,wednesday,thursday,friday")]
        work_days: String,
    },
    /// Create a new company
    Company {
        #[clap(subcommand)]
        command: commands::CompanyCommand,
    },
    /// Create a new project
    Project {
        #[clap(subcommand)]
        command: commands::ProjectCommand,
    },
    /// Create a new task
    Task {
        #[clap(subcommand)]
        command: commands::TaskCommand,
    },
    /// Create a new resource
    Resource {
        #[clap(subcommand)]
        command: commands::ResourceCommand,
    },
    /// List entities
    List {
        #[clap(subcommand)]
        command: commands::ListCommand,
    },
    /// Update entities
    Update {
        #[clap(subcommand)]
        command: commands::UpdateCommand,
    },
    /// Delete entities
    Delete {
        #[clap(subcommand)]
        command: commands::DeleteCommand,
    },
    /// Link entities
    Link {
        #[clap(subcommand)]
        command: commands::LinkCommand,
    },
    /// Remove links
    Unlink {
        #[clap(subcommand)]
        command: commands::UnlinkCommand,
    },
    /// Generate reports
    Report {
        #[clap(subcommand)]
        command: commands::ReportCommand,
    },
    /// Validate system
    Validate {
        #[clap(subcommand)]
        command: commands::ValidateCommand,
    },
    /// Build static site
    Build {
        /// Output directory
        #[clap(short, long, default_value = "dist")]
        output: PathBuf,
        /// Base URL for the site
        #[clap(long, default_value = "https://example.com")]
        base_url: String,
    },
    /// Template management
    Template {
        #[clap(subcommand)]
        command: commands::TemplateCommand,
    },
}

impl Cli {
    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.command {
            Commands::Init {
                name,
                email,
                timezone,
                work_hours_start,
                work_hours_end,
                work_days,
            } => handlers::init_handler::handle_init(
                name, email, timezone, work_hours_start, work_hours_end, work_days,
            ),
            Commands::Company { command } => handlers::company_handler::handle_company_command(command),
            Commands::Project { command } => handlers::project_handler::handle_project_command(command),
            Commands::Task { command } => handlers::task_handler::handle_task_command(command),
            Commands::Resource { command } => handlers::resource_handler::handle_resource_command(command),
            Commands::List { command } => handlers::list_handler::handle_list_command(command),
            Commands::Update { command } => handlers::update_handler::handle_update_command(command),
            Commands::Delete { command } => handlers::delete_handler::handle_delete_command(command),
            Commands::Link { command } => handlers::link_handler::handle_link_command(command),
            Commands::Unlink { command } => handlers::unlink_handler::handle_unlink_command(command),
            Commands::Report { command } => handlers::report_handler::handle_report_command(command),
            Commands::Validate { command } => handlers::validate_handler::handle_validate_command(command),
            Commands::Build { output, base_url } => handlers::build_handler::handle_build(output, base_url),
            Commands::Template { command } => handlers::template_handler::handle_template_command(command),
        }
    }
}
