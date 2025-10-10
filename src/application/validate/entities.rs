use super::specifications::{ProjectHasAssignedResourcesSpec, ProjectHasTasksSpec};
use super::types::ValidationResult;
use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::specification::{AndSpecification, Specification};

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

    pub fn execute(&self) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Load all entities
        let companies = self.company_repository.find_all()?;
        let resources = self.resource_repository.find_all()?;
        let projects = self.project_repository.find_all()?;

        // Validate entity relationships using specifications
        results.extend(self.validate_entity_relationships(&companies, &resources, &projects)?);

        // Validate entity completeness using specifications
        results.extend(self.validate_entity_completeness(&projects)?);

        Ok(results)
    }

    fn validate_entity_relationships(
        &self,
        companies: &[crate::domain::company_management::company::Company],
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Check for orphaned entities
        let company_codes: std::collections::HashSet<_> = companies.iter().map(|c| c.code.clone()).collect();
        let resource_codes: std::collections::HashSet<_> = resources.iter().map(|r| r.code()).collect();

        // Check projects referencing non-existent companies
        for project in projects {
            let ref_company = project.company_code();
            if !company_codes.contains(ref_company) {
                results.push(
                    ValidationResult::error(
                        "PROJECT_INVALID_COMPANY".to_string(),
                        "Project references non-existent company".to_string(),
                    )
                    .with_entity("Project".to_string(), project.code().to_string())
                    .with_details(format!(
                        "Project '{}' references company '{}' which does not exist",
                        project.code(),
                        ref_company
                    )),
                );
            }
        }

        // Check companies with no projects
        for company in companies {
            let has_projects = projects.iter().any(|p| p.company_code() == company.code);
            if !has_projects {
                results.push(
                    ValidationResult::info("COMPANY_NO_PROJECTS".to_string(), "Company has no projects".to_string())
                        .with_entity("Company".to_string(), company.code.clone())
                        .with_details("Company exists but has no associated projects".to_string()),
                );
            }
        }

        // Check resources assigned to non-existent tasks
        for project in projects {
            for task in project.tasks().values() {
                for resource_code in task.assigned_resources() {
                    if !resource_codes.contains(&resource_code.as_str()) {
                        results.push(
                            ValidationResult::error(
                                "TASK_INVALID_RESOURCE".to_string(),
                                "Task references non-existent resource".to_string(),
                            )
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details(format!(
                                "Task '{}' references resource '{}' which does not exist",
                                task.code(),
                                resource_code
                            )),
                        );
                    }
                }
            }
        }

        Ok(results)
    }

    fn validate_entity_completeness(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Create composite specification for project completeness
        let project_completeness_spec = AndSpecification::new()
            .add_specification(Box::new(ProjectHasTasksSpec))
            .add_specification(Box::new(ProjectHasAssignedResourcesSpec));

        for project in projects {
            if !project_completeness_spec.is_satisfied_by(project)
                && let Some(explanation) = project_completeness_spec.explain_why_not_satisfied(project)
            {
                results.push(
                    ValidationResult::warning(
                        "PROJECT_INCOMPLETE".to_string(),
                        "Project may be incomplete".to_string(),
                    )
                    .with_entity("Project".to_string(), project.code().to_string())
                    .with_details(explanation),
                );
            }
        }

        Ok(results)
    }
}
