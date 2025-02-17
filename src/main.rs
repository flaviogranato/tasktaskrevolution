mod application;
mod domain;
mod infrastructure;
mod interface;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cli = interface::cli::Cli::parse();
    interface::cli::run(cli)
}
