use crate::infrastructure::persistence::manifests::company_manifest::CompanyManifest;
use crate::domain::company_management::company::Company;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_company_manifest_conversion() {
        let company = Company::new(
            "TECH-001".to_string(),
            "Tech Corp".to_string(),
            "test-user".to_string(),
        ).unwrap();

        println!("Company: {:?}", company);

        let manifest = CompanyManifest::from(&company);
        println!("Manifest: {:?}", manifest);

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        println!("YAML:\n{}", yaml);
    }
}
