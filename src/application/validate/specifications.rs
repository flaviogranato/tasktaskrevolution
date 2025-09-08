use crate::domain::{
    
    
    company_management::company::Company, project_management::any_project::AnyProject,
    resource_management::any_resource::AnyResource, shared::specification::Specification,
};

/// Specification to check if a project has valid date range
pub struct ValidProjectDateRangeSpec;

impl Specification<AnyProject> for ValidProjectDateRangeSpec {
    fn is_satisfied_by(&self, project: &AnyProject) -> bool {
        if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
            start < end
        } else {
            true // If dates are not set, consider valid
        }
    }

    fn description(&self) -> &str {
        "Project must have valid date range (start < end)"
    }

    fn explain_why_not_satisfied(&self, project: &AnyProject) -> Option<String> {
        if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
            if start >= end {
                Some(format!(
                    "Project '{}' has invalid date range: start ({}) >= end ({})",
                    project.code(),
                    start.format("%d/%m/%Y"),
                    end.format("%d/%m/%Y")
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Specification to check if a task is within project timeline
pub struct TaskWithinProjectTimelineSpec;

impl Specification<AnyProject> for TaskWithinProjectTimelineSpec {
    fn is_satisfied_by(&self, project: &AnyProject) -> bool {
        if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
            project.tasks().values().all(|task| {
                let task_start = task.start_date();
                let task_end = task.due_date();
                *task_start >= start && *task_end <= end
            })
        } else {
            true // If project dates are not set, consider valid
        }
    }

    fn description(&self) -> &str {
        "All tasks must be within project timeline"
    }

    fn explain_why_not_satisfied(&self, project: &AnyProject) -> Option<String> {
        if let (Some(start), Some(end)) = (project.start_date(), project.end_date()) {
            let out_of_bounds_tasks: Vec<_> = project
                .tasks()
                .values()
                .filter(|task| {
                    let task_start = task.start_date();
                    let task_end = task.due_date();
                    *task_start < start || *task_end > end
                })
                .collect();

            if out_of_bounds_tasks.is_empty() {
                None
            } else {
                let task_details: Vec<_> = out_of_bounds_tasks
                    .iter()
                    .map(|task| format!("'{}'", task.code()))
                    .collect();
                Some(format!(
                    "Tasks {} are outside project timeline ({} to {})",
                    task_details.join(", "),
                    start.format("%d/%m/%Y"),
                    end.format("%d/%m/%Y")
                ))
            }
        } else {
            None
        }
    }
}

/// Specification to check if a resource has valid vacation periods
pub struct ValidResourceVacationSpec;

impl Specification<AnyResource> for ValidResourceVacationSpec {
    fn is_satisfied_by(&self, resource: &AnyResource) -> bool {
        if let Some(vacations) = resource.vacations() {
            vacations
                .iter()
                .all(|period| period.start_date.date_naive() < period.end_date.date_naive())
        } else {
            true // No vacations means valid
        }
    }

    fn description(&self) -> &str {
        "Resource vacation periods must have valid date ranges"
    }

    fn explain_why_not_satisfied(&self, resource: &AnyResource) -> Option<String> {
        if let Some(vacations) = resource.vacations() {
            let invalid_vacations: Vec<_> = vacations
                .iter()
                .filter(|period| period.start_date.date_naive() >= period.end_date.date_naive())
                .collect();

            if invalid_vacations.is_empty() {
                None
            } else {
                let vacation_details: Vec<_> = invalid_vacations
                    .iter()
                    .map(|period| {
                        format!(
                            "{} to {}",
                            period.start_date.format("%d/%m/%Y"),
                            period.end_date.format("%d/%m/%Y")
                        )
                    })
                    .collect();
                Some(format!(
                    "Resource '{}' has invalid vacation periods: {}",
                    resource.code(),
                    vacation_details.join(", ")
                ))
            }
        } else {
            None
        }
    }
}

/// Specification to check if a company has valid settings
pub struct ValidCompanySettingsSpec;

impl Specification<Company> for ValidCompanySettingsSpec {
    fn is_satisfied_by(&self, company: &Company) -> bool {
        !company.code.trim().is_empty() && !company.name.trim().is_empty()
    }

    fn description(&self) -> &str {
        "Company must have valid code and name"
    }

    fn explain_why_not_satisfied(&self, company: &Company) -> Option<String> {
        let mut issues = Vec::new();

        if company.code.trim().is_empty() {
            issues.push("invalid code format".to_string());
        }

        if company.name.trim().is_empty() {
            issues.push("invalid name".to_string());
        }

        if issues.is_empty() {
            None
        } else {
            Some(format!(
                "Company '{}' has {}: {}",
                company.code,
                if issues.len() == 1 { "issue" } else { "issues" },
                issues.join(", ")
            ))
        }
    }
}

/// Specification to check if a project has assigned resources
pub struct ProjectHasAssignedResourcesSpec;

impl Specification<AnyProject> for ProjectHasAssignedResourcesSpec {
    fn is_satisfied_by(&self, project: &AnyProject) -> bool {
        !project.resources().is_empty()
    }

    fn description(&self) -> &str {
        "Project must have at least one assigned resource"
    }

    fn explain_why_not_satisfied(&self, project: &AnyProject) -> Option<String> {
        if project.resources().is_empty() {
            Some(format!("Project '{}' has no assigned resources", project.code()))
        } else {
            None
        }
    }
}

/// Specification to check if a project has tasks
pub struct ProjectHasTasksSpec;

impl Specification<AnyProject> for ProjectHasTasksSpec {
    fn is_satisfied_by(&self, project: &AnyProject) -> bool {
        !project.tasks().is_empty()
    }

    fn description(&self) -> &str {
        "Project must have at least one task"
    }

    fn explain_why_not_satisfied(&self, project: &AnyProject) -> Option<String> {
        if project.tasks().is_empty() {
            Some(format!("Project '{}' has no tasks", project.code()))
        } else {
            None
        }
    }
}
