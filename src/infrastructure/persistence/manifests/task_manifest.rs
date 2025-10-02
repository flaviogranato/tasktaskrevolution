use crate::domain::task_management::{AnyTask, Category as TaskCategory, Priority as TaskPriority, Task, state::*};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid7::{Uuid, uuid7};

// This is a private helper struct to unify the data from different Task<State> types.
#[allow(dead_code)]
struct TaskCore {
    id: Uuid,
    project_code: String,
    code: String,
    name: String,
    description: Option<String>,
    start_date: NaiveDate,
    due_date: NaiveDate,
    actual_end_date: Option<NaiveDate>,
    dependencies: Vec<String>,
    assigned_resources: Vec<String>,
    priority: TaskPriority,
    category: TaskCategory,
}

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: Spec,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Spec {
    pub project_code: String,
    pub assignee: String,
    pub status: Status,
    pub priority: Priority,
    pub estimated_start_date: Option<NaiveDate>,
    pub estimated_end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_end_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub tags: Vec<String>,
    pub effort: Effort,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub acceptance_criteria: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default = "Vec::new")]
    pub comments: Vec<Comment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Status {
    Planned,
    ToDo,
    InProgress,
    Done,
    Blocked,
    Cancelled,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Effort {
    estimated_hours: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual_hours: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Comment {
    author: String,
    message: String,
    timestamp: NaiveDate,
}

impl From<AnyTask> for TaskManifest {
    fn from(any_task: AnyTask) -> Self {
        let (task_core, manifest_status, comments) = match any_task {
            AnyTask::Planned(task) => (
                TaskCore {
                    id: task.id,
                    project_code: task.project_code,
                    code: task.code,
                    name: task.name,
                    description: task.description,
                    start_date: task.start_date,
                    due_date: task.due_date,
                    actual_end_date: task.actual_end_date,
                    dependencies: task.dependencies,
                    assigned_resources: task.assigned_resources,
                    priority: task.priority,
                    category: task.category,
                },
                Status::Planned,
                vec![],
            ),
            AnyTask::InProgress(task) => {
                let mut comments = Vec::new();
                if task.state.progress > 0 {
                    comments.push(Comment {
                        author: "system".to_string(),
                        message: format!("Progresso atual: {}%", task.state.progress),
                        timestamp: chrono::Utc::now().naive_utc().date(),
                    });
                }
                (
                    TaskCore {
                        id: task.id,
                        project_code: task.project_code,
                        code: task.code,
                        name: task.name,
                        description: task.description,
                        start_date: task.start_date,
                        due_date: task.due_date,
                        actual_end_date: task.actual_end_date,
                        dependencies: task.dependencies,
                        assigned_resources: task.assigned_resources,
                        priority: task.priority,
                        category: task.category,
                    },
                    Status::InProgress,
                    comments,
                )
            }
            AnyTask::Completed(task) => (
                TaskCore {
                    id: task.id,
                    project_code: task.project_code,
                    code: task.code,
                    name: task.name,
                    description: task.description,
                    start_date: task.start_date,
                    due_date: task.due_date,
                    actual_end_date: task.actual_end_date,
                    dependencies: task.dependencies,
                    assigned_resources: task.assigned_resources,
                    priority: task.priority,
                    category: task.category,
                },
                Status::Done,
                vec![],
            ),
            AnyTask::Blocked(task) => {
                let comments = vec![Comment {
                    author: "system".to_string(),
                    message: format!("Task blocked: {}", task.state.reason),
                    timestamp: chrono::Utc::now().naive_utc().date(),
                }];
                (
                    TaskCore {
                        id: task.id,
                        project_code: task.project_code,
                        code: task.code,
                        name: task.name,
                        description: task.description,
                        start_date: task.start_date,
                        due_date: task.due_date,
                        actual_end_date: task.actual_end_date,
                        dependencies: task.dependencies,
                        assigned_resources: task.assigned_resources,
                        priority: task.priority,
                        category: task.category,
                    },
                    Status::Blocked,
                    comments,
                )
            }
            AnyTask::Cancelled(task) => (
                TaskCore {
                    id: task.id,
                    project_code: task.project_code,
                    code: task.code,
                    name: task.name,
                    description: task.description,
                    start_date: task.start_date,
                    due_date: task.due_date,
                    actual_end_date: task.actual_end_date,
                    dependencies: task.dependencies,
                    assigned_resources: task.assigned_resources,
                    priority: task.priority,
                    category: task.category,
                },
                Status::Cancelled,
                vec![],
            ),
        };

        TaskManifest {
            api_version: API_VERSION.to_string(),
            kind: "Task".to_string(),
            metadata: Metadata {
                id: Some(task_core.id.to_string()),
                code: task_core.code,
                name: task_core.name,
                description: task_core.description,
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec: Spec {
                project_code: task_core.project_code,
                assignee: task_core
                    .assigned_resources
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "unassigned".to_string()),
                status: manifest_status.clone(),
                priority: Priority::Medium,
                estimated_start_date: Some(task_core.start_date),
                estimated_end_date: Some(task_core.due_date),
                actual_start_date: Some(task_core.start_date),
                actual_end_date: task_core.actual_end_date,
                dependencies: task_core.dependencies,
                tags: task_core.assigned_resources.clone(),
                effort: Effort {
                    estimated_hours: 8.0,
                    actual_hours: if matches!(manifest_status, Status::Done) {
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
}

impl TryFrom<TaskManifest> for AnyTask {
    type Error = String;

    fn try_from(manifest: TaskManifest) -> Result<Self, Self::Error> {
        let mut assigned_resources = manifest.spec.tags.clone();
        if manifest.spec.assignee != "unassigned" && !assigned_resources.contains(&manifest.spec.assignee) {
            assigned_resources.insert(0, manifest.spec.assignee.clone());
        }

        let id = manifest
            .metadata
            .id
            .map(|id_str| Uuid::from_str(&id_str))
            .transpose()
            .map_err(|e| e.to_string())?
            .unwrap_or_else(uuid7);

        let start_date = manifest
            .spec
            .estimated_start_date
            .or(manifest.spec.actual_start_date)
            .unwrap_or_else(|| chrono::Utc::now().naive_utc().date());

        let due_date = manifest
            .spec
            .estimated_end_date
            .unwrap_or_else(|| chrono::Utc::now().naive_utc().date());

        let task = match manifest.spec.status {
            Status::Planned | Status::ToDo => AnyTask::Planned(Task {
                id,
                project_code: manifest.spec.project_code,
                code: manifest.metadata.code,
                name: manifest.metadata.name,
                description: manifest.metadata.description,
                state: Planned,
                start_date,
                due_date,
                actual_end_date: manifest.spec.actual_end_date,
                dependencies: manifest.spec.dependencies,
                assigned_resources,
                priority: TaskPriority::default(),
                category: TaskCategory::default(),
            }),
            Status::InProgress => {
                let progress = manifest
                    .spec
                    .comments
                    .iter()
                    .find(|c| c.message.starts_with("Progresso atual:"))
                    .and_then(|c| {
                        c.message
                            .strip_prefix("Progresso atual: ")
                            .and_then(|s| s.strip_suffix('%'))
                            .and_then(|s| s.parse::<u8>().ok())
                    })
                    .unwrap_or(0);
                AnyTask::InProgress(Task {
                    id,
                    project_code: manifest.spec.project_code,
                    code: manifest.metadata.code,
                    name: manifest.metadata.name,
                    description: manifest.metadata.description,
                    state: InProgress { progress },
                    start_date,
                    due_date,
                    actual_end_date: manifest.spec.actual_end_date,
                    dependencies: manifest.spec.dependencies,
                    assigned_resources,
                    priority: TaskPriority::default(),
                    category: TaskCategory::default(),
                })
            }
            Status::Done => AnyTask::Completed(Task {
                id,
                project_code: manifest.spec.project_code,
                code: manifest.metadata.code,
                name: manifest.metadata.name,
                description: manifest.metadata.description,
                state: Completed,
                start_date,
                due_date,
                actual_end_date: manifest.spec.actual_end_date,
                dependencies: manifest.spec.dependencies,
                assigned_resources,
                priority: TaskPriority::default(),
                category: TaskCategory::default(),
            }),
            Status::Blocked => {
                let reason = manifest
                    .spec
                    .comments
                    .iter()
                    .find(|c| c.message.starts_with("Task blocked:"))
                    .and_then(|c| c.message.strip_prefix("Task blocked: "))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Reason not specified".to_string());
                AnyTask::Blocked(Task {
                    id,
                    project_code: manifest.spec.project_code,
                    code: manifest.metadata.code,
                    name: manifest.metadata.name,
                    description: manifest.metadata.description,
                    state: Blocked { reason },
                    start_date,
                    due_date,
                    actual_end_date: manifest.spec.actual_end_date,
                    dependencies: manifest.spec.dependencies,
                    assigned_resources,
                    priority: TaskPriority::default(),
                    category: TaskCategory::default(),
                })
            }
            Status::Cancelled => AnyTask::Cancelled(Task {
                id,
                project_code: manifest.spec.project_code,
                code: manifest.metadata.code,
                name: manifest.metadata.name,
                description: manifest.metadata.description,
                state: Cancelled,
                start_date,
                due_date,
                actual_end_date: manifest.spec.actual_end_date,
                dependencies: manifest.spec.dependencies,
                assigned_resources,
                priority: TaskPriority::default(),
                category: TaskCategory::default(),
            }),
        };

        Ok(task)
    }
}

#[cfg(test)]
mod convertable_tests {
    use super::*;
    use crate::domain::task_management::state::Planned;

    // Helper to create a date for tests
    fn test_date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    // Helper to create a basic task for tests
    fn create_basic_task() -> Task<Planned> {
        Task {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: "TSK-001".to_string(),
            name: "Test Task".to_string(),
            description: Some("A description".to_string()),
            state: Planned,
            start_date: test_date(2024, 1, 1),
            due_date: test_date(2024, 1, 10),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["res-1".to_string()],
            priority: TaskPriority::default(),
            category: TaskCategory::default(),
        }
    }

    // Helper to create a basic manifest for tests
    fn create_basic_manifest(status: Status) -> TaskManifest {
        TaskManifest {
            api_version: API_VERSION.to_string(),
            kind: "Task".to_string(),
            metadata: Metadata {
                id: Some(Uuid::from(1).to_string()),
                code: "TASK-1".to_string(),
                name: "Basic Task".to_string(),
                description: Some("A simple task for testing".to_string()),
                created_at: None,
                updated_at: None,
                created_by: None,
            },
            spec: Spec {
                project_code: "PROJ-1".to_string(),
                assignee: "res-1".to_string(),
                status,
                priority: Priority::Medium,
                estimated_start_date: Some(test_date(2024, 1, 1)),
                estimated_end_date: Some(test_date(2024, 1, 10)),
                actual_start_date: Some(test_date(2024, 1, 1)),
                actual_end_date: None,
                dependencies: vec![],
                tags: vec!["res-1".to_string()],
                effort: Effort {
                    estimated_hours: 8.0,
                    actual_hours: None,
                },
                acceptance_criteria: vec![],
                comments: vec![],
            },
        }
    }

    // --- Conversion Tests: Task -> Manifest ---

    #[test]
    fn test_task_to_manifest_planned_status() {
        let task = create_basic_task();
        let manifest = TaskManifest::from(AnyTask::Planned(task));

        assert_eq!(manifest.spec.status, Status::Planned);
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_in_progress_status() {
        let task = create_basic_task().start().update_progress(50);
        let manifest = TaskManifest::from(AnyTask::InProgress(task));

        assert_eq!(manifest.spec.status, Status::InProgress);
        assert_eq!(manifest.spec.comments[0].message, "Progresso atual: 50%");
    }

    #[test]
    fn test_task_to_manifest_completed_status() {
        let task = create_basic_task().start().complete();
        let manifest = TaskManifest::from(AnyTask::Completed(task));

        assert_eq!(manifest.spec.status, Status::Done);
        assert_eq!(manifest.spec.effort.actual_hours, Some(8.0));
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_blocked_status() {
        let reason = "Dependency issue".to_string();
        let task = create_basic_task().start().block(reason.clone());
        let manifest = TaskManifest::from(AnyTask::Blocked(task));

        assert_eq!(manifest.spec.status, Status::Blocked);
        assert_eq!(manifest.spec.comments.len(), 1);
        assert_eq!(manifest.spec.comments[0].message, format!("Task blocked: {reason}"));
    }

    #[test]
    fn test_task_to_manifest_cancelled_status() {
        let task = create_basic_task().start().cancel();
        let manifest = TaskManifest::from(AnyTask::Cancelled(task));

        assert_eq!(manifest.spec.status, Status::Cancelled);
        assert!(manifest.spec.comments.is_empty());
    }

    #[test]
    fn test_task_to_manifest_no_assigned_resources() {
        let mut task = create_basic_task();
        task.assigned_resources = vec![];
        let manifest = TaskManifest::from(AnyTask::Planned(task));

        assert_eq!(manifest.spec.assignee, "unassigned");
        assert!(manifest.spec.tags.is_empty());
    }

    #[test]
    fn test_task_to_manifest_no_description() {
        let mut task = create_basic_task();
        task.description = None;
        let manifest = TaskManifest::from(AnyTask::Planned(task));
        assert_eq!(manifest.metadata.description, None);
    }

    // --- Conversion Tests: Manifest -> Task ---

    #[test]
    fn test_manifest_to_task_planned_status() {
        let manifest = create_basic_manifest(Status::Planned);
        let any_task = AnyTask::try_from(manifest).unwrap();

        assert!(matches!(any_task, AnyTask::Planned(_)));
    }

    #[test]
    fn test_manifest_to_task_in_progress_status() {
        let mut manifest = create_basic_manifest(Status::InProgress);
        manifest.spec.comments.push(Comment {
            author: "system".to_string(),
            message: "Progresso atual: 75%".to_string(),
            timestamp: test_date(2024, 1, 5),
        });
        let any_task = AnyTask::try_from(manifest).unwrap();

        if let AnyTask::InProgress(task) = any_task {
            assert_eq!(task.state.progress, 75);
        } else {
            panic!("Incorrect status, expected InProgress");
        }
    }

    #[test]
    fn test_manifest_to_task_completed_status() {
        let manifest = create_basic_manifest(Status::Done);
        let any_task = AnyTask::try_from(manifest).unwrap();
        assert!(matches!(any_task, AnyTask::Completed(_)));
    }

    #[test]
    fn test_manifest_to_task_blocked_status() {
        let mut manifest = create_basic_manifest(Status::Blocked);
        let reason = "Waiting for review".to_string();
        manifest.spec.comments.push(Comment {
            author: "system".to_string(),
            message: format!("Task blocked: {reason}"),
            timestamp: test_date(2024, 1, 5),
        });
        let any_task = AnyTask::try_from(manifest).unwrap();

        if let AnyTask::Blocked(task) = any_task {
            assert_eq!(task.state.reason, reason);
        } else {
            panic!("Incorrect status, expected Blocked");
        }
    }

    #[test]
    fn test_manifest_to_task_cancelled_status() {
        let manifest = create_basic_manifest(Status::Cancelled);
        let any_task = AnyTask::try_from(manifest).unwrap();
        assert!(matches!(any_task, AnyTask::Cancelled(_)));
    }

    // --- Bidirectional Conversion Tests ---

    #[test]
    fn test_bidirectional_conversion_planned_task() {
        let original_task = create_basic_task();
        let manifest = TaskManifest::from(AnyTask::Planned(original_task.clone()));
        let converted_any_task = AnyTask::try_from(manifest).unwrap();

        if let AnyTask::Planned(converted_task) = converted_any_task {
            assert_eq!(original_task.code, converted_task.code);
            assert_eq!(original_task.name, converted_task.name);
        } else {
            panic!("Incorrect status after conversion");
        }
    }

    #[test]
    fn test_bidirectional_conversion_in_progress_task() {
        let original_task = create_basic_task().start().update_progress(50);

        let manifest = TaskManifest::from(AnyTask::InProgress(original_task.clone()));
        let converted_any_task = AnyTask::try_from(manifest).unwrap();

        if let AnyTask::InProgress(converted_task) = converted_any_task {
            assert_eq!(original_task.code, converted_task.code);
            assert_eq!(original_task.state.progress, converted_task.state.progress);
        } else {
            panic!("Incorrect status after conversion");
        }
    }

    #[test]
    fn test_bidirectional_conversion_completed_task() {
        let original_task = create_basic_task().start().complete();
        let manifest = TaskManifest::from(AnyTask::Completed(original_task.clone()));
        let converted_any = AnyTask::try_from(manifest).unwrap();

        assert!(matches!(converted_any, AnyTask::Completed(_)));
        if let AnyTask::Completed(converted) = converted_any {
            assert_eq!(original_task.code, converted.code);
            assert!(converted.actual_end_date.is_some());
        }
    }

    #[test]
    fn test_bidirectional_conversion_blocked_task() {
        let reason = "Waiting for dependency".to_string();
        let original_task = create_basic_task().start().block(reason.clone());
        let manifest = TaskManifest::from(AnyTask::Blocked(original_task.clone()));
        let converted_any = AnyTask::try_from(manifest).unwrap();

        assert!(matches!(converted_any, AnyTask::Blocked(_)));
        if let AnyTask::Blocked(converted) = converted_any {
            assert_eq!(original_task.code, converted.code);
            assert_eq!(converted.state.reason, reason);
        }
    }

    #[test]
    fn test_bidirectional_conversion_cancelled_task() {
        let original_task = create_basic_task().start().cancel();
        let manifest = TaskManifest::from(AnyTask::Cancelled(original_task.clone()));
        let converted_any = AnyTask::try_from(manifest).unwrap();

        assert!(matches!(converted_any, AnyTask::Cancelled(_)));
        if let AnyTask::Cancelled(converted) = converted_any {
            assert_eq!(original_task.code, converted.code);
        }
    }
}

#[cfg(test)]
mod yaml_parsing_tests {
    use super::*;

    #[test]
    fn test_yaml_parsing_success() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                code: "TASK-001"
                name: "Test Task"
                description: "A test task"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "planned"
                priority: "medium"
                estimatedStartDate: "2024-01-01"
                estimatedEndDate: "2024-01-15"
                dependencies:
                    - "TASK-000"
                tags:
                    - "development"
                effort:
                    estimatedHours: 40.0
                acceptanceCriteria:
                    - "Task completed successfully"
        "#;

        let manifest: TaskManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Task");
        assert_eq!(manifest.metadata.code, "TASK-001");
        assert_eq!(manifest.metadata.name, "Test Task");
        assert_eq!(manifest.metadata.description, Some("A test task".to_string()));
        assert_eq!(manifest.spec.project_code, "PROJ-001");
        assert_eq!(manifest.spec.assignee, "DEV-001");
        assert_eq!(manifest.spec.status, Status::Planned);
        assert_eq!(manifest.spec.priority, Priority::Medium);
        assert_eq!(manifest.spec.estimated_start_date, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(manifest.spec.estimated_end_date, Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()));
        assert_eq!(manifest.spec.dependencies, vec!["TASK-000"]);
        assert_eq!(manifest.spec.tags, vec!["development"]);
        assert_eq!(manifest.spec.effort.estimated_hours, 40.0);
        assert_eq!(manifest.spec.acceptance_criteria, vec!["Task completed successfully"]);
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_syntax() {
        let yaml_str = "invalid: yaml: content: [";
        let result: Result<TaskManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_missing_required_field() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                # Missing required fields: code, name
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "planned"
                priority: "medium"
        "#;

        let result: Result<TaskManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_field_type() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                code: "TASK-001"
                name: "Test Task"
                description: "A test task"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "invalid_status"  # Invalid enum value
                priority: "medium"
                estimatedStartDate: "2024-01-01"
                estimatedEndDate: "2024-01-15"
        "#;

        let result: Result<TaskManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_date_format() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                code: "TASK-001"
                name: "Test Task"
                description: "A test task"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "planned"
                priority: "medium"
                estimatedStartDate: "invalid-date"  # Invalid date format
                estimatedEndDate: "2024-01-15"
        "#;

        let result: Result<TaskManifest, _> = serde_yaml::from_str(yaml_str);
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();
        
        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_success_with_optional_fields() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                code: "TASK-001"
                name: "Complex Test Task"
                description: "A comprehensive test task with all fields"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "in_progress"
                priority: "high"
                estimatedStartDate: "2024-01-01"
                estimatedEndDate: "2024-01-15"
                actualStartDate: "2024-01-01"
                actualEndDate: "2024-01-14"
                dependencies:
                    - "TASK-000"
                    - "TASK-002"
                tags:
                    - "testing"
                    - "complex"
                effort:
                    estimatedHours: 80.0
                    actualHours: 75.5
                acceptanceCriteria:
                    - "All tests pass"
                    - "Code review completed"
                comments:
                    - text: "Started working on this task"
                      author: "DEV-001"
                      createdAt: "2024-01-01T09:00:00Z"
        "#;

        let manifest: TaskManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.metadata.description, Some("A comprehensive test task with all fields".to_string()));
        assert_eq!(manifest.spec.status, Status::InProgress);
        assert_eq!(manifest.spec.actual_end_date, Some(NaiveDate::from_ymd_opt(2024, 1, 14).unwrap()));
        assert_eq!(manifest.spec.priority, Priority::High);
        assert_eq!(manifest.spec.tags, vec!["testing", "complex"]);
        assert_eq!(manifest.spec.dependencies, vec!["TASK-000", "TASK-002"]);
        assert_eq!(manifest.spec.effort.estimated_hours, 80.0);
        assert_eq!(manifest.spec.effort.actual_hours, Some(75.5));
        assert_eq!(manifest.spec.acceptance_criteria, vec!["All tests pass", "Code review completed"]);
        assert_eq!(manifest.spec.comments.len(), 1);
    }

    #[test]
    fn test_yaml_parsing_success_minimal_task() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
                id: "01996dev-0000-0000-0000-000000task"
                code: "TASK-001"
                name: "Minimal Task"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                projectCode: "PROJ-001"
                assignee: "DEV-001"
                status: "planned"
                priority: "medium"
                effort:
                    estimatedHours: 0.0
        "#;

        let manifest: TaskManifest = serde_yaml::from_str(yaml_str).unwrap();
        
        assert_eq!(manifest.metadata.name, "Minimal Task");
        assert_eq!(manifest.spec.project_code, "PROJ-001");
        assert_eq!(manifest.spec.assignee, "DEV-001");
        assert_eq!(manifest.spec.status, Status::Planned);
        assert_eq!(manifest.spec.priority, Priority::Medium);
        assert!(manifest.spec.dependencies.is_empty());
        assert!(manifest.spec.tags.is_empty());
        assert!(manifest.spec.acceptance_criteria.is_empty());
        assert!(manifest.spec.comments.is_empty());
    }
}
