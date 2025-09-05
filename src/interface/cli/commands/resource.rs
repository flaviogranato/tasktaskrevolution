use clap::Subcommand;

#[derive(Subcommand)]
pub enum ResourceCommand {
    /// Create a new resource
    Create {
        /// Resource name
        #[clap(short, long)]
        name: String,
        /// Resource code
        #[clap(short, long)]
        code: String,
        /// Resource email
        #[clap(short, long)]
        email: String,
        /// Resource description
        #[clap(short, long)]
        description: Option<String>,
    },
    /// Create time off for resource
    TimeOff {
        /// Resource code
        #[clap(short, long)]
        resource: String,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: String,
        /// End date (YYYY-MM-DD)
        #[clap(long)]
        end_date: String,
        /// Hours
        #[clap(short, long)]
        hours: u32,
        /// Description
        #[clap(short, long)]
        description: Option<String>,
    },
    /// Create vacation for resource
    Vacation {
        /// Resource code
        #[clap(short, long)]
        resource: String,
        /// Start date (YYYY-MM-DD)
        #[clap(long)]
        start_date: String,
        /// End date (YYYY-MM-DD)
        #[clap(long)]
        end_date: String,
        /// Description
        #[clap(short, long)]
        description: Option<String>,
        /// With compensation
        #[clap(long)]
        with_compensation: bool,
    },
    /// Describe a resource
    Describe {
        /// Resource code
        #[clap(short, long)]
        code: String,
    },
    /// Update a resource
    Update {
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
    /// Deactivate a resource
    Deactivate {
        /// Resource code
        #[clap(short, long)]
        code: String,
    },
}
