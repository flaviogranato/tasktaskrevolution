use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Test that global list resources shows company and project codes
#[test]
fn test_global_list_resources_shows_company_and_project_codes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Create resources in different companies and projects
    create_company_resource(
        &temp,
        "TECH-001",
        "dev-001",
        "João Silva",
        "joao@techcorp.com",
        "Developer",
    )?;
    create_company_resource(
        &temp,
        "DESIGN-001",
        "designer-001",
        "Maria Santos",
        "maria@design.com",
        "Designer",
    )?;
    create_project_resource(
        &temp,
        "TECH-001",
        "PROJ-001",
        "qa-001",
        "Ana Costa",
        "ana@techcorp.com",
        "QA",
    )?;

    // Test global listing (from root)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["list", "resources"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("COMPANY"))
        .stdout(predicate::str::contains("PROJECTS"))
        .stdout(predicate::str::contains("TECH-001"))
        .stdout(predicate::str::contains("DESIGN-001"))
        .stdout(predicate::str::contains("PROJ-001"));

    temp.close()?;
    Ok(())
}

/// Test that company-level list resources does not show company and project codes
#[test]
fn test_company_list_resources_does_not_show_company_and_project_codes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Create resources in the company
    create_company_resource(
        &temp,
        "TECH-001",
        "dev-001",
        "João Silva",
        "joao@techcorp.com",
        "Developer",
    )?;

    // Test company-level listing (from company directory)
    let company_dir = temp.path().join("companies").join("TECH-001");
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(&company_dir);
    cmd.args(["list", "resources"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("CODE"))
        .stdout(predicate::str::contains("EMAIL"))
        .stdout(predicate::str::contains("TYPE"))
        .stdout(predicate::str::contains("STATUS"))
        .stdout(predicate::str::contains("COMPANY").not())
        .stdout(predicate::str::contains("PROJECTS").not());

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

    // Create companies directory structure
    std::fs::create_dir_all(temp.path().join("companies"))?;

    Ok(())
}

fn create_company_resource(
    temp: &TempDir,
    company_code: &str,
    resource_code: &str,
    name: &str,
    email: &str,
    resource_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create company directory and file
    let company_dir = temp.path().join("companies").join(company_code);
    std::fs::create_dir_all(&company_dir)?;

    let company_yaml = format!(
        r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  code: {}
  name: {} Company
  description: Test company
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "test@example.com"
spec:
  timezone: America/Sao_Paulo
  size: small
  status: active
  workDays: [Monday, Tuesday, Wednesday, Thursday, Friday]
  vacationRules:
    maxConcurrentVacations: 10
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
"#,
        company_code, company_code
    );
    std::fs::write(company_dir.join("company.yaml"), company_yaml)?;

    // Create resources directory and file
    let resources_dir = company_dir.join("resources");
    std::fs::create_dir_all(&resources_dir)?;

    let resource_yaml = format!(
        r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  name: {}
  email: {}
  code: {}
  resourceType: {}
  status: Available
spec:
  scope: Company
  timeOffBalance: 0
  timeOffHistory: []
"#,
        name, email, resource_code, resource_type
    );
    std::fs::write(resources_dir.join(format!("{}.yaml", resource_code)), resource_yaml)?;

    Ok(())
}

fn create_project_resource(
    temp: &TempDir,
    company_code: &str,
    project_code: &str,
    resource_code: &str,
    name: &str,
    email: &str,
    resource_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create project directory structure
    let project_dir = temp
        .path()
        .join("companies")
        .join(company_code)
        .join("projects")
        .join(project_code);
    std::fs::create_dir_all(&project_dir)?;

    // Create project.yaml
    let project_yaml = format!(
        r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  code: {}
  name: {} Project
  description: Test project
spec:
  timezone: America/Sao_Paulo
  status: Planned
  vacationRules:
    maxConcurrentVacations: 20
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
"#,
        project_code, project_code
    );
    std::fs::write(project_dir.join("project.yaml"), project_yaml)?;

    // Create resources directory and file
    let resources_dir = project_dir.join("resources");
    std::fs::create_dir_all(&resources_dir)?;

    let resource_yaml = format!(
        r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  name: {}
  email: {}
  code: {}
  resourceType: {}
  status: Available
spec:
  scope: Company
  timeOffBalance: 0
  timeOffHistory: []
"#,
        name, email, resource_code, resource_type
    );
    std::fs::write(resources_dir.join(format!("{}.yaml", resource_code)), resource_yaml)?;

    Ok(())
}
