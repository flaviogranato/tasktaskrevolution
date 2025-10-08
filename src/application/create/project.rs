use crate::application::errors::AppError;
use crate::domain::company_settings::repository::ConfigRepository;
use crate::domain::project_management::{AnyProject, builder::ProjectBuilder, repository::ProjectRepository};
use crate::infrastructure::persistence::config_repository::FileConfigRepository;

pub struct CreateProjectUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> CreateProjectUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        name: &str,
        description: Option<&str>,
        company_code: String,
        code: Option<String>,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<AnyProject, AppError> {
        // Get the next available code or use provided code
        let code = match code {
            Some(c) => c,
            None => self.repository.get_next_code()?,
        };

        // Use the unified builder
        let mut project = ProjectBuilder::new()
            .name(name.to_string())
            .code(code)
            .company_code(company_code.clone())
            .created_by("system".to_string()); // TODO: Get from config

        // Add dates if provided
        if let Some(start) = start_date {
            project = project.start_date(start);
        }
        if let Some(end) = end_date {
            project = project.end_date(end);
        }

        // Add description if provided
        if let Some(desc) = description {
            project = project.description(Some(desc.to_string()));
        }

        // Load config to get default timezone
        let config_repo = FileConfigRepository::new();
        if let Ok((config, _)) = config_repo.load() {
            // Apply default timezone from config if not already set
            project = project.timezone(config.default_timezone);
        }

        let project = project.build()?; // This returns Result<Project, AppError>
        let any_project: AnyProject = project.into();

        self.repository.save(any_project.clone())?;
        Ok(any_project)
    }

    #[allow(dead_code)]
    #[cfg(test)]
    pub fn get_repository(&self) -> &R {
        &self.repository
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use std::cell::RefCell;

    struct MockProjectRepository {
        should_fail: bool,
        saved_config: RefCell<Option<AnyProject>>,
        project: AnyProject,
    }

    impl MockProjectRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_config: RefCell::new(None),
                project: ProjectBuilder::new()
                    .name("John".to_string())
                    .code("proj-1".to_string())
                    .company_code("COMP-001".to_string())
                    .created_by("system".to_string())
                    .end_date(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
                    .build()
                    .unwrap()
                    .into(),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Mocked save error".to_string(),
                });
            }
            *self.saved_config.borrow_mut() = Some(project);
            Ok(())
        }

        fn load(&self) -> DomainResult<AnyProject> {
            Ok(self.project.clone())
        }

        fn get_next_code(&self) -> DomainResult<String> {
            Ok("proj-1".to_string()) // Always return a fixed code for tests
        }

        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            Ok(vec![self.project.clone()])
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            if self.project.code() == code {
                Ok(Some(self.project.clone()))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_create_project_success() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");

        let result = use_case.execute(name, description, "TEST_COMPANY".to_string(), None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockProjectRepository::new(true);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");

        let result = use_case.execute(name, description, "TEST_COMPANY".to_string(), None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");
        let _ = use_case.execute(name, description, "TEST_COMPANY".to_string(), None, None, None);

        let saved_config = use_case.get_repository().saved_config.borrow();
        assert!(saved_config.is_some());
        let any_project = saved_config.as_ref().unwrap();
        assert_eq!(any_project.name(), name);
        // AnyProject is no longer an enum with variants, so we can access the project directly
        // Compare descriptions by converting both to Option<String>
        let expected_desc = description.as_ref().map(|s| s.to_string());
        let actual_desc = any_project.description().map(|s| s.to_string());
        assert_eq!(actual_desc, expected_desc);
    }

    #[test]
    fn test_create_project_with_custom_code() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Custom Project";
        let custom_code = "CUSTOM-001".to_string();

        let result = use_case.execute(
            name,
            None,
            "TEST_COMPANY".to_string(),
            Some(custom_code.clone()),
            None,
            None,
        );
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.code(), custom_code);
        assert_eq!(project.name(), name);
    }

    #[test]
    fn test_create_project_with_dates() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Project with Dates";
        let start_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let result = use_case.execute(
            name,
            None,
            "TEST_COMPANY".to_string(),
            None,
            Some(start_date),
            Some(end_date),
        );
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), name);
        assert_eq!(project.start_date(), Some(start_date));
        assert_eq!(project.end_date(), Some(end_date));
    }

    #[test]
    fn test_create_project_with_description() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Project with Description";
        let description = "This is a detailed project description";

        let result = use_case.execute(name, Some(description), "TEST_COMPANY".to_string(), None, None, None);
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), name);
        assert_eq!(project.description(), Some(&description.to_string()));
    }

    #[test]
    fn test_create_project_use_case_creation() {
        let mock_repo = MockProjectRepository::new(false);
        let _use_case = CreateProjectUseCase::new(mock_repo);

        // Test that the use case was created successfully
        // If we get here, creation succeeded
    }

    #[test]
    fn test_create_project_repository_error() {
        let mock_repo = MockProjectRepository::new(true);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Test Project";

        let result = use_case.execute(name, None, "TEST_COMPANY".to_string(), None, None, None);
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::ValidationError { field, message } => {
                assert_eq!(field, "repository");
                assert_eq!(message, "Mocked save error");
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_create_project_minimal_parameters() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Minimal Project";

        let result = use_case.execute(name, None, "TEST_COMPANY".to_string(), None, None, None);
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), name);
        assert_eq!(project.company_code(), "TEST_COMPANY");
        assert_eq!(project.created_by(), "system");
    }

    #[test]
    fn test_create_project_with_all_parameters() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "Complete Project";
        let description = "A complete project with all parameters";
        let custom_code = "COMPLETE-001".to_string();
        let start_date = chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let end_date = chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        let result = use_case.execute(
            name,
            Some(description),
            "TEST_COMPANY".to_string(),
            Some(custom_code.clone()),
            Some(start_date),
            Some(end_date),
        );
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), name);
        assert_eq!(project.description(), Some(&description.to_string()));
        assert_eq!(project.code(), custom_code);
        assert_eq!(project.start_date(), Some(start_date));
        assert_eq!(project.end_date(), Some(end_date));
        assert_eq!(project.company_code(), "TEST_COMPANY");
    }
}
