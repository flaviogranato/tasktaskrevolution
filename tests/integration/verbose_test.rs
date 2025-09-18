use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Test that debug output is hidden by default
#[test]
fn test_debug_output_hidden_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Test command without verbose flag
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["list", "resources"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DEBUG").not())
        .stdout(predicate::str::contains("[INFO]").not());

    temp.close()?;
    Ok(())
}

/// Test that debug output is shown with verbose flag
#[test]
fn test_debug_output_shown_with_verbose_flag() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Test command with verbose flag
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["--verbose", "list", "resources"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[INFO] Current context:"));

    temp.close()?;
    Ok(())
}

/// Test that debug output is shown with -v flag
#[test]
fn test_debug_output_shown_with_v_flag() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Test command with -v flag
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["-v", "list", "resources"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[INFO] Current context:"));

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
