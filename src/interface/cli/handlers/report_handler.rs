use crate::{
    application::report::{task::TaskReportUseCase, vacation::VacationReportUseCase},
    infrastructure::persistence::{
        project_repository::FileProjectRepository,
        task_repository::FileTaskRepository,
        resource_repository::FileResourceRepository,
    },
};
use super::super::commands::ReportCommand;
use std::path::PathBuf;

pub fn handle_report_command(command: ReportCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ReportCommand::Tasks { project, company, output } => {
            let project_repository = FileProjectRepository::new();
            let report_use_case = TaskReportUseCase::new(project_repository);

            match report_use_case.execute(project, company, &output) {
                Ok(_) => {
                    println!("✅ Task report generated successfully!");
                    println!("   Output: {}", output.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate task report: {}", e);
                    Err(e.into())
                }
            }
        }
        ReportCommand::Vacation { resource, output } => {
            let project_repository = FileProjectRepository::new();
            let resource_repository = FileResourceRepository::new(".");
            let report_use_case = VacationReportUseCase::new(project_repository, resource_repository);

            match report_use_case.execute(resource, &output) {
                Ok(_) => {
                    println!("✅ Vacation report generated successfully!");
                    println!("   Output: {}", output.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate vacation report: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
