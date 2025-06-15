use crate::domain::shared::errors::DomainError;
use crate::domain::task_management::{Task, TaskStatus, repository::TaskRepository};
use chrono::NaiveDate;

pub struct CreateTaskUseCase<T: TaskRepository> {
    repository: T,
}

impl<T: TaskRepository> CreateTaskUseCase<T> {
    pub fn new(repository: T) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        code: String,
        name: String,
        description: Option<String>,
        start_date: NaiveDate,
        due_date: NaiveDate,
        assigned_resources: Vec<String>,
    ) -> Result<(), DomainError> {
        // Validar que a data de início não é posterior à data de vencimento
        if start_date > due_date {
            return Err(DomainError::Generic(
                "Data de início não pode ser posterior à data de vencimento".to_string(),
            ));
        }

        // Verificar se já existe uma task com o mesmo código
        if let Ok(Some(_)) = self.repository.find_by_code(&code) {
            return Err(DomainError::Generic(format!(
                "Task com código '{}' já existe",
                code
            )));
        }

        // Criar a task
        let task = Task {
            id: format!("TASK-{}", code),
            code: code.clone(),
            name: name.clone(),
            description,
            status: TaskStatus::Planned,
            start_date,
            due_date,
            actual_end_date: None,
            assigned_resources,
        };

        // Salvar a task
        self.repository.save(task)?;

        println!("Task {} criada com sucesso.", name);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::shared::errors::DomainError;
    use crate::domain::task_management::{Task, TaskStatus};
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    struct MockTaskRepository {
        should_fail: bool,
        saved_task: RefCell<Option<Task>>,
        tasks: RefCell<HashMap<String, Task>>,
    }

    impl MockTaskRepository {
        fn new(should_fail: bool) -> Self {
            Self { should_fail, saved_task: RefCell::new(None), tasks: RefCell::new(HashMap::new()) }
        }

        fn add_existing_task(&self, task: Task) {
            self.tasks.borrow_mut().insert(task.code.clone(), task);
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: Task) -> Result<(), DomainError> {
            if self.should_fail {
                return Err(DomainError::Generic("Erro mockado ao salvar".to_string()));
            }

            *self.saved_task.borrow_mut() = Some(task.clone());
            self.tasks.borrow_mut().insert(task.code.clone(), task);
            Ok(())
        }

        fn load(&self, _path: &Path) -> Result<Task, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_code(&self, code: &str) -> Result<Option<Task>, DomainError> {
            Ok(self.tasks.borrow().get(code).cloned())
        }

        fn find_by_id(&self, _id: &str) -> Result<Option<Task>, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_all(&self) -> Result<Vec<Task>, DomainError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn delete(&self, _code: &str) -> Result<(), DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn update_status(&self, _code: &str, _new_status: TaskStatus) -> Result<Task, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_assignee(&self, _assignee: &str) -> Result<Vec<Task>, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_status(&self, _status: &TaskStatus) -> Result<Vec<Task>, DomainError> {
            unimplemented!("Not needed for these tests")
        }

        fn find_by_date_range(&self, _start_date: NaiveDate, _end_date: NaiveDate) -> Result<Vec<Task>, DomainError> {
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

        let result = use_case.execute(
            "TSK001".to_string(),
            "Implementar autenticação".to_string(),
            Some("Implementar sistema de login com JWT".to_string()),
            start_date,
            due_date,
            vec!["dev1".to_string(), "dev2".to_string()],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_task_failure() {
        let mock_repo = MockTaskRepository::new(true);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let result = use_case.execute(
            "TSK001".to_string(),
            "Implementar autenticação".to_string(),
            Some("Implementar sistema de login com JWT".to_string()),
            start_date,
            due_date,
            vec!["dev1".to_string()],
        );

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

        let _ = use_case.execute(
            code.clone(),
            name.clone(),
            description.clone(),
            start_date,
            due_date,
            assigned_resources.clone(),
        );

        let saved_task = use_case.repository.saved_task.borrow();
        assert!(saved_task.is_some());

        let task = saved_task.as_ref().unwrap();
        assert_eq!(task.code, code);
        assert_eq!(task.name, name);
        assert_eq!(task.description, description);
        assert_eq!(task.start_date, start_date);
        assert_eq!(task.due_date, due_date);
        assert_eq!(task.assigned_resources, assigned_resources);
        assert!(matches!(task.status, TaskStatus::Planned));
        assert!(task.actual_end_date.is_none());
        assert_eq!(task.id, format!("TASK-{}", code));
    }

    #[test]
    fn test_create_task_invalid_date_range() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);

        // Data de início posterior à data de vencimento
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 30).unwrap();
        let due_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let result = use_case.execute(
            "TSK001".to_string(),
            "Task inválida".to_string(),
            None,
            start_date,
            due_date,
            vec![],
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Data de início não pode ser posterior")
        );
    }

    #[test]
    fn test_create_task_duplicate_code() {
        let mock_repo = MockTaskRepository::new(false);

        // Adicionar uma task existente
        let existing_task = Task {
            id: "TASK-TSK001".to_string(),
            code: "TSK001".to_string(),
            name: "Task existente".to_string(),
            description: None,
            status: TaskStatus::Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            assigned_resources: vec![],
        };
        mock_repo.add_existing_task(existing_task);

        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let result = use_case.execute(
            "TSK001".to_string(), // Mesmo código da task existente
            "Nova task".to_string(),
            None,
            start_date,
            due_date,
            vec![],
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("já existe"));
    }

    #[test]
    fn test_create_task_without_description() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let result = use_case.execute(
            "TSK001".to_string(),
            "Task sem descrição".to_string(),
            None, // Sem descrição
            start_date,
            due_date,
            vec!["dev1".to_string()],
        );

        assert!(result.is_ok());

        let saved_task = use_case.repository.saved_task.borrow();
        assert!(saved_task.is_some());
        assert!(saved_task.as_ref().unwrap().description.is_none());
    }

    #[test]
    fn test_create_task_without_assigned_resources() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);
        let (start_date, due_date) = create_test_dates();

        let result = use_case.execute(
            "TSK001".to_string(),
            "Task sem recursos".to_string(),
            Some("Task sem recursos atribuídos".to_string()),
            start_date,
            due_date,
            vec![], // Sem recursos atribuídos
        );

        assert!(result.is_ok());

        let saved_task = use_case.repository.saved_task.borrow();
        assert!(saved_task.is_some());
        assert!(saved_task.as_ref().unwrap().assigned_resources.is_empty());
    }

    #[test]
    fn test_create_task_same_start_and_due_date() {
        let mock_repo = MockTaskRepository::new(false);
        let use_case = CreateTaskUseCase::new(mock_repo);

        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        let result = use_case.execute(
            "TSK001".to_string(),
            "Task de um dia".to_string(),
            Some("Task que começa e termina no mesmo dia".to_string()),
            date,
            date, // Mesma data
            vec!["dev1".to_string()],
        );

        assert!(result.is_ok());

        let saved_task = use_case.repository.saved_task.borrow();
        assert!(saved_task.is_some());

        let task = saved_task.as_ref().unwrap();
        assert_eq!(task.start_date, task.due_date);
    }
}
