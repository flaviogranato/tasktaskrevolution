use crate::domain::project_management::{AnyProject, repository::ProjectRepository};
use crate::application::errors::AppError;

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
    }

    impl MockProjectRepository {
        fn new(projects: Vec<AnyProject>) -> Self {
            Self {
                projects: RefCell::new(projects),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            self.projects.borrow_mut().push(project);
            Ok(())
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            Ok(self.projects.borrow().clone())
        }
        fn load(&self) -> Result<AnyProject, AppError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, AppError> {
            unimplemented!()
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
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
}
