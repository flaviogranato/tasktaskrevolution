use crate::domain::{
    project_management::repository::ProjectRepository, resource_management::repository::ResourceRepository,
};
use csv::Writer;
use std::error::Error;
use std::io;

/// `VacationReportUseCase` generates a CSV report with vacation periods
/// of all resources, associated with the current project.
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

    /// Executes report generation, writing the result to a provided `Writer`.
    ///
    /// # Arguments
    ///
    /// * `writer` - A `csv::Writer` where the report will be written.
    ///
    /// # Errors
    ///
    /// Returns an error if there is a failure loading data from repositories or
    /// writing to the `writer`.
    pub fn execute<W: io::Write>(&self, writer: &mut Writer<W>) -> Result<(), Box<dyn Error>> {
        // Write CSV header
        writer.write_record(["Resource", "Project", "Start Date", "End Date", "Layoff"])?;

        // Load the project from the current directory.
        // The logic assumes there is a single reference project in the context.
        let project = self.project_repository.load()?;
        let resources = self.resource_repository.find_all()?;

        // Iterate over resources and their vacation periods
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
    use crate::application::errors::AppError;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::domain::resource_management::{
        AnyResource,
        resource::{Period, PeriodType, Resource},
        state::Available,
    };
    use chrono::{Local, TimeZone};

    // --- Mocks ---

    struct MockProjectRepository {
        project: AnyProject,
    }
    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> Result<(), AppError> {
            unimplemented!()
        }
        fn load(&self) -> Result<AnyProject, AppError> {
            Ok(self.project.clone())
        }
        fn get_next_code(&self) -> Result<String, AppError> {
            Ok("proj-1".to_string())
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            if self.project.code() == code {
                Ok(Some(self.project.clone()))
            } else {
                Ok(None)
            }
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            Ok(vec![self.project.clone()])
        }
    }

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }
    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, AppError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(self.resources.clone())
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(None)
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            self.save(resource)
        }
        fn save_time_off(&self, _r: &str, _h: u32, _d: &str, _desc: Option<String>) -> Result<AnyResource, AppError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _r: &str,
            _s: &str,
            _e: &str,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }
        fn check_if_layoff_period(&self, _s: &chrono::DateTime<Local>, _e: &chrono::DateTime<Local>) -> bool {
            unimplemented!()
        }
        fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    // --- Teste Principal ---

    #[test]
    fn test_vacation_report_generation() {
        // 1. Setup: Create test data
        let project: AnyProject = ProjectBuilder::new()
            .code("proj-1".to_string())
            .name("TTRProject".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
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

        assert_eq!(lines.next().unwrap(), "Resource,Project,Start Date,End Date,Layoff");
        let alice_line = lines.next().unwrap();
        assert!(alice_line.starts_with("Alice,TTRProject,"));
        assert!(alice_line.ends_with(",false"));
        assert!(lines.next().is_none()); // Bob should not be in the report
    }
}
