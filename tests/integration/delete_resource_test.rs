use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Test that delete resource command updates status to Inactive instead of creating duplicates
#[test]
fn test_delete_resource_updates_status_to_inactive() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Create initial resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "--name",
        "João Silva",
        "--type",
        "Developer",
        "--email",
        "joao@techcorp.com",
        "--code",
        "dev-001",
        "--company",
        "TECH-CORP",
        "--description",
        "Desenvolvedor Senior",
    ]);

    cmd.assert().success();

    // Verify initial resource file exists
    let resource_file = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("resources")
        .join("dev-001.yaml");

    assert!(resource_file.exists(), "Resource file should exist after creation");

    // Verify initial status is Available
    let content = std::fs::read_to_string(&resource_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("status"))
            .and_then(|s| s.as_str()),
        Some("Available")
    );

    // Delete (deactivate) the resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["delete", "resource", "--code", "dev-001", "--company", "TECH-CORP"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Resource deactivated successfully!"));

    // Verify only one file exists (no duplication)
    let resources_dir = temp.path().join("companies").join("TECH-CORP").join("resources");

    let files: Vec<_> = std::fs::read_dir(&resources_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();

    assert_eq!(files.len(), 1, "Should have exactly one resource file, not duplicated");

    // Verify the file was updated with Inactive status
    let content = std::fs::read_to_string(&resource_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("status"))
            .and_then(|s| s.as_str()),
        Some("Inactive")
    );

    // Verify other fields remain unchanged
    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str()),
        Some("João Silva")
    );

    temp.close()?;
    Ok(())
}

/// Test that delete resource command shows error when resource not found
#[test]
fn test_delete_resource_shows_error_when_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Try to delete non-existent resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["delete", "resource", "--code", "non-existent", "--company", "TECH-CORP"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to deactivate resource"));

    temp.close()?;
    Ok(())
}

fn setup_basic_environment(temp: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    // Create config.yaml in root
    let config_yaml = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  version: "0.5.6"
  description: "TTR Configuration"
spec:
  defaults:
    timezone: "America/Sao_Paulo"
    workDays: [Monday, Tuesday, Wednesday, Thursday, Friday]
  vacationRules:
    maxConcurrentVacations: 10
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
"#;
    std::fs::write(temp.path().join("config.yaml"), config_yaml)?;

    // Create basic directory structure
    let companies_dir = temp.path().join("companies").join("TECH-CORP");
    std::fs::create_dir_all(&companies_dir)?;

    // Create company.yaml
    let company_yaml = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  code: TECH-CORP
  name: Tech Corp
  description: Technology company
spec:
  timezone: America/Sao_Paulo
  workDays: [Monday, Tuesday, Wednesday, Thursday, Friday]
  vacationRules:
    maxConcurrentVacations: 10
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
"#;
    std::fs::write(companies_dir.join("company.yaml"), company_yaml)?;

    Ok(())
}
