use super::business_rules::ValidateBusinessRulesUseCase;
use super::data_integrity::ValidateDataIntegrityUseCase;
use super::entities::ValidateEntitiesUseCase;
use super::types::ValidationResult;
use crate::domain::{
    company_management::repository::CompanyRepository, project_management::repository::ProjectRepository,
    resource_management::repository::ResourceRepository, shared::errors::DomainError,
};

pub struct ValidateSystemUseCase<P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: P,
    resource_repository: R,
    company_repository: C,
}

impl<P, R, C> ValidateSystemUseCase<P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    pub fn new(project_repository: P, resource_repository: R, company_repository: C) -> Self {
        Self {
            project_repository,
            resource_repository,
            company_repository,
        }
    }

    pub fn execute(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let mut all_results = Vec::new();

        // 1. Validate data integrity first (foundation)
        println!("Validating data integrity...");
        let data_integrity_results = self.validate_data_integrity()?;
        all_results.extend(data_integrity_results);

        // 2. Validate entities and relationships
        println!("Validating entities and relationships...");
        let entity_results = self.validate_entities()?;
        all_results.extend(entity_results);

        // 3. Validate business rules
        println!("Validating business rules...");
        let business_results = self.validate_business_rules()?;
        all_results.extend(business_results);

        // 4. Generate summary
        self.print_summary(&all_results);

        Ok(all_results)
    }

    fn validate_data_integrity(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let use_case = ValidateDataIntegrityUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_entities(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let use_case = ValidateEntitiesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_business_rules(&self) -> Result<Vec<ValidationResult>, DomainError> {
        let use_case = ValidateBusinessRulesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn print_summary(&self, results: &[ValidationResult]) {
        let errors = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Error))
            .count();
        let warnings = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Warning))
            .count();
        let info = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Info))
            .count();

        println!("\nVALIDATION SUMMARY:");
        println!("===================");
        println!("Errors:   {}", errors);
        println!("Warnings: {}", warnings);
        println!("Info:     {}", info);
        println!("Total:    {}", results.len());

        if errors == 0 && warnings == 0 {
            println!("System validation completed successfully!");
        } else if errors == 0 {
            println!("System validation completed with warnings");
        } else {
            println!("System validation failed with errors");
        }
    }
}
