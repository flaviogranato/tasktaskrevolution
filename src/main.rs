#[allow(non_snake_case)]
mod application;
#[allow(non_snake_case)]
mod domain;
#[allow(non_snake_case)]
mod infrastructure;
#[allow(non_snake_case)]
mod interface;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let cli = interface::cli::Cli::parse();
    tokio::runtime::Runtime::new()?.block_on(interface::cli::run(cli))
}
