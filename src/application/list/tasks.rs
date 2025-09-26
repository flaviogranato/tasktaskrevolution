#![allow(unused_imports)]
use crate::application::errors::AppError;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::task_management::{Category, Priority, any_task::AnyTask};

pub struct ListTasksUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> ListTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, project_code: &str, company_code: &str) -> Result<Vec<AnyTask>, AppError> {
        let project = self.repository.find_by_code(project_code)?;
        match project {
            Some(p) => {
                // Verify the project belongs to the correct company
                if p.company_code() == company_code {
                    let tasks = p.tasks().values().cloned().collect::<Vec<_>>();
                    Ok(tasks)
                } else {
                    Err(AppError::ProjectNotFound {
                        code: project_code.to_string(),
                    })
                }
            }
            None => Err(AppError::ProjectNotFound {
                code: project_code.to_string(),
            }),
        }
    }

    pub fn execute_all_by_company(&self, company_code: &str) -> Result<Vec<AnyTask>, AppError> {
        let projects = self.repository.find_all()?;
        let mut all_tasks = Vec::new();

        for project in projects {
            if project.company_code() == company_code {
                let tasks = project.tasks().values().cloned().collect::<Vec<_>>();
                all_tasks.extend(tasks);
            }
        }

        Ok(all_tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::errors::AppError;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::domain::task_management::{state::Planned, task::Task};
    use chrono::NaiveDate;

    use uuid7::uuid7;

    struct MockProjectRepository {
        project: Option<AnyProject>,
        projects: Vec<AnyProject>,
        should_fail: bool,
    }

    impl MockProjectRepository {
        fn new_with_project(project: AnyProject) -> Self {
            Self {
                project: Some(project),
                projects: vec![],
                should_fail: false,
            }
        }

        fn new_with_projects(projects: Vec<AnyProject>) -> Self {
            Self {
                project: None,
                projects,
                should_fail: false,
            }
        }

        fn new_with_failure() -> Self {
            Self {
                project: None,
                projects: vec![],
                should_fail: true,
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn load(&self) -> Result<AnyProject, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "load".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            self.project.clone().ok_or(AppError::ProjectNotFound {
                code: "not-found".to_string(),
            })
        }
        
        fn save(&self, _project: AnyProject) -> Result<(), AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "save".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(())
        }
        
        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_all".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            Ok(self.projects.clone())
        }
        
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            if self.should_fail {
                return Err(AppError::IoError {
                    operation: "find_by_code".to_string(),
                    details: "Mock failure".to_string(),
                });
            }
            
            if let Some(ref project) = self.project {
                if project.code() == code {
                    return Ok(Some(project.clone()));
                }
            }
            
            for project in &self.projects {
                if project.code() == code {
                    return Ok(Some(project.clone()));
                }
            }
            
            Ok(None)
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
    }

    fn create_test_task(code: &str, name: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        }
        .into()
    }

    fn create_project_with_tasks(tasks: Vec<AnyTask>) -> AnyProject {
        let mut builder = ProjectBuilder::new()
            .code("PROJ-1".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string());

        for task in tasks {
            builder = builder.add_task(task);
        }

        builder.build().unwrap().into()
    }

    #[test]
    fn test_list_tasks_success() {
        let tasks = vec![
            create_test_task("TSK-1", "First task"),
            create_test_task("TSK-2", "Second task"),
        ];
        let project = create_project_with_tasks(tasks);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute("PROJ-1", "COMP-001").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "First task"));
        assert!(result.iter().any(|t| t.code() == "TSK-2"));
    }

    #[test]
    fn test_list_tasks_empty() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute("PROJ-1", "COMP-001").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_list_tasks_project_not_found() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute("NONEXISTENT", "COMP-001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ProjectNotFound { code } => assert_eq!(code, "NONEXISTENT"),
            _ => panic!("Expected ProjectNotFound error"),
        }
    }

    #[test]
    fn test_list_tasks_wrong_company() {
        let tasks = vec![create_test_task("TSK-1", "First task")];
        let project = create_project_with_tasks(tasks);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute("PROJ-1", "WRONG-COMPANY");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ProjectNotFound { code } => assert_eq!(code, "PROJ-1"),
            _ => panic!("Expected ProjectNotFound error"),
        }
    }

    #[test]
    fn test_list_tasks_repository_error() {
        let mock_repo = MockProjectRepository::new_with_failure();
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute("PROJ-1", "COMP-001");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::IoError { operation, details } => {
                assert_eq!(operation, "find_by_code");
                assert_eq!(details, "Mock failure");
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_execute_all_by_company_success() {
        let tasks1 = vec![create_test_task("TSK-1", "Task 1")];
        let tasks2 = vec![create_test_task("TSK-2", "Task 2")];
        let project1 = create_project_with_tasks(tasks1);
        let project2 = create_project_with_tasks(tasks2);
        
        let projects = vec![project1, project2];
        let mock_repo = MockProjectRepository::new_with_projects(projects);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute_all_by_company("COMP-001").unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "Task 1"));
        assert!(result.iter().any(|t| t.name() == "Task 2"));
    }

    #[test]
    fn test_execute_all_by_company_empty() {
        let mock_repo = MockProjectRepository::new_with_projects(vec![]);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute_all_by_company("COMP-001").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_all_by_company_wrong_company() {
        let tasks = vec![create_test_task("TSK-1", "Task 1")];
        let project = create_project_with_tasks(tasks);
        let projects = vec![project];
        let mock_repo = MockProjectRepository::new_with_projects(projects);
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute_all_by_company("WRONG-COMPANY").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_execute_all_by_company_repository_error() {
        let mock_repo = MockProjectRepository::new_with_failure();
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute_all_by_company("COMP-001");
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
    fn test_list_tasks_use_case_creation() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository::new_with_project(project);
        let _use_case = ListTasksUseCase::new(mock_repo);
        
        // Test that the use case was created successfully
        assert!(true); // If we get here, creation succeeded
    }
}
