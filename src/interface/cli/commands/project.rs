use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProjectCommand {
    /// Create a new project
    Create {
        /// Project name
        #[clap(short, long)]
        name: String,
        /// Project code
        #[clap(long)]
        code: String,
        /// Company code
        #[clap(short, long)]
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
    },
    /// Create project from template
    FromTemplate {
        /// Template name
        #[clap(short, long)]
        template: String,
        /// Project name
        #[clap(short, long)]
        name: String,
        /// Project code
        #[clap(long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
        /// Template parameters (key=value format)
        #[clap(long, num_args = 0..)]
        params: Vec<String>,
    },
    /// Describe a project
    Describe {
        /// Project code
        #[clap(long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
    },
    /// Update a project
    Update {
        /// Project code
        #[clap(long)]
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
    /// Cancel a project
    Cancel {
        /// Project code
        #[clap(long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
    },
    /// Assign resource to task
    AssignResource {
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
        /// Task code
        #[clap(short, long)]
        task: String,
        /// Resource code
        #[clap(short, long)]
        resource: String,
    },
}
