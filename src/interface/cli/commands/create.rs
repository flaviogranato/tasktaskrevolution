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
        /// Project code
        #[clap(long)]
        code: String,
        /// Company code
        #[clap(long)]
        company: String,
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
        /// Task code
        #[clap(long)]
        code: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
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
        /// Resource code
        #[clap(long)]
        code: String,
        /// Resource email
        #[clap(short, long)]
        email: String,
        /// Company code
        #[clap(long)]
        company: String,
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
