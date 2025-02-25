use crate::infrastructure::persistence::{
    project_repository::FileProjectRepository, resource_repository::FileResourceRepository,
};
use csv::Writer;
use std::{io, path::PathBuf};

pub struct VacationReportUseCase {
    project_repository: FileProjectRepository,
    resource_repository: FileResourceRepository,
}

impl VacationReportUseCase {
    pub fn new() -> Self {
        Self {
            project_repository: FileProjectRepository::new(),
            resource_repository: FileResourceRepository::new(),
        }
    }

    pub fn execute(&self) -> Result<(), io::Error> {
        let mut writer = Writer::from_path("vacation_report.csv")?;
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
        println!("✅ Relatório de férias gerado com sucesso: vacation_report.csv");
        Ok(())
    }
}
