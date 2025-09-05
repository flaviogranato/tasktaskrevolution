use clap::Subcommand;

#[derive(Subcommand)]
pub enum LinkCommand {
    /// Link tasks
    Tasks {
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
        #[clap(short, long)]
        company: String,
    },
}
