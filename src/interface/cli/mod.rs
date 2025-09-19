use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

pub mod command_executor;
pub mod commands;
pub mod context_manager;
pub mod handlers;
pub mod simplified_executor;
pub mod table_formatter;

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"),
       version = env!("CARGO_PKG_VERSION"),
       about = env!("CARGO_PKG_DESCRIPTION"),
       long_about = None,
       name = "ttr")]
pub struct Cli {
    /// Enable verbose output
    #[clap(short, long, global = true)]
    pub verbose: bool,
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
        /// Company name
        #[clap(long, default_value = "Default Company")]
        company_name: String,
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
    /// Create new entities
    Create {
        #[clap(subcommand)]
        command: commands::CreateCommand,
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
    /// Task management
    Task {
        #[clap(subcommand)]
        command: commands::TaskCommand,
    },
    /// Migration tools
    Migrate {
        #[clap(subcommand)]
        command: commands::MigrateCommand,
    },
}

impl Cli {
    /// Check if verbose output is enabled
    pub fn is_verbose() -> bool {
        std::env::var("TTR_VERBOSE").unwrap_or_default() == "1"
    }

    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        // Set global verbose flag
        unsafe {
            std::env::set_var("TTR_VERBOSE", if self.verbose { "1" } else { "0" });
        }

        match self.command {
            Commands::Init {
                name,
                email,
                company_name,
                timezone,
                work_hours_start,
                work_hours_end,
                work_days,
            } => command_executor::execute_init(
                name,
                email,
                company_name,
                timezone,
                work_hours_start,
                work_hours_end,
                work_days,
            ),
            Commands::Create { command } => simplified_executor::SimplifiedExecutor::execute_create(command),
            Commands::List { command } => simplified_executor::SimplifiedExecutor::execute_list(command),
            Commands::Update { command } => simplified_executor::SimplifiedExecutor::execute_update(command),
            Commands::Delete { command } => simplified_executor::SimplifiedExecutor::execute_delete(command),
            Commands::Link { command } => handlers::link_handler::handle_link_command(command),
            Commands::Unlink { command } => handlers::unlink_handler::handle_unlink_command(command),
            Commands::Report { command } => handlers::report_handler::handle_report_command(command),
            Commands::Validate { command } => command_executor::execute_validate(command),
            Commands::Build { output, base_url } => command_executor::execute_build(output, base_url),
            Commands::Template { command } => handlers::template_handler::handle_template_command(command),
            Commands::Task { command } => handlers::task_handler::handle_task_command(command),
            Commands::Migrate { command } => handlers::migrate_handler::handle_migrate_command(command),
        }
    }
}
