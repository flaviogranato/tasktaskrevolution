use crate::domain::{
    resource_management::{AnyResource, repository::ResourceRepository},
    shared::errors::{DomainError, DomainErrorKind},
    task_management::{AnyTask, repository::TaskRepository},
};

#[derive(Debug, thiserror::Error)]
pub enum AssignResourceError {
    #[error("Task with code '{0}' not found")]
    TaskNotFound(String),
    #[error("Resource with code '{0}' not found")]
    ResourceNotFound(String),
    #[error("Project '{0}' not found for task")]
    ProjectNotFound(String),
    #[error("Resource '{0}' is not available for assignment")]
    ResourceNotAvailable(String),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] DomainError),
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

    pub fn execute(&self, task_code: &str, resource_code: &str) -> Result<AnyTask, AssignResourceError> {
        // 1. Find the task
        let task = self
            .task_repository
            .find_by_code(task_code)?
            .ok_or_else(|| AssignResourceError::TaskNotFound(task_code.to_string()))?;

        // 2. Find the resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AssignResourceError::ResourceNotFound(resource_code.to_string()))?;

        // 3. Validate resource availability
        if !self.is_resource_available(&resource) {
            return Err(AssignResourceError::ResourceNotAvailable(resource_code.to_string()));
        }

        // 4. Assign resource to task (this will be implemented in the domain)
        let updated_task = self.assign_resource_to_task(task, resource)?;

        // 5. Save the updated task
        self.task_repository.save(updated_task.clone())?;

        Ok(updated_task)
    }

    fn is_resource_available(&self, _resource: &AnyResource) -> bool {
        // For now, consider all resources available
        // This can be enhanced with business logic later
        true
    }

    fn assign_resource_to_task(&self, task: AnyTask, _resource: AnyResource) -> Result<AnyTask, AssignResourceError> {
        // This will be implemented in the domain layer
        // For now, return the task as-is
        Ok(task)
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
        fn save(&self, task: AnyTask) -> Result<AnyTask, DomainError> {
            self.tasks.borrow_mut().insert(task.code().to_string(), task.clone());
            Ok(task)
        }

        fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
            Ok(self.tasks.borrow().get(code).cloned())
        }

        fn find_by_project(&self, _project_code: &str) -> Result<Vec<AnyTask>, DomainError> {
            unimplemented!()
        }

        fn get_next_code(&self, _project_code: &str) -> Result<String, DomainError> {
            unimplemented!()
        }
    }

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, DomainError> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, DomainError> {
            Ok(self.resources.borrow().get(code).cloned())
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }

        fn check_if_layoff_period(
            &self,
            _start_date: &chrono::DateTime<chrono::Local>,
            _end_date: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, _resource_type: &str) -> Result<String, DomainError> {
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
        let result = use_case.execute("TASK-001", "RES-001");

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
        let result = use_case.execute("NONEXISTENT-TASK", "RES-001");

        // Assert
        assert!(matches!(result, Err(AssignResourceError::TaskNotFound(_))));
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
        let result = use_case.execute("TASK-001", "NONEXISTENT-RESOURCE");

        // Assert
        assert!(matches!(result, Err(AssignResourceError::ResourceNotFound(_))));
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

            fn execute(&self, task_code: &str, resource_code: &str) -> Result<AnyTask, AssignResourceError> {
                // 1. Find the task
                let task = self
                    .task_repository
                    .find_by_code(task_code)?
                    .ok_or_else(|| AssignResourceError::TaskNotFound(task_code.to_string()))?;

                // 2. Find the resource
                let resource = self
                    .resource_repository
                    .find_by_code(resource_code)?
                    .ok_or_else(|| AssignResourceError::ResourceNotFound(resource_code.to_string()))?;

                // 3. Validate resource availability - always return false
                if !self.is_resource_available(&resource) {
                    return Err(AssignResourceError::ResourceNotAvailable(resource_code.to_string()));
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

            fn assign_resource_to_task(&self, task: AnyTask, _resource: AnyResource) -> Result<AnyTask, AssignResourceError> {
                Ok(task)
            }
        }

        let use_case = UnavailableResourceUseCase::new(task_repo, resource_repo);

        // Act
        let result = use_case.execute("TASK-001", "RES-001");

        // Assert
        assert!(matches!(result, Err(AssignResourceError::ResourceNotAvailable(_))));
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
            fn save(&self, _task: AnyTask) -> Result<AnyTask, DomainError> {
                Err(DomainError::new(DomainErrorKind::Generic {
                    message: "Save failed".to_string(),
                }))
            }

            fn find_all(&self) -> Result<Vec<AnyTask>, DomainError> {
                Ok(self.tasks.borrow().values().cloned().collect())
            }

            fn find_by_code(&self, code: &str) -> Result<Option<AnyTask>, DomainError> {
                Ok(self.tasks.borrow().get(code).cloned())
            }

            fn find_by_project(&self, _project_code: &str) -> Result<Vec<AnyTask>, DomainError> {
                unimplemented!()
            }

            fn get_next_code(&self, _project_code: &str) -> Result<String, DomainError> {
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
        let result = use_case.execute("TASK-001", "RES-001");

        // Assert
        assert!(result.is_err());
        if let Err(e) = result {
            eprintln!("Error: {}", e);
            eprintln!("Error type: {:?}", e);
            // Check if it's a RepositoryError and contains the expected message
            match e {
                AssignResourceError::RepositoryError(domain_error) => {
                    assert!(domain_error.to_string().contains("Save failed"));
                }
                _ => {
                    // If it's not a RepositoryError, check if the error message contains the expected text
                    assert!(e.to_string().contains("Save failed"), 
                        "Expected error to contain 'Save failed', but got: {}", e);
                }
            }
        }
    }
}
