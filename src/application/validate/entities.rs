use crate::domain::{
    project_management::repository::ProjectRepository,
    resource_management::repository::ResourceRepository,
    company_management::repository::CompanyRepository,
    shared::errors::DomainError,
};
use super::types::{ValidationResult, ValidationSeverity};

pub struct ValidateEntitiesUseCase<'a, P, R, C> 
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: &'a P,
    resource_repository: &'a R,
    company_repository: &'a C,
}

impl<'a, P, R, C> ValidateEntitiesUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    pub fn new(project_repository: &'a P, resource_repository: &'a R, company_repository: &'a C) -> Self {
        Self {
            project_repository,
            resource_repository,
            company_repository,
        }
    }

    pub fn execute(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();

        // Load all entities
        let companies = self.company_repository.find_all()?;
        let resources = self.resource_repository.find_all()?;
        let projects = self.project_repository.find_all()?;

        // Validate company references
        results.extend(self.validate_company_references(&companies, &resources, &projects)?);

        // Validate resource assignments
        results.extend(self.validate_resource_assignments(&resources, &projects)?);

        // Validate task dependencies
        results.extend(self.validate_task_dependencies(&projects)?);

        // Validate orphaned entities
        results.extend(self.validate_orphaned_entities(&companies, &resources, &projects)?);

        Ok(results)
    }

    fn validate_company_references(
        &self,
        companies: &[crate::domain::company_management::company::Company],
        _resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();
        let company_codes: std::collections::HashSet<_> = companies.iter().map(|c| c.code.clone()).collect();

        // Check if projects reference valid companies
        for project in projects {
            let ref_company = project.company_code();
            if !company_codes.contains(ref_company) {
                results.push(
                    ValidationResult::error(format!("Project references non-existent company"))
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_details(format!("Referenced company '{}' does not exist", ref_company))
                );
            }
        }

        Ok(results)
    }

    fn validate_resource_assignments(
        &self,
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();
        let resource_codes: std::collections::HashSet<_> = resources.iter().map(|r| r.code()).collect();

        for project in projects {
            for task in project.tasks().values() {
                // Check if assigned resources exist
                for assigned_resource in task.assigned_resources() {
                    if !resource_codes.contains(assigned_resource.as_str()) {
                        results.push(
                            ValidationResult::error(format!("Task assigned to non-existent resource"))
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details(format!("Assigned resource '{}' does not exist", assigned_resource))
                        );
                    }
                }

                // Warning: Tasks without assigned resources
                if task.assigned_resources().is_empty() {
                    results.push(
                        ValidationResult::warning(format!("Task has no assigned resources"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details("Consider assigning resources to ensure task completion".to_string())
                    );
                }
            }
        }

        Ok(results)
    }

    fn validate_task_dependencies(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();

        for project in projects {
            let task_codes: std::collections::HashSet<_> = project.tasks().keys().collect();

            for task in project.tasks().values() {
                // Check if dependencies exist
                for dependency in task.dependencies() {
                    if !task_codes.contains(dependency) {
                        results.push(
                            ValidationResult::error(format!("Task depends on non-existent task"))
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details(format!("Dependency '{}' does not exist", dependency))
                        );
                    }
                }

                // Check for circular dependencies (basic check)
                if self.has_circular_dependency(task, project, &mut std::collections::HashSet::new()) {
                    results.push(
                        ValidationResult::error(format!("Circular dependency detected"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details("Task creates a circular dependency chain".to_string())
                    );
                }
            }
        }

        Ok(results)
    }

    fn has_circular_dependency(
        &self,
        task: &crate::domain::task_management::any_task::AnyTask,
        project: &crate::domain::project_management::any_project::AnyProject,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if visited.contains(task.code()) {
            return true;
        }

        visited.insert(task.code().to_string());

        for dependency_code in task.dependencies() {
            if let Some(dependency) = project.tasks().get(dependency_code) {
                if self.has_circular_dependency(dependency, project, visited) {
                    return true;
                }
            }
        }

        visited.remove(task.code());
        false
    }

    fn validate_orphaned_entities(
        &self,
        companies: &[crate::domain::company_management::company::Company],
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();

        // Check for companies without projects
        for company in companies {
            let has_projects = projects.iter().any(|p| p.company_code() == company.code);

            if !has_projects {
                results.push(
                    ValidationResult::warning(format!("Company has no associated projects"))
                        .with_entity("Company".to_string(), company.code.to_string())
                        .with_details("Consider adding projects to this company".to_string())
                );
            }
        }

        // Check for resources without projects
        for resource in resources {
            let is_assigned = projects.iter().any(|p| {
                p.tasks().values().any(|t| t.assigned_resources().contains(&resource.code().to_string()))
            });

            if !is_assigned {
                results.push(
                    ValidationResult::info(format!("Resource is not assigned to any tasks"))
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_details("Resource is available for assignment".to_string())
                );
            }
        }

        Ok(results)
    }
}
