//! TaskTaskRevolution CLI
//!
//! Interface de linha de comando para o TTR.
//! A lógica principal está em lib.rs para facilitar testes.

fn main() {
    match task_task_revolution::run() {
        Ok(()) => {
            // Success - exit with code 0
            std::process::exit(0);
        }
        Err(e) => {
            // Error - print to stderr and exit with appropriate code
            eprintln!("Error: {}", e);

            // Try to determine appropriate exit code based on error type
            let exit_code = if e.to_string().contains("validation") {
                task_task_revolution::interface::cli::exit_codes::CliError::Validation
            } else if e.to_string().contains("permission") || e.to_string().contains("Permission denied") {
                task_task_revolution::interface::cli::exit_codes::CliError::Permission
            } else if e.to_string().contains("not found") || e.to_string().contains("No such file") {
                task_task_revolution::interface::cli::exit_codes::CliError::NotFound
            } else if e.to_string().contains("conflict") {
                task_task_revolution::interface::cli::exit_codes::CliError::Conflict
            } else if e.to_string().contains("invalid argument") {
                task_task_revolution::interface::cli::exit_codes::CliError::InvalidArgument
            } else {
                task_task_revolution::interface::cli::exit_codes::CliError::Internal
            };

            exit_code.exit();
        }
    }
}
