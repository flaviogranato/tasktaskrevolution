use crate::infrastructure::persistence::{
    project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
};
use csv::Writer;
use std::{io, path::PathBuf};

pub struct VacationReportUseCase {
    project_repository: FileProjectRepository,
    resource_repository: FileResourceRepository,
}

#[derive(Debug)]
pub struct VacationReportResult {
    pub success: bool,
    pub message: String,
    pub file_path: String,
}

impl VacationReportUseCase {
    pub fn new() -> Self {
        Self {
            project_repository: FileProjectRepository::new(),
            resource_repository: FileResourceRepository::new(),
        }
    }

    pub fn execute(&self) -> Result<VacationReportResult, io::Error> {
        let file_path = "vacation_report.csv";
        let mut writer = Writer::from_path(file_path)?;
        writer.write_record(&["Recurso", "Projeto", "Data Início", "Data Fim"])?;

        if let Ok(project) = self.project_repository.load_project(&PathBuf::from(".")) {
            if let Ok(resources) = self.resource_repository.load_resources() {
                for resource in resources {
                    for periods in &resource.vacations {
                        for period in periods {
                            writer.write_record(&[
                                &resource.name,
                                &project.name,
                                &period.start_date.to_rfc3339(),
                                &period.end_date.to_rfc3339(),
                            ])?;
                        }
                    }
                }
            }
        }

        writer.flush()?;
        
        Ok(VacationReportResult {
            success: true,
            message: "Relatório de férias gerado com sucesso".to_string(),
            file_path: file_path.to_string(),
        })
    }
}
