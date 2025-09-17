use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ReportCommand {
    /// Generate task report
    Tasks {
        /// Project code
        #[clap(short, long)]
        project: String,
        /// Company code
        #[clap(short, long)]
        company: String,
        /// Output file (optional - will auto-generate if not provided)
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate vacation report
    Vacation {
        /// Resource code
        #[clap(short, long)]
        resource: String,
        /// Output file (optional - will auto-generate if not provided)
        #[clap(short, long)]
        output: Option<PathBuf>,
    },
}
