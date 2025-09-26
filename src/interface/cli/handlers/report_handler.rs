use crate::interface::cli::commands::report::execute_report;
use crate::interface::cli::commands::ReportCommand;

pub fn handle_report_command(command: ReportCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ReportCommand::Generate(args) => {
            execute_report(args)?;
        }
    }
    Ok(())
}