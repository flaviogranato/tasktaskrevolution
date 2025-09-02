#![allow(dead_code)]

use super::super::task_management::any_task::AnyTask;
use crate::domain::shared::errors::{DomainError, DomainErrorKind};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ProjectStatus {
    Planned,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

impl ProjectStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, ProjectStatus::InProgress | ProjectStatus::OnHold)
    }

    pub fn can_transition_to(&self, new_status: &ProjectStatus) -> bool {
        match (self, new_status) {
            // Planned -> InProgress, Cancelled
            (ProjectStatus::Planned, ProjectStatus::InProgress) => true,
            (ProjectStatus::Planned, ProjectStatus::Cancelled) => true,

            // InProgress -> OnHold, Completed
            (ProjectStatus::InProgress, ProjectStatus::OnHold) => true,
            (ProjectStatus::InProgress, ProjectStatus::Completed) => true,

            // OnHold -> InProgress, Cancelled
            (ProjectStatus::OnHold, ProjectStatus::InProgress) => true,
            (ProjectStatus::OnHold, ProjectStatus::Cancelled) => true,

            // Completed -> (não pode mudar)
            (ProjectStatus::Completed, _) => false,

            // Cancelled -> (não pode mudar)
            (ProjectStatus::Cancelled, _) => false,

            // Outras transições não são permitidas
            _ => false,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Planned => write!(f, "Planned"),
            ProjectStatus::InProgress => write!(f, "In Progress"),
            ProjectStatus::OnHold => write!(f, "On Hold"),
            ProjectStatus::Completed => write!(f, "Completed"),
            ProjectStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ProjectPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl ProjectPriority {
    pub fn weight(&self) -> u8 {
        match self {
            ProjectPriority::Low => 1,
            ProjectPriority::Medium => 2,
            ProjectPriority::High => 3,
            ProjectPriority::Critical => 4,
        }
    }
}

// ============================================================================
// STRUCTS
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub priority: ProjectPriority,

    // Datas
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,

    // Metadados
    pub company_code: String,
    pub manager_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,

    // Relacionamentos
    pub tasks: HashMap<String, AnyTask>,
    pub resources: HashMap<String, ResourceAssignment>,

    // Configurações
    pub settings: ProjectSettings,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub timezone: Option<String>,
    pub vacation_rules: Option<VacationRules>,
    pub work_hours: Option<WorkHours>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHours {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VacationRules {
    pub allowed_days_per_year: u32,
    pub carry_over_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignment {
    pub resource_id: String,
    pub task_id: String,
    pub allocation_percentage: u8,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            timezone: Some("UTC".to_string()),
            vacation_rules: Some(VacationRules {
                allowed_days_per_year: 20,
                carry_over_days: 5,
            }),
            work_hours: Some(WorkHours {
                start: "08:00".to_string(),
                end: "18:00".to_string(),
            }),
        }
    }
}

impl Default for VacationRules {
    fn default() -> Self {
        Self {
            allowed_days_per_year: 20,
            carry_over_days: 5,
        }
    }
}

impl Default for WorkHours {
    fn default() -> Self {
        Self {
            start: "08:00".to_string(),
            end: "18:00".to_string(),
        }
    }
}

impl Project {
    pub fn new(code: String, name: String, company_code: String, created_by: String) -> Result<Self, DomainError> {
        if code.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: "Project code cannot be empty".to_string(),
            }));
        }

        if name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Project name cannot be empty".to_string(),
            }));
        }

        let now = Utc::now();

        Ok(Self {
            id: uuid7::uuid7().to_string(),
            code,
            name,
            description: None,
            status: ProjectStatus::Planned,
            priority: ProjectPriority::Medium,
            start_date: None,
            end_date: None,
            actual_start_date: None,
            actual_end_date: None,
            company_code,
            manager_id: None,
            created_at: now,
            updated_at: now,
            created_by,
            tasks: HashMap::new(),
            resources: HashMap::new(),
            settings: ProjectSettings::default(),
            metadata: HashMap::new(),
        })
    }

    pub fn change_status(&mut self, new_status: ProjectStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(&new_status) {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: format!("Cannot transition from {:?} to {:?}", self.status, new_status),
            }));
        }

        // Validações específicas por status
        match new_status {
            ProjectStatus::InProgress => {
                self.validate_can_start()?;
                self.actual_start_date = Some(Utc::now().date_naive());
            }
            ProjectStatus::Completed => {
                self.validate_can_complete()?;
                self.actual_end_date = Some(Utc::now().date_naive());
            }
            _ => {}
        }

        self.status = new_status;
        self.updated_at = Utc::now();

        Ok(())
    }

    fn validate_can_start(&self) -> Result<(), DomainError> {
        if self.tasks.is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "tasks".to_string(),
                message: "Project must have at least one task to start".to_string(),
            }));
        }
        Ok(())
    }

    fn validate_can_complete(&self) -> Result<(), DomainError> {
        let all_tasks_completed = self.tasks.values().all(|task| task.status() == "Completed");

        if !all_tasks_completed {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "tasks".to_string(),
                message: "All tasks must be completed before marking project as complete".to_string(),
            }));
        }
        Ok(())
    }

    pub fn add_task(&mut self, task: AnyTask) -> Result<(), DomainError> {
        // Use the task code as the ID
        let task_id = task.code().to_string();

        if self.tasks.contains_key(&task_id) {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "task_id".to_string(),
                message: "Task with this ID already exists".to_string(),
            }));
        }

        self.tasks.insert(task_id, task);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn remove_task(&mut self, task_id: &str) -> Result<(), DomainError> {
        if let Some(_task) = self.tasks.get(task_id) {
            // Verificar se a tarefa pode ser removida
            // Implementar quando tivermos acesso ao AnyTask
        }

        self.tasks.remove(task_id);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn assign_resource(&mut self, assignment: ResourceAssignment) -> Result<(), DomainError> {
        let key = format!("{}_{}", assignment.resource_id, assignment.task_id);

        if self.resources.contains_key(&key) {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "resource_assignment".to_string(),
                message: "Resource is already assigned to this task".to_string(),
            }));
        }

        self.resources.insert(key, assignment);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn remove_resource_assignment(&mut self, resource_id: &str, task_id: &str) -> Result<(), DomainError> {
        let key = format!("{}_{}", resource_id, task_id);

        if let Some(_assignment) = self.resources.get(&key) {
            // Verificar se a alocação pode ser removida
            // Implementar quando necessário
        }

        self.resources.remove(&key);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    pub fn has_resources(&self) -> bool {
        !self.resources.is_empty()
    }

    pub fn is_on_schedule(&self) -> bool {
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            let now = Utc::now().date_naive();
            now >= start && now <= end
        } else {
            true // Se não há datas definidas, consideramos no prazo
        }
    }

    pub fn completion_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed_tasks = self
            .tasks
            .values()
            .filter(|_task| {
                // Assumindo que AnyTask tem um método is_completed
                // Se não tiver, podemos implementar uma verificação diferente
                false // Placeholder - implementar quando tivermos acesso ao AnyTask
            })
            .count();

        (completed_tasks as f64 / self.tasks.len() as f64) * 100.0
    }

    // Getters simples
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn status(&self) -> ProjectStatus {
        self.status
    }

    pub fn priority(&self) -> ProjectPriority {
        self.priority
    }

    pub fn company_code(&self) -> &str {
        &self.company_code
    }

    pub fn created_by(&self) -> &str {
        &self.created_by
    }

    pub fn tasks(&self) -> &HashMap<String, AnyTask> {
        &self.tasks
    }

    pub fn resources(&self) -> &HashMap<String, ResourceAssignment> {
        &self.resources
    }

    // Validation methods
    pub fn is_code_valid(&self) -> bool {
        !self.code.trim().is_empty()
    }

    pub fn is_name_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn is_date_range_valid(&self) -> bool {
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            start <= end
        } else {
            true
        }
    }

    pub fn validate(&self) -> Result<Vec<String>, String> {
        let mut errors = Vec::new();

        if !self.is_code_valid() {
            errors.push("Project code cannot be empty".to_string());
        }

        if !self.is_name_valid() {
            errors.push("Project name cannot be empty".to_string());
        }

        if !self.is_date_range_valid() {
            errors.push("Project end date must be after start date".to_string());
        }

        Ok(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation_with_valid_data() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        )
        .unwrap();

        assert_eq!(project.code(), "PROJ-001");
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.company_code(), "COMP-001");
        assert_eq!(project.status(), ProjectStatus::Planned);
        assert_eq!(project.priority(), ProjectPriority::Medium);
        assert_eq!(project.created_by(), "user@example.com");
    }

    #[test]
    fn test_project_creation_with_empty_code() {
        let result = Project::new(
            "".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_project_creation_with_empty_name() {
        let result = Project::new(
            "PROJ-001".to_string(),
            "".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_project_status_transitions() {
        let mut project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        )
        .unwrap();

        // Add a task to the project so it can be started
        let task = crate::domain::task_management::any_task::AnyTask::Planned(
            crate::domain::task_management::builder::TaskBuilder::new()
                .project_code("PROJ-001".to_string())
                .name("Test Task".to_string())
                .code("TASK-001".to_string())
                .dates(
                    chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                    chrono::NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
                )
                .unwrap()
                .validate_vacations(&[])
                .unwrap()
                .build()
                .unwrap(),
        );
        project.add_task(task).unwrap();

        // Planned -> InProgress
        assert!(project.change_status(ProjectStatus::InProgress).is_ok());
        assert_eq!(project.status(), ProjectStatus::InProgress);

        // Complete the task first so the project can be completed
        let task_code = "TASK-001".to_string();
        if let Some(task) = project.tasks.get_mut(&task_code) {
            let completed_task = task.clone().complete();
            project.tasks.insert(task_code, completed_task);
        }

        // InProgress -> Completed
        assert!(project.change_status(ProjectStatus::Completed).is_ok());
        assert_eq!(project.status(), ProjectStatus::Completed);

        // Completed -> Planned (não deve funcionar)
        assert!(project.change_status(ProjectStatus::Planned).is_err());
        assert_eq!(project.status(), ProjectStatus::Completed);
    }

    #[test]
    fn test_project_validation() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user@example.com".to_string(),
        )
        .unwrap();

        let errors = project.validate().unwrap();
        assert!(errors.is_empty());
    }
}
