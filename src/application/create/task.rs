// Priority and Category are used in Task initializations
use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::TaskBuilder,
};
use chrono::NaiveDate;

pub struct CreateTaskArgs {
    pub company_code: String,
    pub project_code: String,
    pub name: String,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub assigned_resources: Vec<String>,
}

pub struct CreateTaskUseCase<R: ProjectRepository> {
    repository: R,
}

impl<R: ProjectRepository> CreateTaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn execute(&self, args: CreateTaskArgs) -> Result<(), DomainError> {
        let CreateTaskArgs {
            company_code: _company_code, // TODO: Use this for hierarchical task saving
            project_code,
            name,
            start_date,
            due_date,
            assigned_resources,
        } = args;

        // 1. Load the project aggregate.
        let mut project = self
            .repository
            .find_by_code(&project_code)?
            .ok_or_else(|| DomainError::ProjectNotFound {
                code: project_code.clone(),
            })?;

        // 2. Delegate task creation to the project aggregate.
        // This is a placeholder for the future implementation of `project.add_task(...)`
        // For now, we'll keep the builder logic here.
        if start_date > due_date {
            return Err(DomainError::ValidationError {
                field: "dates".to_string(),
                message: "Data de início não pode ser posterior à data de vencimento".to_string(),
            });
        }

        let next_task_code = format!("task-{}", project.tasks().len() + 1);

        let builder = TaskBuilder::new()
            .project_code(project_code)
            .name(name.clone())
            .code(next_task_code)
            .dates(start_date, due_date)
            .map_err(|e| DomainError::ValidationError {
                field: "task".to_string(),
                message: e.to_string(),
            })?;

        let task = if assigned_resources.is_empty() {
            builder
                .validate_vacations(&[])
                .unwrap()
                .build()
                .map_err(|e| DomainError::ValidationError {
                    field: "task".to_string(),
                    message: e.to_string(),
                })
        } else {
            let mut iter = assigned_resources.into_iter();
            let builder_with_res = builder.assign_resource(iter.next().unwrap());
            let final_builder = iter.fold(builder_with_res, |b, r| b.assign_resource(r));
            final_builder
                .validate_vacations(&[])
                .unwrap()
                .build()
                .map_err(|e| DomainError::ValidationError {
                    field: "task".to_string(),
                    message: e.to_string(),
                })
        }?;

        // Add the task to the project (this part will be moved into a project method later)
        project.add_task(task.into());

        // 3. Save the entire project aggregate.
        self.repository.save(project)?;

        println!("Task {name} created successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockProjectRepository {
        should_fail: bool,
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    impl MockProjectRepository {
        fn new(should_fail: bool) -> Self {
            let mut projects = HashMap::new();
            let project = ProjectBuilder::new()
                .code("PROJ-1".to_string())
                .name("Test Project".to_string())
                .company_code("COMP-001".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into();
            projects.insert("PROJ-1".to_string(), project);

            Self {
                should_fail,
                projects: RefCell::new(projects),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Erro mockado ao salvar".to_string(),
                });
            }
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            Ok(self.projects.borrow().get(code).cloned())
        }

        // Unimplemented methods
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

    fn create_test_dates() -> (NaiveDate, NaiveDate) {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 1, 30).unwrap();
        (start_date, due_date)
    }

    #[test]
    fn test_create_task_success() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Implementar autenticação".to_string(),
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        let project = use_case.repository.find_by_code("PROJ-1").unwrap().unwrap();
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Implementar autenticação");
    }

    #[test]
    fn test_create_task_fails_if_project_not_found() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-NONEXISTENT".to_string(),
            name: "Task for nonexistent project".to_string(),
            start_date,
            due_date,
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_task_fails_if_start_date_after_due_date() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        #[allow(unused_variables)]
        let (start_date, due_date) = create_test_dates();

        // Test with start_date > due_date
        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with invalid dates".to_string(),
            start_date: due_date + chrono::Duration::days(1), // start_date > due_date
            due_date,
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string()
                    .contains("Data de início não pode ser posterior à data de vencimento")
            );
        }
    }

    #[test]
    fn test_create_task_with_same_start_and_due_date() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        #[allow(unused_variables)]
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with same dates".to_string(),
            start_date,           // Use the same date for both
            due_date: start_date, // Use the same date for both
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        if let Err(e) = &result {
            eprintln!("Error creating task with same dates: {}", e);
        }

        assert!(result.is_ok(), "Expected Ok, but got Err: {:?}", result);
    }

    #[test]
    fn test_create_task_without_assigned_resources() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task without resources".to_string(),
            start_date,
            due_date,
            assigned_resources: vec![], // Empty resources vector
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        let project = use_case.repository.find_by_code("PROJ-1").unwrap().unwrap();
        // Count should be 1 since we're starting with a fresh project
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Task without resources");
    }

    #[test]
    fn test_create_task_with_multiple_assigned_resources() {
        let mock_repo = MockProjectRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with multiple resources".to_string(),
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string(), "dev2".to_string(), "dev3".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        let project = use_case.repository.find_by_code("PROJ-1").unwrap().unwrap();
        // Count should be 1 since we're starting with a fresh project
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Task with multiple resources");
    }

    #[test]
    fn test_create_task_repository_save_failure() {
        let mock_repo = MockProjectRepository::new(true); // This will make save() fail
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task that will fail to save".to_string(),
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Erro mockado ao salvar"));
        }
    }
}
