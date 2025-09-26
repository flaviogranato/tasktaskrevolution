use crate::application::errors::AppError;
use crate::domain::project_management::{AnyProject, repository::ProjectRepository};

pub struct ListProjectsUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> ListProjectsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<AnyProject>, AppError> {
        self.repository.find_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use std::cell::RefCell;

    struct MockProjectRepository {
        projects: RefCell<Vec<AnyProject>>,
        should_fail: bool,
    }

    impl MockProjectRepository {
        fn new(projects: Vec<AnyProject>) -> Self {
            Self {
                projects: RefCell::new(projects),
                should_fail: false,
            }
        }

        fn new_with_failure() -> Self {
            Self {
                projects: RefCell::new(vec![]),
                should_fail: true,
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            self.projects.borrow_mut().push(project);
            Ok(())
        }
        
        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_all".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.projects.borrow().clone())
        }
        
        fn load(&self) -> Result<AnyProject, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "load".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            self.projects.borrow().first().cloned().ok_or(AppError::ProjectNotFound {
                code: "not-found".to_string(),
            })
        }
        
        fn get_next_code(&self) -> Result<String, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "get_next_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok("PROJ-NEXT".to_string())
        }
        
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_by_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.projects.borrow().iter().find(|p| p.code() == code).cloned())
        }
    }

    #[test]
    fn test_list_projects_success() {
        let projects = vec![
            ProjectBuilder::new()
                .code("proj-a".to_string())
                .name("Project A".to_string())
                .company_code("COMP-001".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into(),
            ProjectBuilder::new()
                .code("proj-b".to_string())
                .name("Project B".to_string())
                .company_code("COMP-001".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into(),
        ];
        let mock_repo = MockProjectRepository::new(projects);
        let use_case = ListProjectsUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.name() == "Project A"));
        assert!(result.iter().any(|p| p.name() == "Project B"));
    }

    #[test]
    fn test_list_projects_empty() {
        let mock_repo = MockProjectRepository::new(vec![]);
        let use_case = ListProjectsUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_list_projects_repository_error() {
        let mock_repo = MockProjectRepository::new_with_failure();
        let use_case = ListProjectsUseCase::new(mock_repo);

        let result = use_case.execute();
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_all");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_list_projects_use_case_creation() {
        let mock_repo = MockProjectRepository::new(vec![]);
        let _use_case = ListProjectsUseCase::new(mock_repo);
        
        // Test that the use case was created successfully
        assert!(true); // If we get here, creation succeeded
    }

    #[test]
    fn test_list_projects_with_different_company_codes() {
        let projects = vec![
            ProjectBuilder::new()
                .code("proj-a".to_string())
                .name("Project A".to_string())
                .company_code("COMP-001".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into(),
            ProjectBuilder::new()
                .code("proj-b".to_string())
                .name("Project B".to_string())
                .company_code("COMP-002".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into(),
        ];
        let mock_repo = MockProjectRepository::new(projects);
        let use_case = ListProjectsUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.company_code() == "COMP-001"));
        assert!(result.iter().any(|p| p.company_code() == "COMP-002"));
    }
}
