use crate::domain::task_management::{repository::TaskRepository, task::TaskStatus};
use csv::Writer;
use std::error::Error;
use std::io;

/// `TaskReportUseCase` gera um relatório em formato CSV com todas as tarefas.
pub struct TaskReportUseCase<T: TaskRepository> {
    task_repository: T,
}

impl<T: TaskRepository> TaskReportUseCase<T> {
    /// Cria uma nova instância do caso de uso com o repositório necessário.
    pub fn new(task_repository: T) -> Self {
        Self { task_repository }
    }

    /// Executa a geração do relatório, escrevendo o resultado em um `Writer` fornecido.
    pub fn execute<W: io::Write>(&self, writer: &mut Writer<W>) -> Result<(), Box<dyn Error>> {
        // Escrever o cabeçalho do CSV
        writer.write_record([
            "Code",
            "Name",
            "Status",
            "Progress",
            "StartDate",
            "DueDate",
            "Assignees",
        ])?;

        let tasks = self.task_repository.find_all()?;

        // Iterar sobre as tarefas e escrever os registros
        for task in tasks {
            let (status_str, progress_str) = match task.status {
                TaskStatus::Planned => ("Planned", "0".to_string()),
                TaskStatus::InProgress { progress } => ("InProgress", progress.to_string()),
                TaskStatus::Completed => ("Completed", "100".to_string()),
                TaskStatus::Blocked { .. } => ("Blocked", "N/A".to_string()),
                TaskStatus::Cancelled => ("Cancelled", "N/A".to_string()),
            };

            let assignees_str = task.assigned_resources.join(", ");

            writer.write_record([
                &task.code,
                &task.name,
                status_str,
                &progress_str,
                &task.start_date.to_string(),
                &task.due_date.to_string(),
                &assignees_str,
            ])?;
        }

        writer.flush()?;
        Ok(())
    }
}

// ===================================
// TESTES
// ===================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::shared::errors::DomainError;
    use crate::domain::task_management::{repository::TaskRepository, task::Task};
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::path::Path;

    // --- Mock ---
    struct MockTaskRepository {
        tasks: RefCell<Vec<Task>>,
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, _task: Task) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn load(&self, _path: &Path) -> Result<Task, DomainError> {
            unimplemented!()
        }
        fn find_by_code(&self, _code: &str) -> Result<Option<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_id(&self, _id: &str) -> Result<Option<Task>, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<Task>, DomainError> {
            Ok(self.tasks.borrow().clone())
        }
        fn delete(&self, _id: &str) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn update_status(&self, _code: &str, _new_status: TaskStatus) -> Result<Task, DomainError> {
            unimplemented!()
        }
        fn find_by_assignee(&self, _assignee: &str) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_status(&self, _status: &TaskStatus) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_date_range(&self, _start: NaiveDate, _end: NaiveDate) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
    }

    // --- Teste Principal ---
    #[test]
    fn test_task_report_generation() {
        // 1. Setup: Criar dados de teste
        let task1 = Task {
            id: "TASK-001".to_string(),
            code: "TSK-001".to_string(),
            name: "Implement Login".to_string(),
            description: None,
            status: TaskStatus::InProgress { progress: 50 },
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: vec!["Alice".to_string()],
        };
        let task2 = Task {
            id: "TASK-002".to_string(),
            code: "TSK-002".to_string(),
            name: "Setup Database".to_string(),
            description: None,
            status: TaskStatus::Completed,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
            actual_end_date: None,
            assigned_resources: vec!["Bob".to_string(), "Charlie".to_string()],
        };

        let mock_repo = MockTaskRepository {
            tasks: RefCell::new(vec![task1, task2]),
        };
        let use_case = TaskReportUseCase::new(mock_repo);

        // 2. Act: Executar e escrever para um buffer
        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        // 3. Assert: Verificar o conteúdo do CSV
        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );
        assert_eq!(
            lines.next().unwrap(),
            "TSK-001,Implement Login,InProgress,50,2025-01-01,2025-01-10,Alice"
        );
        assert_eq!(
            lines.next().unwrap(),
            "TSK-002,Setup Database,Completed,100,2025-01-02,2025-01-05,\"Bob, Charlie\""
        );
        assert!(lines.next().is_none());
    }
}
