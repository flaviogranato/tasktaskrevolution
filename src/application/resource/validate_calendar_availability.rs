//! Calendar Availability Validation Use Case
//!
//! This module implements the use case for validating resource availability
//! based on calendars, vacations, and working hours.

use crate::application::errors::AppError;
use crate::application::shared::code_resolver::CodeResolverTrait;
use crate::domain::resource_management::repository::ResourceRepository;
use chrono::NaiveDate;
use std::collections::HashMap;

/// Calendar availability validation use case
pub struct ValidateCalendarAvailabilityUseCase {
    resource_repository: Box<dyn ResourceRepository>,
    code_resolver: Box<dyn CodeResolverTrait>,
}

/// Calendar availability result
#[derive(Debug, Clone)]
pub struct CalendarAvailabilityResult {
    pub resource_code: String,
    pub is_available: bool,
    pub conflicts: Vec<AvailabilityConflict>,
    pub working_days: Vec<NaiveDate>,
    pub non_working_days: Vec<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct AvailabilityConflict {
    pub conflict_type: AvailabilityConflictType,
    pub date: NaiveDate,
    pub message: String,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AvailabilityConflictType {
    Vacation,           // Resource is on vacation
    Holiday,           // Company holiday
    NonWorkingDay,     // Weekend or non-working day
    Leave,             // Personal leave
    OverAllocation,    // Resource is over-allocated
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictSeverity {
    Low,      // Minor conflict, can be resolved easily
    Medium,   // Moderate conflict, requires attention
    High,     // Major conflict, impacts project timeline
    Critical, // Critical conflict, project cannot proceed
}

impl ValidateCalendarAvailabilityUseCase {
    pub fn new(
        resource_repository: Box<dyn ResourceRepository>,
        code_resolver: Box<dyn CodeResolverTrait>,
    ) -> Self {
        Self {
            resource_repository,
            code_resolver,
        }
    }

    /// Validate calendar availability for a specific resource during a time period
    pub fn validate_resource_availability(
        &self,
        resource_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<CalendarAvailabilityResult, AppError> {
        // Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(|e| AppError::from(e))?;

        // Load resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AppError::validation_error("resource", "Resource not found"))?;

        let mut conflicts = Vec::new();
        let mut working_days = Vec::new();
        let mut non_working_days = Vec::new();

        // Check each day in the range
        for date in start_date.iter_days().take_while(|&d| d <= end_date) {
            if resource.is_available_on_date(date) {
                working_days.push(date);
            } else {
                non_working_days.push(date);
                
                // Determine the specific conflict type
                let conflict = self.determine_conflict_type(&resource, date);
                conflicts.push(conflict);
            }
        }

        let is_available = conflicts.is_empty();

        Ok(CalendarAvailabilityResult {
            resource_code: resource_code.to_string(),
            is_available,
            conflicts,
            working_days,
            non_working_days,
        })
    }

    /// Validate calendar availability for multiple resources
    pub fn validate_multiple_resources_availability(
        &self,
        resource_codes: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, CalendarAvailabilityResult>, AppError> {
        let mut results = HashMap::new();

        for resource_code in resource_codes {
            let result = self.validate_resource_availability(
                resource_code,
                start_date,
                end_date,
            )?;
            results.insert(resource_code.clone(), result);
        }

        Ok(results)
    }

    /// Get working days for a resource in a date range
    pub fn get_working_days(
        &self,
        resource_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>, AppError> {
        let result = self.validate_resource_availability(resource_code, start_date, end_date)?;
        Ok(result.working_days)
    }

    /// Get non-working days for a resource in a date range
    pub fn get_non_working_days(
        &self,
        resource_code: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>, AppError> {
        let result = self.validate_resource_availability(resource_code, start_date, end_date)?;
        Ok(result.non_working_days)
    }

    /// Check if a resource is available on a specific date
    pub fn is_available_on_date(
        &self,
        resource_code: &str,
        date: NaiveDate,
    ) -> Result<bool, AppError> {
        // Resolve resource code to ID
        let resource_id = self
            .code_resolver
            .resolve_resource_code(resource_code)
            .map_err(|e| AppError::from(e))?;

        // Load resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| AppError::validation_error("resource", "Resource not found"))?;

        // Check if resource is available on this specific date
        Ok(resource.is_available_on_date(date))
    }

    /// Determine the specific type of availability conflict
    fn determine_conflict_type(
        &self,
        resource: &crate::domain::resource_management::any_resource::AnyResource,
        date: NaiveDate,
    ) -> AvailabilityConflict {
        // Check if it's a holiday
        if resource.is_holiday(date) {
            return AvailabilityConflict {
                conflict_type: AvailabilityConflictType::Holiday,
                date,
                message: format!("Resource {} is on holiday on {}", resource.code(), date),
                severity: ConflictSeverity::High,
            };
        }

        // Check if it's a leave period
        if resource.is_on_leave(date) {
            return AvailabilityConflict {
                conflict_type: AvailabilityConflictType::Leave,
                date,
                message: format!(
                    "Resource {} is on leave on {}",
                    resource.code(),
                    date
                ),
                severity: ConflictSeverity::High,
            };
        }

        // Check if it's a non-working day
        if !resource.is_working_day(date) {
            return AvailabilityConflict {
                conflict_type: AvailabilityConflictType::NonWorkingDay,
                date,
                message: format!(
                    "Resource {} is not available on {} (non-working day)",
                    resource.code(),
                    date
                ),
                severity: ConflictSeverity::Medium,
            };
        }


        // Default case - should not happen if is_available_on_date is working correctly
        AvailabilityConflict {
            conflict_type: AvailabilityConflictType::NonWorkingDay,
            date,
            message: format!("Resource {} is not available on {}", resource.code(), date),
            severity: ConflictSeverity::Medium,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::any_resource::AnyResource;
    use crate::domain::resource_management::resource::Resource;
    use crate::domain::resource_management::resource::ResourceScope;
    use chrono::NaiveDate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl MockResourceRepository {
        fn new() -> Self {
            Self {
                resources: RefCell::new(HashMap::new()),
            }
        }

        fn add_resource(&self, resource: AnyResource) {
            self.resources.borrow_mut().insert(resource.id().to_string(), resource);
        }
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            let resource_id = resource.id().to_string();
            self.resources.borrow_mut().insert(resource_id.clone(), resource.clone());
            Ok(resource)
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(self.resources.borrow().values().cloned().map(|r| (r, "company".to_string(), vec![])).collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().values().find(|r| r.code() == code).cloned())
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<AnyResource> {
            Err(DomainError::validation_error("resource", "Not implemented in mock"))
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
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
        fn find_by_id(&self, id: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().get(id).cloned())
        }
    }

    struct MockCodeResolver {
        resource_codes: RefCell<HashMap<String, String>>, // code -> id
    }

    impl MockCodeResolver {
        fn new() -> Self {
            Self {
                resource_codes: RefCell::new(HashMap::new()),
            }
        }

        fn add_resource(&self, code: &str, id: &str) {
            self.resource_codes.borrow_mut().insert(code.to_string(), id.to_string());
        }
    }

    impl CodeResolverTrait for MockCodeResolver {
        fn resolve_company_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "company",
                "Not implemented in mock",
            )))
        }

        fn resolve_project_code(&self, _code: &str) -> DomainResult<String> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
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

        fn validate_project_code(&self, _code: &str) -> DomainResult<()> {
            Err(DomainError::from(AppError::validation_error(
                "project",
                "Not implemented in mock",
            )))
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

    fn create_test_resource(code: &str) -> AnyResource {
        Resource::new(
            code.to_string(),
            format!("Test {}", code),
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

    #[test]
    fn test_validate_resource_availability_success() {
        let resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();

        let resource = create_test_resource("DEV-001");
        resource_repo.add_resource(resource.clone());
        code_resolver.add_resource("DEV-001", &resource.id().to_string());

        let use_case = ValidateCalendarAvailabilityUseCase::new(
            Box::new(resource_repo),
            Box::new(code_resolver),
        );

        // Use only weekdays to avoid weekend conflicts
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(); // Wednesday
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 3).unwrap(); // Friday

        let result = use_case
            .validate_resource_availability("DEV-001", start_date, end_date)
            .unwrap();

        assert_eq!(result.resource_code, "DEV-001");
        assert!(result.is_available);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_validate_resource_availability_with_conflicts() {
        let resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();

        let resource = create_test_resource("DEV-001");
        resource_repo.add_resource(resource.clone());
        code_resolver.add_resource("DEV-001", &resource.id().to_string());

        let use_case = ValidateCalendarAvailabilityUseCase::new(
            Box::new(resource_repo),
            Box::new(code_resolver),
        );

        // Test with a weekend date
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 4).unwrap(); // Saturday
        let end_date = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();   // Sunday

        let result = use_case
            .validate_resource_availability("DEV-001", start_date, end_date)
            .unwrap();

        assert_eq!(result.resource_code, "DEV-001");
        assert!(!result.is_available);
        assert!(!result.conflicts.is_empty());
    }

    #[test]
    fn test_is_available_on_date() {
        let resource_repo = MockResourceRepository::new();
        let code_resolver = MockCodeResolver::new();

        let resource = create_test_resource("DEV-001");
        resource_repo.add_resource(resource.clone());
        code_resolver.add_resource("DEV-001", &resource.id().to_string());

        let use_case = ValidateCalendarAvailabilityUseCase::new(
            Box::new(resource_repo),
            Box::new(code_resolver),
        );

        // Test with a weekday
        let weekday = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap(); // Monday
        let is_available = use_case
            .is_available_on_date("DEV-001", weekday)
            .unwrap();

        assert!(is_available);
    }
}
