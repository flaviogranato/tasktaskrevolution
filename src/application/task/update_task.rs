use crate::domain::{
    project_management::repository::ProjectRepository, shared::errors::DomainError, task_management::any_task::AnyTask,
};
use chrono::NaiveDate;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateTaskError {
    #[error("Project with code '{0}' not found.")]
    ProjectNotFound(String),
    #[error("Task with code '{0}' not found in project.")]
    TaskNotFound(String),
    #[error("An unexpected domain rule was violated: {0}")]
    DomainError(String),
    #[error("A repository error occurred: {0}")]
    RepositoryError(#[from] DomainError),
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

        // 2. Delegate the update to the project aggregate.
        // This method ensures all domain invariants are respected.
        project
            .update_task(task_code, args.name, args.description, args.start_date, args.due_date)
            .map_err(UpdateTaskError::DomainError)?;

        // 3. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 4. Return the updated task to the caller.
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
    use std::{cell::RefCell, collections::HashMap};
    use uuid7::uuid7;

    // --- Mocks ---
    struct MockProjectRepository {
        projects: RefCell<HashMap<String, AnyProject>>,
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
            assigned_resources: vec![],
        }
        .into()
    }

    fn setup_test_project(tasks: Vec<AnyTask>) -> AnyProject {
        let mut project: AnyProject = ProjectBuilder::new("Test Project".to_string())
            .code("PROJ-1".to_string())
            .build()
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
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
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
        assert_eq!(updated_task.description().as_deref().unwrap(), "Initial Description");
    }

    #[test]
    fn test_update_task_fails_if_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let use_case = UpdateTaskUseCase::new(project_repo);

        let args = UpdateTaskArgs {
            name: Some("New Name".to_string()),
            ..Default::default()
        };

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", args);
        assert!(matches!(result, Err(UpdateTaskError::ProjectNotFound(_))));
    }
}
