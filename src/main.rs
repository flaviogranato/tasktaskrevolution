#[allow(non_snake_case)]
mod application;
#[allow(non_snake_case)]
mod domain;
#[allow(non_snake_case)]
mod infrastructure;
#[allow(non_snake_case)]
mod interface;

use clap::Parser;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Inicializa o sistema de logging e telemetria
    // infrastructure::telemetry::init();
    info!("Iniciando TaskTaskRevolution...");

    let cli = interface::cli::Cli::parse();
    interface::cli::run(cli)
}
