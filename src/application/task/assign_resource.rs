use crate::domain::{
    project_management::repository::ProjectRepository, resource_management::repository::ResourceRepository,
    shared::errors::DomainError, task_management::any_task::AnyTask,
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssignResourceError {
    #[error("Project not found for task with code '{0}'.")]
    ProjectNotFound(String),
    #[error("One or more resources not found: {0:?}")]
    ResourcesNotFound(Vec<String>),
    #[error("Domain rule violation: {0}")]
    DomainError(String),
    #[error(transparent)]
    RepositoryError(#[from] DomainError),
}

pub struct AssignResourceToTaskUseCase<PR, RR>
where
    PR: ProjectRepository,
    RR: ResourceRepository,
{
    project_repository: PR,
    resource_repository: RR,
}

impl<PR, RR> AssignResourceToTaskUseCase<PR, RR>
where
    PR: ProjectRepository,
    RR: ResourceRepository,
{
    pub fn new(project_repository: PR, resource_repository: RR) -> Self {
        Self {
            project_repository,
            resource_repository,
        }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        resource_codes: &[&str],
    ) -> Result<AnyTask, AssignResourceError> {
        // 1. Validate that all resources exist.
        let all_resources = self.resource_repository.find_all()?;
        let existing_resource_codes: HashSet<String> = all_resources.iter().map(|r| r.code().to_string()).collect();

        let not_found_resources: Vec<String> = resource_codes
            .iter()
            .filter(|rc| !existing_resource_codes.contains(**rc))
            .map(|s| s.to_string())
            .collect();

        if !not_found_resources.is_empty() {
            return Err(AssignResourceError::ResourcesNotFound(not_found_resources));
        }

        // 2. Load the project aggregate.
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| AssignResourceError::ProjectNotFound(project_code.to_string()))?;

        // 3. Delegate the assignment to the project aggregate.
        project
            .assign_resource_to_task(task_code, resource_codes)
            .map_err(AssignResourceError::DomainError)?;

        // 4. Save the updated project aggregate.
        self.project_repository.save(project.clone())?;

        // 5. Return the updated task.
        let updated_task = project
            .tasks()
            .get(task_code)
            .cloned()
            .ok_or_else(|| AssignResourceError::DomainError("Task disappeared after assignment".to_string()))?;

        Ok(updated_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        resource_management::{AnyResource, resource::Resource},
        task_management::{state::Planned, task::Task},
    };
    use chrono::{DateTime, Local, NaiveDate};
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

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> Result<Vec<AnyResource>, DomainError> {
            Ok(self.resources.clone())
        }
        fn save(&self, _resource: AnyResource) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> Result<AnyResource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(&self, _start: &DateTime<Local>, _end: &DateTime<Local>) -> bool {
            unimplemented!()
        }
        fn get_next_code(&self, resource_type: &str) -> Result<String, DomainError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    // --- Helpers ---

    fn create_test_task(code: &str, assignees: Vec<&str>) -> AnyTask {
        Task::<Planned> {
            id: uuid7(),
            project_code: "PROJ-1".to_string(),
            code: code.to_string(),
            name: "Test Task".to_string(),
            description: None,
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2025, 1, 10).unwrap(),
            actual_end_date: None,
            assigned_resources: assignees.into_iter().map(String::from).collect(),
        }
        .into()
    }

    fn create_test_resource(name: &str) -> AnyResource {
        Resource::new(
            format!("dev-{name}"),
            name.to_string(),
            None,
            "Developer".to_string(),
            None,
            0,
        )
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

    #[test]
    fn test_assign_new_resources_success() {
        let project = setup_test_project(vec![create_test_task("TSK-1", vec!["dev-res-1"])]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1"), create_test_resource("res-2")],
        };
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);

        let result = use_case.execute("PROJ-1", "TSK-1", &["dev-res-2"]);

        assert!(result.is_ok());
        let updated_task = result.unwrap();
        let mut assignees = updated_task.assigned_resources().to_vec();
        assignees.sort();
        assert_eq!(assignees, vec!["dev-res-1", "dev-res-2"]);
    }

    #[test]
    fn test_assign_fails_if_project_not_found() {
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::new()),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", &["dev-res-1"]);

        assert!(matches!(result, Err(AssignResourceError::ProjectNotFound(_))));
    }

    #[test]
    fn test_assign_fails_if_resource_not_found() {
        let project = setup_test_project(vec![create_test_task("TSK-1", vec![])]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);

        let result = use_case.execute("PROJ-1", "TSK-1", &["res-NONEXISTENT"]);

        assert!(matches!(result, Err(AssignResourceError::ResourcesNotFound(_))));
    }
}
