#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
use crate::domain::task_management::{Category, Priority, any_task::AnyTask};
use std::fmt;

#[derive(Debug)]
pub enum RemoveDependencyError {
    ProjectNotFound(String),
    TaskNotFound(String),
    DependencyNotFound(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for RemoveDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoveDependencyError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            RemoveDependencyError::TaskNotFound(code) => write!(f, "Task with code '{}' not found.", code),
            RemoveDependencyError::DependencyNotFound(code) => write!(f, "Dependency '{}' not found for task.", code),
            RemoveDependencyError::AppError(message) => write!(f, "Domain error: {}", message),
            RemoveDependencyError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for RemoveDependencyError {}

impl From<AppError> for RemoveDependencyError {
    fn from(err: AppError) -> Self {
        RemoveDependencyError::RepositoryError(err)
    }
}

/// `RemoveTaskDependencyUseCase` is responsible for removing a dependency between two tasks.
pub struct RemoveTaskDependencyUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    code_resolver: CR,
}

impl<PR, CR> RemoveTaskDependencyUseCase<PR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    CR: CodeResolverTrait,
{
    pub fn new(project_repository: PR, code_resolver: CR) -> Self {
        Self {
            project_repository,
            code_resolver,
        }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        dependency_code: &str,
    ) -> Result<AnyTask, RemoveDependencyError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(RemoveDependencyError::RepositoryError)?;

        // 2. Load the project aggregate using ID
        let mut project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| RemoveDependencyError::ProjectNotFound(project_code.to_string()))?;

        // 3. Ensure both tasks exist within the project
        if !project.tasks().contains_key(task_code) {
            return Err(RemoveDependencyError::TaskNotFound(task_code.to_string()));
        }
        if !project.tasks().contains_key(dependency_code) {
            return Err(RemoveDependencyError::DependencyNotFound(dependency_code.to_string()));
        }

        // 4. Check if the dependency actually exists
        let task = project.tasks().get(task_code).unwrap();
        let dependencies = match task {
            AnyTask::Planned(t) => &t.dependencies,
            AnyTask::InProgress(t) => &t.dependencies,
            AnyTask::Blocked(t) => &t.dependencies,
            AnyTask::Completed(t) => &t.dependencies,
            AnyTask::Cancelled(t) => &t.dependencies,
        };

        if !dependencies.contains(&dependency_code.to_string()) {
            return Err(RemoveDependencyError::DependencyNotFound(dependency_code.to_string()));
        }

        // 5. Validate that removing the dependency won't break critical constraints
        if self.is_task_blocked_by_dependency(&project, task_code, dependency_code)? {
            return Err(RemoveDependencyError::AppError(
                "Cannot remove dependency: task is currently blocked by another dependency.".to_string(),
            ));
        }

        // 6. Remove the dependency from the task
        let updated_task = project
            .remove_dependency_from_task(task_code, dependency_code)
            .map_err(RemoveDependencyError::AppError)?;

        // 7. Save the updated project
        self.project_repository.save(project.clone())?;

        Ok(updated_task)
    }

    fn is_task_blocked_by_dependency(
        &self,
        project: &crate::domain::project_management::AnyProject,
        task_code: &str,
        dependency_code: &str,
    ) -> Result<bool, RemoveDependencyError> {
        // This is a simplified check - in a real system, you might want more sophisticated validation
        // For now, we'll just check if the task is currently blocked and the dependency is the only blocker

        let task = project.tasks().get(task_code).unwrap();
        let dependencies = match task {
            AnyTask::Planned(t) => &t.dependencies,
            AnyTask::InProgress(t) => &t.dependencies,
            AnyTask::Blocked(t) => &t.dependencies,
            AnyTask::Completed(t) => &t.dependencies,
            AnyTask::Cancelled(t) => &t.dependencies,
        };

        // If task has only one dependency and it's the one we're trying to remove,
        // and the task is currently blocked, we should prevent removal
        if dependencies.len() == 1 && dependencies.contains(&dependency_code.to_string()) {
            // Check if task is in blocked state
            match task {
                AnyTask::Blocked(_) => return Ok(true),
                _ => return Ok(false),
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        task_management::{state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), AppError> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, AppError> {
            Ok(self.projects.borrow().get(code).cloned())
        }

        fn load(&self) -> Result<AnyProject, AppError> {
            unimplemented!()
        }

        fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
            unimplemented!()
        }

        fn get_next_code(&self) -> Result<String, AppError> {
            unimplemented!()
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, id: &str) -> Result<Option<AnyProject>, AppError> {
            // For testing, map "project-id" to the first project
            if id == "project-id" {
                Ok(self.projects.borrow().values().next().cloned())
            } else {
                Ok(None)
            }
        }
    }

    struct MockCodeResolver {
        should_fail: bool,
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_project_code(&self, _code: &str) -> Result<String, AppError> {
            if self.should_fail {
                Err(AppError::ValidationError {
                    field: "code_resolver".to_string(),
                    message: "Mock failure".to_string(),
                })
            } else {
                Ok("project-id".to_string())
            }
        }

        fn resolve_resource_code(&self, _code: &str) -> Result<String, AppError> {
            Ok("resource-id".to_string())
        }

        fn resolve_task_code(&self, _code: &str) -> Result<String, AppError> {
            Ok("task-id".to_string())
        }

        fn resolve_company_code(&self, _code: &str) -> Result<String, AppError> {
            Ok("company-id".to_string())
        }

        fn validate_company_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn validate_project_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn validate_resource_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn validate_task_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str, dependencies: Vec<String>) -> Task<Planned> {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: format!("Test Task {}", code),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies,
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        }
    }

    fn setup_test_project(tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new()
            .name("Test Project".to_string())
            .code("PROJ-1".to_string())
            .company_code("COMP-001".to_string())
            .created_by("system".to_string())
            .build()
            .unwrap()
            .into();
        for task in tasks {
            project.add_task(task);
        }
        project
    }

    // --- Tests ---
    #[test]
    fn test_remove_dependency_success() {
        // Arrange
        let task_a = create_test_task("TASK-A", vec!["TASK-B".to_string()]);
        let task_b = create_test_task("TASK-B", vec![]);

        let project = setup_test_project(vec![task_a.into(), task_b.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project.clone())])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(result.is_ok());
        let updated_task = result.unwrap();
        assert_eq!(updated_task.code(), "TASK-A");

        // Verify dependency was removed by checking the returned task
        let dependencies = match updated_task {
            AnyTask::Planned(t) => t.dependencies.clone(),
            _ => panic!("Expected Planned task"),
        };
        assert!(!dependencies.contains(&"TASK-B".to_string()));
    }

    #[test]
    fn test_remove_dependency_project_not_found() {
        // Arrange
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("NONEXISTENT-PROJ", "TASK-A", "TASK-B");

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::ProjectNotFound(_))));
    }

    #[test]
    fn test_remove_dependency_task_not_found() {
        // Arrange
        let task_b = create_test_task("TASK-B", vec![]);
        let project = setup_test_project(vec![task_b.into()]);

        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "NONEXISTENT-TASK", "TASK-B");

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::TaskNotFound(_))));
    }

    #[test]
    fn test_remove_dependency_not_found() {
        // Arrange
        let task_a = create_test_task("TASK-A", vec![]);
        let task_b = create_test_task("TASK-B", vec![]);

        let project = setup_test_project(vec![task_a.into(), task_b.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::DependencyNotFound(_))));
    }

    #[test]
    fn test_remove_dependency_dependency_not_found_in_task() {
        // Arrange
        let task_a = create_test_task("TASK-A", vec!["TASK-C".to_string()]); // TASK-A depends on TASK-C
        let task_b = create_test_task("TASK-B", vec![]); // TASK-B has no dependencies

        let project = setup_test_project(vec![task_a.into(), task_b.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B"); // Try to remove TASK-B from TASK-A's dependencies

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::DependencyNotFound(_))));
    }

    #[test]
    fn test_remove_dependency_task_blocked_by_dependency() {
        // Arrange
        use crate::domain::task_management::state::Blocked;

        let task_a = create_test_task("TASK-A", vec!["TASK-B".to_string()]);
        let task_b = create_test_task("TASK-B", vec![]);

        // Convert task_a to blocked state
        let blocked_task = Task::<Blocked> {
            id: task_a.id,
            project_code: task_a.project_code,
            code: task_a.code,
            name: task_a.name,
            description: task_a.description,
            state: Blocked {
                reason: "Waiting for dependency".to_string(),
            },
            start_date: task_a.start_date,
            due_date: task_a.due_date,
            actual_end_date: task_a.actual_end_date,
            dependencies: task_a.dependencies,
            assigned_resources: task_a.assigned_resources,
            priority: task_a.priority,
            category: task_a.category,
        };

        let project = setup_test_project(vec![blocked_task.into(), task_b.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::AppError(_))));
    }

    #[test]
    fn test_remove_dependency_task_not_blocked_by_dependency() {
        // Arrange
        use crate::domain::task_management::state::Blocked;

        let task_a = create_test_task("TASK-A", vec!["TASK-B".to_string(), "TASK-C".to_string()]); // Has 2 dependencies
        let task_b = create_test_task("TASK-B", vec![]);
        let task_c = create_test_task("TASK-C", vec![]);

        // Convert task_a to blocked state
        let blocked_task = Task::<Blocked> {
            id: task_a.id,
            project_code: task_a.project_code,
            code: task_a.code,
            name: task_a.name,
            description: task_a.description,
            state: Blocked {
                reason: "Waiting for dependency".to_string(),
            },
            start_date: task_a.start_date,
            due_date: task_a.due_date,
            actual_end_date: task_a.actual_end_date,
            dependencies: task_a.dependencies,
            assigned_resources: task_a.assigned_resources,
            priority: task_a.priority,
            category: task_a.category,
        };

        let project = setup_test_project(vec![blocked_task.into(), task_b.into(), task_c.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project.clone())])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(result.is_ok()); // Should succeed because task has multiple dependencies
    }

    #[test]
    fn test_remove_dependency_task_not_blocked() {
        // Arrange
        let task_a = create_test_task("TASK-A", vec!["TASK-B".to_string()]);
        let task_b = create_test_task("TASK-B", vec![]);

        let project = setup_test_project(vec![task_a.into(), task_b.into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project.clone())])),
        };

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(project_repo, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(result.is_ok()); // Should succeed because task is not blocked
    }

    #[test]
    fn test_remove_dependency_repository_error() {
        // Arrange
        #[allow(unused_variables)]
        let task_a = create_test_task("TASK-A", vec!["TASK-B".to_string()]);
        #[allow(unused_variables)]
        let task_b = create_test_task("TASK-B", vec![]);

        // Create a mock repository that fails on save
        struct FailingMockProjectRepository;

        impl ProjectRepository for FailingMockProjectRepository {
            fn save(&self, _project: AnyProject) -> Result<(), AppError> {
                Err(AppError::ValidationError {
                    field: "repository".to_string(),
                    message: "Repository save failed".to_string(),
                })
            }

            fn find_by_code(&self, _code: &str) -> Result<Option<AnyProject>, AppError> {
                let project = setup_test_project(vec![
                    create_test_task("TASK-A", vec!["TASK-B".to_string()]).into(),
                    create_test_task("TASK-B", vec![]).into(),
                ]);
                Ok(Some(project))
            }

            fn load(&self) -> Result<AnyProject, AppError> {
                unimplemented!()
            }

            fn find_all(&self) -> Result<Vec<AnyProject>, AppError> {
                unimplemented!()
            }

            fn get_next_code(&self) -> Result<String, AppError> {
                unimplemented!()
            }
        }

        impl ProjectRepositoryWithId for FailingMockProjectRepository {
            fn find_by_id(&self, id: &str) -> Result<Option<AnyProject>, AppError> {
                if id == "project-id" {
                    let project = setup_test_project(vec![
                        create_test_task("TASK-A", vec!["TASK-B".to_string()]).into(),
                        create_test_task("TASK-B", vec![]).into(),
                    ]);
                    Ok(Some(project))
                } else {
                    Ok(None)
                }
            }
        }

        let code_resolver = MockCodeResolver { should_fail: false };
        let use_case = RemoveTaskDependencyUseCase::new(FailingMockProjectRepository, code_resolver);

        // Act
        let result = use_case.execute("PROJ-1", "TASK-A", "TASK-B");

        // Assert
        assert!(matches!(result, Err(RemoveDependencyError::RepositoryError(_))));
    }
}
