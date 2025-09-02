use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use std::fmt;

#[derive(Debug)]
pub enum LinkTaskError {
    ProjectNotFound(String),
    TaskNotFound(String),
    DependencyNotFound(String),
    CircularDependencyDetected(Vec<String>),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for LinkTaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkTaskError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            LinkTaskError::TaskNotFound(code) => write!(f, "Task with code '{}' not found.", code),
            LinkTaskError::DependencyNotFound(code) => write!(f, "Dependency task with code '{}' not found.", code),
            LinkTaskError::CircularDependencyDetected(tasks) => {
                write!(f, "Circular dependency detected between tasks: {:?}", tasks)
            }
            LinkTaskError::DomainError(message) => write!(f, "Domain error: {}", message),
            LinkTaskError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for LinkTaskError {}

impl From<DomainError> for LinkTaskError {
    fn from(err: DomainError) -> Self {
        LinkTaskError::RepositoryError(err)
    }
}

/// `LinkTaskUseCase` is responsible for creating a dependency between two tasks.
pub struct LinkTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> LinkTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    pub fn new(project_repository: PR) -> Self {
        Self { project_repository }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        dependency_code: &str,
    ) -> Result<AnyTask, LinkTaskError> {
        if task_code == dependency_code {
            return Err(LinkTaskError::DomainError(
                "A task cannot depend on itself.".to_string(),
            ));
        }

        // 1. Load the project aggregate that contains the tasks.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| LinkTaskError::ProjectNotFound(project_code.to_string()))?;

        // 2. Ensure both tasks exist within the project.
        if !project.tasks().contains_key(task_code) {
            return Err(LinkTaskError::TaskNotFound(task_code.to_string()));
        }
        if !project.tasks().contains_key(dependency_code) {
            return Err(LinkTaskError::DependencyNotFound(dependency_code.to_string()));
        }

        // 3. Check for circular dependencies.
        // We perform a DFS traversal starting from the dependency to see if it eventually leads back to the original task.
        let mut stack = vec![dependency_code.to_string()];
        let mut visited = std::collections::HashSet::new();

        while let Some(current_code) = stack.pop() {
            if current_code == task_code {
                return Err(LinkTaskError::CircularDependencyDetected(vec![
                    task_code.to_string(),
                    dependency_code.to_string(),
                ]));
            }

            // To avoid infinite loops on existing cycles, we only process each node once.
            if !visited.insert(current_code.clone()) {
                continue;
            }

            if let Some(task) = project.tasks().get(&current_code) {
                let dependencies = match task {
                    AnyTask::Planned(t) => &t.dependencies,
                    AnyTask::InProgress(t) => &t.dependencies,
                    AnyTask::Blocked(t) => &t.dependencies,
                    AnyTask::Completed(t) => &t.dependencies,
                    AnyTask::Cancelled(t) => &t.dependencies,
                };
                for dep in dependencies {
                    stack.push(dep.clone());
                }
            }
        }

        // 4. Add the dependency to the task.
        let updated_task = project
            .add_dependency_to_task(task_code, dependency_code)
            .map_err(LinkTaskError::DomainError)?;

        // 5. Save the entire project aggregate with the modified task.
        self.project_repository.save(project.clone())?;

        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{any_project::AnyProject, builder::ProjectBuilder},
        task_management::{any_task::AnyTask, state::Planned, task::Task},
    };
    use chrono::NaiveDate;
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
        should_fail_save: bool, // Added for testing repository errors
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), DomainError> {
            if self.should_fail_save {
                Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::Generic {
                        message: "Simulated save failure".to_string(),
                    },
                ))
            } else {
                self.projects.borrow_mut().insert(project.code().to_string(), project);
                Ok(())
            }
        }
        fn find_by_code(&self, code: &str) -> Result<Option<AnyProject>, DomainError> {
            Ok(self.projects.borrow().get(code).cloned())
        }
        fn load(&self) -> Result<AnyProject, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
            unimplemented!()
        }
        fn get_next_code(&self) -> Result<String, DomainError> {
            unimplemented!()
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: format!("Task {code}"),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
        }
        .into()
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
    fn test_link_task_success() {
        let project = setup_test_project(vec![create_test_task("A"), create_test_task("B")]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-1", "B", "A");

        assert!(result.is_ok());
        let updated_task = result.unwrap();

        let deps = match updated_task {
            AnyTask::Planned(t) => t.dependencies,
            _ => panic!("Expected a planned task"),
        };
        assert_eq!(deps, vec!["A".to_string()]);
    }

    #[test]
    fn test_link_task_fails_if_task_not_found() {
        let project = setup_test_project(vec![create_test_task("A")]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-1", "B_NONEXISTENT", "A");
        assert!(matches!(result, Err(LinkTaskError::TaskNotFound(_))));
    }

    #[test]
    fn test_link_task_fails_if_dependency_not_found() {
        let project = setup_test_project(vec![create_test_task("A")]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-1", "A", "B_NONEXISTENT");
        assert!(matches!(result, Err(LinkTaskError::DependencyNotFound(_))));
    }

    #[test]
    fn test_link_task_fails_on_self_dependency() {
        let project = setup_test_project(vec![create_test_task("A")]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-1", "A", "A");
        assert!(matches!(result, Err(LinkTaskError::DomainError(_))));
    }

    #[test]
    fn test_link_task_fails_on_circular_dependency() {
        // B depends on A (B -> A)
        let task_a = create_test_task("A");
        let mut task_b = create_test_task("B");
        if let AnyTask::Planned(t) = &mut task_b {
            t.dependencies.push("A".to_string());
        }

        let project = setup_test_project(vec![task_a, task_b]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        // Try to create dependency A -> B, which would create a cycle (A -> B -> A)
        let result = use_case.execute("PROJ-1", "A", "B");

        assert!(matches!(result, Err(LinkTaskError::CircularDependencyDetected(_))));
    }

    #[test]
    fn test_link_task_with_different_task_states() {
        // Create tasks with different states to test the match statement
        let mut task_a = create_test_task("A");
        let mut task_b = create_test_task("B");
        let mut task_c = create_test_task("C");
        let mut task_d = create_test_task("D");
        let mut task_e = create_test_task("E");

        // Add some dependencies to test the different match arms
        if let AnyTask::Planned(t) = &mut task_a {
            t.dependencies.push("X".to_string());
        }
        if let AnyTask::InProgress(t) = &mut task_b {
            t.dependencies.push("Y".to_string());
        }
        if let AnyTask::Blocked(t) = &mut task_c {
            t.dependencies.push("Z".to_string());
        }
        if let AnyTask::Completed(t) = &mut task_d {
            t.dependencies.push("W".to_string());
        }
        if let AnyTask::Cancelled(t) = &mut task_e {
            t.dependencies.push("V".to_string());
        }

        let project = setup_test_project(vec![task_a, task_b, task_c, task_d, task_e]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        // This should succeed and test all the different task state match arms
        let result = use_case.execute("PROJ-1", "A", "B");
        assert!(result.is_ok());
    }

    #[test]
    fn test_link_task_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()), // Empty repository
            should_fail_save: false,
        };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("NONEXISTENT_PROJECT", "A", "B");
        assert!(matches!(result, Err(LinkTaskError::ProjectNotFound(_))));
    }

    #[test]
    fn test_link_task_repository_error() {
        let task_a = create_test_task("A");
        let task_b = create_test_task("B");
        let project = setup_test_project(vec![task_a, task_b]);

        // Create a mock repository that fails on save
        struct FailingMockProjectRepository {
            project: AnyProject,
        }

        impl ProjectRepository for FailingMockProjectRepository {
            fn find_by_code(&self, _code: &str) -> Result<Option<AnyProject>, DomainError> {
                Ok(Some(self.project.clone()))
            }

            fn save(&self, _project: AnyProject) -> Result<(), DomainError> {
                Err(DomainError::new(
                    crate::domain::shared::errors::DomainErrorKind::Generic {
                        message: "Repository save failed".to_string(),
                    },
                ))
            }

            fn find_all(&self) -> Result<Vec<AnyProject>, DomainError> {
                Ok(vec![self.project.clone()])
            }

            fn load(&self) -> Result<AnyProject, DomainError> {
                Ok(self.project.clone())
            }

            fn get_next_code(&self) -> Result<String, DomainError> {
                Ok("PROJ-1".to_string())
            }
        }

        let project_repo = FailingMockProjectRepository { project };
        let use_case = LinkTaskUseCase::new(project_repo);

        let result = use_case.execute("PROJ-1", "A", "B");
        assert!(result.is_err());
    }
}
