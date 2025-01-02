mod domain;
mod interface;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = interface::cli::Cli::parse();

    interface::cli::run(cli)?;
    Ok(())
}
