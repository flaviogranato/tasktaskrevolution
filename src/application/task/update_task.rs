#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
use crate::domain::task_management::{Category, Priority, any_task::AnyTask, repository::TaskRepository};
use chrono::NaiveDate;
use std::fmt;

#[derive(Debug)]
pub enum UpdateAppError {
    ProjectNotFound(String),
    TaskNotFound(String),
    AppError(String),
    RepositoryError(AppError),
}

impl fmt::Display for UpdateAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateAppError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            UpdateAppError::TaskNotFound(code) => write!(f, "Task with code '{}' not found in project.", code),
            UpdateAppError::AppError(message) => write!(f, "An unexpected domain rule was violated: {}", message),
            UpdateAppError::RepositoryError(err) => write!(f, "A repository error occurred: {}", err),
        }
    }
}

impl std::error::Error for UpdateAppError {}

impl From<AppError> for UpdateAppError {
    fn from(err: AppError) -> Self {
        UpdateAppError::RepositoryError(err)
    }
}

impl From<crate::domain::shared::errors::DomainError> for UpdateAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        UpdateAppError::RepositoryError(err.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTaskArgs {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
}

pub struct UpdateTaskUseCase<PR, TR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    TR: TaskRepository,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    task_repository: TR,
    code_resolver: CR,
}

impl<PR, TR, CR> UpdateTaskUseCase<PR, TR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    TR: TaskRepository,
    CR: CodeResolverTrait,
{
    pub fn new(project_repository: PR, task_repository: TR, code_resolver: CR) -> Self {
        Self {
            project_repository,
            task_repository,
            code_resolver,
        }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        args: UpdateTaskArgs,
    ) -> Result<AnyTask, UpdateAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(UpdateAppError::RepositoryError)?;

        // 2. Load the project aggregate using ID
        let mut project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| UpdateAppError::ProjectNotFound(project_code.to_string()))?;

        // Check if a reschedule is needed before args is moved.
        let needs_reschedule = args.due_date.is_some();

        // 3. Delegate the update to the project aggregate.
        // This method ensures all domain invariants are respected.
        let updated_task = project
            .update_task(task_code, args.name, args.description, args.start_date, args.due_date)
            .map_err(UpdateAppError::AppError)?;

        // 4. If the due date was changed, reschedule all dependent tasks.
        if needs_reschedule {
            project
                .reschedule_dependents_of(task_code)
                .map_err(UpdateAppError::AppError)?;
        }

        // 5. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 6. Save the updated task individually in the project's tasks directory
        self.task_repository
            .save_in_hierarchy(updated_task.clone(), project.company_code(), project_code)?;

        // 7. Return the updated task to the caller.
        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        task_management::{repository::TaskRepository, state::Planned, task::Task},
    };
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    struct MockCodeResolver {
        // Mock doesn't need to resolve anything for UpdateTaskUseCase
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {}
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn resolve_project_code(&self, _code: &str) -> Result<String, AppError> {
            Ok("mock-project-id".to_string())
        }

        fn resolve_resource_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("resource", "Not implemented in mock"))
        }

        fn resolve_task_code(&self, _code: &str) -> Result<String, AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }

        fn validate_company_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("company", "Not implemented in mock"))
        }

        fn validate_project_code(&self, _code: &str) -> Result<(), AppError> {
            Ok(())
        }

        fn validate_resource_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("resource", "Not implemented in mock"))
        }

        fn validate_task_code(&self, _code: &str) -> Result<(), AppError> {
            Err(AppError::validation_error("task", "Not implemented in mock"))
        }
    }

    impl MockTaskRepository {
        fn new() -> Self {
            Self {
                tasks: RefCell::new(HashMap::new()),
            }
        }
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
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn get_next_code(&self, _project_code: &str) -> Result<String, AppError> {
            Ok("TASK-001".to_string())
        }
    }

    // --- Mocks ---
    #[derive(Clone)]
    struct MockProjectRepository {
        projects: Rc<RefCell<HashMap<String, AnyProject>>>,
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
        fn find_by_id(&self, _id: &str) -> Result<Option<AnyProject>, AppError> {
            // For tests, we'll return the first project in the map
            Ok(self.projects.borrow().values().next().cloned())
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str, name: &str) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: Some("Initial Description".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
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

    // TODO: Enable this test once `AnyProject::update_task` is implemented.
    #[test]
    fn test_update_task_name_success() {
        let project = setup_test_project(vec![create_test_task("TSK-1", "Old Name")]);
        let project_repo = MockProjectRepository {
            projects: Rc::new(RefCell::new(HashMap::from([(project.code().to_string(), project)]))),
        };
        let task_repo = MockTaskRepository::new();
        let code_resolver = MockCodeResolver::new();
        let use_case = UpdateTaskUseCase::new(project_repo, task_repo, code_resolver);

        let args = UpdateTaskArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-1", "TSK-1", args);

        assert!(result.is_ok());
        let updated_task = result.unwrap();
        assert_eq!(updated_task.name(), "New Name");
        assert_eq!(updated_task.description().unwrap(), "Initial Description");
    }

    #[test]
    fn test_update_task_fails_if_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: Rc::new(RefCell::new(HashMap::new())),
        };
        let task_repo = MockTaskRepository::new();
        let code_resolver = MockCodeResolver::new();
        let use_case = UpdateTaskUseCase::new(project_repo, task_repo, code_resolver);

        let args = UpdateTaskArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", args);
        assert!(matches!(result, Err(UpdateAppError::ProjectNotFound(_))));
    }

    fn create_task_with_deps(code: &str, start_date: NaiveDate, due_date: NaiveDate, deps: Vec<&str>) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: format!("Task {code}"),
            description: None,
            state: Planned,
            start_date,
            due_date,
            actual_end_date: None,
            dependencies: deps.into_iter().map(String::from).collect(),
            assigned_resources: vec![],
            priority: Priority::default(),
            category: Category::default(),
        }
        .into()
    }

    #[test]
    fn test_update_task_reschedules_dependents() {
        let d = |day| NaiveDate::from_ymd_opt(2025, 1, day).unwrap();

        // A depends on nothing
        let task_a = create_task_with_deps("A", d(1), d(5), vec![]);
        // B depends on A
        let task_b = create_task_with_deps("B", d(6), d(10), vec!["A"]);
        // C depends on B
        let task_c = create_task_with_deps("C", d(11), d(15), vec!["B"]);

        let project = setup_test_project(vec![task_a, task_b, task_c]);
        let project_repo = MockProjectRepository {
            projects: Rc::new(RefCell::new(HashMap::from([(project.code().to_string(), project)]))),
        };
        let task_repo = MockTaskRepository::new();
        let code_resolver = MockCodeResolver::new();
        let use_case = UpdateTaskUseCase::new(project_repo.clone(), task_repo, code_resolver);

        // We delay task A by 3 days (it now ends on day 8 instead of 5)
        let args = UpdateTaskArgs {
            due_date: Some(d(8)),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-1", "A", args);
        assert!(result.is_ok());

        // Verification
        let final_project = project_repo.find_by_code("PROJ-1").unwrap().unwrap();
        let final_tasks = final_project.tasks();

        let final_a = final_tasks.get("A").unwrap();
        assert_eq!(*final_a.due_date(), d(8)); // Original task is updated

        let final_b = final_tasks.get("B").unwrap();
        assert_eq!(*final_b.start_date(), d(9)); // B now starts the day after A ends
        assert_eq!(*final_b.due_date(), d(13)); // B keeps its 4-day duration

        let final_c = final_tasks.get("C").unwrap();
        assert_eq!(*final_c.start_date(), d(14)); // C now starts the day after B ends
        assert_eq!(*final_c.due_date(), d(18)); // C keeps its 4-day duration
    }
}
