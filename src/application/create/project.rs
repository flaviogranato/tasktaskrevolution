use crate::domain::project_management::{
    AnyProject, builder::ProjectBuilder, repository::ProjectRepository,
};
use crate::application::errors::AppError;

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
    ) -> Result<AnyProject, AppError> {
        // Get the next available code
        let code = self.repository.get_next_code()?;

        // Use the unified builder
        let mut project = ProjectBuilder::new()
            .name(name.to_string())
            .code(code)
            .company_code(company_code)
            .created_by("system".to_string()) // TODO: Get from config
            .end_date(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        // Add description if provided
        if let Some(desc) = description {
            project = project.description(Some(desc.to_string()));
        }

        let project = project.build()?; // This returns Result<Project, AppError>
        let any_project: AnyProject = project.into();

        self.repository.save(any_project.clone())?;
        println!("Project {name} created");
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
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::application::errors::AppError;
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
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            if self.should_fail {
                return Err(AppError::ValidationError {
                    field: "repository".to_string(),
                    message: "Erro mockado ao salvar".to_string(),
                });
            }
            *self.saved_config.borrow_mut() = Some(project);
            Ok(())
        }

        fn load(&self) -> Result<AnyProject, AppError> {
            Ok(self.project.clone())
        }

        fn get_next_code(&self) -> Result<String, AppError> {
            Ok("proj-1".to_string()) // Always return a fixed code for tests
        }

        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            Ok(vec![self.project.clone()])
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
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

        let result = use_case.execute(name, description, "TEST_COMPANY".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_failure() {
        let mock_repo = MockProjectRepository::new(true);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");

        let result = use_case.execute(name, description, "TEST_COMPANY".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_config_saved() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateProjectUseCase::new(mock_repo);
        let name = "John";
        let description = Some("a simple test project");
        let _ = use_case.execute(name, description, "TEST_COMPANY".to_string());

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
}
