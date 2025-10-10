use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::company_management::company::{Company, CompanySize, CompanyStatus};

const API_VERSION: &str = "tasktaskrevolution.io/v1alpha1";

/// Manifest for serializing/deserializing Company entities to/from YAML.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompanyManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: CompanyMetadata,
    pub spec: CompanySpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<CompanyStatusManifest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompanyMetadata {
    pub id: String,
    pub code: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CompanySpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    pub size: CompanySizeManifest,
    pub status: CompanyStatusManifest,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CompanySizeManifest {
    Small,
    Medium,
    Large,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CompanyStatusManifest {
    Active,
    Inactive,
    Suspended,
}

impl From<&Company> for CompanyManifest {
    fn from(company: &Company) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind: "Company".to_string(),
            metadata: CompanyMetadata {
                id: company.id.clone(),
                code: company.code.clone(),
                name: company.name.clone(),
                created_at: company.created_at,
                updated_at: company.updated_at,
                created_by: company.created_by.clone(),
                labels: None,
                annotations: None,
                namespace: None,
            },
            spec: CompanySpec {
                description: company.description.clone(),
                tax_id: company.tax_id.clone(),
                address: company.address.clone(),
                email: company.email.clone(),
                phone: company.phone.clone(),
                website: company.website.clone(),
                industry: company.industry.clone(),
                size: CompanySizeManifest::from(&company.size),
                status: CompanyStatusManifest::from(&company.status),
            },
            status: Some(CompanyStatusManifest::from(&company.status)),
        }
    }
}

impl From<&CompanySize> for CompanySizeManifest {
    fn from(size: &CompanySize) -> Self {
        match size {
            CompanySize::Small => CompanySizeManifest::Small,
            CompanySize::Medium => CompanySizeManifest::Medium,
            CompanySize::Large => CompanySizeManifest::Large,
        }
    }
}

impl From<&CompanyStatus> for CompanyStatusManifest {
    fn from(status: &CompanyStatus) -> Self {
        match status {
            CompanyStatus::Active => CompanyStatusManifest::Active,
            CompanyStatus::Inactive => CompanyStatusManifest::Inactive,
            CompanyStatus::Suspended => CompanyStatusManifest::Suspended,
        }
    }
}

impl CompanyManifest {
    pub fn to(&self) -> Company {
        Company {
            id: self.metadata.id.clone(),
            code: self.metadata.code.clone(),
            name: self.metadata.name.clone(),
            description: self.spec.description.clone(),
            tax_id: self.spec.tax_id.clone(),
            address: self.spec.address.clone(),
            email: self.spec.email.clone(),
            phone: self.spec.phone.clone(),
            website: self.spec.website.clone(),
            industry: self.spec.industry.clone(),
            size: self.spec.size.to(),
            status: self.spec.status.to(),
            created_at: self.metadata.created_at,
            updated_at: self.metadata.updated_at,
            created_by: self.metadata.created_by.clone(),
        }
    }
}

impl CompanySizeManifest {
    pub fn to(&self) -> CompanySize {
        match self {
            CompanySizeManifest::Small => CompanySize::Small,
            CompanySizeManifest::Medium => CompanySize::Medium,
            CompanySizeManifest::Large => CompanySize::Large,
        }
    }
}

impl CompanyStatusManifest {
    pub fn to(&self) -> CompanyStatus {
        match self {
            CompanyStatusManifest::Active => CompanyStatus::Active,
            CompanyStatusManifest::Inactive => CompanyStatus::Inactive,
            CompanyStatusManifest::Suspended => CompanyStatus::Suspended,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::company_management::company::{Company, CompanySize, CompanyStatus};

    #[test]
    fn test_company_manifest_creation() {
        let company = Company::new(
            "COMP-001".to_string(),
            "TechConsulting Ltda".to_string(),
            "user@example.com".to_string(),
        )
        .unwrap();

        let manifest = CompanyManifest::from(&company);

        assert_eq!(manifest.api_version, API_VERSION);
        assert_eq!(manifest.kind, "Company");
        assert_eq!(manifest.metadata.code, "COMP-001");
        assert_eq!(manifest.metadata.name, "TechConsulting Ltda");
        assert_eq!(manifest.metadata.created_by, "user@example.com");
        assert_eq!(manifest.spec.size, CompanySizeManifest::Medium);
        assert_eq!(manifest.spec.status, CompanyStatusManifest::Active);
    }

    #[test]
    fn test_company_manifest_conversion() {
        let original_company = Company::new(
            "COMP-002".to_string(),
            "Outra Empresa Ltda".to_string(),
            "admin@example.com".to_string(),
        )
        .unwrap();

        let manifest = CompanyManifest::from(&original_company);
        let converted_company = manifest.to();

        assert_eq!(original_company.id, converted_company.id);
        assert_eq!(original_company.code, converted_company.code);
        assert_eq!(original_company.name, converted_company.name);
        assert_eq!(original_company.created_by, converted_company.created_by);
        assert_eq!(original_company.size, converted_company.size);
        assert_eq!(original_company.status, converted_company.status);
    }

    #[test]
    fn test_company_size_manifest_conversion() {
        let sizes = vec![CompanySize::Small, CompanySize::Medium, CompanySize::Large];

        for size in sizes {
            let manifest = CompanySizeManifest::from(&size);
            let converted = manifest.to();
            assert_eq!(size, converted);
        }
    }

    #[test]
    fn test_company_status_manifest_conversion() {
        let statuses = vec![CompanyStatus::Active, CompanyStatus::Inactive, CompanyStatus::Suspended];

        for status in statuses {
            let manifest = CompanyStatusManifest::from(&status);
            let converted = manifest.to();
            assert_eq!(status, converted);
        }
    }

    #[test]
    fn test_yaml_parsing_success() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                code: "TECH-CORP"
                name: "Tech Corp"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                description: "A technology company"
                size: "medium"
                status: "active"
        "#;

        let manifest: CompanyManifest = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Company");
        assert_eq!(manifest.metadata.code, "TECH-CORP");
        assert_eq!(manifest.metadata.name, "Tech Corp");
        assert_eq!(manifest.spec.description, Some("A technology company".to_string()));
        assert_eq!(manifest.spec.size, CompanySizeManifest::Medium);
        assert_eq!(manifest.spec.status, CompanyStatusManifest::Active);
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_syntax() {
        let yaml_str = "invalid: yaml: content: [";
        let result: Result<CompanyManifest, _> = serde_yaml::from_str(yaml_str);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();

        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_missing_required_field() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                # Missing required fields: code, name, createdAt, updatedAt, createdBy
            spec:
                size: "medium"
                status: "active"
        "#;

        let result: Result<CompanyManifest, _> = serde_yaml::from_str(yaml_str);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();

        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_field_type() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                code: "TECH-CORP"
                name: "Tech Corp"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                size: "invalid_size"  # Invalid enum value
                status: "active"
        "#;

        let result: Result<CompanyManifest, _> = serde_yaml::from_str(yaml_str);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();

        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_invalid_date_format() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                code: "TECH-CORP"
                name: "Tech Corp"
                createdAt: "invalid-date"  # Invalid date format
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                size: "medium"
                status: "active"
        "#;

        let result: Result<CompanyManifest, _> = serde_yaml::from_str(yaml_str);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let app_error: crate::application::errors::AppError = error.into();

        let error_message = format!("{}", app_error);
        assert!(error_message.contains("Serialization error for format 'YAML'"));
    }

    #[test]
    fn test_yaml_parsing_failure_wrong_api_version() {
        let yaml_str = r#"
            apiVersion: wrong.api.version/v1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                code: "TECH-CORP"
                name: "Tech Corp"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                size: "medium"
                status: "active"
        "#;

        let result: Result<CompanyManifest, _> = serde_yaml::from_str(yaml_str);

        // This should still parse successfully as we don't validate API version
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.api_version, "wrong.api.version/v1");
    }

    #[test]
    fn test_yaml_parsing_success_with_optional_fields() {
        let yaml_str = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Company
            metadata:
                id: "01996dev-0000-0000-0000-000000techc"
                code: "TECH-CORP"
                name: "Tech Corp"
                createdAt: "2024-01-01T00:00:00Z"
                updatedAt: "2024-01-01T00:00:00Z"
                createdBy: "system"
            spec:
                description: "A technology company"
                taxId: "12.345.678/0001-90"
                address: "123 Tech Street"
                email: "contact@techcorp.com"
                phone: "+55 11 99999-9999"
                website: "https://techcorp.com"
                industry: "Technology"
                size: "large"
                status: "active"
        "#;

        let manifest: CompanyManifest = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(manifest.spec.description, Some("A technology company".to_string()));
        assert_eq!(manifest.spec.tax_id, Some("12.345.678/0001-90".to_string()));
        assert_eq!(manifest.spec.address, Some("123 Tech Street".to_string()));
        assert_eq!(manifest.spec.email, Some("contact@techcorp.com".to_string()));
        assert_eq!(manifest.spec.phone, Some("+55 11 99999-9999".to_string()));
        assert_eq!(manifest.spec.website, Some("https://techcorp.com".to_string()));
        assert_eq!(manifest.spec.industry, Some("Technology".to_string()));
        assert_eq!(manifest.spec.size, CompanySizeManifest::Large);
    }
}
