use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Test that update resource command updates existing file instead of creating duplicate
#[test]
fn test_update_resource_updates_existing_file() -> Result<(), Box<dyn std::error::Error>> {
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

    // Update the resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "resource",
        "--code",
        "dev-001",
        "--company",
        "TECH-CORP",
        "--name",
        "João Silva Santos",
        "--email",
        "joao.silva@techcorp.com",
        "--description",
        "Tech Lead Senior",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Resource updated successfully!"));

    // Verify only one file exists (no duplication)
    let resources_dir = temp.path().join("companies").join("TECH-CORP").join("resources");

    let files: Vec<_> = std::fs::read_dir(&resources_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();

    assert_eq!(files.len(), 1, "Should have exactly one resource file, not duplicated");

    // Verify the file was updated with new content
    let content = std::fs::read_to_string(&resource_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str()),
        Some("João Silva Santos")
    );

    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("email"))
            .and_then(|e| e.as_str()),
        Some("joao.silva@techcorp.com")
    );

    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("resourceType"))
            .and_then(|t| t.as_str()),
        Some("Tech Lead Senior")
    );

    temp.close()?;
    Ok(())
}

/// Test that update resource command shows error when resource not found
#[test]
fn test_update_resource_shows_error_when_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Try to update non-existent resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "resource",
        "--code",
        "non-existent",
        "--company",
        "TECH-CORP",
        "--name",
        "New Name",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to update resource"));

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
