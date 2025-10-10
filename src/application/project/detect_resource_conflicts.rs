//! Resource Conflict Detection Use Case
//!
//! This module implements the use case for detecting resource conflicts
//! during task creation and resource assignment.

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::repository::TaskRepository;
use chrono::NaiveDate;
use std::collections::HashMap;

/// Resource conflict detection use case
pub struct DetectResourceConflictsUseCase {
    #[allow(dead_code)]
    project_repository: Box<dyn ProjectRepository>,
    resource_repository: Box<dyn ResourceRepository>,
    task_repository: Box<dyn TaskRepository>,
    code_resolver: Box<dyn CodeResolverTrait>,
}

/// Resource conflict information
#[derive(Debug, Clone)]
pub struct ResourceConflict {
    pub resource_code: String,
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub message: String,
    pub conflicting_tasks: Vec<String>,
    pub suggested_resolutions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    DoubleBooking,           // Resource assigned to multiple tasks at same time
    VacationConflict,       // Resource on vacation during task period
    CapacityExceeded,       // Resource allocation exceeds capacity
    SkillMismatch,          // Resource lacks required skills
    AvailabilityConflict,   // Resource not available during task period
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictSeverity {
    Low,      // Minor conflict, can be resolved easily
    Medium,   // Moderate conflict, requires attention
    High,     // Major conflict, impacts project timeline
    Critical, // Critical conflict, project cannot proceed
}

impl DetectResourceConflictsUseCase {
    pub fn new(
        project_repository: Box<dyn ProjectRepository>,
        resource_repository: Box<dyn ResourceRepository>,
        task_repository: Box<dyn TaskRepository>,
        code_resolver: Box<dyn CodeResolverTrait>,
    ) -> Self {
        Self {
            project_repository,
            resource_repository,
            task_repository,
            code_resolver,
        }
    }

    /// Detect conflicts for a specific resource during a time period
    pub fn detect_conflicts_for_resource(
        &self,
        resource_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        exclude_task_code: Option<&str>,
    ) -> Result<Vec<ResourceConflict>, AppError> {
        let mut conflicts = Vec::new();

        // Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(AppError::from)?;

        // Load resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AppError::validation_error("resource", "Resource not found"))?;

        // Check vacation conflicts
        if let Some(vacation_conflict) = self.check_vacation_conflict(&resource, start_date, end_date) {
            conflicts.push(vacation_conflict);
        }

        // Check availability conflicts
        if let Some(availability_conflict) = self.check_availability_conflict(&resource, start_date, end_date) {
            conflicts.push(availability_conflict);
        }

        // Check double booking conflicts
        if let Some(double_booking_conflicts) = self.check_double_booking_conflicts(
            &resource_id,
            start_date,
            end_date,
            exclude_task_code,
        )? {
            conflicts.extend(double_booking_conflicts);
        }

        // Check capacity conflicts
        if let Some(capacity_conflict) = self.check_capacity_conflict(&resource, start_date, end_date) {
            conflicts.push(capacity_conflict);
        }

        Ok(conflicts)
    }

    /// Detect conflicts for multiple resources
    pub fn detect_conflicts_for_resources(
        &self,
        resource_codes: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
        exclude_task_code: Option<&str>,
    ) -> Result<HashMap<String, Vec<ResourceConflict>>, AppError> {
        let mut all_conflicts = HashMap::new();

        for resource_code in resource_codes {
            let conflicts = self.detect_conflicts_for_resource(
                resource_code,
                start_date,
                end_date,
                exclude_task_code,
            )?;
            
            if !conflicts.is_empty() {
                all_conflicts.insert(resource_code.clone(), conflicts);
            }
        }

        Ok(all_conflicts)
    }

    /// Check if resource is on vacation during the specified period
    fn check_vacation_conflict(
        &self,
        resource: &crate::domain::resource_management::any_resource::AnyResource,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Option<ResourceConflict> {
        // Check if resource has vacation periods that overlap with task dates
        for date in start_date.iter_days().take_while(|&d| d <= end_date) {
            if !resource.is_available_on_date(date) {
                return Some(ResourceConflict {
                    resource_code: resource.code().to_string(),
                    conflict_type: ConflictType::VacationConflict,
                    severity: ConflictSeverity::High,
                    message: format!(
                        "Resource {} is not available on {} (vacation or holiday)",
                        resource.code(),
                        date
                    ),
                    conflicting_tasks: vec![],
                    suggested_resolutions: vec![
                        "Adjust task dates to avoid vacation period".to_string(),
                        "Assign alternative resource".to_string(),
                        "Reschedule task for when resource is available".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Check if resource is available during working hours
    fn check_availability_conflict(
        &self,
        resource: &crate::domain::resource_management::any_resource::AnyResource,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Option<ResourceConflict> {
        // Check if any date in the range is not a working day
        for date in start_date.iter_days().take_while(|&d| d <= end_date) {
            if !resource.is_available_on_date(date) {
                return Some(ResourceConflict {
                    resource_code: resource.code().to_string(),
                    conflict_type: ConflictType::AvailabilityConflict,
                    severity: ConflictSeverity::Medium,
                    message: format!(
                        "Resource {} is not available on {} (non-working day)",
                        resource.code(),
                        date
                    ),
                    conflicting_tasks: vec![],
                    suggested_resolutions: vec![
                        "Adjust task dates to working days only".to_string(),
                        "Assign alternative resource".to_string(),
                    ],
                });
            }
        }

        None
    }

    /// Check for double booking conflicts
    fn check_double_booking_conflicts(
        &self,
        resource_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        exclude_task_code: Option<&str>,
    ) -> Result<Option<Vec<ResourceConflict>>, AppError> {
        // Find all tasks that use this resource in the same time period
        let all_tasks = self.task_repository.find_all()?;
        let mut conflicting_tasks = Vec::new();

        for task in all_tasks {
            // Skip the task we're excluding (for updates)
            if let Some(exclude_code) = exclude_task_code
                && task.code() == exclude_code {
                continue;
            }

            // Check if task uses this resource and overlaps in time
            if task.assigned_resources().contains(&resource_id.to_string()) {
                let task_start = task.start_date();
                let task_end = task.due_date();

                // Check for date overlap
                if start_date <= *task_end && end_date >= *task_start {
                    conflicting_tasks.push(task.code().to_string());
                }
            }
        }

        if conflicting_tasks.is_empty() {
            return Ok(None);
        }

        // Resolve resource ID back to code for the conflict message
        let resource_code = resource_id.to_string();

        Ok(Some(vec![ResourceConflict {
            resource_code: resource_code.clone(),
            conflict_type: ConflictType::DoubleBooking,
            severity: ConflictSeverity::Critical,
            message: format!(
                "Resource {} is already assigned to {} other tasks during this period",
                resource_code,
                conflicting_tasks.len()
            ),
            conflicting_tasks,
            suggested_resolutions: vec![
                "Reduce resource allocation percentage".to_string(),
                "Adjust task dates to avoid overlap".to_string(),
                "Assign alternative resource".to_string(),
                "Reschedule conflicting tasks".to_string(),
            ],
        }]))
    }

    /// Check if resource capacity would be exceeded
    fn check_capacity_conflict(
        &self,
        resource: &crate::domain::resource_management::any_resource::AnyResource,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Option<ResourceConflict> {
        // This is a simplified check - in a real implementation,
        // you would check current allocations and calculate total percentage
        // For now, we'll just check if the resource can handle 100% allocation
        let max_allocation = resource.max_allocation_percentage();
        if max_allocation < 100 {
            return Some(ResourceConflict {
                resource_code: resource.code().to_string(),
                conflict_type: ConflictType::CapacityExceeded,
                severity: ConflictSeverity::Medium,
                message: format!(
                    "Resource {} cannot be allocated at 100% capacity (max: {}%)",
                    resource.code(),
                    max_allocation
                ),
                conflicting_tasks: vec![],
                suggested_resolutions: vec![
                    "Reduce allocation percentage".to_string(),
                    "Extend task duration".to_string(),
                    "Assign additional resources".to_string(),
                ],
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::AnyProject;
    use crate::domain::project_management::repository::ProjectRepositoryWithId;
    use crate::domain::resource_management::any_resource::AnyResource;
    use crate::domain::resource_management::repository::ResourceRepositoryWithId;
    use crate::domain::resource_management::resource::Resource;
    use crate::domain::resource_management::resource::ResourceScope;
    use crate::domain::task_management::any_task::AnyTask;
    use crate::domain::task_management::repository::TaskRepositoryWithId;
    use crate::domain::task_management::task::Task;
    use crate::domain::task_management::state::Planned;
    use crate::domain::task_management::{Category, Priority};
    use crate::domain::shared::errors::{DomainError, DomainResult};
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use uuid7::Uuid;

    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    impl MockProjectRepository {
        fn new() -> Self {
            Self {
                projects: RefCell::new(HashMap::new()),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn load(&self) -> DomainResult<AnyProject> {
            Err(DomainError::validation_error("project", "Not implemented in mock"))
        }

        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            Ok(self.projects.borrow().values().cloned().collect())
        }

        fn save(&self, project: AnyProject) -> DomainResult<()> {
            self.projects.borrow_mut().insert(project.id().to_string(), project);
            Ok(())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().values().find(|p| p.code() == code).cloned())
        }

        fn get_next_code(&self) -> DomainResult<String> {
            Ok("PROJ-001".to_string())
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().get(id).cloned())
        }
    }

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl MockResourceRepository {
        fn new() -> Self {
            Self {
                resources: RefCell::new(HashMap::new()),
            }
        }

        fn add_resource(&self, resource: AnyResource) {
            self.resources.borrow_mut().insert(resource.id().to_string(), resource);
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            let resource_id = resource.id().to_string();
            self.resources.borrow_mut().insert(resource_id.clone(), resource.clone());
            Ok(resource)
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(self.resources.borrow().values().cloned().map(|r| (r, "company".to_string(), vec![])).collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().values().find(|r| r.code() == code).cloned())
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<AnyResource> {
            Err(DomainError::validation_error("resource", "Not implemented in mock"))
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            Err(DomainError::validation_error("resource", "Not implemented in mock"))
        }

        fn check_if_layoff_period(&self, _start_date: &chrono::DateTime<chrono::Local>, _end_date: &chrono::DateTime<chrono::Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            Ok("RES-001".to_string())
        }
    }

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().get(id).cloned())
        }
    }

    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl MockTaskRepository {
        fn new() -> Self {
            Self {
                tasks: RefCell::new(HashMap::new()),
            }
        }

        fn add_task(&self, task: AnyTask) {
            self.tasks.borrow_mut().insert(task.id().to_string(), task);
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: AnyTask) -> DomainResult<AnyTask> {
            self.tasks.borrow_mut().insert(task.code().to_string(), task.clone());
            Ok(task)
        }

        fn save_in_hierarchy(&self, task: AnyTask, _company_code: &str, _project_code: &str) -> DomainResult<AnyTask> {
            self.save(task)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyTask>> {
            Ok(self.tasks.borrow().values().find(|t| t.code() == code).cloned())
        }

        fn find_by_project(&self, _project_code: &str) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_all_by_project(&self, _company_code: &str, _project_code: &str) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn get_next_code(&self, _project_code: &str) -> DomainResult<String> {
            Ok("TASK-001".to_string())
        }
    }

    impl TaskRepositoryWithId for MockTaskRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyTask>> {
            Ok(self.tasks.borrow().get(id).cloned())
        }
    }

    struct MockCodeResolver {
        resource_codes: RefCell<HashMap<String, String>>, // code -> id
        resource_ids: RefCell<HashMap<String, String>>,   // id -> code
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                resource_codes: RefCell::new(HashMap::new()),
                resource_ids: RefCell::new(HashMap::new()),
            }
        }

        fn add_resource(&self, code: &str, id: &str) {
            self.resource_codes.borrow_mut().insert(code.to_string(), id.to_string());
            self.resource_ids.borrow_mut().insert(id.to_string(), code.to_string());
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
        }

        fn resolve_resource_code(&self, code: &str) -> DomainResult<String> {
            self.resource_codes.borrow().get(code).cloned().ok_or_else(|| {
                DomainError::from(AppError::validation_error(
                    "resource",
                    format!("Resource '{}' not found", code),
                ))
            })
        }

        fn resolve_task_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }

        fn validate_company_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn validate_project_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
        }

        fn validate_resource_code(&self, code: &str) -> DomainResult<()> {
            self.resolve_resource_code(code)?;
            Ok(())
        }

        fn validate_task_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }

    }

    fn create_test_resource(code: &str) -> AnyResource {
        Resource::new(
            code.to_string(),
            format!("Test {}", code),
            None,
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            0,
        )
        .into()
    }

    fn create_test_task(code: &str, resource_id: &str, start_date: NaiveDate, end_date: NaiveDate) -> AnyTask {
        Task::<Planned> {
            id: Uuid::from_fields_v7(chrono::Utc::now().timestamp_millis() as u64, 0, 0),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: format!("Test {}", code),
            description: None,
            state: Planned,
            start_date,
            due_date: end_date,
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![resource_id.to_string()],
            priority: Priority::default(),
            category: Category::default(),
        }
        .into()
    }

    #[test]
    fn test_detect_vacation_conflict() {
        let project_repo = Box::new(MockProjectRepository::new());
        let resource_repo = Box::new(MockResourceRepository::new());
        let task_repo = Box::new(MockTaskRepository::new());
        let code_resolver = Box::new(MockCodeResolver::new());

        let resource = create_test_resource("DEV-001");
        resource_repo.add_resource(resource.clone());
        code_resolver.add_resource("DEV-001", &resource.id().to_string());

        let use_case = DetectResourceConflictsUseCase::new(
            project_repo,
            resource_repo,
            task_repo,
            code_resolver,
        );

        // Use only weekdays to avoid weekend conflicts
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(); // Wednesday
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(); // Friday

        let conflicts = use_case
            .detect_conflicts_for_resource("DEV-001", start_date, end_date, None)
            .unwrap();

        // Should not have conflicts for a basic resource
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_detect_double_booking_conflict() {
        let project_repo = Box::new(MockProjectRepository::new());
        let resource_repo = Box::new(MockResourceRepository::new());
        let task_repo = Box::new(MockTaskRepository::new());
        let code_resolver = Box::new(MockCodeResolver::new());

        let resource = create_test_resource("DEV-001");
        resource_repo.add_resource(resource.clone());
        code_resolver.add_resource("DEV-001", &resource.id().to_string());

        // Add an existing task that uses the same resource
        let existing_task = create_test_task(
            "TASK-001",
            &resource.id().to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 5).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        );
        task_repo.add_task(existing_task);

        let use_case = DetectResourceConflictsUseCase::new(
            project_repo,
            resource_repo,
            task_repo,
            code_resolver,
        );

        let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();

        let conflicts = use_case
            .detect_conflicts_for_resource("DEV-001", start_date, end_date, None)
            .unwrap();

        // Should detect double booking conflict
        assert!(!conflicts.is_empty());
        assert!(conflicts.iter().any(|c| c.conflict_type == ConflictType::DoubleBooking));
    }
}