use clap::Subcommand;

#[derive(Subcommand)]
pub enum TaskCommand {
    /// Create a new task
    Create {
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
    /// Describe a task
    Describe {
        /// Task code
        #[clap(long)]
        code: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
    },
    /// Update a task
    Update {
        /// Task code
        #[clap(long)]
        code: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
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
    /// Delete a task
    Delete {
        /// Task code
        #[clap(long)]
        code: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
    },
    /// Link tasks
    Link {
        /// Source task code
        #[clap(short, long)]
        from: String,
        /// Target task code
        #[clap(short, long)]
        to: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
    },
    /// Remove task link
    Unlink {
        /// Source task code
        #[clap(short, long)]
        from: String,
        /// Target task code
        #[clap(short, long)]
        to: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
    },
    /// Assign resource to task
    AssignResource {
        /// Task code
        #[clap(short, long)]
        task: String,
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(long)]
        company: String,
        /// Resource code
        #[clap(short, long)]
        resource: String,
    },
}
