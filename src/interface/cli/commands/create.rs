use clap::Subcommand;

#[derive(Subcommand)]
pub enum CreateCommand {
    /// Create a new company
    Company {
        /// Company name
        #[clap(short, long)]
        name: String,
        /// Company code
        #[clap(short, long)]
        code: String,
        /// Company description
        #[clap(short, long)]
        description: Option<String>,
    },
    /// Create a new project
    Project {
        /// Project name
        #[clap(short, long)]
        name: String,
        /// Project code (optional, will be auto-generated if not provided)
        #[clap(long)]
        code: Option<String>,
        /// Company code (optional if in company context)
        #[clap(long)]
        company: Option<String>,
        /// Project description
        #[clap(short, long)]
        description: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: String,
        /// End date (YYYY-MM-DD)
        #[clap(long)]
        end_date: String,
        /// Template name (optional)
        #[clap(long)]
        template: Option<String>,
        /// Template variables (comma-separated key=value pairs)
        #[clap(long)]
        template_vars: Option<String>,
    },
    /// Create a new task
    Task {
        /// Task name
        #[clap(short, long)]
        name: String,
        /// Task code (optional, will be auto-generated if not provided)
        #[clap(long)]
        code: Option<String>,
        /// Project code (optional if in project context)
        #[clap(short, long)]
        project: Option<String>,
        /// Company code (optional if in company/project context)
        #[clap(long)]
        company: Option<String>,
        /// Task description
        #[clap(short, long)]
        description: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: String,
        /// Due date (YYYY-MM-DD)
        #[clap(long)]
        due_date: String,
        /// Assigned resources (comma-separated codes)
        #[clap(long)]
        assigned_resources: Option<String>,
    },
    /// Create a new resource
    Resource {
        /// Resource name
        #[clap(short, long)]
        name: String,
        /// Resource code (optional, will be auto-generated if not provided)
        #[clap(long)]
        code: Option<String>,
        /// Resource email
        #[clap(short, long)]
        email: String,
        /// Company code (optional if in company/project context)
        #[clap(long)]
        company: Option<String>,
        /// Resource description
        #[clap(short, long)]
        description: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: String,
        /// End date (YYYY-MM-DD)
        #[clap(long)]
        end_date: String,
    },
}
