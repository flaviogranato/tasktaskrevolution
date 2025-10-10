#![allow(dead_code)]

use crate::application::errors::AppError;
use crate::domain::resource_management::{
    any_resource::AnyResource,
    repository::ResourceRepository,
    resource::{WipLimits, WipStatus},
};
use crate::domain::shared::errors::DomainError;
use crate::interface::cli::table_formatter::TableFormatter;
use clap::{Args, Subcommand};
use std::fmt;

#[derive(Debug, Subcommand)]
pub enum WipCommand {
    /// Set WIP limits for a resource
    Set(WipSetArgs),
    /// Get WIP status for a resource
    Status(WipStatusArgs),
    /// List WIP status for all resources
    List(WipListArgs),
    /// Disable WIP limits for a resource
    Disable(WipDisableArgs),
}

#[derive(Debug, Args)]
pub struct WipSetArgs {
    /// Resource code
    #[arg(short, long)]
    pub resource: String,
    /// Maximum concurrent tasks
    #[arg(short, long, default_value = "5")]
    pub max_tasks: u32,
    /// Maximum concurrent projects
    #[arg(short, long, default_value = "3")]
    pub max_projects: u32,
    /// Maximum allocation percentage
    #[arg(short, long, default_value = "100")]
    pub max_allocation: u8,
}

#[derive(Debug, Args)]
pub struct WipStatusArgs {
    /// Resource code
    #[arg(short, long)]
    pub resource: String,
}

#[derive(Debug, Args)]
pub struct WipListArgs {
    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,
    /// Show only resources with WIP limits enabled
    #[arg(short, long)]
    pub enabled_only: bool,
}

#[derive(Debug, Args)]
pub struct WipDisableArgs {
    /// Resource code
    #[arg(short, long)]
    pub resource: String,
}

#[derive(Debug)]
pub enum WipAppError {
    ResourceNotFound(String),
    InvalidWipLimits(String),
    RepositoryError(AppError),
}

impl fmt::Display for WipAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WipAppError::ResourceNotFound(code) => write!(f, "Resource with code '{}' not found.", code),
            WipAppError::InvalidWipLimits(message) => write!(f, "Invalid WIP limits: {}", message),
            WipAppError::RepositoryError(err) => write!(f, "Repository error: {}", err),
        }
    }
}

impl std::error::Error for WipAppError {}

impl From<AppError> for WipAppError {
    fn from(err: AppError) -> Self {
        WipAppError::RepositoryError(err)
    }
}

impl From<DomainError> for WipAppError {
    fn from(domain_error: DomainError) -> Self {
        let app_error: AppError = domain_error.into();
        app_error.into()
    }
}

pub struct WipUseCase<RR>
where
    RR: ResourceRepository,
{
    resource_repository: RR,
}

impl<RR> WipUseCase<RR>
where
    RR: ResourceRepository,
{
    pub fn new(resource_repository: RR) -> Self {
        Self { resource_repository }
    }

    pub fn set_wip_limits(
        &self,
        resource_code: &str,
        max_tasks: u32,
        max_projects: u32,
        max_allocation: u8,
    ) -> Result<(), WipAppError> {
        // Find the resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| WipAppError::ResourceNotFound(resource_code.to_string()))?;

        // Create WIP limits
        let wip_limits = WipLimits::new(max_tasks, max_projects, max_allocation);
        if !wip_limits.is_valid() {
            return Err(WipAppError::InvalidWipLimits(
                "WIP limits configuration is invalid".to_string(),
            ));
        }

        // Update resource with new WIP limits
        let updated_resource = match resource {
            AnyResource::Available(mut res) => {
                res.set_wip_limits(wip_limits).map_err(WipAppError::InvalidWipLimits)?;
                AnyResource::Available(res)
            }
            AnyResource::Assigned(mut res) => {
                res.set_wip_limits(wip_limits).map_err(WipAppError::InvalidWipLimits)?;
                AnyResource::Assigned(res)
            }
            AnyResource::Inactive(mut res) => {
                res.set_wip_limits(wip_limits).map_err(WipAppError::InvalidWipLimits)?;
                AnyResource::Inactive(res)
            }
        };

        // Save the updated resource
        self.resource_repository
            .save(updated_resource)
            .map_err(|e: DomainError| WipAppError::RepositoryError(e.into()))?;

        Ok(())
    }

    pub fn get_wip_status(&self, resource_code: &str) -> Result<WipStatusInfo, WipAppError> {
        // Find the resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| WipAppError::ResourceNotFound(resource_code.to_string()))?;

        // Extract WIP status information
        let (wip_limits, wip_status, active_tasks, current_allocation) = match resource {
            AnyResource::Available(ref res) => (
                res.get_wip_limits().cloned(),
                res.get_wip_status(),
                res.get_active_task_count(),
                res.get_current_allocation_percentage(),
            ),
            AnyResource::Assigned(ref res) => (
                res.get_wip_limits().cloned(),
                res.get_wip_status(),
                res.get_active_task_count(),
                res.get_current_allocation_percentage(),
            ),
            AnyResource::Inactive(ref res) => (
                res.get_wip_limits().cloned(),
                res.get_wip_status(),
                res.get_active_task_count(),
                res.get_current_allocation_percentage(),
            ),
        };

        Ok(WipStatusInfo {
            resource_code: resource_code.to_string(),
            resource_name: resource.name().to_string(),
            wip_limits,
            wip_status,
            active_tasks,
            current_allocation,
        })
    }

    pub fn list_wip_status(
        &self,
        status_filter: Option<&str>,
        enabled_only: bool,
    ) -> Result<Vec<WipStatusInfo>, WipAppError> {
        // Get all resources
        let resources = self.resource_repository.find_all()?;

        let mut status_list = Vec::new();

        for resource in resources {
            let (wip_limits, wip_status, active_tasks, current_allocation) = match resource {
                AnyResource::Available(ref res) => (
                    res.get_wip_limits().cloned(),
                    res.get_wip_status(),
                    res.get_active_task_count(),
                    res.get_current_allocation_percentage(),
                ),
                AnyResource::Assigned(ref res) => (
                    res.get_wip_limits().cloned(),
                    res.get_wip_status(),
                    res.get_active_task_count(),
                    res.get_current_allocation_percentage(),
                ),
                AnyResource::Inactive(ref res) => (
                    res.get_wip_limits().cloned(),
                    res.get_wip_status(),
                    res.get_active_task_count(),
                    res.get_current_allocation_percentage(),
                ),
            };

            // Apply filters
            if enabled_only && wip_limits.is_none() {
                continue;
            }

            if let Some(filter_status) = status_filter
                && !wip_status
                    .to_string()
                    .to_lowercase()
                    .contains(&filter_status.to_lowercase())
            {
                continue;
            }

            status_list.push(WipStatusInfo {
                resource_code: resource.code().to_string(),
                resource_name: resource.name().to_string(),
                wip_limits,
                wip_status,
                active_tasks,
                current_allocation,
            });
        }

        Ok(status_list)
    }

    pub fn disable_wip_limits(&self, resource_code: &str) -> Result<(), WipAppError> {
        // Find the resource
        let resource = self
            .resource_repository
            .find_by_code(resource_code)?
            .ok_or_else(|| WipAppError::ResourceNotFound(resource_code.to_string()))?;

        // Disable WIP limits
        let updated_resource = match resource {
            AnyResource::Available(mut res) => {
                res.disable_wip_limits();
                AnyResource::Available(res)
            }
            AnyResource::Assigned(mut res) => {
                res.disable_wip_limits();
                AnyResource::Assigned(res)
            }
            AnyResource::Inactive(mut res) => {
                res.disable_wip_limits();
                AnyResource::Inactive(res)
            }
        };

        // Save the updated resource
        self.resource_repository
            .save(updated_resource)
            .map_err(|e: DomainError| WipAppError::RepositoryError(e.into()))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WipStatusInfo {
    pub resource_code: String,
    pub resource_name: String,
    pub wip_limits: Option<WipLimits>,
    pub wip_status: WipStatus,
    pub active_tasks: u32,
    pub current_allocation: u32,
}

impl WipStatusInfo {
    pub fn display_table(status_list: &[WipStatusInfo]) {
        if status_list.is_empty() {
            println!("No resources found.");
            return;
        }

        let mut table = TableFormatter::new(vec![
            "RESOURCE".to_string(),
            "NAME".to_string(),
            "WIP STATUS".to_string(),
            "ACTIVE TASKS".to_string(),
            "ALLOCATION %".to_string(),
            "MAX TASKS".to_string(),
            "MAX ALLOCATION %".to_string(),
        ]);

        for status in status_list {
            let max_tasks = status
                .wip_limits
                .as_ref()
                .map(|l| l.max_concurrent_tasks.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let max_allocation = status
                .wip_limits
                .as_ref()
                .map(|l| l.max_allocation_percentage.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            table.add_row(vec![
                status.resource_code.clone(),
                status.resource_name.clone(),
                status.wip_status.to_string(),
                status.active_tasks.to_string(),
                format!("{}%", status.current_allocation),
                max_tasks,
                max_allocation,
            ]);
        }

        println!("{}", table);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::resource::Resource;
    use crate::domain::shared::errors::DomainResult;
    use std::{cell::RefCell, collections::HashMap};

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> DomainResult<AnyResource> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_all(&self) -> DomainResult<Vec<AnyResource>> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> DomainResult<Option<AnyResource>> {
            Ok(self.resources.borrow().get(code).cloned())
        }
        fn find_by_company(&self, _company_code: &str) -> DomainResult<Vec<AnyResource>> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> DomainResult<Vec<(AnyResource, String, Vec<String>)>> {
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> DomainResult<AnyResource> {
            self.save(resource)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> DomainResult<AnyResource> {
            unimplemented!()
        }

        fn check_if_layoff_period(
            &self,
            _start_date: &chrono::DateTime<chrono::Local>,
            _end_date: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, _resource_type: &str) -> DomainResult<String> {
            unimplemented!()
        }
    }

    fn create_test_resource(code: &str, name: &str) -> AnyResource {
        use crate::domain::resource_management::resource::ResourceScope;
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            160,
        )
        .into()
    }

    #[test]
    fn test_set_wip_limits_success() {
        // Arrange
        let resource = create_test_resource("RES-001", "Test Resource");
        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource)])),
        };
        let use_case = WipUseCase::new(repo);

        // Act
        let result = use_case.set_wip_limits("RES-001", 3, 2, 80);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_wip_limits_resource_not_found() {
        // Arrange
        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::new()),
        };
        let use_case = WipUseCase::new(repo);

        // Act
        let result = use_case.set_wip_limits("NONEXISTENT", 3, 2, 80);

        // Assert
        assert!(matches!(result, Err(WipAppError::ResourceNotFound(_))));
    }

    #[test]
    fn test_get_wip_status_success() {
        // Arrange
        let resource = create_test_resource("RES-001", "Test Resource");
        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource)])),
        };
        let use_case = WipUseCase::new(repo);

        // Act
        let result = use_case.get_wip_status("RES-001");

        // Assert
        assert!(result.is_ok());
        let status = result.unwrap();
        assert_eq!(status.resource_code, "RES-001");
        assert_eq!(status.resource_name, "Test Resource");
    }

    #[test]
    fn test_disable_wip_limits_success() {
        // Arrange
        let resource = create_test_resource("RES-001", "Test Resource");
        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource)])),
        };
        let use_case = WipUseCase::new(repo);

        // Act
        let result = use_case.disable_wip_limits("RES-001");

        // Assert
        assert!(result.is_ok());
    }
}
