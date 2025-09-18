#![allow(dead_code, clippy::to_string_in_format_args)]

use crate::application::errors::AppError;
use crate::domain::resource_management::{
    any_resource::AnyResource, repository::ResourceRepository, resource::WipStatus,
};
use crate::interface::cli::table_formatter::TableFormatter;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipReport {
    pub generated_at: DateTime<Local>,
    pub total_resources: usize,
    pub resources_with_wip_limits: usize,
    pub resources_within_limits: usize,
    pub resources_near_limit: usize,
    pub resources_exceeded: usize,
    pub resources_disabled: usize,
    pub average_active_tasks: f64,
    pub average_allocation: f64,
    pub resource_details: Vec<WipResourceDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WipResourceDetail {
    pub resource_code: String,
    pub resource_name: String,
    pub resource_type: String,
    pub wip_status: WipStatus,
    pub active_tasks: u32,
    pub current_allocation: u32,
    pub max_tasks: Option<u32>,
    pub max_allocation: Option<u8>,
    pub wip_limits_enabled: bool,
    pub utilization_percentage: f64,
}

#[derive(Debug)]
pub enum WipReportError {
    RepositoryError(AppError),
    ReportGenerationError(String),
}

impl fmt::Display for WipReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WipReportError::RepositoryError(err) => write!(f, "Repository error: {}", err),
            WipReportError::ReportGenerationError(message) => write!(f, "Report generation error: {}", message),
        }
    }
}

impl std::error::Error for WipReportError {}

impl From<AppError> for WipReportError {
    fn from(err: AppError) -> Self {
        WipReportError::RepositoryError(err)
    }
}

pub struct WipReportUseCase<RR>
where
    RR: ResourceRepository,
{
    resource_repository: RR,
}

impl<RR> WipReportUseCase<RR>
where
    RR: ResourceRepository,
{
    pub fn new(resource_repository: RR) -> Self {
        Self { resource_repository }
    }

    pub fn generate_wip_report(&self) -> Result<WipReport, WipReportError> {
        // Get all resources
        let resources = self.resource_repository.find_all()?;

        let mut resource_details = Vec::new();
        let mut total_resources = 0;
        let mut resources_with_wip_limits = 0;
        let mut resources_within_limits = 0;
        let mut resources_near_limit = 0;
        let mut resources_exceeded = 0;
        let mut resources_disabled = 0;
        let mut total_active_tasks = 0;
        let mut total_allocation = 0;

        for resource in resources {
            total_resources += 1;

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

            let wip_limits_enabled = wip_limits.is_some() && wip_limits.as_ref().unwrap().enabled;
            if wip_limits_enabled {
                resources_with_wip_limits += 1;
            }

            match wip_status {
                WipStatus::WithinLimits => resources_within_limits += 1,
                WipStatus::NearLimit => resources_near_limit += 1,
                WipStatus::Exceeded => resources_exceeded += 1,
                WipStatus::Disabled => resources_disabled += 1,
            }

            total_active_tasks += active_tasks as usize;
            total_allocation += current_allocation as usize;

            let utilization_percentage = if let Some(ref limits) = wip_limits {
                if limits.enabled && limits.max_concurrent_tasks > 0 {
                    (active_tasks as f64 / limits.max_concurrent_tasks as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            };

            resource_details.push(WipResourceDetail {
                resource_code: resource.code().to_string(),
                resource_name: resource.name().to_string(),
                resource_type: resource.resource_type().to_string(),
                wip_status,
                active_tasks,
                current_allocation,
                max_tasks: wip_limits.as_ref().map(|l| l.max_concurrent_tasks),
                max_allocation: wip_limits.as_ref().map(|l| l.max_allocation_percentage),
                wip_limits_enabled,
                utilization_percentage,
            });
        }

        let average_active_tasks = if total_resources > 0 {
            total_active_tasks as f64 / total_resources as f64
        } else {
            0.0
        };

        let average_allocation = if total_resources > 0 {
            total_allocation as f64 / total_resources as f64
        } else {
            0.0
        };

        Ok(WipReport {
            generated_at: Local::now(),
            total_resources,
            resources_with_wip_limits,
            resources_within_limits,
            resources_near_limit,
            resources_exceeded,
            resources_disabled,
            average_active_tasks,
            average_allocation,
            resource_details,
        })
    }

    pub fn generate_wip_summary(&self) -> Result<String, WipReportError> {
        let report = self.generate_wip_report()?;

        let mut summary = String::new();
        summary.push_str("=== WIP Status Report ===\n");
        summary.push_str(&format!(
            "Generated at: {}\n",
            report.generated_at.format("%Y-%m-%d %H:%M:%S")
        ));
        summary.push_str(&format!("Total Resources: {}\n", report.total_resources));
        summary.push_str(&format!(
            "Resources with WIP Limits: {}\n",
            report.resources_with_wip_limits
        ));
        summary.push_str(&format!(
            "Resources within limits: {}\n",
            report.resources_within_limits
        ));
        summary.push_str(&format!("Resources near limit: {}\n", report.resources_near_limit));
        summary.push_str(&format!("Resources exceeded: {}\n", report.resources_exceeded));
        summary.push_str(&format!("Resources with disabled WIP: {}\n", report.resources_disabled));
        summary.push_str(&format!("Average active tasks: {:.1}\n", report.average_active_tasks));
        summary.push_str(&format!("Average allocation: {:.1}%\n", report.average_allocation));

        Ok(summary)
    }

    pub fn display_wip_report_table(&self) -> Result<(), WipReportError> {
        let report = self.generate_wip_report()?;

        if report.resource_details.is_empty() {
            println!("No resources found.");
            return Ok(());
        }

        let mut table = TableFormatter::new(vec![
            "RESOURCE".to_string(),
            "NAME".to_string(),
            "TYPE".to_string(),
            "WIP STATUS".to_string(),
            "ACTIVE TASKS".to_string(),
            "ALLOCATION %".to_string(),
            "UTILIZATION %".to_string(),
            "WIP ENABLED".to_string(),
        ]);

        for detail in &report.resource_details {
            table.add_row(vec![
                detail.resource_code.clone(),
                detail.resource_name.clone(),
                detail.resource_type.clone(),
                detail.wip_status.to_string(),
                detail.active_tasks.to_string(),
                format!("{}%", detail.current_allocation),
                format!("{:.1}%", detail.utilization_percentage),
                if detail.wip_limits_enabled { "Yes" } else { "No" }.to_string(),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    pub fn export_wip_report_csv(&self, file_path: &str) -> Result<(), WipReportError> {
        let report = self.generate_wip_report()?;
        let mut csv_content = String::new();

        // CSV header
        csv_content.push_str("Resource Code,Resource Name,Resource Type,WIP Status,Active Tasks,Allocation %,Utilization %,WIP Enabled,Max Tasks,Max Allocation %\n");

        // CSV data
        for detail in &report.resource_details {
            csv_content.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                detail.resource_code,
                detail.resource_name,
                detail.resource_type,
                detail.wip_status.to_string(),
                detail.active_tasks,
                detail.current_allocation,
                detail.utilization_percentage,
                if detail.wip_limits_enabled { "Yes" } else { "No" },
                detail
                    .max_tasks
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                detail
                    .max_allocation
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
            ));
        }

        std::fs::write(file_path, csv_content)
            .map_err(|e| WipReportError::ReportGenerationError(format!("Failed to write CSV file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resource_management::resource::Resource;
    use std::{cell::RefCell, collections::HashMap};

    struct MockResourceRepository {
        resources: RefCell<HashMap<String, AnyResource>>,
    }

    impl ResourceRepository for MockResourceRepository {
        fn save(&self, resource: AnyResource) -> Result<AnyResource, AppError> {
            self.resources
                .borrow_mut()
                .insert(resource.code().to_string(), resource.clone());
            Ok(resource)
        }

        fn find_all(&self) -> Result<Vec<AnyResource>, AppError> {
            Ok(self.resources.borrow().values().cloned().collect())
        }

        fn find_by_code(&self, code: &str) -> Result<Option<AnyResource>, AppError> {
            Ok(self.resources.borrow().get(code).cloned())
        }
        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            Ok(vec![])
        }
            Ok(vec![])
        }

        fn save_in_hierarchy(
            &self,
            resource: AnyResource,
            _company_code: &str,
            _project_code: Option<&str>,
        ) -> Result<AnyResource, AppError> {
            self.save(resource)
        }

        fn save_time_off(
            &self,
            _resource_name: &str,
            _hours: u32,
            _date: &str,
            _description: Option<String>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn save_vacation(
            &self,
            _resource_name: &str,
            _start_date: &str,
            _end_date: &str,
            _is_time_off_compensation: bool,
            _compensated_hours: Option<u32>,
        ) -> Result<AnyResource, AppError> {
            unimplemented!()
        }

        fn check_if_layoff_period(
            &self,
            _start_date: &chrono::DateTime<chrono::Local>,
            _end_date: &chrono::DateTime<chrono::Local>,
        ) -> bool {
            unimplemented!()
        }

        fn get_next_code(&self, _resource_type: &str) -> Result<String, AppError> {
            unimplemented!()
        }
    }

    fn create_test_resource(code: &str, name: &str, resource_type: &str) -> AnyResource {
        Resource::new(
            code.to_string(),
            name.to_string(),
            None,
            resource_type.to_string(),
            None,
            None,
            None,
            160,
        )
        .into()
    }

    #[test]
    fn test_generate_wip_report() {
        // Arrange
        let resource1 = create_test_resource("RES-001", "Developer 1", "Developer");
        let resource2 = create_test_resource("RES-002", "Developer 2", "Developer");

        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([
                (resource1.code().to_string(), resource1),
                (resource2.code().to_string(), resource2),
            ])),
        };
        let use_case = WipReportUseCase::new(repo);

        // Act
        let result = use_case.generate_wip_report();

        // Assert
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.total_resources, 2);
        assert_eq!(report.resource_details.len(), 2);
    }

    #[test]
    fn test_generate_wip_summary() {
        // Arrange
        let resource = create_test_resource("RES-001", "Developer 1", "Developer");
        let repo = MockResourceRepository {
            resources: RefCell::new(HashMap::from([(resource.code().to_string(), resource)])),
        };
        let use_case = WipReportUseCase::new(repo);

        // Act
        let result = use_case.generate_wip_summary();

        // Assert
        assert!(result.is_ok());
        let summary = result.unwrap();
        assert!(summary.contains("WIP Status Report"));
        assert!(summary.contains("Total Resources: 1"));
    }
}
