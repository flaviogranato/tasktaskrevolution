use super::super::commands::ReportCommand;
use crate::{
    application::report::{task::TaskReportUseCase, vacation::VacationReportUseCase},
    infrastructure::persistence::{
        project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
    },
};
use chrono::Utc;
use csv::Writer;
use std::fs::File;
use std::path::PathBuf;

/// Generate automatic file name based on entity type and timestamp
fn generate_filename(entity_type: &str, project_code: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}_{}.csv", entity_type, project_code, timestamp);
    PathBuf::from(filename)
}

pub fn handle_report_command(command: ReportCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ReportCommand::Tasks {
            project,
            company,
            output,
        } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let report_use_case = TaskReportUseCase::new(project_repository);

            // Generate filename if not provided
            let output_path = output.unwrap_or_else(|| generate_filename("tasks", &project));

            let file = File::create(&output_path)?;
            let mut writer = Writer::from_writer(file);
            match report_use_case.execute(&project, &company, &mut writer) {
                Ok(_) => {
                    println!("✅ Task report generated successfully!");
                    println!("   Output: {}", output_path.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate task report: {}", e);
                    Err(e)
                }
            }
        }
        ReportCommand::Vacation { resource, output } => {
            let project_repository = FileProjectRepository::with_base_path(".".into());
            let resource_repository = FileResourceRepository::new(".");
            let report_use_case = VacationReportUseCase::new(project_repository, resource_repository);

            // Generate filename if not provided
            let output_path = output.unwrap_or_else(|| generate_filename("vacation", &resource));

            let file = File::create(&output_path)?;
            let mut writer = Writer::from_writer(file);
            match report_use_case.execute(&mut writer) {
                Ok(_) => {
                    println!("✅ Vacation report generated successfully!");
                    println!("   Output: {}", output_path.display());
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to generate vacation report: {}", e);
                    Err(e)
                }
            }
        }
    }
}
