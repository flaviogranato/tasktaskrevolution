use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DescribeTaskError {
    #[error("Project with code '{0}' not found.")]
    ProjectNotFound(String),
    #[error("Task with code '{0}' not found in project '{1}'.")]
    TaskNotFound(String, String),
    #[error("A repository error occurred: {0}")]
    RepositoryError(#[from] DomainError),
}

pub struct DescribeTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> DescribeTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(&self, project_code: &str, task_code: &str) -> Result<AnyTask, DescribeTaskError> {
        let project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| DescribeTaskError::ProjectNotFound(project_code.to_string()))?;

        let task = project
            .tasks()
            .get(task_code)
            .cloned()
            .ok_or_else(|| DescribeTaskError::TaskNotFound(task_code.to_string(), project_code.to_string()))?;

        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::builder::ProjectBuilder,
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            Ok(self.projects.borrow().get(code).cloned())
        }
        fn load(&self) -> Result<AnyProject, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, DomainError> {
            unimplemented!()
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: Some("A test task.".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: vec!["dev-1".to_string()],
        }
        .into()
    }

    fn create_test_project(code: &str, tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new("Test Project".to_string())
            .code(code.to_string())
            .build()
            .into();
        for task in tasks {
            project.add_task(task);
        }
        project
    }

    #[test]
    fn test_describe_task_success() {
        let project_code = "PROJ-1";
        let task_code = "TSK-1";
        let project = create_test_project(project_code, vec![create_test_task(task_code)]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_code.to_string(), project)])),
        };
        let use_case = DescribeTaskUseCase::new(project_repo);

        let result = use_case.execute(project_code, task_code);

        assert!(result.is_ok());
        let found_task = result.unwrap();
        assert_eq!(found_task.code(), task_code);
        assert!(!found_task.assigned_resources().is_empty());
    }

    #[test]
    fn test_describe_task_not_found() {
        let project_code = "PROJ-1";
        let project = create_test_project(project_code, vec![]); // No tasks
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project_code.to_string(), project)])),
        };
        let use_case = DescribeTaskUseCase::new(project_repo);

        let result = use_case.execute(project_code, "TSK-NONEXISTENT");

        assert!(matches!(result, Err(DescribeTaskError::TaskNotFound(_, _))));
    }

    #[test]
    fn test_describe_task_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let use_case = DescribeTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1");

        assert!(matches!(result, Err(DescribeTaskError::ProjectNotFound(_))));
    }
}
