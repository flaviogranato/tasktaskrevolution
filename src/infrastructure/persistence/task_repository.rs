use crate::domain::{
    shared::{convertable::Convertable, errors::DomainError},
    task_management::{Task, TaskStatus, repository::TaskRepository},
};
use crate::infrastructure::persistence::manifests::task_manifest::TaskManifest;
use globwalk::glob;
use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileTaskRepository {
    base_path: PathBuf,
}

#[allow(dead_code)]
impl FileTaskRepository {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("."),
        }
    }

    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn get_task_file_path(&self, task_code: &str) -> PathBuf {
        self.base_path.join("tasks").join(format!("{task_code}.yaml"))
    }

    fn get_tasks_directory(&self) -> PathBuf {
        self.base_path.join("tasks")
    }

    fn load_manifest(&self, path: &Path) -> Result<TaskManifest, DomainError> {
        let yaml =
            fs::read_to_string(path).map_err(|e| DomainError::Generic(format!("Erro ao ler arquivo de task: {e}")))?;

        serde_yaml::from_str(&yaml).map_err(|e| DomainError::Generic(format!("Erro ao deserializar task: {e}")))
    }

    fn save_manifest(&self, task_manifest: &TaskManifest) -> Result<(), DomainError> {
        let file_path = self.get_task_file_path(&task_manifest.metadata.code);

        let yaml = serde_yaml::to_string(task_manifest)
            .map_err(|e| DomainError::Generic(format!("Erro ao serializar task: {e}")))?;

        // Criar diretório tasks se não existir
        fs::create_dir_all(file_path.parent().unwrap())
            .map_err(|e| DomainError::Generic(format!("Erro ao criar diretório tasks: {e}")))?;

        fs::write(file_path, yaml).map_err(|e| DomainError::Generic(format!("Erro ao salvar task: {e}")))?;

        Ok(())
    }

    // ========================================================================
    // FUNCIONALIDADES ESPECÍFICAS PARA GERENCIAMENTO DE TASKS
    // ========================================================================

    /// Encontra todas as tasks de um projeto específico
    pub fn find_by_project(&self, project_code: &str) -> Result<Vec<Task>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| task.id.starts_with(project_code))
            .collect())
    }

    /// Atualiza os recursos atribuídos a uma task
    pub fn update_assigned_resources(&self, code: &str, resources: Vec<String>) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        task.assigned_resources = resources;
        self.save(task.clone())?;
        Ok(task)
    }

    /// Adiciona um recurso à lista de recursos atribuídos
    pub fn add_assignee(&self, code: &str, assignee: String) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        if !task.assigned_resources.contains(&assignee) {
            task.assigned_resources.push(assignee);
            self.save(task.clone())?;
        }
        Ok(task)
    }

    /// Remove um recurso da lista de recursos atribuídos
    pub fn remove_assignee(&self, code: &str, assignee: &str) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        task.assigned_resources.retain(|r| r != assignee);
        self.save(task.clone())?;
        Ok(task)
    }

    /// Atualiza o progresso de uma task em andamento
    pub fn update_progress(&self, code: &str, progress: u8) -> Result<Task, DomainError> {
        if progress > 100 {
            return Err(DomainError::Generic(
                "Progresso não pode ser maior que 100%".to_string(),
            ));
        }

        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        // Atualizar status para InProgress com o novo progresso
        task.status = TaskStatus::InProgress { progress };

        // Se progresso é 100%, marcar como completada
        if progress == 100 {
            task.status = TaskStatus::Completed;
            if task.actual_end_date.is_none() {
                task.actual_end_date = Some(chrono::Utc::now().naive_utc().date());
            }
        }

        self.save(task.clone())?;
        Ok(task)
    }

    /// Bloqueia uma task com uma razão específica
    pub fn block_task(&self, code: &str, reason: String) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        task.status = TaskStatus::Blocked { reason };
        self.save(task.clone())?;
        Ok(task)
    }

    /// Desbloqueia uma task, retornando ao status anterior ou Planned
    pub fn unblock_task(&self, code: &str) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        // Se estava bloqueada, voltar para Planned
        if matches!(task.status, TaskStatus::Blocked { .. }) {
            task.status = TaskStatus::Planned;
            self.save(task.clone())?;
        }
        Ok(task)
    }

    /// Encontra tasks que estão atrasadas (due_date passou e não estão completas)
    pub fn find_overdue_tasks(&self) -> Result<Vec<Task>, DomainError> {
        let today = chrono::Utc::now().naive_utc().date();
        let all_tasks = self.find_all()?;

        Ok(all_tasks
            .into_iter()
            .filter(|task| {
                task.due_date < today && !matches!(task.status, TaskStatus::Completed | TaskStatus::Cancelled)
            })
            .collect())
    }

    /// Encontra tasks que vencem em um número específico de dias
    pub fn find_tasks_due_in_days(&self, days: i64) -> Result<Vec<Task>, DomainError> {
        let target_date = chrono::Utc::now().naive_utc().date() + chrono::Duration::days(days);
        let all_tasks = self.find_all()?;

        Ok(all_tasks
            .into_iter()
            .filter(|task| {
                task.due_date == target_date && !matches!(task.status, TaskStatus::Completed | TaskStatus::Cancelled)
            })
            .collect())
    }

    /// Atualiza as datas de uma task
    pub fn update_dates(
        &self,
        code: &str,
        start_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        if let Some(start) = start_date {
            task.start_date = start;
        }

        if let Some(due) = due_date {
            task.due_date = due;
        }

        // Validar que start_date <= due_date
        if task.start_date > task.due_date {
            return Err(DomainError::Generic(
                "Data de início não pode ser posterior à data de vencimento".to_string(),
            ));
        }

        self.save(task.clone())?;
        Ok(task)
    }

    /// Clona uma task existente com um novo código
    pub fn clone_task(
        &self,
        original_code: &str,
        new_code: &str,
        new_name: Option<String>,
    ) -> Result<Task, DomainError> {
        let original_task = self.find_by_code(original_code)?.ok_or_else(|| {
            DomainError::Generic(format!("Task original com código '{original_code}' não encontrada"))
        })?;

        // Verificar se o novo código já existe
        if self.find_by_code(new_code)?.is_some() {
            return Err(DomainError::Generic(format!("Task com código '{new_code}' já existe")));
        }

        let mut new_task = original_task.clone();
        new_task.code = new_code.to_string();
        new_task.id = format!("TASK-{new_code}");
        new_task.name = new_name.unwrap_or_else(|| format!("{} (Cópia)", original_task.name));
        new_task.status = TaskStatus::Planned; // Nova task sempre começa como Planned
        new_task.actual_end_date = None; // Limpar data de conclusão

        self.save(new_task.clone())?;
        Ok(new_task)
    }

    pub fn get_task_statistics(&self) -> Result<TaskStatistics, DomainError> {
        let all_tasks = self.find_all()?;
        let current_date = chrono::Local::now().date_naive();

        // ✅ Substituir Default::default() + atribuições por:
        let stats = TaskStatistics {
            total: all_tasks.len(),
            planned: all_tasks
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Planned))
                .count(),
            in_progress: all_tasks
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::InProgress { .. }))
                .count(),
            completed: all_tasks
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Completed))
                .count(),
            blocked: all_tasks
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Blocked { .. }))
                .count(),
            cancelled: all_tasks
                .iter()
                .filter(|t| matches!(t.status, TaskStatus::Cancelled))
                .count(),
            overdue: all_tasks
                .iter()
                .filter(|t| t.due_date < current_date && !matches!(t.status, TaskStatus::Completed))
                .count(),
        };

        Ok(stats)
    }
    /// Exporta todas as tasks para um formato de relatório
    pub fn export_tasks_report(&self) -> Result<String, DomainError> {
        let all_tasks = self.find_all()?;
        let mut report = String::new();

        report.push_str("# Relatório de Tasks\n\n");
        report.push_str(&format!("Total de tasks: {}\n\n", all_tasks.len()));

        for task in all_tasks {
            report.push_str(&format!("## {} - {}\n", task.code, task.name));
            report.push_str(&format!("- **Status:** {:?}\n", task.status));
            report.push_str(&format!("- **Início:** {}\n", task.start_date));
            report.push_str(&format!("- **Vencimento:** {}\n", task.due_date));

            if !task.assigned_resources.is_empty() {
                report.push_str(&format!("- **Recursos:** {}\n", task.assigned_resources.join(", ")));
            }

            if let Some(description) = &task.description {
                report.push_str(&format!("- **Descrição:** {description}\n"));
            }

            report.push('\n');
        }

        Ok(report)
    }
}

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct TaskStatistics {
    pub total: usize,
    pub planned: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub blocked: usize,
    pub cancelled: usize,
    pub overdue: usize,
}

#[allow(dead_code)]
impl TaskStatistics {
    pub fn completion_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }

    pub fn overdue_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.overdue as f64 / self.total as f64) * 100.0
        }
    }
}

// ========================================================================
// IMPLEMENTAÇÃO DO TRAIT TASKREPOSITORY
// ========================================================================

impl TaskRepository for FileTaskRepository {
    fn save(&self, task: Task) -> Result<(), DomainError> {
        let task_manifest = <TaskManifest as Convertable<Task>>::from(task);
        self.save_manifest(&task_manifest)
    }

    fn load(&self, path: &Path) -> Result<Task, DomainError> {
        let manifest = self.load_manifest(path)?;
        Ok(<TaskManifest as Convertable<Task>>::to(&manifest))
    }

    fn find_by_code(&self, code: &str) -> Result<Option<Task>, DomainError> {
        let file_path = self.get_task_file_path(code);

        if !file_path.exists() {
            return Ok(None);
        }

        match self.load_manifest(&file_path) {
            Ok(manifest) => Ok(Some(<TaskManifest as Convertable<Task>>::to(&manifest))),
            Err(_) => Ok(None),
        }
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Task>, DomainError> {
        // Para tasks, vamos usar o código como identificador principal
        // mas também buscar por ID no conteúdo dos arquivos
        let tasks = self.find_all()?;
        Ok(tasks.into_iter().find(|task| task.id == id))
    }

    fn find_all(&self) -> Result<Vec<Task>, DomainError> {
        let pattern = self.base_path.join("**/tasks/**/*.yaml");
        let walker = glob(pattern.to_str().unwrap()).map_err(|e| DomainError::Generic(e.to_string()))?;

        let mut tasks = Vec::new();
        for entry in walker {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                match self.load_manifest(&file_path) {
                    Ok(manifest) => {
                        tasks.push(<TaskManifest as Convertable<Task>>::to(&manifest));
                    }
                    Err(e) => {
                        // Log do erro mas continua processando outros arquivos
                        eprintln!("Erro ao carregar task de {:?}: {}", file_path, e);
                    }
                }
            }
        }

        Ok(tasks)
    }

    fn delete(&self, id: &str) -> Result<(), DomainError> {
        let task_to_delete = self
            .find_by_id(id)?
            .ok_or_else(|| DomainError::Generic(format!("Task com id '{id}' não encontrada")))?;

        let file_path = self.get_task_file_path(&task_to_delete.code);

        fs::remove_file(file_path).map_err(|e| DomainError::Generic(format!("Erro ao deletar o arquivo da task: {e}")))
    }

    fn update_status(&self, code: &str, new_status: TaskStatus) -> Result<Task, DomainError> {
        let mut task = self
            .find_by_code(code)?
            .ok_or_else(|| DomainError::Generic(format!("Task com código '{code}' não encontrada")))?;

        task.status = new_status;

        // Se a task foi completada, definir data de conclusão
        if matches!(task.status, TaskStatus::Completed) && task.actual_end_date.is_none() {
            task.actual_end_date = Some(chrono::Utc::now().naive_utc().date());
        }

        self.save(task.clone())?;
        Ok(task)
    }

    fn find_by_assignee(&self, assignee: &str) -> Result<Vec<Task>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| task.assigned_resources.contains(&assignee.to_string()))
            .collect())
    }

    fn find_by_status(&self, status: &TaskStatus) -> Result<Vec<Task>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| std::mem::discriminant(&task.status) == std::mem::discriminant(status))
            .collect())
    }

    fn find_by_date_range(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<Vec<Task>, DomainError> {
        let all_tasks = self.find_all()?;
        Ok(all_tasks
            .into_iter()
            .filter(|task| task.start_date >= start_date && task.due_date <= end_date)
            .collect())
    }
}

impl Default for FileTaskRepository {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================================================
// TESTES
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_management::{Task, TaskStatus};
    use chrono::NaiveDate;
    use tempfile::tempdir;

    fn create_test_task(code: &str, name: &str) -> Task {
        Task {
            id: format!("TASK-{code}"),
            code: code.to_string(),
            name: name.to_string(),
            description: Some(format!("Descrição da task {name}")),
            status: TaskStatus::Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
            actual_end_date: None,
            assigned_resources: vec!["dev1".to_string()],
        }
    }

    #[test]
    fn test_save_and_find_by_code() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let task = create_test_task("TSK001", "Test Task 1");
        repo.save(task.clone()).unwrap();

        let found_task = repo.find_by_code("TSK001").unwrap();
        assert!(found_task.is_some());
        assert_eq!(found_task.unwrap().name, "Test Task 1");
    }

    #[test]
    fn test_find_all() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let task1 = create_test_task("TSK001", "Test Task 1");
        let task2 = create_test_task("TSK002", "Test Task 2");

        repo.save(task1).unwrap();
        repo.save(task2).unwrap();

        let tasks = repo.find_all().unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.iter().any(|t| t.code == "TSK001"));
        assert!(tasks.iter().any(|t| t.code == "TSK002"));
    }

    #[test]
    fn test_update_progress() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let task = create_test_task("TSK001", "Test Task 1");
        repo.save(task).unwrap();

        // Atualizar progresso para 75%
        let updated_task = repo.update_progress("TSK001", 75).unwrap();
        assert!(matches!(updated_task.status, TaskStatus::InProgress { progress: 75 }));

        // Atualizar progresso para 100% deve completar a task
        let completed_task = repo.update_progress("TSK001", 100).unwrap();
        assert!(matches!(completed_task.status, TaskStatus::Completed));
        assert!(completed_task.actual_end_date.is_some());
    }

    #[test]
    fn test_find_by_project() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let mut task1 = create_test_task("TSK001", "Test Task 1");
        task1.id = "PROJ1-TSK001".to_string();

        let mut task2 = create_test_task("TSK002", "Test Task 2");
        task2.id = "PROJ2-TSK002".to_string();

        repo.save(task1).unwrap();
        repo.save(task2).unwrap();

        let proj1_tasks = repo.find_by_project("PROJ1").unwrap();
        assert_eq!(proj1_tasks.len(), 1);
        assert_eq!(proj1_tasks[0].code, "TSK001");
    }

    #[test]
    fn test_task_statistics() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let mut task1 = create_test_task("TSK001", "Planned Task");
        task1.status = TaskStatus::Planned;

        let mut task2 = create_test_task("TSK002", "In Progress Task");
        task2.status = TaskStatus::InProgress { progress: 50 };

        let mut task3 = create_test_task("TSK003", "Completed Task");
        task3.status = TaskStatus::Completed;

        repo.save(task1).unwrap();
        repo.save(task2).unwrap();
        repo.save(task3).unwrap();

        let stats = repo.get_task_statistics().unwrap();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.planned, 1);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.completed, 1);
        let expected_rate = 100.0 / 3.0; // 33.333...
        let actual_rate = stats.completion_rate();
        assert!(
            (actual_rate - expected_rate).abs() < 0.001,
            "Expected completion rate around {expected_rate:.3}, got {actual_rate:.3}"
        );
    }

    #[test]
    fn test_add_and_remove_assignee() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        // Criar task com um recurso inicial "dev1"
        let task = create_test_task("TSK001", "Test Task Assignee");
        repo.save(task).unwrap();

        // Adicionar "dev2"
        let updated_task = repo.add_assignee("TSK001", "dev2".to_string()).unwrap();
        assert_eq!(updated_task.assigned_resources.len(), 2);
        assert!(updated_task.assigned_resources.contains(&"dev1".to_string()));
        assert!(updated_task.assigned_resources.contains(&"dev2".to_string()));

        // Tentar adicionar "dev2" novamente (não deve duplicar)
        let same_task = repo.add_assignee("TSK001", "dev2".to_string()).unwrap();
        assert_eq!(same_task.assigned_resources.len(), 2);

        // Remover "dev1"
        let removed_task = repo.remove_assignee("TSK001", "dev1").unwrap();
        assert_eq!(removed_task.assigned_resources.len(), 1);
        assert!(!removed_task.assigned_resources.contains(&"dev1".to_string()));
        assert!(removed_task.assigned_resources.contains(&"dev2".to_string()));
    }

    #[test]
    fn test_block_and_unblock_task() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let task = create_test_task("TSK001", "Test Task Block");
        repo.save(task).unwrap();

        // Bloquear a task
        let reason = "Aguardando dependência externa".to_string();
        let blocked_task = repo.block_task("TSK001", reason.clone()).unwrap();
        assert!(matches!(
            blocked_task.status,
            TaskStatus::Blocked { reason: r } if r == reason
        ));

        // Desbloquear a task
        let unblocked_task = repo.unblock_task("TSK001").unwrap();
        assert!(matches!(unblocked_task.status, TaskStatus::Planned));
    }

    #[test]
    fn test_clone_task() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let original_task = create_test_task("TSK-ORIG", "Original Task");
        repo.save(original_task.clone()).unwrap();

        let new_code = "TSK-CLONE";
        let new_name = "Cloned Task".to_string();
        let cloned_task = repo.clone_task("TSK-ORIG", new_code, Some(new_name.clone())).unwrap();

        // Verificar a task clonada
        assert_eq!(cloned_task.code, new_code);
        assert_eq!(cloned_task.name, new_name);
        assert_eq!(cloned_task.id, format!("TASK-{new_code}"));
        assert!(matches!(cloned_task.status, TaskStatus::Planned));
        assert_eq!(cloned_task.assigned_resources, original_task.assigned_resources);
        assert!(cloned_task.actual_end_date.is_none());

        // Verificar se a nova task foi salva
        let found_cloned_task = repo.find_by_code(new_code).unwrap();
        assert!(found_cloned_task.is_some());
        assert_eq!(found_cloned_task.unwrap().name, new_name);

        // Verificar se a task original não foi alterada
        let found_original_task = repo.find_by_code("TSK-ORIG").unwrap().unwrap();
        assert_eq!(found_original_task.name, "Original Task");
        assert!(matches!(found_original_task.status, TaskStatus::Planned));
    }

    #[test]
    fn test_delete_task() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());

        let task1 = create_test_task("TSK-DEL", "Task to be Deleted");
        let task2 = create_test_task("TSK-KEEP", "Task to be Kept");

        repo.save(task1.clone()).unwrap();
        repo.save(task2.clone()).unwrap();

        // Deletar a primeira task
        repo.delete(&task1.id).unwrap();

        // Verificar se a task foi deletada
        let deleted_task = repo.find_by_code("TSK-DEL").unwrap();
        assert!(deleted_task.is_none());

        // Verificar se a outra task ainda existe
        let kept_task = repo.find_by_code("TSK-KEEP").unwrap();
        assert!(kept_task.is_some());

        let all_tasks = repo.find_all().unwrap();
        assert_eq!(all_tasks.len(), 1);
        assert_eq!(all_tasks[0].code, "TSK-KEEP");
    }

    #[test]
    fn test_date_queries() {
        let temp_dir = tempdir().unwrap();
        let repo = FileTaskRepository::with_base_path(temp_dir.path().to_path_buf());
        let today = chrono::Utc::now().naive_utc().date();

        // Tarefa atrasada
        let mut overdue_task = create_test_task("TSK-OVERDUE", "Overdue Task");
        overdue_task.due_date = today - chrono::Duration::days(1);
        repo.save(overdue_task).unwrap();

        // Tarefa não atrasada
        let mut future_task = create_test_task("TSK-FUTURE", "Future Task");
        future_task.due_date = today + chrono::Duration::days(10);
        repo.save(future_task).unwrap();

        // Verificar find_overdue_tasks
        let overdue_tasks = repo.find_overdue_tasks().unwrap();
        assert_eq!(overdue_tasks.len(), 1);
        assert_eq!(overdue_tasks[0].code, "TSK-OVERDUE");

        // Atualizar data da tarefa atrasada
        let new_due_date = today + chrono::Duration::days(20);
        repo.update_dates("TSK-OVERDUE", None, Some(new_due_date)).unwrap();

        // Verificar novamente as tarefas atrasadas
        let overdue_tasks_after_update = repo.find_overdue_tasks().unwrap();
        assert!(overdue_tasks_after_update.is_empty());

        // Verificar se a data foi realmente atualizada
        let updated_task = repo.find_by_code("TSK-OVERDUE").unwrap().unwrap();
        assert_eq!(updated_task.due_date, new_due_date);
    }
}
