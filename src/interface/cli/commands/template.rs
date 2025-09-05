use clap::Subcommand;

#[derive(Subcommand)]
pub enum TemplateCommand {
    /// List available templates
    List,
    /// Show template details
    Show {
        /// Template name
        #[clap(short, long)]
        name: String,
    },
    /// Create project from template
    Create {
        /// Template name
        #[clap(short, long)]
        template: String,
        /// Project name
        #[clap(short, long)]
        name: String,
        /// Project code
        #[clap(short, long)]
        code: String,
        /// Company code
        #[clap(short, long)]
        company: String,
        /// Template parameters (key=value format)
        #[clap(long, num_args = 0..)]
        params: Vec<String>,
    },
}
