use crate::domain::{
    project::repository::ProjectRepository, resource::repository::ResourceRepository,
};
use crate::infrastructure::persistence::{
    project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
};
use chrono::{DateTime, Local};
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
        writer.write_record(["Recurso", "Projeto", "Data Início", "Data Fim", "Layoff"])?;

        if let Ok(project) = self.project_repository.load(&PathBuf::from(".")) {
            if let Ok(resources) = self.resource_repository.find_all() {
                for resource in resources {
                    if let Some(periods) = &resource.vacations {
                        for period in periods {
                            writer.write_record([
                                &resource.name,
                                &project.name,
                                &period.start_date.to_rfc3339(),
                                &period.end_date.to_rfc3339(),
                                &period.is_layoff.to_string(),
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

    fn generate_report(
        &self,
        writer: &mut csv::Writer<Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writer.write_record(["Recurso", "Projeto", "Data Início", "Data Fim", "Layoff"])?;

        if let Ok(project) = self.project_repository.load(&PathBuf::from(".")) {
            if let Ok(resources) = self.resource_repository.find_all() {
                for resource in resources {
                    if let Some(periods) = &resource.vacations {
                        for period in periods {
                            writer.write_record([
                                &resource.name,
                                &project.name,
                                &period.start_date.to_rfc3339(),
                                &period.end_date.to_rfc3339(),
                                &period.is_layoff.to_string(),
                            ])?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn generate(
        &self,
        _range_start: Option<DateTime<Local>>,
        _range_end: Option<DateTime<Local>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(Vec::new());

        self.generate_report(&mut wtr)?;

        wtr.flush()?;

        Ok(String::from_utf8(wtr.into_inner()?)?)
    }
}

impl Default for VacationReportUseCase {
    fn default() -> Self {
        Self::new()
    }
}
