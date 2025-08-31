
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid7::Uuid;

use crate::domain::shared::errors::{DomainError, DomainErrorKind};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub tasks: HashMap<String, Task>,
    pub resources: HashMap<String, ResourceAssignment>,
    
    // Configurações
    pub settings: ProjectSettings,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub allow_overtime: bool,
    pub require_approval: bool,
    pub auto_resource_leveling: bool,
    pub critical_path_alert: bool,
    pub budget_alert_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    
    // Datas
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub actual_start_date: Option<NaiveDate>,
    pub actual_end_date: Option<NaiveDate>,
    
    // Esforço
    pub estimated_effort: Option<Duration>,
    pub actual_effort: Option<Duration>,
    pub remaining_effort: Option<Duration>,
    
    // Dependências
    pub predecessors: Vec<String>,
    pub successors: Vec<String>,
    
    // Recursos
    pub assigned_resources: Vec<String>,
    
    // Metadados
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAssignment {
    pub resource_id: String,
    pub task_id: String,
    pub allocation_percentage: u8,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub hours: u32,
    pub minutes: u32,
}

impl Duration {
    pub fn new(hours: u32, minutes: u32) -> Self {
        Self { hours, minutes }
    }
    
    pub fn from_minutes(total_minutes: u32) -> Self {
        Self {
            hours: total_minutes / 60,
            minutes: total_minutes % 60,
        }
    }
    
    pub fn to_minutes(&self) -> u32 {
        self.hours * 60 + self.minutes
    }
    
    pub fn add(&self, other: &Duration) -> Duration {
        let total_minutes = self.to_minutes() + other.to_minutes();
        Duration::from_minutes(total_minutes)
    }
    
    pub fn subtract(&self, other: &Duration) -> Result<Duration, DomainError> {
        let self_minutes = self.to_minutes();
        let other_minutes = other.to_minutes();
        
        if other_minutes > self_minutes {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "duration".to_string(),
                message: "Cannot subtract larger duration from smaller duration".to_string(),
            }));
        }
        
        Ok(Duration::from_minutes(self_minutes - other_minutes))
    }
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl Project {
    pub fn new(
        code: String,
        name: String,
        company_code: String,
        created_by: String,
    ) -> Result<Self, DomainError> {
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
        
        if company_code.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "company_code".to_string(),
                message: "Company code cannot be empty".to_string(),
            }));
        }
        
        let now = Utc::now();
        
        Ok(Self {
            id: Uuid::new_v7().to_string(),
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
    
    pub fn add_task(&mut self, task: Task) -> Result<(), DomainError> {
        if self.tasks.contains_key(&task.id) {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "task_id".to_string(),
                message: "Task with this ID already exists".to_string(),
            }));
        }
        
        self.tasks.insert(task.id.clone(), task);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn remove_task(&mut self, task_id: &str) -> Result<(), DomainError> {
        if let Some(task) = self.tasks.get(task_id) {
            if task.status == TaskStatus::InProgress {
                return Err(DomainError::new(DomainErrorKind::Generic {
                    message: "Cannot remove task that is in progress".to_string(),
                }));
            }
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
        
        if let Some(assignment) = self.resources.get(&key) {
            if assignment.start_date <= Utc::now().date_naive() {
                return Err(DomainError::new(DomainErrorKind::Generic {
                    message: "Cannot remove resource assignment that has already started".to_string(),
                }));
            }
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
        if let (Some(planned_end), Some(actual_end)) = (self.end_date, self.actual_end_date) {
            actual_end <= planned_end
        } else if let Some(planned_end) = self.end_date {
            let today = Utc::now().date_naive();
            today <= planned_end
        } else {
            false
        }
    }
    
    pub fn completion_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        
        let completed_tasks = self.tasks.values()
            .filter(|task| task.status == TaskStatus::Completed)
            .count();
        
        (completed_tasks as f64 / self.tasks.len() as f64) * 100.0
    }
    
    // Validações privadas
    fn validate_can_start(&self) -> Result<(), DomainError> {
        if !self.has_tasks() {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: "Project must have at least one task to start".to_string(),
            }));
        }
        
        if self.start_date.is_none() || self.end_date.is_none() {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: "Project must have start and end dates defined".to_string(),
            }));
        }
        
        if !self.has_resources() {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: "Project must have at least one resource assigned".to_string(),
            }));
        }
        
        Ok(())
    }
    
    fn validate_can_complete(&self) -> Result<(), DomainError> {
        for task in self.tasks.values() {
            if task.status != TaskStatus::Completed {
                return Err(DomainError::new(DomainErrorKind::Generic {
                    message: "All tasks must be completed to finish project".to_string(),
                }));
            }
        }
        
        Ok(())
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            allow_overtime: false,
            require_approval: true,
            auto_resource_leveling: false,
            critical_path_alert: true,
            budget_alert_threshold: Some(0.9), // 90%
        }
    }
}

impl Task {
    pub fn new(
        name: String,
        created_by: String,
    ) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Task name cannot be empty".to_string(),
            }));
        }
        
        let now = Utc::now();
        
        Ok(Self {
            id: Uuid::new_v7().to_string(),
            name,
            description: None,
            status: TaskStatus::NotStarted,
            priority: TaskPriority::Medium,
            start_date: None,
            end_date: None,
            actual_start_date: None,
            actual_end_date: None,
            estimated_effort: None,
            actual_effort: None,
            remaining_effort: None,
            predecessors: Vec::new(),
            successors: Vec::new(),
            assigned_resources: Vec::new(),
            created_at: now,
            updated_at: now,
            created_by,
        })
    }
    
    pub fn change_status(&mut self, new_status: TaskStatus) -> Result<(), DomainError> {
        if !self.can_transition_to(&new_status) {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: format!("Cannot transition from {:?} to {:?}", self.status, new_status),
            }));
        }
        
        // Validações específicas por status
        match new_status {
            TaskStatus::InProgress => {
                self.actual_start_date = Some(Utc::now().date_naive());
            }
            TaskStatus::Completed => {
                self.actual_end_date = Some(Utc::now().date_naive());
                self.remaining_effort = Some(Duration::new(0, 0));
            }
            _ => {}
        }
        
        self.status = new_status;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn add_predecessor(&mut self, task_id: String) -> Result<(), DomainError> {
        if task_id == self.id {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "predecessor".to_string(),
                message: "Task cannot depend on itself".to_string(),
            }));
        }
        
        if !self.predecessors.contains(&task_id) {
            self.predecessors.push(task_id);
            self.updated_at = Utc::now();
        }
        
        Ok(())
    }
    
    pub fn remove_predecessor(&mut self, task_id: &str) -> Result<(), DomainError> {
        if self.status == TaskStatus::InProgress {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: "Cannot remove predecessor from task that is in progress".to_string(),
            }));
        }
        
        self.predecessors.retain(|id| id != task_id);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn assign_resource(&mut self, resource_id: String) -> Result<(), DomainError> {
        if !self.assigned_resources.contains(&resource_id) {
            self.assigned_resources.push(resource_id);
            self.updated_at = Utc::now();
        }
        
        Ok(())
    }
    
    pub fn remove_resource(&mut self, resource_id: &str) -> Result<(), DomainError> {
        if self.status == TaskStatus::InProgress {
            return Err(DomainError::new(DomainErrorKind::Generic {
                message: "Cannot remove resource from task that is in progress".to_string(),
            }));
        }
        
        self.assigned_resources.retain(|id| id != resource_id);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }
    
    pub fn is_started(&self) -> bool {
        matches!(self.status, TaskStatus::InProgress | TaskStatus::Completed)
    }
    
    pub fn is_delayed(&self) -> bool {
        if let (Some(planned_end), Some(actual_end)) = (self.end_date, self.actual_end_date) {
            actual_end > planned_end
        } else if let Some(planned_end) = self.end_date {
            let today = Utc::now().date_naive();
            today > planned_end
        } else {
            false
        }
    }
    
    // Validações privadas
    fn can_transition_to(&self, new_status: &TaskStatus) -> bool {
        match (self.status, new_status) {
            // NotStarted -> InProgress, Cancelled
            (TaskStatus::NotStarted, TaskStatus::InProgress) => true,
            (TaskStatus::NotStarted, TaskStatus::Cancelled) => true,
            
            // InProgress -> OnHold, Completed
            (TaskStatus::InProgress, TaskStatus::OnHold) => true,
            (TaskStatus::InProgress, TaskStatus::Completed) => true,
            
            // OnHold -> InProgress, Cancelled
            (TaskStatus::OnHold, TaskStatus::InProgress) => true,
            (TaskStatus::OnHold, TaskStatus::Cancelled) => true,
            
            // Completed -> (não pode mudar)
            (TaskStatus::Completed, _) => false,
            
            // Cancelled -> (não pode mudar)
            (TaskStatus::Cancelled, _) => false,
            
            // Outras transições não são permitidas
            _ => false,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_project_creation() {
        let project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );
        
        assert!(project.is_ok());
        
        let project = project.unwrap();
        assert_eq!(project.code, "PROJ-001");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.company_code, "COMP-001");
        assert_eq!(project.status, ProjectStatus::Planned);
        assert_eq!(project.priority, ProjectPriority::Medium);
        assert!(project.tasks.is_empty());
        assert!(project.resources.is_empty());
    }
    
    #[test]
    fn test_project_creation_with_empty_code() {
        let result = Project::new(
            "".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );
        
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(matches!(error.kind(), DomainErrorKind::ValidationError { .. }));
        }
    }
    
    #[test]
    fn test_project_creation_with_empty_name() {
        let result = Project::new(
            "PROJ-001".to_string(),
            "".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        );
        
        assert!(result.is_err());
        
        if let Err(error) = result {
            assert!(matches!(error.kind(), DomainErrorKind::ValidationError { .. }));
        }
    }
    
    #[test]
    fn test_project_status_transitions() {
        let mut project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Planned -> InProgress (deve falhar sem tarefas e recursos)
        let result = project.change_status(ProjectStatus::InProgress);
        assert!(result.is_err());
        
        // Adicionar tarefa
        let task = Task::new("Test Task".to_string(), "user-001".to_string()).unwrap();
        project.add_task(task).unwrap();
        
        // Adicionar recurso
        let assignment = ResourceAssignment {
            resource_id: "RES-001".to_string(),
            task_id: "TASK-001".to_string(),
            allocation_percentage: 100,
            start_date: Utc::now().date_naive(),
            end_date: None,
            role: "Developer".to_string(),
            created_at: Utc::now(),
        };
        project.assign_resource(assignment).unwrap();
        
        // Definir datas
        project.start_date = Some(Utc::now().date_naive());
        project.end_date = Some(Utc::now().date_naive() + chrono::Duration::days(30));
        
        // Agora deve funcionar
        let result = project.change_status(ProjectStatus::InProgress);
        assert!(result.is_ok());
        assert_eq!(project.status, ProjectStatus::InProgress);
    }
    
    #[test]
    fn test_project_completion_validation() {
        let mut project = Project::new(
            "PROJ-001".to_string(),
            "Test Project".to_string(),
            "COMP-001".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Adicionar tarefa não completada
        let mut task = Task::new("Test Task".to_string(), "user-001".to_string()).unwrap();
        project.add_task(task.clone()).unwrap();
        
        // Tentar completar projeto deve falhar
        let result = project.change_status(ProjectStatus::Completed);
        assert!(result.is_err());
        
        // Completar tarefa
        task.change_status(TaskStatus::Completed).unwrap();
        project.tasks.insert(task.id.clone(), task);
        
        // Agora deve funcionar
        let result = project.change_status(ProjectStatus::Completed);
        assert!(result.is_ok());
        assert_eq!(project.status, ProjectStatus::Completed);
    }
    
    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test Task".to_string(),
            "user-001".to_string(),
        );
        
        assert!(task.is_ok());
        
        let task = task.unwrap();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::NotStarted);
        assert_eq!(task.priority, TaskPriority::Medium);
        assert!(task.predecessors.is_empty());
        assert!(task.assigned_resources.is_empty());
    }
    
    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new(
            "Test Task".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // NotStarted -> InProgress
        let result = task.change_status(TaskStatus::InProgress);
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.actual_start_date.is_some());
        
        // InProgress -> Completed
        let result = task.change_status(TaskStatus::Completed);
        assert!(result.is_ok());
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.actual_end_date.is_some());
        
        // Completed -> InProgress (deve falhar)
        let result = task.change_status(TaskStatus::InProgress);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_duration_operations() {
        let duration1 = Duration::new(2, 30); // 2h 30m
        let duration2 = Duration::new(1, 45); // 1h 45m
        
        // Adição
        let sum = duration1.add(&duration2);
        assert_eq!(sum.hours, 4);
        assert_eq!(sum.minutes, 15);
        
        // Subtração
        let diff = duration1.subtract(&duration2).unwrap();
        assert_eq!(diff.hours, 0);
        assert_eq!(diff.minutes, 45);
        
        // Subtração inválida
        let result = duration2.subtract(&duration1);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_duration_from_minutes() {
        let duration = Duration::from_minutes(125); // 2h 5m
        assert_eq!(duration.hours, 2);
        assert_eq!(duration.minutes, 5);
        
        let total_minutes = duration.to_minutes();
        assert_eq!(total_minutes, 125);
    }
}
