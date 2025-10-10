use crate::application::errors::AppError;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::query_engine::{QueryEngine, QueryResult};
use crate::domain::shared::query_parser::{Query, QueryValue};
use crate::domain::task_management::repository::TaskRepository;

/// Tipos de entidades que podem ser consultadas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Project,
    Task,
    Resource,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Project => write!(f, "project"),
            EntityType::Task => write!(f, "task"),
            EntityType::Resource => write!(f, "resource"),
        }
    }
}

impl std::str::FromStr for EntityType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "project" => Ok(EntityType::Project),
            "task" => Ok(EntityType::Task),
            "resource" => Ok(EntityType::Resource),
            _ => Err(AppError::validation_error("entity_type", format!("Invalid entity type: {}", s))),
        }
    }
}

/// Executor de queries integrado com repositórios
pub struct QueryExecutor {
    project_repository: Box<dyn ProjectRepository>,
    task_repository: Box<dyn TaskRepository>,
    resource_repository: Box<dyn ResourceRepository>,
}

impl QueryExecutor {
    pub fn new(
        project_repository: Box<dyn ProjectRepository>,
        task_repository: Box<dyn TaskRepository>,
        resource_repository: Box<dyn ResourceRepository>,
    ) -> Self {
        Self {
            project_repository,
            task_repository,
            resource_repository,
        }
    }

    /// Executa uma query para um tipo específico de entidade
    pub fn execute_query(&self, entity_type: EntityType, query: Query) -> Result<QueryResult<QueryValue>, AppError> {
        match entity_type {
            EntityType::Project => self.execute_project_query(query),
            EntityType::Task => self.execute_task_query(query),
            EntityType::Resource => self.execute_resource_query(query),
        }
    }

    /// Executa query em projetos
    fn execute_project_query(&self, query: Query) -> Result<QueryResult<QueryValue>, AppError> {
        let projects = self.project_repository.find_all()
            .map_err(AppError::from)?;
        
        let result = QueryEngine::execute(&query, projects)
            .map_err(|e| AppError::ValidationError {
                field: "query".to_string(),
                message: format!("Query execution error: {:?}", e),
            })?;
        
        // Converter resultado para QueryValue
        let items: Vec<QueryValue> = result.items
            .into_iter()
            .map(|p| QueryValue::String(serde_json::to_string(&p).unwrap_or_default()))
            .collect();
        
        Ok(QueryResult {
            items,
            total_count: result.total_count,
            filtered_count: result.filtered_count,
            aggregation_result: result.aggregation_result,
        })
    }

    /// Executa query em tarefas
    fn execute_task_query(&self, query: Query) -> Result<QueryResult<QueryValue>, AppError> {
        let tasks = self.task_repository.find_all()
            .map_err(AppError::from)?;
        
        let result = QueryEngine::execute(&query, tasks)
            .map_err(|e| AppError::ValidationError {
                field: "query".to_string(),
                message: format!("Query execution error: {:?}", e),
            })?;
        
        // Converter resultado para QueryValue
        let items: Vec<QueryValue> = result.items
            .into_iter()
            .map(|t| QueryValue::String(serde_json::to_string(&t).unwrap_or_default()))
            .collect();
        
        Ok(QueryResult {
            items,
            total_count: result.total_count,
            filtered_count: result.filtered_count,
            aggregation_result: result.aggregation_result,
        })
    }

    /// Executa query em recursos
    fn execute_resource_query(&self, query: Query) -> Result<QueryResult<QueryValue>, AppError> {
        let resources = self.resource_repository.find_all()
            .map_err(AppError::from)?;
        
        let result = QueryEngine::execute(&query, resources)
            .map_err(|e| AppError::ValidationError {
                field: "query".to_string(),
                message: format!("Query execution error: {:?}", e),
            })?;
        
        // Converter resultado para QueryValue
        let items: Vec<QueryValue> = result.items
            .into_iter()
            .map(|r| QueryValue::String(serde_json::to_string(&r).unwrap_or_default()))
            .collect();
        
        Ok(QueryResult {
            items,
            total_count: result.total_count,
            filtered_count: result.filtered_count,
            aggregation_result: result.aggregation_result,
        })
    }

    /// Executa query e retorna entidades completas
    pub fn execute_query_with_entities(&self, entity_type: EntityType, query: Query) -> Result<QueryResult<Box<dyn std::fmt::Debug>>, AppError> {
        match entity_type {
            EntityType::Project => {
                let projects = self.project_repository.find_all()?;
                let result = QueryEngine::execute(&query, projects)
                    .map_err(|e| AppError::ValidationError {
                        field: "query".to_string(),
                        message: format!("Query execution error: {:?}", e),
                    })?;
                Ok(QueryResult {
                    items: result.items.into_iter().map(|p| Box::new(p) as Box<dyn std::fmt::Debug>).collect(),
                    total_count: result.total_count,
                    filtered_count: result.filtered_count,
                    aggregation_result: result.aggregation_result,
                })
            }
            EntityType::Task => {
                let tasks = self.task_repository.find_all()?;
                let result = QueryEngine::execute(&query, tasks)
                    .map_err(|e| AppError::ValidationError {
                        field: "query".to_string(),
                        message: format!("Query execution error: {:?}", e),
                    })?;
                Ok(QueryResult {
                    items: result.items.into_iter().map(|t| Box::new(t) as Box<dyn std::fmt::Debug>).collect(),
                    total_count: result.total_count,
                    filtered_count: result.filtered_count,
                    aggregation_result: result.aggregation_result,
                })
            }
            EntityType::Resource => {
                let resources = self.resource_repository.find_all()?;
                let result = QueryEngine::execute(&query, resources)
                    .map_err(|e| AppError::ValidationError {
                        field: "query".to_string(),
                        message: format!("Query execution error: {:?}", e),
                    })?;
                Ok(QueryResult {
                    items: result.items.into_iter().map(|r| Box::new(r) as Box<dyn std::fmt::Debug>).collect(),
                    total_count: result.total_count,
                    filtered_count: result.filtered_count,
                    aggregation_result: result.aggregation_result,
                })
            }
        }
    }

    /// Lista campos disponíveis para um tipo de entidade
    pub fn get_available_fields(&self, entity_type: EntityType) -> Vec<String> {
        match entity_type {
            EntityType::Project => vec![
                "id".to_string(),
                "code".to_string(),
                "name".to_string(),
                "description".to_string(),
                "status".to_string(),
                "priority".to_string(),
                "start_date".to_string(),
                "end_date".to_string(),
                "actual_start_date".to_string(),
                "actual_end_date".to_string(),
                "company_code".to_string(),
                "manager_id".to_string(),
                "created_by".to_string(),
                "created_at".to_string(),
                "updated_at".to_string(),
                "task_count".to_string(),
                "resource_count".to_string(),
                "is_active".to_string(),
                "priority_weight".to_string(),
            ],
            EntityType::Task => vec![
                "id".to_string(),
                "project_code".to_string(),
                "code".to_string(),
                "name".to_string(),
                "description".to_string(),
                "status".to_string(),
                "start_date".to_string(),
                "due_date".to_string(),
                "actual_end_date".to_string(),
                "priority".to_string(),
                "category".to_string(),
                "dependency_count".to_string(),
                "assigned_resource_count".to_string(),
                "is_overdue".to_string(),
                "days_until_due".to_string(),
                "priority_weight".to_string(),
            ],
            EntityType::Resource => vec![
                "id".to_string(),
                "code".to_string(),
                "name".to_string(),
                "email".to_string(),
                "resource_type".to_string(),
                "scope".to_string(),
                "project_id".to_string(),
                "start_date".to_string(),
                "end_date".to_string(),
                "time_off_balance".to_string(),
                "vacation_count".to_string(),
                "time_off_history_count".to_string(),
                "task_assignment_count".to_string(),
                "active_task_count".to_string(),
                "current_allocation_percentage".to_string(),
                "is_wip_limits_exceeded".to_string(),
                "wip_status".to_string(),
                "is_available".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project_management::{AnyProject, repository::ProjectRepositoryWithId};
    use crate::domain::resource_management::{AnyResource, repository::ResourceRepositoryWithId};
    use crate::domain::task_management::{AnyTask, repository::TaskRepositoryWithId};
    use std::cell::RefCell;

    // Mock repositories for testing
    struct MockProjectRepository {
        projects: RefCell<Vec<AnyProject>>,
    }

    impl MockProjectRepository {
        fn new(projects: Vec<AnyProject>) -> Self {
            Self {
                projects: RefCell::new(projects),
            }
        }
    }

    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _project: AnyProject) -> crate::domain::shared::errors::DomainResult<()> {
            Ok(())
        }

        fn find_all(&self) -> crate::domain::shared::errors::DomainResult<Vec<AnyProject>> {
            Ok(self.projects.borrow().clone())
        }

        fn load(&self) -> crate::domain::shared::errors::DomainResult<AnyProject> {
            Err(crate::domain::shared::errors::DomainError::ProjectNotFound {
                code: "not-found".to_string(),
            })
        }

        fn find_by_code(&self, _code: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyProject>> {
            Ok(None)
        }

        fn get_next_code(&self) -> crate::domain::shared::errors::DomainResult<String> {
            Ok("PROJ-001".to_string())
        }
    }

    impl ProjectRepositoryWithId for MockProjectRepository {
        fn find_by_id(&self, _id: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyProject>> {
            Ok(None)
        }
    }

    struct MockTaskRepository {
        tasks: RefCell<Vec<AnyTask>>,
    }

    impl MockTaskRepository {
        fn new(tasks: Vec<AnyTask>) -> Self {
            Self {
                tasks: RefCell::new(tasks),
            }
        }
    }

    impl TaskRepository for MockTaskRepository {
        fn save(&self, _task: AnyTask) -> crate::domain::shared::errors::DomainResult<AnyTask> {
            Ok(AnyTask::Planned(crate::domain::task_management::task::Task {
                id: uuid7::uuid7(),
                project_code: "PROJ-001".to_string(),
                code: "TASK-001".to_string(),
                name: "Test Task".to_string(),
                description: None,
                state: crate::domain::task_management::state::Planned,
                start_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                due_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
                actual_end_date: None,
                dependencies: vec![],
                assigned_resources: vec![],
                priority: crate::domain::task_management::priority::Priority::default(),
                category: crate::domain::task_management::category::Category::default(),
            }))
        }

        fn save_in_hierarchy(&self, task: AnyTask, _company_code: &str, _project_code: &str) -> crate::domain::shared::errors::DomainResult<AnyTask> {
            Ok(task)
        }

        fn find_all(&self) -> crate::domain::shared::errors::DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().clone())
        }

        fn find_by_code(&self, _code: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyTask>> {
            Ok(None)
        }

        fn find_by_project(&self, _project_code: &str) -> crate::domain::shared::errors::DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().clone())
        }

        fn find_all_by_project(&self, _company_code: &str, _project_code: &str) -> crate::domain::shared::errors::DomainResult<Vec<AnyTask>> {
            Ok(self.tasks.borrow().clone())
        }

        fn get_next_code(&self, _project_code: &str) -> crate::domain::shared::errors::DomainResult<String> {
            Ok("TASK-001".to_string())
        }
    }

    impl TaskRepositoryWithId for MockTaskRepository {
        fn find_by_id(&self, _id: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyTask>> {
            Ok(None)
        }
    }

    struct MockResourceRepository {
        resources: RefCell<Vec<AnyResource>>,
    }

    impl MockResourceRepository {
        fn new(resources: Vec<AnyResource>) -> Self {
            Self {
                resources: RefCell::new(resources),
            }
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> crate::domain::shared::errors::DomainResult<AnyResource> {
            Ok(resource)
        }

        fn save_in_hierarchy(&self, resource: AnyResource, _company_code: &str, _project_code: Option<&str>) -> crate::domain::shared::errors::DomainResult<AnyResource> {
            Ok(resource)
        }

        fn find_all(&self) -> crate::domain::shared::errors::DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().clone())
        }

        fn find_by_company(&self, _company_code: &str) -> crate::domain::shared::errors::DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().clone())
        }

        fn find_all_with_context(&self) -> crate::domain::shared::errors::DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(vec![])
        }

        fn find_by_code(&self, _code: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyResource>> {
            Ok(None)
        }

        fn save_time_off(&self, _name: &str, _hours: u32, _date: &str, _desc: Option<String>) -> crate::domain::shared::errors::DomainResult<AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ValidationError {
                field: "resource".to_string(),
                message: "Not implemented in mock".to_string(),
            })
        }

        fn save_vacation(&self, _name: &str, _start: &str, _end: &str, _comp: bool, _hours: Option<u32>) -> crate::domain::shared::errors::DomainResult<AnyResource> {
            Err(crate::domain::shared::errors::DomainError::ValidationError {
                field: "resource".to_string(),
                message: "Not implemented in mock".to_string(),
            })
        }

        fn check_if_layoff_period(&self, _start: &chrono::DateTime<chrono::Local>, _end: &chrono::DateTime<chrono::Local>) -> bool {
            false
        }

        fn get_next_code(&self, _resource_type: &str) -> crate::domain::shared::errors::DomainResult<String> {
            Ok("RES-001".to_string())
        }
    }

    impl ResourceRepositoryWithId for MockResourceRepository {
        fn find_by_id(&self, _id: &str) -> crate::domain::shared::errors::DomainResult<Option<AnyResource>> {
            Ok(None)
        }
    }

    #[test]
    fn test_entity_type_parsing() {
        assert_eq!("project".parse::<EntityType>().unwrap(), EntityType::Project);
        assert_eq!("task".parse::<EntityType>().unwrap(), EntityType::Task);
        assert_eq!("resource".parse::<EntityType>().unwrap(), EntityType::Resource);
        assert!("invalid".parse::<EntityType>().is_err());
    }

    #[test]
    fn test_get_available_fields() {
        let executor = QueryExecutor::new(
            Box::new(MockProjectRepository::new(vec![])),
            Box::new(MockTaskRepository::new(vec![])),
            Box::new(MockResourceRepository::new(vec![])),
        );

        let project_fields = executor.get_available_fields(EntityType::Project);
        assert!(project_fields.contains(&"code".to_string()));
        assert!(project_fields.contains(&"name".to_string()));
        assert!(project_fields.contains(&"status".to_string()));

        let task_fields = executor.get_available_fields(EntityType::Task);
        assert!(task_fields.contains(&"code".to_string()));
        assert!(task_fields.contains(&"priority".to_string()));

        let resource_fields = executor.get_available_fields(EntityType::Resource);
        assert!(resource_fields.contains(&"name".to_string()));
        assert!(resource_fields.contains(&"resource_type".to_string()));
    }
}
