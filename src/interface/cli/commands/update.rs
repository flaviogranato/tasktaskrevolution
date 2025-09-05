use clap::Subcommand;

#[derive(Subcommand)]
pub enum UpdateCommand {
    /// Update a project
    Project {
        /// Project code
        #[clap(short, long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
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
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(short, long)]
        company: String,
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
        /// New resource name
        #[clap(long)]
        name: Option<String>,
        /// New resource email
        #[clap(long)]
        email: Option<String>,
        /// New resource description
        #[clap(long)]
        description: Option<String>,
    },
}
