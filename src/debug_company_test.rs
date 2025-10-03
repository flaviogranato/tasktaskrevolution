use crate::infrastructure::persistence::manifests::company_manifest::CompanyManifest;
use crate::domain::company_management::company::{Company, CompanySize, CompanyStatus};
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_serialization() {
        let company = Company::new(
            "TECH-001".to_string(),
            "Tech Corp".to_string(),
            "test-user".to_string(),
        ).unwrap();

        println!("Company ID: {}", company.id);

        let manifest = CompanyManifest::from(&company);
        let yaml = serde_yaml::to_string(&manifest).unwrap();
        
        println!("Generated YAML:");
        println!("{}", yaml);

        // Try to deserialize it back
        let parsed: CompanyManifest = serde_yaml::from_str(&yaml).unwrap();
        println!("Parsed back successfully: {:?}", parsed);
    }
}
