use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::any_task::AnyTask;

pub struct ListTasksUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> ListTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<AnyTask>, DomainError> {
        let project = self.repository.load()?;
        let tasks = project.tasks().values().cloned().collect();
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        shared::errors::DomainError,
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;

    use uuid7::uuid7;

    struct MockProjectRepository {
        project: AnyProject,
    }

    impl ProjectRepository for MockProjectRepository {
        fn load(&self) -> Result<AnyProject, DomainError> {
            Ok(self.project.clone())
        }
        // Unimplemented methods
        fn save(&self, _project: AnyProject) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            unimplemented!()
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<AnyProject>, DomainError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, DomainError> {
            unimplemented!()
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
        let mock_repo = MockProjectRepository { project };
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|t| t.name() == "First task"));
        assert!(result.iter().any(|t| t.code() == "TSK-2"));
    }

    #[test]
    fn test_list_tasks_empty() {
        let project = create_project_with_tasks(vec![]);
        let mock_repo = MockProjectRepository { project };
        let use_case = ListTasksUseCase::new(mock_repo);

        let result = use_case.execute().unwrap();
        assert!(result.is_empty());
    }
}
