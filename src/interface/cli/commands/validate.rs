use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ValidateCommand {
    /// Validate business rules
    BusinessRules(ValidateArgs),
    /// Validate data integrity
    DataIntegrity(ValidateArgs),
    /// Validate entities
    Entities(ValidateArgs),
    /// Validate system
    System(ValidateArgs),
}

#[derive(Args)]
pub struct ValidateArgs {
    /// Output format (json, html, table)
    #[clap(long, default_value = "table")]
    pub format: String,

    /// Output file for results
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Fail on validation errors (strict mode)
    #[clap(long)]
    pub strict: bool,

    /// Include warnings in output
    #[clap(long)]
    pub include_warnings: bool,

    /// Verbose output
    #[clap(short, long)]
    pub verbose: bool,
}
