use clap::Subcommand;

#[derive(Subcommand)]
pub enum WorkspaceCommand {
    /// Initialize workspace with examples for onboarding
    Init {
        /// Manager name
        #[clap(short, long, default_value = "Project Manager")]
        name: String,
        /// Manager email
        #[clap(short, long, default_value = "manager@example.com")]
        email: String,
        /// Company name
        #[clap(long, default_value = "Tech Corp")]
        company_name: String,
        /// Company code
        #[clap(long, default_value = "TECH-001")]
        company_code: String,
        /// Default timezone
        #[clap(short, long, default_value = "UTC")]
        timezone: String,
        /// Skip interactive prompts
        #[clap(long)]
        yes: bool,
    },
}
