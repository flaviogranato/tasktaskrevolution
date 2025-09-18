#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::domain::project_management::{any_project::AnyProject, repository::ProjectRepository};
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::task_management::{Category, Priority};
use std::fmt;

#[derive(Debug)]
pub enum AssignResourceToAppError {
    ProjectNotFound(String),
    TaskNotFound(String),
    ResourceNotFound(String),
    ResourceAlreadyAssigned(String, String),
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
        resource_code: &str,
    ) -> Result<AnyProject, AssignResourceToAppError> {
        // 1. Find the project
        let mut project = self
            .project_repository
            .find_by_code(project_code)?
            .ok_or_else(|| AssignResourceToAppError::ProjectNotFound(project_code.to_string()))?;

        // 2. Find the resource
        let _resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AssignResourceToAppError::ResourceNotFound(resource_code.to_string()))?;

        // 3. Validate that the task exists in the project
        if !project.tasks().contains_key(task_code) {
            return Err(AssignResourceToAppError::TaskNotFound(task_code.to_string()));
        }

        // 4. Assign the resource to the task
        project
            .assign_resource_to_task(task_code, &[resource_code])
            .map_err(AssignResourceToAppError::AppError)?;

        // 5. Save the updated project
        self.project_repository.save(project.clone())?;

        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::{AnyProject, builder::ProjectBuilder},
        resource_management::{AnyResource, resource::Resource},
        task_management::{AnyTask, state::Planned, task::Task},
    };
    use chrono::{DateTime, Local, NaiveDate};
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

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(self.resources.clone())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.iter().find(|r| r.code() == code).cloned())
        }

        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
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

        fn save(&self, _resource: AnyResource) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, resource_type: &str) -> Result<String, AppError> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    // --- Helpers ---
    fn create_test_task(code: &str, assignees: &[&str]) -> Task<Planned> {
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
            dependencies: vec![],
            assigned_resources: assignees.iter().map(|&s| s.to_string()).collect(),
            priority: Priority::default(),
            category: Category::default(),
        }
    }

    fn create_test_resource(name: &str) -> AnyResource {
        Resource::new(
            format!("dev-{name}"),
            name.to_string(),
            None,
            "Developer".to_string(),
            None,
            None,
            None,
            0,
        )
        .into()
    }

    fn setup_test_project(tasks: Vec<AnyTask>) -> AnyProject {
        let mut builder = ProjectBuilder::new()
            .code("PROJ-1".to_string())
            .name("Test Project".to_string())
            .company_code("COMP-001".to_string())
            .created_by("test-user".to_string());

        for task in tasks {
            builder = builder.add_task(task);
        }

        builder.build().unwrap().into()
    }

    // --- Tests ---
    #[test]
    fn test_assign_new_resources_success() {
        let project = setup_test_project(vec![create_test_task("TSK-1", &["dev-res-1"]).into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1"), create_test_resource("res-2")],
        };
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);

        let result = use_case.execute("PROJ-1", "TSK-1", "dev-res-2");

        if let Err(ref e) = result {
            eprintln!("Error: {}", e);
        }

        assert!(result.is_ok());
        let updated_project = result.unwrap();
        let updated_task = updated_project.tasks().get("TSK-1").unwrap();
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

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", "dev-res-1");

        assert!(matches!(result, Err(AssignResourceToAppError::ProjectNotFound(_))));
    }

    #[test]
    fn test_assign_fails_if_resource_not_found() {
        let project = setup_test_project(vec![create_test_task("TSK-1", &[]).into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project)])),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo);

        let result = use_case.execute("PROJ-1", "TSK-1", "res-NONEXISTENT");

        assert!(matches!(result, Err(AssignResourceToAppError::ResourceNotFound(_))));
    }
}
