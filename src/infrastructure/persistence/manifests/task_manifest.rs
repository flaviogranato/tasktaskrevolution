use crate::domain::shared::convertable::Convertable;
use crate::domain::task_management::{Task, TaskStatus};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
    pub project_code: String,
    pub assignee: String,
    status: Status,
    priority: Priority,
    pub estimated_start_date: Option<NaiveDate>,
    pub estimated_end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    effort: Effort,
    pub acceptance_criteria: Vec<String>,
    comments: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    ToDo,
    InProgress,
    Done,
    Blocked,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Effort {
    estimated_hours: f32,
    actual_hours: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct Comment {
    author: String,
    message: String,
    timestamp: NaiveDate,
}

impl Convertable<Task> for TaskManifest {
    fn from(source: Task) -> Self {
        let mut comments = Vec::new();

        let status = match source.status {
            TaskStatus::Completed => Status::Done,
            TaskStatus::Planned => Status::ToDo,
            TaskStatus::InProgress { progress } => {
                comments.push(Comment {
                    author: "system".to_string(),
                    message: format!("Progresso atual: {progress}%"),
                    timestamp: chrono::Utc::now().naive_utc().date(),
                });
                Status::InProgress
            }
            TaskStatus::Blocked { ref reason } => {
                comments.push(Comment {
                    author: "system".to_string(),
                    message: format!("Tarefa bloqueada: {reason}"),
                    timestamp: chrono::Utc::now().naive_utc().date(),
                });
                Status::Blocked
            }
            TaskStatus::Cancelled => Status::Cancelled,
        };

        TaskManifest {
            api_version: "v1".to_string(),
            kind: "Task".to_string(),
            metadata: Metadata {
                code: source.code,
                name: source.name,
                description: source.description,
            },
            spec: Spec {
                project_code: source.id.clone(),
                assignee: source
                    .assigned_resources
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unassigned".to_string()),
                status,
                priority: Priority::Medium,
                estimated_start_date: Some(source.start_date),
                estimated_end_date: Some(source.due_date),
                actual_start_date: Some(source.start_date),
                actual_end_date: source.actual_end_date,
                dependencies: Vec::new(),
                tags: source.assigned_resources.clone(),
                effort: Effort {
                    estimated_hours: 8.0,
                    actual_hours: if matches!(source.status, TaskStatus::Completed) {
                        Some(8.0)
                    } else {
                        None
                    },
                },
                acceptance_criteria: Vec::new(),
                comments,
            },
        }
    }

    fn to(&self) -> Task {
        let status = match self.spec.status {
            Status::ToDo => TaskStatus::Planned,
            Status::InProgress => {
                let progress = self
                    .spec
                    .comments
                    .iter()
                    .find(|c| c.message.starts_with("Progresso atual:"))
                    .and_then(|c| {
                        let progress_str = c
                            .message
                            .strip_prefix("Progresso atual: ")
                            .and_then(|s| s.strip_suffix("%"))?;
                        progress_str.parse::<u8>().ok()
                    })
                    .unwrap_or(50);
                TaskStatus::InProgress { progress }
            }
            Status::Done => TaskStatus::Completed,
            Status::Blocked => {
                let reason = self
                    .spec
                    .comments
                    .iter()
                    .find(|c| c.message.starts_with("Tarefa bloqueada:"))
                    .and_then(|c| c.message.strip_prefix("Tarefa bloqueada: "))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Motivo não especificado".to_string());
                TaskStatus::Blocked { reason }
            }
            Status::Cancelled => TaskStatus::Cancelled,
        };

        let mut assigned_resources = Vec::new();

        if self.spec.assignee != "unassigned" {
            assigned_resources.push(self.spec.assignee.clone());
        }

        for tag in &self.spec.tags {
            if tag != "unassigned" && !assigned_resources.contains(tag) {
                assigned_resources.push(tag.clone());
            }
        }

        if assigned_resources.is_empty() && !self.spec.tags.is_empty() {
            assigned_resources = self.spec.tags.clone();
        }

        Task {
            id: self.spec.project_code.clone(),
            code: self.metadata.code.clone(),
            name: self.metadata.name.clone(),
            description: self.metadata.description.clone(),
            status,
            start_date: self
                .spec
                .estimated_start_date
                .or(self.spec.actual_start_date)
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
            due_date: self
                .spec
                .estimated_end_date
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
            actual_end_date: self.spec.actual_end_date,
            assigned_resources,
        }
    }
}

#[cfg(test)]
mod convertable_tests {
    use super::*;
    use crate::domain::shared::convertable::Convertable;
    use crate::domain::task_management::{Task, TaskStatus};
    use chrono::NaiveDate;

    // Helper function para criar uma data de teste
    fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    // Helper function para criar uma Task básica
    fn create_basic_task() -> Task {
        Task {
            id: "TASK-001".to_string(),
            code: "TSK001".to_string(),
            name: "Tarefa de Teste".to_string(),
            description: Some("Descrição da tarefa de teste".to_string()),
            status: TaskStatus::Planned,
            start_date: test_date(2024, 1, 15),
            due_date: test_date(2024, 1, 30),
            actual_end_date: None,
            assigned_resources: vec!["dev1".to_string(), "dev2".to_string()],
        }
    }

    // Helper function para criar um TaskManifest básico
    fn create_basic_manifest() -> TaskManifest {
        TaskManifest {
            api_version: "v1".to_string(),
            kind: "Task".to_string(),
            metadata: Metadata {
                code: "TSK001".to_string(),
                name: "Tarefa de Teste".to_string(),
                description: Some("Descrição da tarefa de teste".to_string()),
            },
            spec: Spec {
                project_code: "TASK-001".to_string(),
                assignee: "dev1".to_string(),
                status: Status::ToDo,
                priority: Priority::Medium,
                estimated_start_date: Some(test_date(2024, 1, 15)),
                estimated_end_date: Some(test_date(2024, 1, 30)),
                actual_start_date: Some(test_date(2024, 1, 15)),
                actual_end_date: None,
                dependencies: vec![],
                tags: vec!["dev1".to_string(), "dev2".to_string()],
                effort: Effort {
                    estimated_hours: 8.0,
                    actual_hours: None,
                },
                acceptance_criteria: vec![],
                comments: vec![],
            },
        }
    }

    // =============================================================================
    // TESTES DE CONVERSÃO TASK → TASKMANIFEST
    // =============================================================================

    #[test]
    fn test_task_to_manifest_planned_status() {
        let task = create_basic_task();
        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());

        assert_eq!(manifest.api_version, "v1");
        assert_eq!(manifest.kind, "Task");
        assert_eq!(manifest.metadata.code, task.code);
        assert_eq!(manifest.metadata.name, task.name);
        assert_eq!(manifest.metadata.description, task.description);
        assert_eq!(manifest.spec.project_code, task.id);
        assert_eq!(manifest.spec.assignee, "dev1");
        assert_eq!(manifest.spec.status, Status::ToDo);
        assert_eq!(manifest.spec.priority, Priority::Medium);
        assert_eq!(manifest.spec.estimated_start_date, Some(task.start_date));
        assert_eq!(manifest.spec.estimated_end_date, Some(task.due_date));
        assert_eq!(manifest.spec.actual_end_date, task.actual_end_date);
        assert_eq!(manifest.spec.tags, task.assigned_resources);
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_in_progress_status() {
        let mut task = create_basic_task();
        task.status = TaskStatus::InProgress { progress: 75 };

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.spec.status, Status::InProgress);
        assert_eq!(manifest.spec.comments.len(), 1);
        assert_eq!(manifest.spec.comments[0].author, "system");
        assert_eq!(manifest.spec.comments[0].message, "Progresso atual: 75%");
    }

    #[test]
    fn test_task_to_manifest_completed_status() {
        let mut task = create_basic_task();
        task.status = TaskStatus::Completed;
        task.actual_end_date = Some(test_date(2024, 1, 28));

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.spec.status, Status::Done);
        assert_eq!(manifest.spec.effort.actual_hours, Some(8.0));
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_blocked_status() {
        let mut task = create_basic_task();
        task.status = TaskStatus::Blocked {
            reason: "Aguardando aprovação do cliente".to_string(),
        };

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.spec.status, Status::Blocked);
        assert_eq!(manifest.spec.comments.len(), 1);
        assert_eq!(manifest.spec.comments[0].author, "system");
        assert_eq!(
            manifest.spec.comments[0].message,
            "Tarefa bloqueada: Aguardando aprovação do cliente"
        );
    }

    #[test]
    fn test_task_to_manifest_cancelled_status() {
        let mut task = create_basic_task();
        task.status = TaskStatus::Cancelled;

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.spec.status, Status::Cancelled);
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_no_assigned_resources() {
        let mut task = create_basic_task();
        task.assigned_resources = vec![];

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.spec.assignee, "unassigned");
        assert!(manifest.spec.tags.is_empty());
    }

    #[test]
    fn test_task_to_manifest_no_description() {
        let mut task = create_basic_task();
        task.description = None;

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        assert_eq!(manifest.metadata.description, None);
    }

    #[test]
    fn test_task_to_manifest_single_assigned_resource() {
        let mut task = create_basic_task();
        task.assigned_resources = vec!["single_dev".to_string()];

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());

        assert_eq!(manifest.spec.assignee, "single_dev");
        assert_eq!(manifest.spec.tags, task.assigned_resources);
    }

    // =============================================================================
    // TESTES DE CONVERSÃO TASKMANIFEST → TASK
    // =============================================================================

    #[test]
    fn test_manifest_to_task_todo_status() {
        let manifest = create_basic_manifest();
        let task = manifest.to();

        assert_eq!(task.id, "TASK-001");
        assert_eq!(task.code, "TSK001");
        assert_eq!(task.name, "Tarefa de Teste");
        assert_eq!(task.description, Some("Descrição da tarefa de teste".to_string()));
        assert_eq!(task.status, TaskStatus::Planned);
        assert_eq!(task.start_date, test_date(2024, 1, 15));
        assert_eq!(task.due_date, test_date(2024, 1, 30));
        assert_eq!(task.actual_end_date, None);
        assert_eq!(task.assigned_resources, vec!["dev1", "dev2"]);
    }

    #[test]
    fn test_manifest_to_task_in_progress_status() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::InProgress;
        manifest.spec.comments.push(Comment {
            author: "system".to_string(),
            message: "Progresso atual: 80%".to_string(),
            timestamp: test_date(2024, 1, 20),
        });

        let task = manifest.to();

        match task.status {
            TaskStatus::InProgress { progress } => assert_eq!(progress, 80),
            _ => panic!("Expected InProgress status"),
        }
    }

    #[test]
    fn test_manifest_to_task_in_progress_without_progress_comment() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::InProgress;

        let task = manifest.to();

        match task.status {
            TaskStatus::InProgress { progress } => assert_eq!(progress, 50), // valor padrão
            _ => panic!("Expected InProgress status"),
        }
    }

    #[test]
    fn test_manifest_to_task_completed_status() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::Done;
        manifest.spec.actual_end_date = Some(test_date(2024, 1, 28));

        let task = manifest.to();

        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.actual_end_date, Some(test_date(2024, 1, 28)));
    }

    #[test]
    fn test_manifest_to_task_blocked_status() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::Blocked;
        manifest.spec.comments.push(Comment {
            author: "system".to_string(),
            message: "Tarefa bloqueada: Falta de recursos".to_string(),
            timestamp: test_date(2024, 1, 20),
        });

        let task = manifest.to();

        match task.status {
            TaskStatus::Blocked { reason } => assert_eq!(reason, "Falta de recursos"),
            _ => panic!("Expected Blocked status"),
        }
    }

    #[test]
    fn test_manifest_to_task_blocked_without_reason_comment() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::Blocked;

        let task = manifest.to();

        match task.status {
            TaskStatus::Blocked { reason } => assert_eq!(reason, "Motivo não especificado"),
            _ => panic!("Expected Blocked status"),
        }
    }

    #[test]
    fn test_manifest_to_task_cancelled_status() {
        let mut manifest = create_basic_manifest();
        manifest.spec.status = Status::Cancelled;

        let task = manifest.to();

        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_manifest_to_task_unassigned() {
        let mut manifest = create_basic_manifest();
        manifest.spec.assignee = "unassigned".to_string();
        manifest.spec.tags = vec![];

        let task = manifest.to();

        assert!(task.assigned_resources.is_empty());
    }

    #[test]
    fn test_manifest_to_task_no_description() {
        let mut manifest = create_basic_manifest();
        manifest.metadata.description = None;

        let task = manifest.to();

        assert_eq!(task.description, None);
    }

    #[test]
    fn test_manifest_to_task_missing_dates() {
        let mut manifest = create_basic_manifest();
        manifest.spec.estimated_start_date = None;
        manifest.spec.actual_start_date = None;
        manifest.spec.estimated_end_date = None;

        let task = manifest.to();

        // Deve usar data atual como fallback
        let today = chrono::Utc::now().naive_utc().date();
        assert_eq!(task.start_date, today);
        assert_eq!(task.due_date, today);
    }

    #[test]
    fn test_manifest_to_task_tags_as_resources() {
        let mut manifest = create_basic_manifest();
        manifest.spec.assignee = "lead_dev".to_string();
        manifest.spec.tags = vec!["dev1".to_string(), "dev2".to_string(), "lead_dev".to_string()];

        let task = manifest.to();

        // Deve incluir assignee e tags únicos
        assert_eq!(task.assigned_resources.len(), 3);
        assert!(task.assigned_resources.contains(&"lead_dev".to_string()));
        assert!(task.assigned_resources.contains(&"dev1".to_string()));
        assert!(task.assigned_resources.contains(&"dev2".to_string()));
    }

    #[test]
    fn test_manifest_to_task_duplicate_resources() {
        let mut manifest = create_basic_manifest();
        manifest.spec.assignee = "dev1".to_string();
        manifest.spec.tags = vec!["dev1".to_string(), "dev2".to_string()];

        let task = manifest.to();

        // Não deve duplicar recursos
        assert_eq!(task.assigned_resources.len(), 2);
        assert!(task.assigned_resources.contains(&"dev1".to_string()));
        assert!(task.assigned_resources.contains(&"dev2".to_string()));
    }

    // =============================================================================
    // TESTES DE CONVERSÃO BIDIRECIONAL
    // =============================================================================

    #[test]
    fn test_bidirectional_conversion_planned_task() {
        let original_task = create_basic_task();
        let manifest = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let converted_task = manifest.to();

        // Campos que devem ser preservados exatamente
        assert_eq!(converted_task.id, original_task.id);
        assert_eq!(converted_task.code, original_task.code);
        assert_eq!(converted_task.name, original_task.name);
        assert_eq!(converted_task.description, original_task.description);
        assert_eq!(converted_task.status, original_task.status);
        assert_eq!(converted_task.start_date, original_task.start_date);
        assert_eq!(converted_task.due_date, original_task.due_date);
        assert_eq!(converted_task.actual_end_date, original_task.actual_end_date);
        assert_eq!(converted_task.assigned_resources, original_task.assigned_resources);
    }

    #[test]
    fn test_bidirectional_conversion_in_progress_task() {
        let mut original_task = create_basic_task();
        original_task.status = TaskStatus::InProgress { progress: 65 };

        let manifest = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let converted_task = manifest.to();

        // Verifica se o progresso foi preservado
        match (original_task.status, converted_task.status) {
            (TaskStatus::InProgress { progress: orig }, TaskStatus::InProgress { progress: conv }) => {
                assert_eq!(orig, conv);
            }
            _ => panic!("Status should be InProgress for both"),
        }

        // Outros campos devem ser preservados
        assert_eq!(converted_task.id, original_task.id);
        assert_eq!(converted_task.code, original_task.code);
        assert_eq!(converted_task.name, original_task.name);
    }

    #[test]
    fn test_bidirectional_conversion_blocked_task() {
        let mut original_task = create_basic_task();
        original_task.status = TaskStatus::Blocked {
            reason: "Aguardando revisão de código".to_string(),
        };

        let manifest = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let converted_task = manifest.to();

        // Verifica se a razão do bloqueio foi preservada
        match (original_task.status, converted_task.status) {
            (TaskStatus::Blocked { reason: orig }, TaskStatus::Blocked { reason: conv }) => {
                assert_eq!(orig, conv);
            }
            _ => panic!("Status should be Blocked for both"),
        }
    }

    #[test]
    fn test_bidirectional_conversion_completed_task() {
        let mut original_task = create_basic_task();
        original_task.status = TaskStatus::Completed;
        original_task.actual_end_date = Some(test_date(2024, 1, 25));

        let manifest = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.status, TaskStatus::Completed);
        assert_eq!(converted_task.actual_end_date, original_task.actual_end_date);
    }

    #[test]
    fn test_bidirectional_conversion_cancelled_task() {
        let mut original_task = create_basic_task();
        original_task.status = TaskStatus::Cancelled;

        let manifest = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.status, TaskStatus::Cancelled);
        assert_eq!(converted_task.id, original_task.id);
        assert_eq!(converted_task.code, original_task.code);
    }

    #[test]
    fn test_multiple_bidirectional_conversions() {
        let original_task = create_basic_task();

        // Primeira conversão
        let manifest1 = <TaskManifest as Convertable<Task>>::from(original_task.clone());
        let task1 = manifest1.to();

        // Segunda conversão
        let manifest2 = <TaskManifest as Convertable<Task>>::from(task1.clone());
        let task2 = manifest2.to();

        // Terceira conversão
        let manifest3 = <TaskManifest as Convertable<Task>>::from(task2.clone());
        let task3 = manifest3.to();

        // Dados essenciais devem permanecer estáveis após múltiplas conversões
        assert_eq!(task3.id, original_task.id);
        assert_eq!(task3.code, original_task.code);
        assert_eq!(task3.name, original_task.name);
        assert_eq!(task3.description, original_task.description);
        assert_eq!(task3.status, original_task.status);
        assert_eq!(task3.assigned_resources, original_task.assigned_resources);
    }

    // =============================================================================
    // TESTES DE CASOS EXTREMOS E VALIDAÇÃO
    // =============================================================================

    #[test]
    fn test_task_with_empty_strings() {
        let mut task = create_basic_task();
        task.code = "".to_string();
        task.name = "".to_string();

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.code, "");
        assert_eq!(converted_task.name, "");
        assert_eq!(converted_task.id, task.id);
    }

    #[test]
    fn test_task_with_special_characters() {
        let mut task = create_basic_task();
        task.name = "Tarefa com çãrãctéres especiais & símbolos!".to_string();
        task.description = Some("Descrição com 'aspas' e \"aspas duplas\"".to_string());

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.name, task.name);
        assert_eq!(converted_task.description, task.description);
    }

    #[test]
    fn test_progress_edge_cases() {
        // Teste com progresso 0%
        let mut task = create_basic_task();
        task.status = TaskStatus::InProgress { progress: 0 };

        let manifest = <TaskManifest as Convertable<Task>>::from(task);
        let converted_task = manifest.to();

        match converted_task.status {
            TaskStatus::InProgress { progress } => assert_eq!(progress, 0),
            _ => panic!("Expected InProgress status"),
        }

        // Teste com progresso 100%
        let mut task2 = create_basic_task();
        task2.status = TaskStatus::InProgress { progress: 100 };

        let manifest2 = <TaskManifest as Convertable<Task>>::from(task2);
        let converted_task2 = manifest2.to();

        match converted_task2.status {
            TaskStatus::InProgress { progress } => assert_eq!(progress, 100),
            _ => panic!("Expected InProgress status"),
        }
    }

    #[test]
    fn test_long_resource_list() {
        let mut task = create_basic_task();
        task.assigned_resources = (1..=20).map(|i| format!("dev{i}")).collect();

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.assigned_resources.len(), 20);
        assert_eq!(converted_task.assigned_resources, task.assigned_resources);
    }

    #[test]
    fn test_date_edge_cases() {
        let mut task = create_basic_task();
        // Teste com datas muito antigas
        task.start_date = test_date(1900, 1, 1);
        task.due_date = test_date(1900, 12, 31);

        let manifest = <TaskManifest as Convertable<Task>>::from(task.clone());
        let converted_task = manifest.to();

        assert_eq!(converted_task.start_date, task.start_date);
        assert_eq!(converted_task.due_date, task.due_date);
    }
}
