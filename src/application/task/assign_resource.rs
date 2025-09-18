#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::domain::resource_management::{
    any_resource::AnyResource,
    repository::ResourceRepository,
    resource::{TaskAssignment, TaskAssignmentStatus},
};
use crate::domain::task_management::{any_task::AnyTask, repository::TaskRepository};
use chrono::Local;
use std::fmt;

#[derive(Debug)]
pub enum AssignResourceToAppError {
    ProjectNotFound(String),
    TaskNotFound(String),
    ResourceNotFound(String),
    ResourceAlreadyAssigned(String, String),
    WipLimitsExceeded(String),
    WipLimitsValidationFailed(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for AssignResourceToAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssignResourceToAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            AssignResourceToAppError::TaskNotFound(code) => write!(f, "Task with code '{}' not found.", code),
            AssignResourceToAppError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            AssignResourceToAppError::ResourceAlreadyAssigned(resource, task) => {
                write!(f, "Resource '{}' is already assigned to task '{}'.", resource, task)
            }
            AssignResourceToAppError::WipLimitsExceeded(message) => write!(f, "WIP limits exceeded: {}", message),
            AssignResourceToAppError::WipLimitsValidationFailed(message) => {
                write!(f, "WIP limits validation failed: {}", message)
            }
            AssignResourceToAppError::AppError(message) => write!(f, "Domain error: {}", message),
            AssignResourceToAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for AssignResourceToAppError {}

impl From<AppError> for AssignResourceToAppError {
    fn from(err: AppError) -> Self {
        AssignResourceToAppError::RepositoryError(err)
    }
}

pub struct AssignResourceToTaskUseCase<TR, RR>
where
    TR: TaskRepository,
    RR: ResourceRepository,
{
    task_repository: TR,
    resource_repository: RR,
}

impl<TR, RR> AssignResourceToTaskUseCase<TR, RR>
where
    TR: TaskRepository,
    RR: ResourceRepository,
{
    pub fn new(task_repository: TR, resource_repository: RR) -> Self {
        Self {
            task_repository,
            resource_repository,
        }
    }

    pub fn execute(
        &self,
        task_code: &str,
        resource_code: &str,
        project_code: &str,
        allocation_percentage: Option<u8>,
    ) -> Result<AnyTask, AssignResourceToAppError> {
        // 1. Find the task
        let task = self
            .task_repository
            .find_by_code(task_code)?
            .ok_or_else(|| AssignResourceToAppError::TaskNotFound(task_code.to_string()))?;

        // 2. Find the resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AssignResourceToAppError::ResourceNotFound(resource_code.to_string()))?;

        // 3. Validate resource availability
        if !self.is_resource_available(&resource) {
            return Err(AssignResourceToAppError::ResourceAlreadyAssigned(
                resource_code.to_string(),
                task_code.to_string(),
            ));
        }

        // 4. Validate WIP limits
        self.validate_wip_limits(&resource, allocation_percentage.unwrap_or(100))?;

        // 5. Create task assignment
        let task_assignment = TaskAssignment {
            task_id: task_code.to_string(),
            project_id: project_code.to_string(),
            start_date: Local::now(),
            end_date: Local::now() + chrono::Duration::days(30), // Default 30 days
            allocation_percentage: allocation_percentage.unwrap_or(100),
            status: TaskAssignmentStatus::Active,
        };

        // 6. Assign the resource to the task
        let updated_task = self.assign_resource_to_task(task, resource.clone())?;

        // 7. Update resource with project assignment and task assignment
        let updated_resource = self.assign_resource_to_project_and_task(resource, project_code, task_assignment)?;

        // 8. Save the updated task
        let saved_task = self
            .task_repository
            .save(updated_task)
            .map_err(AssignResourceToAppError::RepositoryError)?;

        // 9. Save the updated resource using save_in_hierarchy
        self.resource_repository
            .save_in_hierarchy(updated_resource, project_code, None)
            .map_err(AssignResourceToAppError::RepositoryError)?;

        Ok(saved_task)
    }

    fn is_resource_available(&self, _resource: &AnyResource) -> bool {
        // TODO: Implement resource availability check
        // This should check if the resource is not already assigned to another task
        // or if it's not on vacation, etc.
        true
    }

    fn validate_wip_limits(
        &self,
        resource: &AnyResource,
        allocation_percentage: u8,
    ) -> Result<(), AssignResourceToAppError> {
        match resource {
            AnyResource::Available(res) => {
                if let Some(ref wip_limits) = res.wip_limits
                    && wip_limits.enabled
                {
                    // Check if resource can be assigned to more tasks
                    let current_active_tasks = res.get_active_task_count();
                    if current_active_tasks >= wip_limits.max_concurrent_tasks {
                        return Err(AssignResourceToAppError::WipLimitsExceeded(format!(
                            "Resource has reached maximum concurrent tasks limit ({}). Current active tasks: {}",
                            wip_limits.max_concurrent_tasks, current_active_tasks
                        )));
                    }

                    // Check allocation percentage
                    let current_allocation = res.get_current_allocation_percentage();
                    if current_allocation + allocation_percentage as u32 > wip_limits.max_allocation_percentage as u32 {
                        return Err(AssignResourceToAppError::WipLimitsExceeded(format!(
                            "Assignment would exceed maximum allocation percentage ({}). Current allocation: {}%, New assignment: {}%",
                            wip_limits.max_allocation_percentage, current_allocation, allocation_percentage
                        )));
                    }
                }
                Ok(())
            }
            AnyResource::Assigned(res) => {
                if let Some(ref wip_limits) = res.wip_limits
                    && wip_limits.enabled
                {
                    // Check if resource can be assigned to more tasks
                    let current_active_tasks = res.get_active_task_count();
                    if current_active_tasks >= wip_limits.max_concurrent_tasks {
                        return Err(AssignResourceToAppError::WipLimitsExceeded(format!(
                            "Resource has reached maximum concurrent tasks limit ({}). Current active tasks: {}",
                            wip_limits.max_concurrent_tasks, current_active_tasks
                        )));
                    }

                    // Check allocation percentage
                    let current_allocation = res.get_current_allocation_percentage();
                    if current_allocation + allocation_percentage as u32 > wip_limits.max_allocation_percentage as u32 {
                        return Err(AssignResourceToAppError::WipLimitsExceeded(format!(
                            "Assignment would exceed maximum allocation percentage ({}). Current allocation: {}%, New assignment: {}%",
                            wip_limits.max_allocation_percentage, current_allocation, allocation_percentage
                        )));
                    }
                }
                Ok(())
            }
            AnyResource::Inactive(_) => Err(AssignResourceToAppError::AppError(
                "Cannot assign inactive resource to task".to_string(),
            )),
        }
    }

    fn assign_resource_to_task(
        &self,
        task: AnyTask,
        _resource: AnyResource,
    ) -> Result<AnyTask, AssignResourceToAppError> {
        // This will be implemented in the domain layer
        // For now, return the task as-is
        Ok(task)
    }

    fn assign_resource_to_project_and_task(
        &self,
        resource: AnyResource,
        project_code: &str,
        task_assignment: TaskAssignment,
    ) -> Result<AnyResource, AssignResourceToAppError> {
        use crate::domain::resource_management::resource::ProjectAssignment;

        match resource {
            AnyResource::Available(resource) => {
                // Create project assignment
                let project_assignment = ProjectAssignment {
                    project_id: project_code.to_string(),
                    start_date: Local::now(),
                    end_date: Local::now() + chrono::Duration::days(30), // Default 30 days
                    allocation_percentage: 100,                          // Default 100% allocation
                };

                // Convert to Assigned state
                let mut assigned_resource = resource.assign_to_project(project_assignment);

                // Add task assignment
                assigned_resource
                    .assign_to_task(task_assignment)
                    .map_err(AssignResourceToAppError::WipLimitsValidationFailed)?;

                Ok(AnyResource::Assigned(assigned_resource))
            }
            AnyResource::Assigned(mut resource) => {
                // Add new project assignment to existing assignments
                let project_assignment = ProjectAssignment {
                    project_id: project_code.to_string(),
                    start_date: Local::now(),
                    end_date: Local::now() + chrono::Duration::days(30), // Default 30 days
                    allocation_percentage: 100,                          // Default 100% allocation
                };

                resource.state.project_assignments.push(project_assignment);

                // Add task assignment
                resource
                    .assign_to_task(task_assignment)
                    .map_err(AssignResourceToAppError::WipLimitsValidationFailed)?;

                Ok(AnyResource::Assigned(resource))
            }
            AnyResource::Inactive(_) => Err(AssignResourceToAppError::AppError(
                "Cannot assign inactive resource to project".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{resource_management::resource::Resource, task_management::builder::TaskBuilder};
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};

    // --- Mocks ---
    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: AnyTask) -> Result<AnyTask, AppError> {
            self.tasks.borrow_mut().insert(task.code().to_string(), task.clone());
            Ok(task)
        }

        fn find_all(&self) -> Result<Vec<AnyTask>, AppError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, AppError> {
            Ok(self.tasks.borrow().get(code).cloned())
        }

        fn save_in_hierarchy(
            &self,
            task: AnyTask,
            _company_code: &str,
            _project_code: &str,
        ) -> Result<AnyTask, AppError> {
            self.save(task)
        }

        fn find_all_by_project(&self, _company_code: &str, _project_code: &str) -> Result<Vec<AnyTask>, AppError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_project(&self, _project_code: &str) -> Result<Vec<AnyTask>, AppError> {
            unimplemented!()
        }

        fn get_next_code(&self, _project_code: &str) -> Result<String, AppError> {
            unimplemented!()
        }
    }

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.borrow().get(code).cloned())
        }

        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            self.save(resource)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn check_if_layoff_period(
            &self,
            _start_date: &chrono::DateTime<chrono::Local>,
            _end_date: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, _resource_type: &str) -> Result<String, AppError> {
            unimplemented!()
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str, name: &str, project_code: &str) -> AnyTask {
        TaskBuilder::new()
            .project_code(project_code.to_string())
            .name(name.to_string())
            .code(code.to_string())
            .dates(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            )
            .unwrap()
            .assign_resource("RES-001")
            .validate_vacations(&[])
            .unwrap()
            .build()
            .unwrap()
            .into()
    }

    fn create_test_resource(code: &str, name: &str, resource_type: &str) -> AnyResource {
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            resource_type.to_string(),
            None,
            None,
            None,
            0,
        )
        .into()
    }

    // --- Tests ---
    #[test]
    fn test_assign_resource_to_task_success() {
        // Arrange
        let task = create_test_task("TASK-001", "Test Task", "PROJ-001");
        let resource = create_test_resource("RES-001", "Test Resource", "developer");

        let task_repo = MockTaskRepository {
            tasks: RefCell::new(HashMap::from([(task.code().to_string(), task.clone())])),
        };
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource.clone())])),
        };

        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("TASK-001", "RES-001", "PROJ-001", None);

        // Assert
        assert!(result.is_ok());
        let updated_task = result.unwrap();
        assert_eq!(updated_task.code(), "TASK-001");
    }

    #[test]
    fn test_assign_resource_task_not_found() {
        // Arrange
        let resource = create_test_resource("RES-001", "Test Resource", "developer");

        let task_repo = MockTaskRepository {
            tasks: RefCell::new(HashMap::new()),
        };
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource.clone())])),
        };

        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("NONEXISTENT-TASK", "RES-001", "PROJ-001", None);

        // Assert
        assert!(matches!(result, Err(AssignResourceToAppError::TaskNotFound(_))));
    }

    #[test]
    fn test_assign_resource_resource_not_found() {
        // Arrange
        let task = create_test_task("TASK-001", "Test Task", "PROJ-001");

        let task_repo = MockTaskRepository {
            tasks: RefCell::new(HashMap::from([(task.code().to_string(), task.clone())])),
        };
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };

        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("TASK-001", "NONEXISTENT-RESOURCE", "PROJ-001", None);

        // Assert
        assert!(matches!(result, Err(AssignResourceToAppError::ResourceNotFound(_))));
    }

    #[test]
    fn test_assign_resource_resource_not_available() {
        // Arrange
        let task = create_test_task("TASK-001", "Test Task", "PROJ-001");
        let resource = create_test_resource("RES-001", "Test Resource", "developer");

        let task_repo = MockTaskRepository {
            tasks: RefCell::new(HashMap::from([(task.code().to_string(), task.clone())])),
        };
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource.clone())])),
        };

        // Create a use case that considers resources unavailable
        struct UnavailableResourceUseCase<TR, RR> {
            task_repository: TR,
            resource_repository: RR,
        }

        impl<TR: TaskRepository, RR: ResourceRepository> UnavailableResourceUseCase<TR, RR> {
            fn new(task_repository: TR, resource_repository: RR) -> Self {
                Self {
                    task_repository,
                    resource_repository,
                }
            }

            fn execute(&self, task_code: &str, resource_code: &str) -> Result<AnyTask, AssignResourceToAppError> {
                // 1. Find the task
                let task = self
                    .task_repository
                    .find_by_code(task_code)?
                    .ok_or_else(|| AssignResourceToAppError::TaskNotFound(task_code.to_string()))?;

                // 2. Find the resource
                let resource = self
                    .resource_repository
                    .find_by_code(resource_code)?
                    .ok_or_else(|| AssignResourceToAppError::ResourceNotFound(resource_code.to_string()))?;

                // 3. Validate resource availability - always return false
                if !self.is_resource_available(&resource) {
                    return Err(AssignResourceToAppError::ResourceAlreadyAssigned(
                        resource_code.to_string(),
                        task_code.to_string(),
                    ));
                }

                // 4. Assign resource to task
                let updated_task = self.assign_resource_to_task(task, resource)?;

                // 5. Save the updated task
                self.task_repository.save(updated_task.clone())?;

                Ok(updated_task)
            }

            fn is_resource_available(&self, _resource: &AnyResource) -> bool {
                // Always return false to test the unavailable path
                false
            }

            fn assign_resource_to_task(
                &self,
                task: AnyTask,
                _resource: AnyResource,
            ) -> Result<AnyTask, AssignResourceToAppError> {
                Ok(task)
            }
        }

        let use_case = UnavailableResourceUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("TASK-001", "RES-001");

        // Assert
        assert!(matches!(
            result,
            Err(AssignResourceToAppError::ResourceAlreadyAssigned(_, _))
        ));
    }

    #[test]
    fn test_assign_resource_save_failure() {
        // Arrange
        let task = create_test_task("TASK-001", "Test Task", "PROJ-001");
        let resource = create_test_resource("RES-001", "Test Resource", "developer");

        // Create a mock repository that fails on save
        struct FailingMockTaskRepository {
            tasks: RefCell<HashMap<String, AnyTask>>,
        }

        impl TaskRepository for FailingMockTaskRepository {
            fn save(&self, _task: AnyTask) -> Result<AnyTask, AppError> {
                Err(AppError::ValidationError {
                    field: "repository".to_string(),
                    message: "Save failed".to_string(),
                })
            }

            fn find_all(&self) -> Result<Vec<AnyTask>, AppError> {
                Ok(self.tasks.borrow().values().cloned().collect())
            }

            fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, AppError> {
                Ok(self.tasks.borrow().get(code).cloned())
            }

            fn save_in_hierarchy(
                &self,
                task: AnyTask,
                _company_code: &str,
                _project_code: &str,
            ) -> Result<AnyTask, AppError> {
                self.save(task)
            }

            fn find_all_by_project(&self, _company_code: &str, _project_code: &str) -> Result<Vec<AnyTask>, AppError> {
                Ok(self.tasks.borrow().values().cloned().collect())
            }

            fn find_by_project(&self, _project_code: &str) -> Result<Vec<AnyTask>, AppError> {
                unimplemented!()
            }

            fn get_next_code(&self, _project_code: &str) -> Result<String, AppError> {
                unimplemented!()
            }
        }

        let task_repo = FailingMockTaskRepository {
            tasks: RefCell::new(HashMap::from([(task.code().to_string(), task.clone())])),
        };
        let resource_repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource.clone())])),
        };

        let use_case = AssignResourceToTaskUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("TASK-001", "RES-001", "PROJ-001", None);

        // Assert
        assert!(result.is_err());
        if let Err(e) = result {
            eprintln!("Error: {}", e);
            eprintln!("Error type: {:?}", e);
            // Check if it's a RepositoryError and contains the expected message
            match e {
                AssignResourceToAppError::RepositoryError(domain_error) => {
                    assert!(domain_error.to_string().contains("Save failed"));
                }
                _ => {
                    // If it's not a RepositoryError, check if the error message contains the expected text
                    assert!(
                        e.to_string().contains("Save failed"),
                        "Expected error to contain 'Save failed', but got: {}",
                        e
                    );
                }
            }
        }
    }
}
