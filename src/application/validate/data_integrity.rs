use crate::domain::{
    project_management::repository::ProjectRepository,
    resource_management::repository::ResourceRepository,
    company_management::repository::CompanyRepository,
    shared::errors::DomainError,
};
use super::types::{ValidationResult, ValidationSeverity};

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

    pub fn execute(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();

        // Validate company data integrity
        results.extend(self.validate_companies()?);

        // Validate resource data integrity
        results.extend(self.validate_resources()?);

        // Validate project data integrity
        results.extend(self.validate_projects()?);

        Ok(results)
    }

    fn validate_companies(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();
        let companies = self.company_repository.find_all()?;

        for company in companies {
            // Check required fields
            if company.name.trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Company name is empty"))
                        .with_entity("Company".to_string(), company.code.to_string())
                        .with_field("name".to_string())
                );
            }

            if company.code.trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Company code is empty"))
                        .with_entity("Company".to_string(), company.code.to_string())
                        .with_field("code".to_string())
                );
            }

            // Check code format (should follow pattern company-X)
            if !company.code.starts_with("company-") {
                results.push(
                    ValidationResult::warning(format!("Company code format is non-standard"))
                        .with_entity("Company".to_string(), company.code.to_string())
                        .with_field("code".to_string())
                        .with_details("Expected format: company-X (e.g., company-1)".to_string())
                );
            }

            // Check email format if provided
            if let Some(email) = &company.email {
                if !self.is_valid_email(email) {
                    results.push(
                        ValidationResult::error(format!("Invalid email format"))
                            .with_entity("Company".to_string(), company.code.to_string())
                            .with_field("email".to_string())
                            .with_details(format!("Invalid email: {}", email))
                    );
                }
            }

            // Check tax_id format if provided (Brazilian CNPJ format)
            if let Some(tax_id) = &company.tax_id {
                if !self.is_valid_cnpj(tax_id) {
                    results.push(
                        ValidationResult::warning(format!("Tax ID format may be invalid"))
                            .with_entity("Company".to_string(), company.code.to_string())
                            .with_field("tax_id".to_string())
                            .with_details(format!("Expected CNPJ format: XX.XXX.XXX/XXXX-XX, got: {}", tax_id))
                    );
                }
            }

            // Check dates
            if company.created_at > company.updated_at {
                results.push(
                    ValidationResult::error(format!("Invalid date sequence"))
                        .with_entity("Company".to_string(), company.code.to_string())
                        .with_details("Created date cannot be after updated date".to_string())
                );
            }
        }

        Ok(results)
    }

    fn validate_resources(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();
        let resources = self.resource_repository.find_all()?;

        for resource in resources {
            // Check required fields
            if resource.name().trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Resource name is empty"))
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_field("name".to_string())
                );
            }

            if resource.code().trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Resource code is empty"))
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_field("code".to_string())
                );
            }

            // Check code format
            if !resource.code().starts_with("developer-") && !resource.code().starts_with("manager-") {
                results.push(
                    ValidationResult::warning(format!("Resource code format is non-standard"))
                        .with_entity("Resource".to_string(), resource.code().to_string())
                        .with_field("code".to_string())
                        .with_details("Expected format: developer-X or manager-X".to_string())
                );
            }

            // Check email format if provided
            if let Some(email) = resource.email() {
                if !self.is_valid_email(email) {
                    results.push(
                        ValidationResult::error(format!("Invalid email format"))
                            .with_entity("Resource".to_string(), resource.code().to_string())
                            .with_field("email".to_string())
                            .with_details(format!("Invalid email: {}", email))
                    );
                }
            }

            // Check capacity if provided (simplified for now)
            // TODO: Implement proper capacity validation when available

            // Check vacation periods if provided
            if let Some(vacations) = resource.vacations() {
                for vacation in vacations {
                    if vacation.start_date > vacation.end_date {
                        results.push(
                            ValidationResult::error(format!("Invalid vacation period"))
                                .with_entity("Resource".to_string(), resource.code().to_string())
                                .with_field("vacations".to_string())
                                .with_details("Vacation start date cannot be after end date".to_string())
                        );
                    }
                }
            }
        }

        Ok(results)
    }

    fn validate_projects(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut results = Vec::new();
        let projects = self.project_repository.find_all()?;

        for project in projects {
            // Check required fields
            if project.name().trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Project name is empty"))
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_field("name".to_string())
                );
            }

            if project.code().trim().is_empty() {
                results.push(
                    ValidationResult::error(format!("Project code is empty"))
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_field("code".to_string())
                );
            }

            // Check code format
            if !project.code().starts_with("project-") {
                results.push(
                    ValidationResult::warning(format!("Project code format is non-standard"))
                        .with_entity("Project".to_string(), project.code().to_string())
                        .with_field("code".to_string())
                        .with_details("Expected format: project-X".to_string())
                );
            }

            // Check dates
            if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
                if start >= end {
                    results.push(
                        ValidationResult::error(format!("Invalid project timeline"))
                            .with_entity("Project".to_string(), project.code().to_string())
                            .with_details("Project start date must be before end date".to_string())
                    );
                }
            }

            // Check tasks
            for task in project.tasks().values() {
                // Check task required fields
                if task.name().trim().is_empty() {
                    results.push(
                        ValidationResult::error(format!("Task name is empty"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_field("name".to_string())
                    );
                }

                if task.code().trim().is_empty() {
                    results.push(
                        ValidationResult::error(format!("Task code is empty"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_field("code".to_string())
                    );
                }

                // Check task code format
                if !task.code().starts_with("task-") {
                    results.push(
                        ValidationResult::warning(format!("Task code format is non-standard"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_field("code".to_string())
                            .with_details("Expected format: task-X".to_string())
                    );
                }

                // Check task dates
                let start = task.start_date();
                let end = task.due_date();
                if start >= end {
                    results.push(
                        ValidationResult::error(format!("Invalid task timeline"))
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details("Task start date must be before due date".to_string())
                    );
                }

                // Check effort if provided (simplified for now)
                // TODO: Implement proper effort validation when available

                // Check cost if provided (simplified for now)
                // TODO: Implement proper cost validation when available
            }
        }

        Ok(results)
    }

    // Helper methods
    fn is_valid_email(&self, email: &str) -> bool {
        // Basic email validation
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    fn is_valid_cnpj(&self, cnpj: &str) -> bool {
        // Basic CNPJ format validation (XX.XXX.XXX/XXXX-XX)
        let cleaned = cnpj.chars().filter(|c| c.is_digit(10)).collect::<String>();
        cleaned.len() == 14
    }
}
