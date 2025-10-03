use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

pub mod command_executor;
pub mod commands;
pub mod context_manager;
pub mod handlers;
pub mod logging;
pub mod simplified_executor;
pub mod table_formatter;

#[derive(Parser)]
#[clap(author = env!("CARGO_PKG_AUTHORS"),
       version = env!("CARGO_PKG_VERSION"),
       about = env!("CARGO_PKG_DESCRIPTION"),
       long_about = None,
       name = "ttr")]
pub struct Cli {
    /// Enable verbose output (alias for --debug)
    #[clap(short, long, global = true)]
    pub verbose: bool,
    /// Enable debug output (alias for --verbose)
    #[clap(long, global = true)]
    pub debug: bool,
    /// Enable quiet output (minimal information)
    #[clap(short, long, global = true)]
    pub quiet: bool,
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
    /// Initialize workspace with examples (onboarding)
    Workspace {
        #[clap(subcommand)]
        command: commands::WorkspaceCommand,
    },
    /// Create new entities
    #[clap(alias = "new")]
    Create {
        #[clap(subcommand)]
        command: commands::CreateCommand,
    },
    /// List entities
    #[clap(alias = "ls")]
    List {
        #[clap(subcommand)]
        command: commands::ListCommand,
    },
    /// Update entities
    #[clap(alias = "edit")]
    Update {
        #[clap(subcommand)]
        command: commands::UpdateCommand,
    },
    /// Delete entities
    #[clap(alias = "rm")]
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
    #[clap(alias = "check")]
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
    #[clap(alias = "tmpl")]
    Template {
        #[clap(subcommand)]
        command: commands::TemplateCommand,
    },
    /// Task management
    Task {
        #[clap(subcommand)]
        command: commands::TaskCommand,
    },
    /// Query entities with filtering
    #[clap(alias = "q")]
    Query {
        /// Query string to parse and execute
        #[clap(long)]
        query: String,
        /// Entity type to query (project, task, resource, company)
        #[clap(long, default_value = "project")]
        entity_type: String,
        /// Output format (json, table)
        #[clap(long, default_value = "table")]
        format: String,
    },
    /// Migration tools
    Migrate {
        #[clap(subcommand)]
        command: commands::MigrateCommand,
    },
    /// Serve HTML files locally
    Serve {
        /// Port to serve on
        #[clap(short, long, default_value = "3000")]
        port: u16,
        /// Host to bind to
        #[clap(long, default_value = "localhost")]
        host: String,
        /// Directory to serve from
        #[clap(short, long, default_value = "dist")]
        directory: PathBuf,
        /// Enable live reload
        #[clap(long)]
        live_reload: bool,
        /// Enable CORS for development
        #[clap(long)]
        cors: bool,
        /// Enable debug mode
        #[clap(long)]
        debug: bool,
    },
}

impl Cli {
    /// Check if verbose/debug output is enabled
    pub fn is_verbose() -> bool {
        std::env::var("TTR_VERBOSE").unwrap_or_default() == "1"
    }

    /// Check if quiet output is enabled
    pub fn is_quiet() -> bool {
        std::env::var("TTR_QUIET").unwrap_or_default() == "1"
    }

    /// Initialize logging based on flags
    fn init_logging(&self) {
        let verbose = self.verbose || self.debug;
        let quiet = self.quiet;

        // Set environment variables for backward compatibility
        unsafe {
            std::env::set_var("TTR_VERBOSE", if verbose { "1" } else { "0" });
            std::env::set_var("TTR_QUIET", if quiet { "1" } else { "0" });
        }

        // Initialize logging
        if verbose {
            unsafe {
                std::env::set_var("RUST_LOG", "debug");
            }
        } else if quiet {
            unsafe {
                std::env::set_var("RUST_LOG", "error");
            }
        } else {
            unsafe {
                std::env::set_var("RUST_LOG", "info");
            }
        }

        let _ = env_logger::try_init();
    }

    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize logging
        self.init_logging();

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
            Commands::Workspace { command } => handlers::workspace_handler::handle_workspace_command(command),
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
            Commands::Query {
                query,
                entity_type,
                format,
            } => {
                use crate::interface::cli::commands::query::QueryArgs;
                use crate::interface::cli::commands::query::execute_query;
                execute_query(QueryArgs {
                    query,
                    entity_type,
                    format,
                })
            }
            Commands::Migrate { command } => handlers::migrate_handler::handle_migrate_command(command),
            Commands::Serve {
                port,
                host,
                directory,
                live_reload,
                cors,
                debug,
            } => tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(handlers::serve_handler::handle_serve_command(
                    port,
                    host,
                    directory,
                    live_reload,
                    cors,
                    debug,
                )),
        }
    }

}
