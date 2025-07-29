use crate::domain::{
    project_management::repository::ProjectRepository, resource_management::repository::ResourceRepository,
};
use csv::Writer;
use std::error::Error;
use std::io;

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
        project_management::{
            builder::ProjectBuilder,
            project::{Project, ProjectStatus},
        },
        resource_management::resource::{Period, PeriodType, Resource},
        shared::errors::DomainError,
    };
    use chrono::{Local, TimeZone};

    // --- Mocks ---

    struct MockProjectRepository {
        project: Project,
    }
    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: Project) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn load(&self) -> Result<Project, DomainError> {
            Ok(self.project.clone())
        }
    }

    struct MockResourceRepository {
        resources: Vec<Resource>,
    }
    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: Resource) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.clone())
        }
        fn save_time_off(
            &self,
            _r: String,
            _h: u32,
            _d: String,
            _desc: Option<String>,
        ) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _r: String,
            _s: String,
            _e: String,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(&self, _s: &chrono::DateTime<Local>, _e: &chrono::DateTime<Local>) -> bool {
            unimplemented!()
        }
    }

    // --- Teste Principal ---

    #[test]
    fn test_vacation_report_generation() {
        // 1. Setup: Criar dados de teste
        let project = ProjectBuilder::new("ProjetoTTR".to_string())
            .status(ProjectStatus::InProgress)
            .build();

        let mut resource1 = Resource::new(None, "Alice".to_string(), None, "Dev".to_string(), None, None, 0);
        resource1.vacations = Some(vec![Period {
            start_date: Local.with_ymd_and_hms(2025, 7, 1, 9, 0, 0).unwrap(),
            end_date: Local.with_ymd_and_hms(2025, 7, 10, 18, 0, 0).unwrap(),
            approved: true,
            period_type: PeriodType::Vacation,
            is_time_off_compensation: false,
            compensated_hours: None,
            is_layoff: false,
        }]);

        let resource2 = Resource::new(None, "Bob".to_string(), None, "QA".to_string(), None, None, 0); // Sem férias

        let mut resource3 = Resource::new(None, "Charlie".to_string(), None, "Dev".to_string(), None, None, 0);
        resource3.vacations = Some(vec![
            Period {
                // Férias normais
                start_date: Local.with_ymd_and_hms(2025, 8, 1, 0, 0, 0).unwrap(),
                end_date: Local.with_ymd_and_hms(2025, 8, 5, 0, 0, 0).unwrap(),
                approved: true,
                period_type: PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
                is_layoff: false,
            },
            Period {
                // Período de Layoff
                start_date: Local.with_ymd_and_hms(2025, 12, 20, 0, 0, 0).unwrap(),
                end_date: Local.with_ymd_and_hms(2025, 12, 31, 0, 0, 0).unwrap(),
                approved: true,
                period_type: PeriodType::Vacation,
                is_time_off_compensation: false,
                compensated_hours: None,
                is_layoff: true,
            },
        ]);

        // 2. Setup: Criar mocks e o caso de uso
        let mock_project_repo = MockProjectRepository { project };
        let mock_resource_repo = MockResourceRepository {
            resources: vec![resource1, resource2, resource3],
        };
        let use_case = VacationReportUseCase::new(mock_project_repo, mock_resource_repo);

        // 3. Act: Executar o caso de uso, escrevendo para um buffer na memória
        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        // 4. Assert: Verificar o conteúdo do CSV gerado
        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.lines();

        // Verificar cabeçalho
        assert_eq!(lines.next().unwrap(), "Recurso,Projeto,Data Início,Data Fim,Layoff");

        // Verificar linha da Alice
        let alice_line = lines.next().unwrap();
        assert!(alice_line.starts_with("Alice,ProjetoTTR,"));
        assert!(alice_line.ends_with(",false"));

        // Verificar linhas do Charlie
        let charlie_line1 = lines.next().unwrap();
        assert!(charlie_line1.starts_with("Charlie,ProjetoTTR,"));
        assert!(charlie_line1.ends_with(",false"));

        let charlie_line2 = lines.next().unwrap();
        assert!(charlie_line2.starts_with("Charlie,ProjetoTTR,"));
        assert!(charlie_line2.ends_with(",true"));

        // Verificar que não há mais linhas (Bob não tinha férias)
        assert!(lines.next().is_none());
    }
}
