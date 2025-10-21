use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Financial context that determines the scope of budget and cost operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinancialContext {
    /// Company-level financial operations
    Company {
        company_code: String,
    },
    /// Project-level financial operations
    Project {
        company_code: String,
        project_code: String,
    },
}

impl FinancialContext {
    /// Create a company financial context
    pub fn company(company_code: String) -> Self {
        Self::Company { company_code }
    }

    /// Create a project financial context
    pub fn project(company_code: String, project_code: String) -> Self {
        Self::Project {
            company_code,
            project_code,
        }
    }

    /// Get the company code from context
    pub fn company_code(&self) -> &str {
        match self {
            FinancialContext::Company { company_code } => company_code,
            FinancialContext::Project { company_code, .. } => company_code,
        }
    }

    /// Get the project code if available
    pub fn project_code(&self) -> Option<&str> {
        match self {
            FinancialContext::Company { .. } => None,
            FinancialContext::Project { project_code, .. } => Some(project_code),
        }
    }

    /// Check if this is a project context
    pub fn is_project_context(&self) -> bool {
        matches!(self, FinancialContext::Project { .. })
    }

    /// Check if this is a company context
    pub fn is_company_context(&self) -> bool {
        matches!(self, FinancialContext::Company { .. })
    }

    /// Get the path prefix for file operations
    pub fn path_prefix(&self) -> String {
        match self {
            FinancialContext::Company { company_code } => {
                format!("companies/{}/", company_code)
            }
            FinancialContext::Project {
                company_code,
                project_code,
            } => {
                format!("companies/{}/projects/{}/", company_code, project_code)
            }
        }
    }

    /// Get context metadata for labels
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        match self {
            FinancialContext::Company { company_code } => {
                metadata.insert("company".to_string(), company_code.clone());
                metadata.insert("scope".to_string(), "company".to_string());
            }
            FinancialContext::Project {
                company_code,
                project_code,
            } => {
                metadata.insert("company".to_string(), company_code.clone());
                metadata.insert("project".to_string(), project_code.clone());
                metadata.insert("scope".to_string(), "project".to_string());
            }
        }
        metadata
    }
}

impl std::fmt::Display for FinancialContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinancialContext::Company { company_code } => {
                write!(f, "Company: {}", company_code)
            }
            FinancialContext::Project {
                company_code,
                project_code,
            } => {
                write!(f, "Project: {}/{}", company_code, project_code)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_context() {
        let context = FinancialContext::company("TECH-CORP".to_string());
        assert_eq!(context.company_code(), "TECH-CORP");
        assert_eq!(context.project_code(), None);
        assert!(context.is_company_context());
        assert!(!context.is_project_context());
        assert_eq!(context.path_prefix(), "companies/TECH-CORP/");
    }

    #[test]
    fn test_project_context() {
        let context = FinancialContext::project("TECH-CORP".to_string(), "ECOMM-001".to_string());
        assert_eq!(context.company_code(), "TECH-CORP");
        assert_eq!(context.project_code(), Some("ECOMM-001"));
        assert!(!context.is_company_context());
        assert!(context.is_project_context());
        assert_eq!(context.path_prefix(), "companies/TECH-CORP/projects/ECOMM-001/");
    }

    #[test]
    fn test_context_metadata() {
        let company_context = FinancialContext::company("TECH-CORP".to_string());
        let metadata = company_context.metadata();
        assert_eq!(metadata.get("company"), Some(&"TECH-CORP".to_string()));
        assert_eq!(metadata.get("scope"), Some(&"company".to_string()));

        let project_context = FinancialContext::project("TECH-CORP".to_string(), "ECOMM-001".to_string());
        let metadata = project_context.metadata();
        assert_eq!(metadata.get("company"), Some(&"TECH-CORP".to_string()));
        assert_eq!(metadata.get("project"), Some(&"ECOMM-001".to_string()));
        assert_eq!(metadata.get("scope"), Some(&"project".to_string()));
    }
}
