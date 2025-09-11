#![allow(dead_code)]

use super::types::ValidationResult;

use crate::application::errors::AppError;
use crate::domain::company_management::repository::CompanyRepository;
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::resource_management::{repository::ResourceRepository, resource::Period};
use chrono::{DateTime, FixedOffset, Local, NaiveDate, Offset};

pub struct ValidateBusinessRulesUseCase<'a, P, R, C>
where
    P: ProjectRepository,
    R: ResourceRepository,
    C: CompanyRepository,
{
    project_repository: &'a P,
    resource_repository: &'a R,
    company_repository: &'a C,
}

impl<'a, P, R, C> ValidateBusinessRulesUseCase<'a, P, R, C>
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

        // Validate vacation rules
        results.extend(self.validate_vacation_rules(&companies, &resources, &projects)?);

        // Validate resource allocation
        results.extend(self.validate_resource_allocation(&resources, &projects)?);

        // Validate project timeline
        results.extend(self.validate_project_timeline(&projects)?);

        // Validate cost constraints
        results.extend(self.validate_cost_constraints(&projects)?);

        Ok(results)
    }

    fn validate_vacation_rules(
        &self,
        _companies: &[crate::domain::company_management::company::Company],
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        _projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        // Check vacation overlaps between resources
        for (i, resource1) in resources.iter().enumerate() {
            if let Some(vacations1) = resource1.vacations() {
                // Check overlap with other resources
                for resource2 in resources.iter().skip(i + 1) {
                    if let Some(vacations2) = resource2.vacations() {
                        for period1 in vacations1 {
                            for period2 in vacations2 {
                                if self.check_vacation_overlap(period1, period2) {
                                    results.push(
                                        ValidationResult::warning("Vacation overlap detected".to_string())
                                            .with_entity("Resource".to_string(), resource1.code().to_string())
                                            .with_details(format!(
                                                "Resource '{}' and '{}' have overlapping vacations between {} and {}",
                                                resource1.name(),
                                                resource2.name(),
                                                period1.start_date.format("%d/%m/%Y"),
                                                period1.end_date.format("%d/%m/%Y")
                                            )),
                                    );
                                }
                            }
                        }
                    }
                }

                // TODO: Implement layoff period validation when VacationRules methods are available
                // For now, we'll skip this validation
            }
        }

        // TODO: Implement minimum resource validation when VacationRules methods are available
        // For now, we'll skip this validation

        Ok(results)
    }

    fn validate_resource_allocation(
        &self,
        _resources: &[crate::domain::resource_management::any_resource::AnyResource],
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        for project in projects {
            for task in project.tasks().values() {
                let assigned_resources = task.assigned_resources();

                // Check if task has resources assigned
                if assigned_resources.is_empty() {
                    results.push(
                        ValidationResult::warning("Task has no assigned resources".to_string())
                            .with_entity("Task".to_string(), task.code().to_string())
                            .with_details("Task may not be completed without resource assignment".to_string()),
                    );
                    continue;
                }

                // Check resource capacity vs task effort (simplified for now)
                // TODO: Implement proper effort and capacity validation when these fields are available
            }
        }

        Ok(results)
    }

    fn validate_project_timeline(
        &self,
        projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let mut results = Vec::new();

        for project in projects {
            // Check if project has start and end dates
            if let (Some(start_date), Some(end_date)) = (project.start_date(), project.end_date()) {
                if start_date >= end_date {
                    results.push(
                        ValidationResult::error("Invalid project timeline".to_string())
                            .with_entity("Project".to_string(), project.code().to_string())
                            .with_details("Project start date must be before end date".to_string()),
                    );
                }

                // Check if tasks fit within project timeline
                for task in project.tasks().values() {
                    let task_start = task.start_date();
                    let task_end = task.due_date();
                    if *task_start < start_date {
                        results.push(
                            ValidationResult::error("Task starts before project".to_string())
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details("Task start date is before project start date".to_string()),
                        );
                    }
                    if *task_end > end_date {
                        results.push(
                            ValidationResult::error("Task extends beyond project".to_string())
                                .with_entity("Task".to_string(), task.code().to_string())
                                .with_details("Task due date is after project end date".to_string()),
                        );
                    }
                }
            }
        }

        Ok(results)
    }

    fn validate_cost_constraints(
        &self,
        _projects: &[crate::domain::project_management::any_project::AnyProject],
    ) -> Result<Vec<ValidationResult>, AppError> {
        let results = Vec::new();

        // TODO: Implement cost validation when budget and cost fields are available
        // For now, this is a placeholder

        Ok(results)
    }

    // Helper methods
    fn check_vacation_overlap(&self, period1: &Period, period2: &Period) -> bool {
        period1.start_date <= period2.end_date && period2.start_date <= period1.end_date
    }

    fn check_layoff_overlap(&self, vacation_period: &Period, layoff_period: &(String, String)) -> bool {
        let layoff_start = NaiveDate::parse_from_str(&layoff_period.0, "%Y-%m-%d")
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default();
        let layoff_end = NaiveDate::parse_from_str(&layoff_period.1, "%Y-%m-%d")
            .unwrap_or_default()
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default();

        let offset = Local::now().offset().fix();
        let layoff_start: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_start, offset);
        let layoff_end: DateTime<FixedOffset> = DateTime::from_naive_utc_and_offset(layoff_end, offset);

        vacation_period.start_date <= layoff_end && layoff_start <= vacation_period.end_date
    }

    fn count_available_resources_during_project(
        &self,
        resources: &[crate::domain::resource_management::any_resource::AnyResource],
        project: &crate::domain::project_management::any_project::AnyProject,
        _vacation_rules: &crate::domain::project_management::project::VacationRules,
    ) -> usize {
        // Simplified implementation - count resources not on vacation during project period
        resources
            .iter()
            .filter(|resource| {
                if let Some(vacations) = resource.vacations() {
                    // Check if resource is available during project period
                    if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
                        !vacations.iter().any(|vacation| {
                            // TODO: Fix this when Period struct is properly implemented
                            // For now, we'll use a simple date comparison
                            // Convert DateTime to NaiveDate for comparison
                            let vacation_start = vacation.start_date.date_naive();
                            let vacation_end = vacation.end_date.date_naive();
                            vacation_start <= end && start <= vacation_end
                        })
                    } else {
                        true
                    }
                } else {
                    true
                }
            })
            .count()
    }
}
