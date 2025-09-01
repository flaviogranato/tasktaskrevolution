use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use chrono::NaiveDate;
use std::fmt;

#[derive(Debug)]
pub enum UpdateTaskError {
    ProjectNotFound(String),
    TaskNotFound(String),
    DomainError(String),
    RepositoryError(DomainError),
}

impl fmt::Display for UpdateTaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateTaskError::ProjectNotFound(code) => write!(f, "Project with code '{}' not found.", code),
            UpdateTaskError::TaskNotFound(code) => write!(f, "Task with code '{}' not found in project.", code),
            UpdateTaskError::DomainError(message) => write!(f, "An unexpected domain rule was violated: {}", message),
            UpdateTaskError::RepositoryError(err) => write!(f, "A repository error occurred: {}", err),
        }
    }
}

impl std::error::Error for UpdateTaskError {}

impl From<DomainError> for UpdateTaskError {
    fn from(err: DomainError) -> Self {
        UpdateTaskError::RepositoryError(err)
    }
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTaskArgs {
    pub name: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
}

pub struct UpdateTaskUseCase<PR>
where
    PR: ProjectRepository,
{
    project_repository: PR,
}

impl<PR> UpdateTaskUseCase<PR>
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
        args: UpdateTaskArgs,
    ) -> Result<AnyTask, UpdateTaskError> {
        // 1. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| UpdateTaskError::ProjectNotFound(project_code.to_string()))?;

        // Check if a reschedule is needed before args is moved.
        let needs_reschedule = args.due_date.is_some();

        // 2. Delegate the update to the project aggregate.
        // TODO: Implement update_task method in AnyProject
        // This method ensures all domain invariants are respected.
        // For now, we'll just return success
        // project
        //     .update_task(task_code, args.name, args.description, args.start_date, args.due_date)
        //     .map_err(UpdateTaskError::DomainError)?;

        // 3. If the due date was changed, reschedule all dependent tasks.
        if needs_reschedule {
            project
                .reschedule_dependents_of(task_code)
                .map_err(UpdateTaskError::DomainError)?;
        }

        // 4. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 5. Return the updated task to the caller.
        let updated_task = project
            .tasks()
            .get(task_code)
            .cloned()
            // This should ideally not happen if update_task succeeded, but we check for safety.
            .ok_or_else(|| UpdateTaskError::TaskNotFound(task_code.to_string()))?;

        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        task_management::{state::Planned, task::Task},
    };
    use std::{cell::RefCell, collections::HashMap, rc::Rc};
    use uuid7::uuid7;

    // --- Mocks ---
    #[derive(Clone)]
    struct MockProjectRepository {
        projects: Rc<RefCell<HashMap<String, AnyProject>>>,
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> Result<(), DomainError> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
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
        let use_case = UpdateTaskUseCase::new(project_repo);

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
        let use_case = UpdateTaskUseCase::new(project_repo);

        let args = UpdateTaskArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", args);
        assert!(matches!(result, Err(UpdateTaskError::ProjectNotFound(_))));
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
        let use_case = UpdateTaskUseCase::new(project_repo.clone());

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
