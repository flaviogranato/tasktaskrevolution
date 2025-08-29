use crate::domain::{project_management::repository::ProjectRepository, task_management::AnyTask};
use csv::Writer;
use std::error::Error;
use std::io;

/// `TaskReportUseCase` gera um relatório em formato CSV com todas as tarefas.
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

        // Iterar sobre as tarefas e escrever os registros
        for any_task in tasks {
            // Extrair dados comuns e específicos do estado de cada tarefa
            let (code, name, start_date, due_date, assigned_resources, status_str, progress_str) =
                match any_task {  // Zero-copy: sem clone!
                    AnyTask::Planned(task) => (
                        &task.code,                  // Referência
                        &task.name,                  // Referência
                        task.start_date,             // Copy é OK
                        task.due_date,               // Copy é OK
                        &task.assigned_resources,    // Referência
                        "Planned",                   // &'static str
                        "0",                         // &'static str
                    ),
                    AnyTask::InProgress(task) => (
                        &task.code,                  // Referência
                        &task.name,                  // Referência
                        task.start_date,             // Copy é OK
                        task.due_date,               // Copy é OK
                        &task.assigned_resources,    // Referência
                        "InProgress",                // &'static str
                        "0",                         // &'static str - simplificado para consistência
                    ),
                    AnyTask::Completed(task) => (
                        &task.code,                  // Referência
                        &task.name,                  // Referência
                        task.start_date,             // Copy é OK
                        task.due_date,               // Copy é OK
                        &task.assigned_resources,    // Referência
                        "Completed",                 // &'static str
                        "100",                       // &'static str
                    ),
                    AnyTask::Blocked(task) => (
                        &task.code,                  // Referência
                        &task.name,                  // Referência
                        task.start_date,             // Copy é OK
                        task.due_date,               // Copy é OK
                        &task.assigned_resources,    // Referência
                        "Blocked",                   // &'static str
                        "N/A",                       // &'static str
                    ),
                    AnyTask::Cancelled(task) => (
                        &task.code,                  // Referência
                        &task.name,                  // Referência
                        task.start_date,             // Copy é OK
                        task.due_date,               // Copy é OK
                        &task.assigned_resources,    // Referência
                        "Cancelled",                 // &'static str
                        "N/A",                       // &'static str
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
        shared::errors::DomainError,
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
        };

        let mut project: AnyProject = ProjectBuilder::new("Test Project".to_string())
            .code("PROJ-1".to_string())
            .build()
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
}
