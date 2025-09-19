use clap::Subcommand;

#[derive(Subcommand)]
pub enum UpdateCommand {
    /// Update a project
    Project {
        /// Project code
        #[clap(short, long)]
        code: String,
        /// Company code (optional if in company/project context)
        #[clap(long)]
        company: Option<String>,
        /// New project name
        #[clap(long)]
        name: Option<String>,
        /// New project description
        #[clap(long)]
        description: Option<String>,
        /// New start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: Option<String>,
        /// New end date (YYYY-MM-DD)
        #[clap(long)]
        end_date: Option<String>,
    },
    /// Update a task
    Task {
        /// Task code
        #[clap(short, long)]
        code: String,
        /// Project code (optional if in project context)
        #[clap(short, long)]
        project: Option<String>,
        /// Company code (optional if in company/project context)
        #[clap(long)]
        company: Option<String>,
        /// New task name
        #[clap(long)]
        name: Option<String>,
        /// New task description
        #[clap(long)]
        description: Option<String>,
        /// New start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: Option<String>,
        /// New due date (YYYY-MM-DD)
        #[clap(long)]
        due_date: Option<String>,
    },
    /// Update a resource
    Resource {
        /// Resource code
        #[clap(short, long)]
        code: String,
        /// Company code (optional if in company context)
        #[clap(long)]
        company: Option<String>,
        /// New resource name
        #[clap(long)]
        name: Option<String>,
        /// New resource type
        #[clap(long = "type")]
        r#type: Option<String>,
        /// New resource email
        #[clap(long)]
        email: Option<String>,
        /// New resource description
        #[clap(long)]
        description: Option<String>,
    },
}
