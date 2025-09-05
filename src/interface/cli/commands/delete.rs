use clap::Subcommand;

#[derive(Subcommand)]
pub enum DeleteCommand {
    /// Delete a project
    Project {
        /// Project code
        #[clap(short, long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
    },
    /// Delete a task
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
    },
    /// Delete a resource
    Resource {
        /// Resource code
        #[clap(short, long)]
        code: String,
    },
}
