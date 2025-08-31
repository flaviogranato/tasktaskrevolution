use chrono::{DateTime, NaiveDate, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid7::Uuid;

use crate::domain::shared::errors::{DomainError, DomainErrorKind};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum AllocationStatus {
    Planned,
    Active,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ResourceType {
    Human,
    Equipment,
    Material,
    Facility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

// ============================================================================
// STRUCTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub code: String,
    pub name: String,
    pub resource_type: ResourceType,
    pub company_code: String,
    pub skills: Vec<Skill>,
    pub availability: ResourceAvailability,
    pub cost_rate: Option<CostRate>,
    pub status: ResourceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub level: SkillLevel,
    pub years_experience: Option<u8>,
    pub certified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAvailability {
    pub working_hours: WorkingHours,
    pub timezone: String,
    pub holidays: Vec<NaiveDate>,
    pub leaves: Vec<LeavePeriod>,
    pub max_allocation_percentage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHours {
    pub monday: WorkDay,
    pub tuesday: WorkDay,
    pub wednesday: WorkDay,
    pub thursday: WorkDay,
    pub friday: WorkDay,
    pub saturday: WorkDay,
    pub sunday: WorkDay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDay {
    pub is_working_day: bool,
    pub start_time: Option<String>, // HH:MM format
    pub end_time: Option<String>,   // HH:MM format
    pub break_times: Vec<BreakTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakTime {
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeavePeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: String,
    pub allocation_percentage: u8, // 0 = completely unavailable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRate {
    pub hourly_rate: f64,
    pub currency: String,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ResourceStatus {
    Active,
    Inactive,
    OnLeave,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub id: String,
    pub resource_id: String,
    pub task_id: String,
    pub project_id: String,
    pub allocation_percentage: u8,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub role: String,
    pub status: AllocationStatus,
    pub actual_hours: Option<f64>,
    pub estimated_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationConflict {
    pub id: String,
    pub resource_id: String,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: String,
    pub conflicting_allocations: Vec<String>,
    pub suggested_resolutions: Vec<Resolution>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ConflictType {
    OverAllocation,      // Resource allocated > 100%
    DoubleBooking,       // Resource allocated to multiple tasks at same time
    SkillMismatch,      // Resource doesn't have required skills
    AvailabilityConflict, // Resource not available during allocation period
    CostOverrun,        // Allocation exceeds budget constraints
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ConflictSeverity {
    Low,      // Minor conflict, can be resolved easily
    Medium,   // Moderate conflict, requires attention
    High,     // Major conflict, impacts project timeline
    Critical, // Critical conflict, project cannot proceed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub id: String,
    pub description: String,
    pub action_required: String,
    pub estimated_effort: Option<Duration>,
    pub cost_impact: Option<f64>,
    pub timeline_impact: Option<Duration>,
}

// ============================================================================
// IMPLEMENTATIONS
// ============================================================================

impl Resource {
    pub fn new(
        code: String,
        name: String,
        resource_type: ResourceType,
        company_code: String,
        created_by: String,
    ) -> Result<Self, DomainError> {
        if code.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "code".to_string(),
                message: "Resource code cannot be empty".to_string(),
            }));
        }
        
        if name.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "name".to_string(),
                message: "Resource name cannot be empty".to_string(),
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
            resource_type,
            company_code,
            skills: Vec::new(),
            availability: ResourceAvailability::default(),
            cost_rate: None,
            status: ResourceStatus::Active,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }
    
    pub fn add_skill(&mut self, skill: Skill) -> Result<(), DomainError> {
        if self.skills.iter().any(|s| s.name == skill.name) {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "skill".to_string(),
                message: "Skill already exists for this resource".to_string(),
            }));
        }
        
        self.skills.push(skill);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn remove_skill(&mut self, skill_name: &str) -> Result<(), DomainError> {
        let initial_count = self.skills.len();
        self.skills.retain(|s| s.name != skill_name);
        
        if self.skills.len() == initial_count {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "skill".to_string(),
                message: "Skill not found for this resource".to_string(),
            }));
        }
        
        self.updated_at = Utc::now();
        Ok(())
    }
    
    pub fn has_skill(&self, skill_name: &str, min_level: Option<SkillLevel>) -> bool {
        self.skills.iter().any(|skill| {
            skill.name == skill_name && 
            min_level.map_or(true, |min| skill.level as u8 >= min as u8)
        })
    }
    
    pub fn is_available_on_date(&self, date: NaiveDate) -> bool {
        // Verificar se é feriado
        if self.availability.holidays.contains(&date) {
            return false;
        }
        
        // Verificar se está de férias
        for leave in &self.availability.leaves {
            if date >= leave.start_date && date <= leave.end_date {
                return false;
            }
        }
        
        // Verificar se é dia útil
        let weekday = date.weekday();
        let work_day = match weekday {
            chrono::Weekday::Mon => &self.availability.working_hours.monday,
            chrono::Weekday::Tue => &self.availability.working_hours.tuesday,
            chrono::Weekday::Wed => &self.availability.working_hours.wednesday,
            chrono::Weekday::Thu => &self.availability.working_hours.thursday,
            chrono::Weekday::Fri => &self.availability.working_hours.friday,
            chrono::Weekday::Sat => &self.availability.working_hours.saturday,
            chrono::Weekday::Sun => &self.availability.working_hours.sunday,
        };
        
        work_day.is_working_day
    }
    
    pub fn get_working_hours_on_date(&self, date: NaiveDate) -> Option<WorkDay> {
        if !self.is_available_on_date(date) {
            return None;
        }
        
        let weekday = date.weekday();
        let work_day = match weekday {
            chrono::Weekday::Mon => &self.availability.working_hours.monday,
            chrono::Weekday::Tue => &self.availability.working_hours.tuesday,
            chrono::Weekday::Wed => &self.availability.working_hours.wednesday,
            chrono::Weekday::Thu => &self.availability.working_hours.thursday,
            chrono::Weekday::Fri => &self.availability.working_hours.friday,
            chrono::Weekday::Sat => &self.availability.working_hours.saturday,
            chrono::Weekday::Sun => &self.availability.working_hours.sunday,
        };
        
        Some(work_day.clone())
    }
    
    pub fn can_allocate_percentage(&self, percentage: u8) -> bool {
        percentage <= self.availability.max_allocation_percentage
    }
}

impl ResourceAllocation {
    pub fn new(
        resource_id: String,
        task_id: String,
        project_id: String,
        allocation_percentage: u8,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        role: String,
        created_by: String,
    ) -> Result<Self, DomainError> {
        if allocation_percentage == 0 || allocation_percentage > 100 {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "allocation_percentage".to_string(),
                message: "Allocation percentage must be between 1 and 100".to_string(),
            }));
        }
        
        if role.trim().is_empty() {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "role".to_string(),
                message: "Role cannot be empty".to_string(),
            }));
        }
        
        let now = Utc::now();
        
        Ok(Self {
            id: Uuid::new_v7().to_string(),
            resource_id,
            task_id,
            project_id,
            allocation_percentage,
            start_date,
            end_date,
            role,
            status: AllocationStatus::Planned,
            actual_hours: None,
            estimated_hours: None,
            created_at: now,
            updated_at: now,
            created_by,
        })
    }
    
    pub fn activate(&mut self) -> Result<(), DomainError> {
        if self.status != AllocationStatus::Planned {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "status".to_string(),
                message: "Can only activate planned allocations".to_string(),
            }));
        }
        
        self.status = AllocationStatus::Active;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn complete(&mut self) -> Result<(), DomainError> {
        if self.status != AllocationStatus::Active {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "status".to_string(),
                message: "Can only complete active allocations".to_string(),
            }));
        }
        
        self.status = AllocationStatus::Completed;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if self.status == AllocationStatus::Completed {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "status".to_string(),
                message: "Cannot cancel completed allocations".to_string(),
            }));
        }
        
        self.status = AllocationStatus::Cancelled;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn update_hours(&mut self, actual_hours: f64, estimated_hours: Option<f64>) -> Result<(), DomainError> {
        if actual_hours < 0.0 {
            return Err(DomainError::new(DomainErrorKind::ValidationError {
                field: "actual_hours".to_string(),
                message: "Actual hours cannot be negative".to_string(),
            }));
        }
        
        if let Some(est_hours) = estimated_hours {
            if est_hours < 0.0 {
                return Err(DomainError::new(DomainErrorKind::ValidationError {
                    field: "estimated_hours".to_string(),
                    message: "Estimated hours cannot be negative".to_string(),
                }));
            }
        }
        
        self.actual_hours = Some(actual_hours);
        self.estimated_hours = estimated_hours;
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    pub fn is_active(&self) -> bool {
        self.status == AllocationStatus::Active
    }
    
    pub fn is_overlapping(&self, other: &ResourceAllocation) -> bool {
        if self.resource_id != other.resource_id {
            return false;
        }
        
        let self_start = self.start_date;
        let self_end = self.end_date.unwrap_or_else(|| self_start + Duration::days(365));
        
        let other_start = other.start_date;
        let other_end = other.end_date.unwrap_or_else(|| other_start + Duration::days(365));
        
        // Verificar sobreposição
        self_start < other_end && other_start < self_end
    }
    
    pub fn get_total_allocation_percentage(&self, other_allocations: &[ResourceAllocation]) -> u8 {
        let mut total = self.allocation_percentage;
        
        for allocation in other_allocations {
            if allocation.id != self.id && 
               allocation.resource_id == self.resource_id &&
               allocation.is_active() &&
               self.is_overlapping(allocation) {
                total += allocation.allocation_percentage;
            }
        }
        
        total
    }
}

impl AllocationConflict {
    pub fn new(
        resource_id: String,
        conflict_type: ConflictType,
        severity: ConflictSeverity,
        description: String,
        conflicting_allocations: Vec<String>,
        suggested_resolutions: Vec<Resolution>,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            resource_id,
            conflict_type,
            severity,
            description,
            conflicting_allocations,
            suggested_resolutions,
            detected_at: Utc::now(),
        }
    }
    
    pub fn add_resolution(&mut self, resolution: Resolution) {
        self.suggested_resolutions.push(resolution);
    }
    
    pub fn is_critical(&self) -> bool {
        self.severity == ConflictSeverity::Critical
    }
    
    pub fn requires_immediate_action(&self) -> bool {
        matches!(self.severity, ConflictSeverity::High | ConflictSeverity::Critical)
    }
}

impl Resolution {
    pub fn new(
        description: String,
        action_required: String,
        estimated_effort: Option<Duration>,
        cost_impact: Option<f64>,
        timeline_impact: Option<Duration>,
    ) -> Self {
        Self {
            id: Uuid::new_v7().to_string(),
            description,
            action_required,
            estimated_effort,
            cost_impact,
            timeline_impact,
        }
    }
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for ResourceAvailability {
    fn default() -> Self {
        Self {
            working_hours: WorkingHours::default(),
            timezone: "UTC".to_string(),
            holidays: Vec::new(),
            leaves: Vec::new(),
            max_allocation_percentage: 100,
        }
    }
}

impl Default for WorkingHours {
    fn default() -> Self {
        let standard_workday = WorkDay {
            is_working_day: true,
            start_time: Some("09:00".to_string()),
            end_time: Some("17:00".to_string()),
            break_times: vec![
                BreakTime {
                    start_time: "12:00".to_string(),
                    end_time: "13:00".to_string(),
                    description: Some("Lunch break".to_string()),
                }
            ],
        };
        
        let weekend = WorkDay {
            is_working_day: false,
            start_time: None,
            end_time: None,
            break_times: Vec::new(),
        };
        
        Self {
            monday: standard_workday.clone(),
            tuesday: standard_workday.clone(),
            wednesday: standard_workday.clone(),
            thursday: standard_workday.clone(),
            friday: standard_workday.clone(),
            saturday: weekend.clone(),
            sunday: weekend,
        }
    }
}

impl Clone for WorkDay {
    fn clone(&self) -> Self {
        Self {
            is_working_day: self.is_working_day,
            start_time: self.start_time.clone(),
            end_time: self.end_time.clone(),
            break_times: self.break_times.clone(),
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
    fn test_resource_creation() {
        let resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            ResourceType::Human,
            "COMP-001".to_string(),
            "user-001".to_string(),
        );
        
        assert!(resource.is_ok());
        
        let resource = resource.unwrap();
        assert_eq!(resource.code, "RES-001");
        assert_eq!(resource.name, "John Doe");
        assert_eq!(resource.resource_type, ResourceType::Human);
        assert_eq!(resource.company_code, "COMP-001");
        assert_eq!(resource.status, ResourceStatus::Active);
        assert!(resource.skills.is_empty());
    }
    
    #[test]
    fn test_resource_creation_with_empty_code() {
        let result = Resource::new(
            "".to_string(),
            "John Doe".to_string(),
            ResourceType::Human,
            "COMP-001".to_string(),
            "user-001".to_string(),
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_resource_skill_management() {
        let mut resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            ResourceType::Human,
            "COMP-001".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Adicionar skill
        let skill = Skill {
            id: "SKILL-001".to_string(),
            name: "Rust Programming".to_string(),
            level: SkillLevel::Advanced,
            years_experience: Some(3),
            certified: true,
        };
        
        let result = resource.add_skill(skill);
        assert!(result.is_ok());
        assert_eq!(resource.skills.len(), 1);
        
        // Verificar se tem a skill
        assert!(resource.has_skill("Rust Programming", None));
        assert!(resource.has_skill("Rust Programming", Some(SkillLevel::Intermediate)));
        assert!(!resource.has_skill("Rust Programming", Some(SkillLevel::Expert)));
        
        // Remover skill
        let result = resource.remove_skill("Rust Programming");
        assert!(result.is_ok());
        assert_eq!(resource.skills.len(), 0);
    }
    
    #[test]
    fn test_resource_availability() {
        let resource = Resource::new(
            "RES-001".to_string(),
            "John Doe".to_string(),
            ResourceType::Human,
            "COMP-001".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        let today = Utc::now().date_naive();
        let weekday = today.weekday();
        
        // Verificar disponibilidade padrão (segunda a sexta)
        let is_available = match weekday {
            chrono::Weekday::Mon | chrono::Weekday::Tue | chrono::Weekday::Wed | 
            chrono::Weekday::Thu | chrono::Weekday::Fri => true,
            _ => false,
        };
        
        assert_eq!(resource.is_available_on_date(today), is_available);
    }
    
    #[test]
    fn test_resource_allocation_creation() {
        let allocation = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-001".to_string(),
            "PROJ-001".to_string(),
            80,
            Utc::now().date_naive(),
            Some(Utc::now().date_naive() + Duration::days(30)),
            "Developer".to_string(),
            "user-001".to_string(),
        );
        
        assert!(allocation.is_ok());
        
        let allocation = allocation.unwrap();
        assert_eq!(allocation.resource_id, "RES-001");
        assert_eq!(allocation.task_id, "TASK-001");
        assert_eq!(allocation.allocation_percentage, 80);
        assert_eq!(allocation.status, AllocationStatus::Planned);
    }
    
    #[test]
    fn test_resource_allocation_invalid_percentage() {
        let result = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-001".to_string(),
            "PROJ-001".to_string(),
            0, // Invalid
            Utc::now().date_naive(),
            None,
            "Developer".to_string(),
            "user-001".to_string(),
        );
        
        assert!(result.is_err());
        
        let result = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-001".to_string(),
            "PROJ-001".to_string(),
            150, // Invalid
            Utc::now().date_naive(),
            None,
            "Developer".to_string(),
            "user-001".to_string(),
        );
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_resource_allocation_lifecycle() {
        let mut allocation = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-001".to_string(),
            "PROJ-001".to_string(),
            80,
            Utc::now().date_naive(),
            None,
            "Developer".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Ativar
        let result = allocation.activate();
        assert!(result.is_ok());
        assert_eq!(allocation.status, AllocationStatus::Active);
        
        // Completar
        let result = allocation.complete();
        assert!(result.is_ok());
        assert_eq!(allocation.status, AllocationStatus::Completed);
        
        // Não pode cancelar após completar
        let result = allocation.cancel();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_allocation_overlap_detection() {
        let allocation1 = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-001".to_string(),
            "PROJ-001".to_string(),
            80,
            Utc::now().date_naive(),
            Some(Utc::now().date_naive() + Duration::days(10)),
            "Developer".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        let allocation2 = ResourceAllocation::new(
            "RES-001".to_string(),
            "TASK-002".to_string(),
            "PROJ-001".to_string(),
            60,
            Utc::now().date_naive() + Duration::days(5),
            Some(Utc::now().date_naive() + Duration::days(15)),
            "Developer".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Deve detectar sobreposição
        assert!(allocation1.is_overlapping(&allocation2));
        
        let allocation3 = ResourceAllocation::new(
            "RES-002".to_string(), // Diferente recurso
            "TASK-003".to_string(),
            "PROJ-001".to_string(),
            100,
            Utc::now().date_naive(),
            Some(Utc::now().date_naive() + Duration::days(10)),
            "Developer".to_string(),
            "user-001".to_string(),
        ).unwrap();
        
        // Não deve detectar sobreposição para recursos diferentes
        assert!(!allocation1.is_overlapping(&allocation3));
    }
    
    #[test]
    fn test_allocation_conflict_detection() {
        let conflict = AllocationConflict::new(
            "RES-001".to_string(),
            ConflictType::OverAllocation,
            ConflictSeverity::High,
            "Resource allocated more than 100%".to_string(),
            vec!["ALLOC-001".to_string(), "ALLOC-002".to_string()],
            Vec::new(),
        );
        
        assert!(conflict.requires_immediate_action());
        assert!(!conflict.is_critical());
        
        let resolution = Resolution::new(
            "Reduce allocation percentage".to_string(),
            "Adjust allocation percentages to stay within 100%".to_string(),
            Some(Duration::hours(2)),
            Some(0.0),
            Some(Duration::days(1)),
        );
        
        conflict.add_resolution(resolution);
        assert_eq!(conflict.suggested_resolutions.len(), 1);
    }
}
