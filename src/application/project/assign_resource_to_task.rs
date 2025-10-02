#![allow(dead_code)]
#![allow(unused_imports)]

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::{
    any_project::AnyProject,
    repository::{ProjectRepository, ProjectRepositoryWithId},
};
use crate::domain::resource_management::repository::{ResourceRepository, ResourceRepositoryWithId};
use crate::domain::resource_management::resource::ResourceScope;
use crate::domain::shared::errors::{DomainError, DomainResult};
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

impl From<crate::domain::shared::errors::DomainError> for AssignResourceToAppError {
    fn from(err: crate::domain::shared::errors::DomainError) -> Self {
        AssignResourceToAppError::RepositoryError(err.into())
    }
}

pub struct AssignResourceToTaskUseCase<PR, RR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    resource_repository: RR,
    code_resolver: CR,
}

impl<PR, RR, CR> AssignResourceToTaskUseCase<PR, RR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    pub fn new(project_repository: PR, resource_repository: RR, code_resolver: CR) -> Self {
        Self {
            project_repository,
            resource_repository,
            code_resolver,
        }
    }

    pub fn execute(
        &self,
        project_code: &str,
        task_code: &str,
        resource_code: &str,
    ) -> Result<AnyProject, AssignResourceToAppError> {
        // 1. Resolve project code to ID
        let project_id = self
            .code_resolver
            .resolve_project_code(project_code)
            .map_err(|e| AssignResourceToAppError::RepositoryError(AppError::from(e)))?;

        // 2. Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(|e| AssignResourceToAppError::RepositoryError(AppError::from(e)))?;

        // 3. Load the project aggregate using ID
        let mut project = self
            .project_repository
            .find_by_id(&project_id)?
            .ok_or_else(|| AssignResourceToAppError::ProjectNotFound(project_code.to_string()))?;

        // 4. Validate that the resource exists
        let _resource = self
            .resource_repository
            .find_by_id(&resource_id)?
            .ok_or_else(|| AssignResourceToAppError::ResourceNotFound(resource_code.to_string()))?;

        // 5. Validate that the task exists in the project
        if !project.tasks().contains_key(task_code) {
            return Err(AssignResourceToAppError::TaskNotFound(task_code.to_string()));
        }

        // 6. Assign the resource to the task
        project
            .assign_resource_to_task(task_code, &[resource_code])
            .map_err(AssignResourceToAppError::AppError)?;

        // 7. Save the updated project
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
        fn save(&self, project: AnyProject) -> DomainResult<()> {
            self.projects.borrow_mut().insert(project.code().to_string(), project);
            Ok(())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().get(code).cloned())
        }

        fn load(&self) -> DomainResult<AnyProject> {
            unimplemented!()
        }

        fn find_all(&self) -> DomainResult<Vec<AnyProject>> {
            unimplemented!()
        }

        fn get_next_code(&self) -> DomainResult<String> {
            unimplemented!()
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().values().find(|p| p.id() == id).cloned())
        }
    }

    struct MockResourceRepository {
        resources: Vec<AnyResource>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.clone())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.iter().find(|r| r.code() == code).cloned())
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        fn save(&self, _resource: AnyResource) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn save_time_off(
            &self,
            _name: &str,
            _hours: u32,
            _date: &str,
            _desc: Option<String>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _name: &str,
            _start: &str,
            _end: &str,
            _comp: bool,
            _hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn check_if_layoff_period(&self, _start_date: &DateTime<Local>, _end_date: &DateTime<Local>) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, resource_type: &str) -> DomainResult<String> {
            Ok(format!("{}-1", resource_type.to_lowercase()))
        }
    }

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.iter().find(|r| r.id().to_string() == id).cloned())
        }
    }

    struct MockCodeResolver {
        project_codes: RefCell<HashMap<String, String>>,  // code -> id
        resource_codes: RefCell<HashMap<String, String>>, // code -> id
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                project_codes: RefCell::new(HashMap::new()),
                resource_codes: RefCell::new(HashMap::new()),
            }
        }

        fn add_project(&self, code: &str, id: &str) {
            self.project_codes.borrow_mut().insert(code.to_string(), id.to_string());
        }

        fn add_resource(&self, code: &str, id: &str) {
            self.resource_codes
                .borrow_mut()
                .insert(code.to_string(), id.to_string());
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, code: &str) -> DomainResult<String> {
            self.project_codes.borrow().get(code).cloned().ok_or_else(|| {
                DomainError::from(AppError::validation_error(
                    "project",
                    format!("Project '{}' not found", code),
                ))
            })
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

        fn validate_project_code(&self, code: &str) -> DomainResult<()> {
            self.resolve_project_code(code)?;
            Ok(())
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
            ResourceScope::Company,
            None,
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
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project.clone())])),
        };
        let resource1 = create_test_resource("res-1");
        let resource2 = create_test_resource("res-2");
        let resource_repo = MockResourceRepository {
            resources: vec![resource1.clone(), resource2.clone()],
        };
        let code_resolver = MockCodeResolver::new();
        code_resolver.add_project("PROJ-1", project.id());
        code_resolver.add_resource("dev-res-2", &resource2.id().to_string());
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "TSK-1", "dev-res-2");

        if let Err(ref e) = result {
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
        let code_resolver = MockCodeResolver::new();
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo, code_resolver);

        let result = use_case.execute("PROJ-NONEXISTENT", "TSK-1", "dev-res-1");

        assert!(matches!(result, Err(AssignResourceToAppError::RepositoryError(_))));
    }

    #[test]
    fn test_assign_fails_if_resource_not_found() {
        let project = setup_test_project(vec![create_test_task("TSK-1", &[]).into()]);
        let project_repo = MockProjectRepository {
            projects: RefCell::new(HashMap::from([(project.code().to_string(), project.clone())])),
        };
        let resource_repo = MockResourceRepository {
            resources: vec![create_test_resource("res-1")],
        };
        let code_resolver = MockCodeResolver::new();
        code_resolver.add_project("PROJ-1", project.id());
        let use_case = AssignResourceToTaskUseCase::new(project_repo, resource_repo, code_resolver);

        let result = use_case.execute("PROJ-1", "TSK-1", "res-NONEXISTENT");

        assert!(matches!(result, Err(AssignResourceToAppError::RepositoryError(_))));
    }
}
