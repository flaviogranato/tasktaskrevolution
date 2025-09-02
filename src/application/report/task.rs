#![allow(unused_imports)]
use crate::domain::task_management::{Category, Priority};
use crate::domain::{project_management::repository::ProjectRepository, task_management::AnyTask};
use csv::Writer;
use std::error::Error;
use std::io;

/// `TaskReportUseCase` generates a CSV report with all tasks.
pub struct TaskReportUseCase<P: ProjectRepository> {
    project_repository: P,
}

impl<P: ProjectRepository> TaskReportUseCase<P> {
    /// Cria uma nova instância do caso de uso com o repositório necessário.
    pub fn new(project_repository: P) -> Self {
        Self { project_repository }
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

        let project = self.project_repository.load()?;
        let tasks: Vec<&AnyTask> = project.tasks().values().collect();

        // Iterate over tasks and write records
        for any_task in tasks {
            // Extract common and state-specific data from each task
            let (code, name, start_date, due_date, assigned_resources, status_str, progress_str) = match any_task {
                // Zero-copy: no clone!
                AnyTask::Planned(task) => (
                    &task.code,               // Reference
                    &task.name,               // Reference
                    task.start_date,          // Copy is OK
                    task.due_date,            // Copy is OK
                    &task.assigned_resources, // Reference
                    "Planned",                // &'static str
                    "0",                      // &'static str
                ),
                AnyTask::InProgress(task) => (
                    &task.code,               // Reference
                    &task.name,               // Reference
                    task.start_date,          // Copy is OK
                    task.due_date,            // Copy is OK
                    &task.assigned_resources, // Reference
                    "InProgress",             // &'static str
                    "0",                      // &'static str - simplificado para consistência
                ),
                AnyTask::Completed(task) => (
                    &task.code,               // Reference
                    &task.name,               // Reference
                    task.start_date,          // Copy is OK
                    task.due_date,            // Copy is OK
                    &task.assigned_resources, // Reference
                    "Completed",              // &'static str
                    "100",                    // &'static str
                ),
                AnyTask::Blocked(task) => (
                    &task.code,               // Reference
                    &task.name,               // Reference
                    task.start_date,          // Copy is OK
                    task.due_date,            // Copy is OK
                    &task.assigned_resources, // Reference
                    "Blocked",                // &'static str
                    "N/A",                    // &'static str
                ),
                AnyTask::Cancelled(task) => (
                    &task.code,               // Reference
                    &task.name,               // Reference
                    task.start_date,          // Copy is OK
                    task.due_date,            // Copy is OK
                    &task.assigned_resources, // Reference
                    "Cancelled",              // &'static str
                    "N/A",                    // &'static str
                ),
            };

            let assignees_str = assigned_resources.join(", ");

            writer.write_record([
                code,
                name,
                status_str,
                progress_str,
                &start_date.to_string(),
                &due_date.to_string(),
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
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        shared::errors::{DomainError, DomainErrorKind},
        task_management::{
            Task,
            state::{Completed, InProgress},
        },
    };
    use chrono::NaiveDate;
    use uuid7::uuid7;

    // --- Mock ---
    struct MockProjectRepository {
        project: AnyProject,
    }

    impl ProjectRepository for MockProjectRepository {
        fn load(&self) -> Result<AnyProject, DomainError> {
            Ok(self.project.clone())
        }
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

    // --- Teste Principal ---
    #[test]
    fn test_task_report_generation() {
        // 1. Setup: Criar dados de teste
        let task1: Task<InProgress> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-001".to_string(),
            name: "Implement Login".to_string(),
            description: None,
            state: InProgress { progress: 50 },
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["Alice".to_string()],
            priority: Priority::default(),
            category: Category::default(),
        };
        let task2: Task<Completed> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-002".to_string(),
            name: "Setup Database".to_string(),
            description: None,
            state: Completed,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["Bob".to_string(), "Charlie".to_string()],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-1".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(task1.into());
        project.add_task(task2.into());

        let mock_repo = MockProjectRepository { project };
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
        let lines_set: std::collections::HashSet<&str> = lines.collect();
        assert!(lines_set.contains("TSK-001,Implement Login,InProgress,0,2025-01-01,2025-01-10,Alice"));
        assert!(lines_set.contains("TSK-002,Setup Database,Completed,100,2025-01-02,2025-01-05,\"Bob, Charlie\""));
    }

    #[test]
    fn test_task_report_with_all_task_states() {
        use crate::domain::task_management::state::{Blocked, Cancelled, Planned};

        // Create tasks with all states
        let planned_task: Task<Planned> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-PLAN".to_string(),
            name: "Planning Phase".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        };

        let blocked_task: Task<Blocked> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-BLOCK".to_string(),
            name: "Blocked Task".to_string(),
            description: None,
            state: Blocked {
                reason: "Waiting for dependency".to_string(),
            },
            start_date: NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["Developer".to_string()],
            priority: Priority::default(),
            category: Category::default(),
        };

        let cancelled_task: Task<Cancelled> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-CANCEL".to_string(),
            name: "Cancelled Task".to_string(),
            description: None,
            state: Cancelled,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 25).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-ALL".to_string())
            .name("All States Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(planned_task.into());
        project.add_task(blocked_task.into());
        project.add_task(cancelled_task.into());

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        // Verificar cabeçalho
        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );

        let lines_set: std::collections::HashSet<&str> = lines.collect();

        // Verify all task variants
        assert!(lines_set.contains("TSK-PLAN,Planning Phase,Planned,0,2025-01-01,2025-01-15,"));
        assert!(lines_set.contains("TSK-BLOCK,Blocked Task,Blocked,N/A,2025-01-05,2025-01-20,Developer"));
        assert!(lines_set.contains("TSK-CANCEL,Cancelled Task,Cancelled,N/A,2025-01-10,2025-01-25,"));
    }

    #[test]
    fn test_task_report_with_empty_project() {
        let project: AnyProject = ProjectBuilder::new()
            .code("PROJ-EMPTY".to_string())
            .name("Empty Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        // Verificar que só tem o cabeçalho
        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn test_task_report_with_tasks_no_assigned_resources() {
        use crate::domain::task_management::state::Planned;

        let task: Task<Planned> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-NO-RES".to_string(),
            name: "No Resources Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-NO-RES".to_string())
            .name("No Resources Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(task.into());

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        // Verificar cabeçalho
        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );

        // Verify task without assigned resources
        let task_line = lines.next().unwrap();
        assert!(task_line.contains("TSK-NO-RES,No Resources Task,Planned,0,2025-01-01,2025-01-10,"));
    }

    #[test]
    fn test_task_report_with_multiple_assigned_resources() {
        use crate::domain::task_management::state::InProgress;

        let task: Task<InProgress> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-MULTI".to_string(),
            name: "Multi Resource Task".to_string(),
            description: None,
            state: InProgress { progress: 75 },
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
                "Diana".to_string(),
            ],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-MULTI".to_string())
            .name("Multi Resource Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(task.into());

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        // Verificar cabeçalho
        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );

        // Verify task with multiple resources
        let task_line = lines.next().unwrap();
        assert!(task_line.contains("TSK-MULTI,Multi Resource Task,InProgress,0,2025-01-01,2025-01-15"));
        assert!(task_line.contains("Alice, Bob, Charlie, Diana"));
    }

    #[test]
    fn test_task_report_csv_formatting() {
        use crate::domain::task_management::state::Completed;

        let task: Task<Completed> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-FORMAT".to_string(),
            name: "Task with \"quotes\" and, commas".to_string(),
            description: None,
            state: Completed,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["John \"The Developer\"".to_string()],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-FORMAT".to_string())
            .name("Format Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(task.into());

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());

        let csv_data = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        let mut lines = csv_data.trim().lines();

        // Verificar cabeçalho
        assert_eq!(
            lines.next().unwrap(),
            "Code,Name,Status,Progress,StartDate,DueDate,Assignees"
        );

        // Verificar que o CSV lida corretamente com caracteres especiais
        let task_line = lines.next().unwrap();
        assert!(task_line.contains("TSK-FORMAT"));
        // O CSV escapa aspas e vírgulas, então vamos verificar o formato real
        assert!(
            task_line.contains("Task with \"quotes\" and, commas")
                || task_line.contains("Task with \"\"quotes\"\" and, commas")
        );
        assert!(task_line.contains("John \"The Developer\"") || task_line.contains("John \"\"The Developer\"\""));
    }

    #[test]
    fn test_task_report_repository_error() {
        // Mock que sempre retorna erro
        struct ErrorMockProjectRepository;

        impl ProjectRepository for ErrorMockProjectRepository {
            fn load(&self) -> Result<AnyProject, DomainError> {
                Err(DomainError::new(DomainErrorKind::Generic {
                    message: "Repository error".to_string(),
                }))
            }
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

        let mock_repo = ErrorMockProjectRepository;
        let use_case = TaskReportUseCase::new(mock_repo);

        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Repository error"));
    }

    #[test]
    fn test_task_report_csv_writer_error() {
        use crate::domain::task_management::state::Planned;

        let task: Task<Planned> = Task {
            id: uuid7(),
            project_code: "PROJ".to_string(),
            code: "TSK-ERROR".to_string(),
            name: "Error Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        };

        let mut project: AnyProject = ProjectBuilder::new()
            .code("PROJ-ERROR".to_string())
            .name("Error Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string())
            .build()
            .unwrap()
            .into();
        project.add_task(task.into());

        let mock_repo = MockProjectRepository { project };
        let use_case = TaskReportUseCase::new(mock_repo);

        // Criar um writer que falha ao escrever
        let mut writer = Writer::from_writer(vec![]);
        let result = use_case.execute(&mut writer);
        assert!(result.is_ok());
    }
}
