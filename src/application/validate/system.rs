use super::business_rules::ValidateBusinessRulesUseCase;
use super::data_integrity::ValidateDataIntegrityUseCase;
use super::entities::ValidateEntitiesUseCase;
use super::types::ValidationResult;
use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::repository::ResourceRepository;

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

    pub fn execute(&self) -> Result<Vec<ValidationResult>, AppError> {
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

    fn validate_data_integrity(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateDataIntegrityUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_entities(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateEntitiesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn validate_business_rules(&self) -> Result<Vec<ValidationResult>, AppError> {
        let use_case = ValidateBusinessRulesUseCase::new(
            &self.project_repository,
            &self.resource_repository,
            &self.company_repository,
        );
        use_case.execute()
    }

    fn print_summary(&self, results: &[ValidationResult]) {
        let errors: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Error))
            .collect();
        let warnings: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Warning))
            .collect();
        let info: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.severity, super::types::ValidationSeverity::Info))
            .collect();

        // Print detailed results
        if !errors.is_empty() {
            println!("\n❌ ERRORS FOUND:");
            println!("=================");
            for (i, result) in errors.iter().enumerate() {
                println!("{}. {}", i + 1, result.message);
                if let Some(entity_type) = &result.entity_type {
                    println!("   Entity: {} ({})", entity_type, result.entity_code.as_deref().unwrap_or("N/A"));
                }
                if let Some(field) = &result.field {
                    println!("   Field: {}", field);
                }
                if let Some(details) = &result.details {
                    println!("   Details: {}", details);
                }
                println!();
            }
        }

        if !warnings.is_empty() {
            println!("\n⚠️  WARNINGS FOUND:");
            println!("===================");
            for (i, result) in warnings.iter().enumerate() {
                println!("{}. {}", i + 1, result.message);
                if let Some(entity_type) = &result.entity_type {
                    println!("   Entity: {} ({})", entity_type, result.entity_code.as_deref().unwrap_or("N/A"));
                }
                if let Some(field) = &result.field {
                    println!("   Field: {}", field);
                }
                if let Some(details) = &result.details {
                    println!("   Details: {}", details);
                }
                println!();
            }
        }

        if !info.is_empty() {
            println!("\nℹ️  INFO:");
            println!("=========");
            for (i, result) in info.iter().enumerate() {
                println!("{}. {}", i + 1, result.message);
                if let Some(entity_type) = &result.entity_type {
                    println!("   Entity: {} ({})", entity_type, result.entity_code.as_deref().unwrap_or("N/A"));
                }
                if let Some(field) = &result.field {
                    println!("   Field: {}", field);
                }
                if let Some(details) = &result.details {
                    println!("   Details: {}", details);
                }
                println!();
            }
        }

        println!("\nVALIDATION SUMMARY:");
        println!("===================");
        println!("Errors:   {}", errors.len());
        println!("Warnings: {}", warnings.len());
        println!("Info:     {}", info.len());
        println!("Total:    {}", results.len());

        if errors.is_empty() && warnings.is_empty() {
            println!("\n✅ System validation completed successfully!");
        } else if errors.is_empty() {
            println!("\n⚠️  System validation completed with warnings");
        } else {
            println!("\n❌ System validation failed with errors");
        }
    }
}
