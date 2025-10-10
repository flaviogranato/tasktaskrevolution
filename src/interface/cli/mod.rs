use clap::{Parser, Subcommand};
use std::{env, path::PathBuf};

pub mod command_executor;
pub mod commands;
pub mod completions;
pub mod context_manager;
pub mod exit_codes;
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
    /// Output logs in JSON format
    #[clap(long, global = true)]
    pub json_logs: bool,
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
    /// Resource management
    Resource {
        #[clap(subcommand)]
        command: commands::ResourceCommand,
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
    /// Search across all files
    #[clap(alias = "s")]
    Search {
        /// Search query/pattern
        query: String,
        /// Entity type to search (project, task, resource, company)
        #[clap(long)]
        entity_type: Option<String>,
        /// Output format (table, json, csv, list, compact, grouped, highlighted)
        #[clap(long, default_value = "table")]
        format: String,
        /// Case sensitive search
        #[clap(long)]
        case_sensitive: bool,
        /// Whole word matching
        #[clap(long)]
        whole_word: bool,
        /// Use regex pattern
        #[clap(long)]
        regex: bool,
        /// Search only in metadata (YAML frontmatter)
        #[clap(long)]
        metadata_only: bool,
        /// Search only in content (not metadata)
        #[clap(long)]
        content_only: bool,
        /// Maximum number of results
        #[clap(long)]
        max_results: Option<usize>,
        /// Number of context lines to show
        #[clap(long, default_value = "2")]
        context_lines: usize,
        /// Filter by file type
        #[clap(long)]
        file_type: Option<String>,
        /// Minimum score threshold
        #[clap(long)]
        min_score: Option<f32>,
        /// Maximum score threshold
        #[clap(long)]
        max_score: Option<f32>,
        /// Minimum number of matches per file
        #[clap(long)]
        min_matches: Option<usize>,
        /// Maximum number of matches per file
        #[clap(long)]
        max_matches: Option<usize>,
        /// Include path pattern
        #[clap(long)]
        include_path: Option<String>,
        /// Exclude path pattern
        #[clap(long)]
        exclude_path: Option<String>,
        /// Show search statistics
        #[clap(long)]
        stats: bool,
        /// Workspace path (defaults to current directory)
        #[clap(long)]
        workspace: Option<String>,
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
    /// Generate shell completions
    Completions {
        /// Shell type (bash, zsh, fish, powershell, elvish)
        #[clap(short, long)]
        shell: Option<String>,
        /// Install completions to files
        #[clap(long)]
        install: bool,
        /// Output directory for installed completions
        #[clap(long)]
        output_dir: Option<String>,
        /// Show installation help
        #[clap(long)]
        help: bool,
    },
    /// Test data validation and management
    TestData {
        #[clap(subcommand)]
        command: commands::TestDataCommand,
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
        let json_format = self.json_logs;

        // Set environment variables for backward compatibility
        unsafe {
            std::env::set_var("TTR_VERBOSE", if verbose { "1" } else { "0" });
            std::env::set_var("TTR_QUIET", if quiet { "1" } else { "0" });
        }

        // Initialize tracing-based logging
        if let Err(e) = logging::Logger::init(verbose, quiet, json_format) {
            eprintln!("Failed to initialize logging: {}", e);
        }
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
            Commands::Resource { command } => handlers::resource_handler::handle_resource_command(command),
            Commands::Query {
                query,
                entity_type,
                format,
            } => {
                use crate::interface::cli::commands::query::QueryArgs;
                use crate::interface::cli::commands::query::execute_query;
                execute_query(QueryArgs {
                    query: Some(query),
                    entity_type,
                    format,
                    field: None,
                    operator: None,
                    value: None,
                    aggregate: None,
                    aggregate_field: None,
                    sort: None,
                    order: "asc".to_string(),
                    limit: None,
                    offset: None,
                    show_fields: false,
                })
            }
            Commands::Search {
                query,
                entity_type,
                format,
                case_sensitive,
                whole_word,
                regex,
                metadata_only,
                content_only,
                max_results,
                context_lines,
                file_type,
                min_score,
                max_score,
                min_matches,
                max_matches,
                include_path,
                exclude_path,
                stats,
                workspace,
            } => {
                use crate::interface::cli::commands::search::SearchArgs;
                use crate::interface::cli::commands::search::execute_search;
                execute_search(SearchArgs {
                    query,
                    entity_type,
                    format,
                    case_sensitive,
                    whole_word,
                    regex,
                    metadata_only,
                    content_only,
                    max_results,
                    context_lines,
                    file_type,
                    min_score,
                    max_score,
                    min_matches,
                    max_matches,
                    include_path,
                    exclude_path,
                    stats,
                    workspace,
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
            Commands::Completions {
                shell,
                install,
                output_dir,
                help,
            } => {
                if help {
                    completions::CompletionCommandHandler::handle_help_command()
                } else if install {
                    completions::CompletionCommandHandler::handle_install_command(output_dir.clone())
                } else {
                    completions::CompletionCommandHandler::handle_completion_command(shell.clone())
                }
            }
            Commands::TestData { command } => {
                let base_path = std::env::current_dir().unwrap().to_string_lossy().to_string();
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(command.execute(&base_path))
                    .map_err(|e| e.into())
            }
        }
    }
}
