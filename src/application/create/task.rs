// Priority and Category are used in Task initializations
use crate::application::errors::AppError;
use crate::application::project::detect_resource_conflicts::DetectResourceConflictsUseCase;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::project_management::repository::{ProjectRepository, ProjectRepositoryWithId};
use crate::domain::resource_management::repository::{ResourceRepository, ResourceRepositoryWithId};
use crate::domain::task_management::{AnyTask, TaskBuilder, repository::TaskRepository};
use chrono::NaiveDate;

pub struct CreateTaskArgs {
    pub company_code: String,
    pub project_code: String,
    pub name: String,
    pub code: Option<String>,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub assigned_resources: Vec<String>,
}

pub struct CreateTaskUseCase<PR, TR, RR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId,
    TR: TaskRepository,
    RR: ResourceRepository + ResourceRepositoryWithId,
    CR: CodeResolverTrait,
{
    project_repository: PR,
    task_repository: TR,
    #[allow(dead_code)]
    resource_repository: RR,
    code_resolver: CR,
}

impl<PR, TR, RR, CR> CreateTaskUseCase<PR, TR, RR, CR>
where
    PR: ProjectRepository + ProjectRepositoryWithId + Clone + 'static,
    TR: TaskRepository + Clone + 'static,
    RR: ResourceRepository + ResourceRepositoryWithId + Clone + 'static,
    CR: CodeResolverTrait + Clone + 'static,
{
    pub fn new(project_repository: PR, task_repository: TR, resource_repository: RR, code_resolver: CR) -> Self {
        Self {
            project_repository,
            task_repository,
            resource_repository,
            code_resolver,
        }
    }

    pub fn execute(&self, args: CreateTaskArgs) -> Result<(), AppError> {
        let CreateTaskArgs {
            company_code: _company_code, // TODO: Use this for hierarchical task saving
            project_code,
            name,
            code,
            start_date,
            due_date,
            assigned_resources,
        } = args;

        // 1. Resolve project code to ID
        let project_id =
            self.code_resolver
                .resolve_project_code(&project_code)
                .map_err(|_e| AppError::ProjectNotFound {
                    code: project_code.clone(),
                })?;

        // 2. Load the project aggregate using ID
        let mut project =
            self.project_repository
                .find_by_id(&project_id)?
                .ok_or_else(|| AppError::ProjectNotFound {
                    code: project_code.clone(),
                })?;

        // 3. Validate resource conflicts if resources are assigned
        if !assigned_resources.is_empty() {
            // Check for resource conflicts
            let conflict_detector = DetectResourceConflictsUseCase::new(
                Box::new(self.project_repository.clone()),
                Box::new(self.resource_repository.clone()),
                Box::new(self.task_repository.clone()),
                Box::new(self.code_resolver.clone()),
            );

            for resource_code in &assigned_resources {
                let conflicts = conflict_detector.detect_conflicts_for_resource(
                    resource_code,
                    start_date,
                    due_date,
                    None, // No task to exclude for new tasks
                )?;

                if !conflicts.is_empty() {
                    let conflict_messages: Vec<String> = conflicts
                        .iter()
                        .map(|c| format!("{}: {}", c.resource_code, c.message))
                        .collect();
                    
                return Err(AppError::validation_error(
                    "resource_conflicts",
                    format!("Resource conflicts detected: {}", conflict_messages.join("; ")),
                ));
                }
            }
            
            println!("Resources assigned: {:?}", assigned_resources);
        }

        // 4. Delegate task creation to the project aggregate.
        // This is a placeholder for the future implementation of `project.add_task(...)`
        // For now, we'll keep the builder logic here.
        if start_date > due_date {
            return Err(AppError::ValidationError {
                field: "dates".to_string(),
                message: "Start date cannot be after due date".to_string(),
            });
        }

        let next_task_code = match code {
            Some(c) => c,
            None => format!("task-{}", project.tasks().len() + 1),
        };

        let _task_code_for_output = next_task_code.clone();
        let project_code_for_save = project_code.clone();

        let builder = TaskBuilder::new()
            .project_code(project_code)
            .name(name.clone())
            .code(next_task_code)
            .dates(start_date, due_date)
            .map_err(|e| AppError::ValidationError {
                field: "task".to_string(),
                message: e.to_string(),
            })?;

        let task = if assigned_resources.is_empty() {
            builder
                .validate_vacations(&[])
                .unwrap()
                .build()
                .map_err(|e| AppError::ValidationError {
                    field: "task".to_string(),
                    message: e.to_string(),
                })
        } else {
            let mut iter = assigned_resources.into_iter();
            let builder_with_res = builder.assign_resource(iter.next().unwrap());
            let final_builder = iter.fold(builder_with_res, |b, r| b.assign_resource(r));
            final_builder
                .validate_vacations(&[])
                .unwrap()
                .build()
                .map_err(|e| AppError::ValidationError {
                    field: "task".to_string(),
                    message: e.to_string(),
                })
        }?;

        // Add the task to the project (this part will be moved into a project method later)
        let task_any: AnyTask = task.into();
        project.add_task(task_any.clone());

        // 5. Save the entire project aggregate.
        self.project_repository.save(project.clone())?;

        // 6. Save the task individually in the project's tasks directory
        self.task_repository
            .save_in_hierarchy(task_any, project.company_code(), &project_code_for_save)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::project_management::{AnyProject, builder::ProjectBuilder};
    use crate::domain::task_management::{AnyTask, repository::TaskRepository};
    use crate::domain::shared::errors::{DomainError, DomainResult};
    use crate::domain::resource_management::{any_resource::AnyResource, resource::{Resource, ResourceScope}, state::Available};
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Clone)]
    struct MockProjectRepository {
        should_fail: bool,
        projects: Rc<RefCell<HashMap<String, AnyProject>>>,
    }

    #[derive(Clone)]
    struct MockCodeResolver {
        project_codes: RefCell<HashMap<String, String>>, // code -> id
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                project_codes: RefCell::new(HashMap::new()),
            }
        }

        fn add_project(&self, code: &str, id: &str) {
            self.project_codes.borrow_mut().insert(code.to_string(), id.to_string());
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
            // For testing, we'll return the code as the ID
            Ok(code.to_string())
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

        fn validate_resource_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "resource",
                "Not implemented in mock",
            )))
        }

        fn validate_task_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "task",
                "Not implemented in mock",
            )))
        }
    }

    #[derive(Clone)]
    struct MockTaskRepository {
        tasks: RefCell<HashMap<String, AnyTask>>,
    }

    impl MockTaskRepository {
        fn new() -> Self {
            Self {
                tasks: RefCell::new(HashMap::new()),
            }
        }
    }

    #[derive(Clone)]
    struct MockResourceRepository {
        resources: Rc<RefCell<HashMap<String, crate::domain::resource_management::any_resource::AnyResource>>>,
    }

    impl MockResourceRepository {
        fn new() -> Self {
            let mut resources = HashMap::new();
            
            // Add test resources
            let dev1 = Resource::<Available>::new(
                "dev1".to_string(),
                "Developer 1".to_string(),
                Some("dev1@test.com".to_string()),
                "Developer".to_string(),
                ResourceScope::Company,
                None,
                None,
                None,
                None,
                0,
            );
            let any_dev1 = AnyResource::Available(dev1);
            resources.insert(any_dev1.id().to_string(), any_dev1);
            
            let dev2 = Resource::<Available>::new(
                "dev2".to_string(),
                "Developer 2".to_string(),
                Some("dev2@test.com".to_string()),
                "Developer".to_string(),
                ResourceScope::Company,
                None,
                None,
                None,
                None,
                0,
            );
            let any_dev2 = AnyResource::Available(dev2);
            resources.insert(any_dev2.id().to_string(), any_dev2);
            
            let dev3 = Resource::<Available>::new(
                "dev3".to_string(),
                "Developer 3".to_string(),
                Some("dev3@test.com".to_string()),
                "Developer".to_string(),
                ResourceScope::Company,
                None,
                None,
                None,
                None,
                0,
            );
            let any_dev3 = AnyResource::Available(dev3);
            resources.insert(any_dev3.id().to_string(), any_dev3);
            
            Self {
                resources: Rc::new(RefCell::new(resources)),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: crate::domain::resource_management::any_resource::AnyResource) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            let resource_id = resource.id().to_string();
            self.resources.borrow_mut().insert(resource_id.clone(), resource.clone());
            Ok(resource)
        }

        fn save_in_hierarchy(
            &self,
            resource: crate::domain::resource_management::any_resource::AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            self.save(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_all_with_context(&self) -> DomainResult<Vec<(crate::domain::resource_management::any_resource::AnyResource, String, Vec<String>)>> {
            Ok(self.resources.borrow().values().cloned().map(|r| (r, "company".to_string(), vec![])).collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(self.resources.borrow().values().find(|r| r.code() == code).cloned())
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(DomainError::validation_error("resource", "Not implemented in mock"))
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<crate::domain::resource_management::any_resource::AnyResource> {
            Err(DomainError::validation_error("resource", "Not implemented in mock"))
        }

        fn check_if_layoff_period(&self, _start_date: &chrono::DateTime<chrono::Local>, _end_date: &chrono::DateTime<chrono::Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            Ok("RES-001".to_string())
        }
    }

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, id: &str) -> DomainResult<Option<crate::domain::resource_management::any_resource::AnyResource>> {
            Ok(self.resources.borrow().get(id).cloned())
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, task: AnyTask) -> DomainResult<AnyTask> {
            self.tasks.borrow_mut().insert(task.code().to_string(), task.clone());
            Ok(task)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyTask>> {
            Ok(self.tasks.borrow().get(code).cloned())
        }

        fn save_in_hierarchy(&self, task: AnyTask, _company_code: &str, _project_code: &str) -> DomainResult<AnyTask> {
            self.save(task)
        }

        fn find_all_by_project(&self, _company_code: &str, _project_code: &str) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn find_by_project(&self, _project_code: &str) -> DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn get_next_code(&self, _project_code: &str) -> DomainResult<String> {
            Ok("TASK-001".to_string())
        }
    }

    impl MockProjectRepository {
        fn new(should_fail: bool) -> Self {
            let mut projects = HashMap::new();
            let project: AnyProject = ProjectBuilder::new()
                .code("PROJ-1".to_string())
                .name("Test Project".to_string())
                .company_code("COMP-001".to_string())
                .created_by("test-user".to_string())
                .build()
                .unwrap()
                .into();
            let project_id = project.id().to_string();
            projects.insert(project_id, project);

            Self {
                should_fail,
                projects: Rc::new(RefCell::new(projects)),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, project: AnyProject) -> DomainResult<()> {
            if self.should_fail {
                return Err(DomainError::ValidationError {
                    field: "repository".to_string(),
                    message: "Mocked save error".to_string(),
                });
            }
            self.projects.borrow_mut().insert(project.id().to_string(), project);
            Ok(())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyProject>> {
            Ok(self.projects.borrow().values().find(|p| p.code() == code).cloned())
        }

        // Unimplemented methods
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
            Ok(self.projects.borrow().get(id).cloned())
        }
    }

    fn create_test_dates() -> (NaiveDate, NaiveDate) {
        // Use weekdays to avoid weekend conflicts
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Monday
        let due_date = NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(); // Friday
        (start_date, due_date)
    }

    #[test]
    fn test_create_task_success() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Implementar autenticação".to_string(),
            code: None,
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        let project = use_case.project_repository.find_by_id(&project_id).unwrap().unwrap();
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Implementar autenticação");
    }

    #[test]
    fn test_create_task_fails_if_project_not_found() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-NONEXISTENT".to_string(),
            name: "Task for nonexistent project".to_string(),
            code: None,
            start_date,
            due_date,
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
    }

    #[test]
    fn test_create_task_fails_if_start_date_after_due_date() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        #[allow(unused_variables)]
        let (start_date, due_date) = create_test_dates();

        // Test with start_date > due_date
        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with invalid dates".to_string(),
            code: None,
            start_date: due_date + chrono::Duration::days(1), // start_date > due_date
            due_date,
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(
                e.to_string()
                    .contains("Start date cannot be after due date")
            );
        }
    }

    #[test]
    fn test_create_task_with_same_start_and_due_date() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        #[allow(unused_variables)]
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with same dates".to_string(),
            code: None,
            start_date,           // Use the same date for both
            due_date: start_date, // Use the same date for both
            assigned_resources: vec![],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        if let Err(_e) = &result {
        }

        assert!(result.is_ok(), "Expected Ok, but got Err: {:?}", result);
    }

    #[test]
    fn test_create_task_without_assigned_resources() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task without resources".to_string(),
            code: None,
            start_date,
            due_date,
            assigned_resources: vec![], // Empty resources vector
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        let project = mock_repo.find_by_id(&project_id).unwrap().unwrap();
        // Count should be 1 since we're starting with a fresh project
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Task without resources");
    }

    #[test]
    fn test_create_task_with_multiple_assigned_resources() {
        let mock_repo = MockProjectRepository::new(false);
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task with multiple resources".to_string(),
            code: None,
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string(), "dev2".to_string(), "dev3".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_ok());
        let project = mock_repo.find_by_id(&project_id).unwrap().unwrap();
        // Count should be 1 since we're starting with a fresh project
        assert_eq!(project.tasks().len(), 1);

        // Find the task by iterating through all tasks since we don't know the exact code
        let task = project.tasks().values().next().unwrap();
        assert_eq!(task.name(), "Task with multiple resources");
    }

    #[test]
    fn test_create_task_repository_save_failure() {
        let mock_repo = MockProjectRepository::new(true); // This will make save() fail
        let mock_task_repo = MockTaskRepository::new();
        let mock_resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();
        // Get the actual project ID from the mock repository
        let project_id = mock_repo.projects.borrow().values().next().unwrap().id().to_string();
        code_resolver.add_project("PROJ-1", &project_id);
        let use_case = CreateTaskUseCase::new(mock_repo.clone(), mock_task_repo, mock_resource_repo, code_resolver);
        let (start_date, due_date) = create_test_dates();

        let args = CreateTaskArgs {
            project_code: "PROJ-1".to_string(),
            name: "Task that will fail to save".to_string(),
            code: None,
            start_date,
            due_date,
            assigned_resources: vec!["dev1".to_string()],
            company_code: "TEST_COMPANY".to_string(),
        };
        let result = use_case.execute(args);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Mocked save error"));
        }
    }
}
