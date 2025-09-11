use super::specifications::{
    ProjectHasTasksSpec, TaskWithinProjectTimelineSpec, ValidCompanySettingsSpec, ValidProjectDateRangeSpec,
    ValidResourceVacationSpec,
};
use super::types::ValidationResult;
use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;
use crate::domain::shared::specification::{AndSpecification, Specification};

pub struct ValidateDataIntegrityUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: &'a P,
    resource_repository: &'a R,
    company_repository: &'a C,
}

impl<'a, P, R, C> ValidateDataIntegrityUseCase<'a, P, R, C>
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

        // Validate using specifications
        results.extend(self.validate_projects_with_specifications(&projects)?);
        results.extend(self.validate_resources_with_specifications(&resources)?);
        results.extend(self.validate_companies_with_specifications(&companies)?);

        Ok(results)
    }

    fn validate_projects_with_specifications(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Create composite specification for projects
        let project_spec = AndSpecification::new()
            .add_specification(Box::new(ValidProjectDateRangeSpec))
            .add_specification(Box::new(TaskWithinProjectTimelineSpec))
            .add_specification(Box::new(ProjectHasTasksSpec));

        for project in projects {
            if !project_spec.is_satisfied_by(project)
                && let Some(explanation) = project_spec.explain_why_not_satisfied(project)
            {
                results.push(
                    ValidationResult::warning(project_spec.description().to_string())
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_details(explanation),
                );
            }
        }

        Ok(results)
    }

    fn validate_resources_with_specifications(
        &self,
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        let resource_spec = ValidResourceVacationSpec;

        for resource in resources {
            if !resource_spec.is_satisfied_by(resource)
                && let Some(explanation) = resource_spec.explain_why_not_satisfied(resource)
            {
                results.push(
                    ValidationResult::warning(resource_spec.description().to_string())
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_details(explanation),
                );
            }
        }

        Ok(results)
    }

    fn validate_companies_with_specifications(
        &self,
        companies: &[crate::domain::company_management::company::Company],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        let company_spec = ValidCompanySettingsSpec;

        for company in companies {
            if !company_spec.is_satisfied_by(company)
                && let Some(explanation) = company_spec.explain_why_not_satisfied(company)
            {
                results.push(
                    ValidationResult::warning(company_spec.description().to_string())
                        .with_entity("Company".to_string(), company.code().to_string())
                        .with_details(explanation),
                );
            }
        }

        Ok(results)
    }
}
