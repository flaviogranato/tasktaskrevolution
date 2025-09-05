use clap::Subcommand;

#[derive(Subcommand)]
pub enum CompanyCommand {
    /// Create a new company
    Create {
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
}
