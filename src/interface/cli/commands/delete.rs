use clap::Subcommand;

#[derive(Subcommand)]
pub enum DeleteCommand {
    /// Delete a project
    Project {
        /// Project code
        #[clap(short, long)]
        code: String,
        /// Company code (optional if in company/project context)
        #[clap(long)]
        company: Option<String>,
    },
    /// Delete a task
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
    },
    /// Delete a resource
    Resource {
        /// Resource code
        #[clap(short, long)]
        code: String,
    },
}
