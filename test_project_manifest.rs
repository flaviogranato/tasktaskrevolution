#[cfg(test)]
mod tests {
    use serde_yaml;
    use std::collections::HashMap;

    #[derive(serde::Deserialize, Debug)]
    struct ProjectManifest {
        metadata: ProjectMetadata,
    }

    #[derive(serde::Deserialize, Debug)]
    struct ProjectMetadata {
        code: String,
        name: String,
        description: String,
        #[serde(rename = "companyCode")]
        company_code: String,
    }

    #[test]
    fn test_project_manifest_deserialization() {
        let yaml = r#"
metadata:
  code: proj-123
  name: Test Project
  description: Test project
  companyCode: TECH-CORP
"#;
        
        match serde_yaml::from_str::<ProjectManifest>(yaml) {
            Ok(manifest) => println!("Success: {:?}", manifest),
            Err(e) => println!("Error: {}", e),
        }
    }
}
