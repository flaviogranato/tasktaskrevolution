use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::{AnyTask, Task, repository::TaskRepository, state::Planned};
use chrono::NaiveDate;

pub struct CreateTaskArgs {
    pub project_code: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub assigned_resources: Vec<String>,
}

pub struct CreateTaskUseCase<T: TaskRepository> {
    repository: T,
}

impl<T: TaskRepository> CreateTaskUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub fn execute(&self, args: CreateTaskArgs) -> Result<(), DomainError> {
        let CreateTaskArgs {
            project_code,
            code,
            name,
            description,
            start_date,
            due_date,
            assigned_resources,
        } = args;

        // Validar que a data de início não é posterior à data de vencimento
        if start_date > due_date {
            return Err(DomainError::Generic(
                "Data de início não pode ser posterior à data de vencimento".to_string(),
            ));
        }

        // Verificar se já existe uma task com o mesmo código
        if let Ok(Some(_)) = self.repository.find_by_code(&code) {
            return Err(DomainError::Generic(format!("Task com código '{code}' já existe")));
        }

        // Criar a task no estado 'Planned'
        let task: Task<Planned> = Task {
            id: format!("TASK-{code}"),
            project_code,
            code,
            name: name.clone(),
            description,
            state: Planned,
            start_date,
            due_date,
            actual_end_date: None,
            assigned_resources,
        };

        // Salvar a task. A conversão para AnyTask permite que o repositório a armazene.
        self.repository.save(task.into())?;

        println!("Task {name} criada com sucesso.");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::task_management::state::Planned;
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    struct MockTaskRepository {
        should_fail: bool,
        saved_task: RefCell<Option<AnyTask>>,
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl MockTaskRepository {
        fn new(should_fail: bool) -> Self {
            Self {
                should_fail,
                saved_task: RefCell::new(None),
                tasks: RefCell::new(HashMap::new()),
            }
        }

        fn add_existing_task(&self, task: AnyTask) {
            self.tasks.borrow_mut().insert(task.code().to_string(), task);
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: AnyTask) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::Generic("Erro mockado ao salvar".to_string()));
            }
            let code = task.code().to_string();
            *self.saved_task.borrow_mut() = Some(task.clone());
            self.tasks.borrow_mut().insert(code, task);
            Ok(())
        }

        fn load(&self, _path: &Path) -> Result<AnyTask, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().get(code).cloned())
        }

        fn find_by_id(&self, _id: &str) -> Result<Option<AnyTask>, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn delete(&self, _code: &str) -> Result<(), DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_assignee(&self, _assignee: &str) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_date_range(
            &self,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!("Not needed for these tests")
        }
    }

    fn create_test_dates() -> (NaiveDate, NaiveDate) {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 1, 30).unwrap();
        (start_date, due_date)
    }

    #[test]
    fn test_create_task_success() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            code: "TSK001".to_string(),
            name: "Implementar autenticação".to_string(),
            description: Some("Implementar sistema de login com JWT".to_string()),
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string(), "dev2".to_string()],
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_task_failure_on_save() {
        let mock_repo = MockTaskRepository::new(true);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            code: "TSK001".to_string(),
            name: "Implementar autenticação".to_string(),
            description: Some("Implementar sistema de login com JWT".to_string()),
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string()],
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
    }

    #[test]
    fn test_verify_task_saved() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();
        let code = "TSK001".to_string();
        let name = "Implementar autenticação".to_string();
        let description = Some("Implementar sistema de login com JWT".to_string());
        let assigned_resources = vec!["dev1".to_string(), "dev2".to_string()];
        let project_code = "PROJ-1".to_string();

        let args = CreateTaskArgs {
            project_code: project_code.clone(),
            code: code.clone(),
            name: name.clone(),
            description: description.clone(),
            start_date,
            due_date,
            assigned_resources: assigned_resources.clone(),
        };
        let _ = use_case.execute(args);

        let saved_task = use_case.repository.saved_task.borrow();
        assert!(saved_task.is_some());

        if let Some(AnyTask::Planned(task)) = saved_task.as_ref() {
            assert_eq!(task.project_code, project_code);
            assert_eq!(task.code, code);
            assert_eq!(task.name, name);
            assert_eq!(task.description, description);
            assert_eq!(task.start_date, start_date);
            assert_eq!(task.due_date, due_date);
            assert_eq!(task.assigned_resources, assigned_resources);
            assert_eq!(task.id, format!("TASK-{code}"));
        } else {
            panic!("Saved task was not in the expected Planned state");
        }
    }

    #[test]
    fn test_create_task_invalid_date_range() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 30).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            code: "TSK001".to_string(),
            name: "Task inválida".to_string(),
            description: None,
            start_date,
            due_date,
            assigned_resources: vec![],
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("posterior"));
    }

    #[test]
    fn test_create_task_duplicate_code() {
        let mock_repo = MockTaskRepository::new(false);

        let existing_task = Task {
            id: "TASK-TSK001".to_string(),
            project_code: "PROJ-1".to_string(),
            code: "TSK001".to_string(),
            name: "Task existente".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            assigned_resources: vec![],
        };
        mock_repo.add_existing_task(existing_task.into());

        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            code: "TSK001".to_string(),
            name: "Nova task".to_string(),
            description: None,
            start_date,
            due_date,
            assigned_resources: vec![],
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("já existe"));
    }
}
