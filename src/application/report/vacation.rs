use crate::domain::{
    project_management::repository::ProjectRepository, resource_management::repository::ResourceRepository,
};
use csv::Writer;
use std::error::Error;
use std::io;

/// `VacationReportUseCase` gera um relatório em formato CSV com os períodos de férias
/// de todos os recursos, associados ao projeto atual.
/// `VacationReportUseCase` gera um relatório em formato CSV com os períodos de férias
/// de todos os recursos, associados ao projeto atual.
pub struct VacationReportUseCase<P: ProjectRepository, R: ResourceRepository> {
    project_repository: P,
    resource_repository: R,
}

impl<P: ProjectRepository, R: ResourceRepository> VacationReportUseCase<P, R> {
    /// Cria uma nova instância do caso de uso com os repositórios necessários.
    pub fn new(project_repository: P, resource_repository: R) -> Self {
        Self {
            project_repository,
            resource_repository,
        }
    }

    /// Executa a geração do relatório, escrevendo o resultado em um `Writer` fornecido.
    ///
    /// # Arguments
    ///
    /// * `writer` - Um `csv::Writer` para onde o relatório será escrito.
    ///
    /// # Errors
    ///
    /// Retorna um erro se houver falha ao carregar os dados dos repositórios ou
    /// ao escrever no `writer`.
    pub fn execute<W: io::Write>(&self, writer: &mut Writer<W>) -> Result<(), Box<dyn Error>> {
        // Escrever o cabeçalho do CSV
        writer.write_record(["Recurso", "Projeto", "Data Início", "Data Fim", "Layoff"])?;

        // Carregar o projeto do diretório atual.
        // A lógica assume que há um único projeto de referência no contexto.
        let project = self.project_repository.load()?;
        let resources = self.resource_repository.find_all()?;

        // Iterar sobre os recursos e seus períodos de férias
        for resource in resources {
            if let Some(periods) = resource.vacations() {
                for period in periods {
                    writer.write_record([
                        resource.name(),
                        project.name(),
                        &period.start_date.to_rfc3339(),
                        &period.end_date.to_rfc3339(),
                        &period.is_layoff.to_string(),
                    ])?;
                }
            }
        }

        // Garantir que todos os dados sejam escritos no buffer/arquivo
        writer.flush()?;

        Ok(())
    }
}

// ===================================
// TESTES
// ===================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        resource_management::{
            AnyResource,
            resource::{Period, PeriodType, Resource},
            state::Available,
        },
        shared::errors::DomainError,
    };
    use chrono::{Local, TimeZone};

    // --- Mocks ---

    struct MockProjectRepository {
        project: AnyProject,
    }
    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn load(&self) -> Result<AnyProject, DomainError> {
            Ok(self.project.clone())
        }
        fn get_next_code(&self) -> Result<String, DomainError> {
            Ok("proj-1".to_string())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            if self.project.code() == code {
                Ok(Some(self.project.clone()))
            } else {
                Ok(None)
            }
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            Ok(vec![self.project.clone()])
        }
    }

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }
    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.clone())
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(None)
        }
        fn save_time_off(
            &self,
            _r: &str,
            _h: u32,
            _d: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _r: &str,
            _s: &str,
            _e: &str,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(&self, _s: &chrono::DateTime<Local>, _e: &chrono::DateTime<Local>) -> bool {
            unimplemented!()
        }
        fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    // --- Teste Principal ---

    #[test]
    fn test_vacation_report_generation() {
        // 1. Setup: Create test data
        let project: AnyProject = ProjectBuilder::new("ProjetoTTR".to_string())
            .code("proj-1".to_string())
            .build()
            .start()
            .into();

        let mut resource1 = Resource::<Available>::new(
            "dev-1".to_string(),
            "Alice".to_string(),
            None,
            "Dev".to_string(),
            None,
            0,
        );
        resource1.vacations = Some(vec![Period {
            start_date: Local.with_ymd_and_hms(2025, 7, 1, 9, 0, 0).unwrap(),
            end_date: Local.with_ymd_and_hms(2025, 7, 10, 18, 0, 0).unwrap(),
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        }]);

        let resource2 =
            Resource::<Available>::new("qa-1".to_string(), "Bob".to_string(), None, "QA".to_string(), None, 0); // No vacation

        let mock_project_repo = MockProjectRepository { project };
        let mock_resource_repo = MockResourceRepository {
            resources: vec![resource1.into(), resource2.into()],
        };

        let use_case = VacationReportUseCase::new(mock_project_repo, mock_resource_repo);

        // 2. Act: Execute and write to a buffer
        let mut writer = csv::Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        // 3. Assert: Verify the CSV content
        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        assert_eq!(lines.next().unwrap(), "Recurso,Projeto,Data Início,Data Fim,Layoff");
        let alice_line = lines.next().unwrap();
        assert!(alice_line.starts_with("Alice,ProjetoTTR,"));
        assert!(alice_line.ends_with(",false"));
        assert!(lines.next().is_none()); // Bob should not be in the report
    }
}
