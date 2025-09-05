use clap::Subcommand;

#[derive(Subcommand)]
pub enum ListCommand {
    /// List all projects
    Projects {
        /// Company code filter
        #[clap(short, long)]
        company: Option<String>,
    },
    /// List all tasks
    Tasks {
        /// Project code filter
        #[clap(short, long)]
        project: Option<String>,
        /// Company code filter
        #[clap(short, long)]
        company: Option<String>,
    },
    /// List all resources
    Resources,
}
