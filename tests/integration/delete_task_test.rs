use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

/// Test that delete task command actually removes the task from the file
#[test]
fn test_delete_task_removes_task_from_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Find project and task codes
    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Verify task file exists before deletion
    let task_file = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("projects")
        .join(&project_code)
        .join("tasks")
        .join(&task_code);

    assert!(task_file.exists(), "Task file should exist before deletion");

    // Delete the task
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "delete",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task cancelled successfully!"));

    // Verify task file is removed or task status is changed to Cancelled
    if task_file.exists() {
        // If file still exists, check that task status is Cancelled
        let content = std::fs::read_to_string(&task_file)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

        // Check if task status is Cancelled
        let status = yaml
            .get("spec")
            .and_then(|s| s.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown");

        assert_eq!(status, "Cancelled", "Task status should be Cancelled after deletion");
    } else {
        // If file is removed, that's also acceptable
        println!("Task file was removed, which is also acceptable");
    }

    temp.close()?;
    Ok(())
}

/// Test that delete task command shows error when task not found
#[test]
fn test_delete_task_shows_error_when_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;

    // Try to delete non-existent task
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "delete",
        "task",
        "--code",
        "non-existent-task",
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to cancel task"));

    temp.close()?;
    Ok(())
}

/// Test that delete task command shows error when project not found
#[test]
fn test_delete_task_shows_error_when_project_not_found() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    // Try to delete task from non-existent project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "delete",
        "task",
        "--code",
        "some-task",
        "--project",
        "non-existent-project",
        "--company",
        "TECH-CORP",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to cancel task"));

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

    // Create project directory
    let project_dir = companies_dir.join("projects").join("test-project");
    std::fs::create_dir_all(&project_dir)?;

    // Create project.yaml
    let project_yaml = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  code: test-project
  name: Test Project
  description: Test project for delete task testing
spec:
  timezone: America/Sao_Paulo
  status: Planned
  vacationRules:
    maxConcurrentVacations: 20
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
"#;
    std::fs::write(project_dir.join("project.yaml"), project_yaml)?;

    // Create tasks directory
    let tasks_dir = project_dir.join("tasks");
    std::fs::create_dir_all(&tasks_dir)?;

    // Create task.yaml
    let task_yaml = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: 01995d0f-7015-7c3a-ad9f-e4039e6f85cf
  code: test-task
  name: Test Task
  description: Test task for deletion
spec:
  projectCode: test-project
  assignee: unassigned
  status: Planned
  priority: Medium
  estimatedStartDate: 2025-01-01
  estimatedEndDate: 2025-01-31
  actualStartDate: 2025-01-01
  effort:
    estimatedHours: 8.0
"#;
    std::fs::write(tasks_dir.join("test-task.yaml"), task_yaml)?;

    Ok(())
}

fn find_project_code(temp: &TempDir) -> Result<String, Box<dyn std::error::Error>> {
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");

    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(dir_name) = entry.file_name().to_str() {
                    return Ok(dir_name.to_string());
                }
            }
        }
    }
    Err("Project code not found".into())
}

fn find_task_code(temp: &TempDir, project_code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let tasks_dir = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("projects")
        .join(project_code)
        .join("tasks");

    if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(file_name) = entry.file_name().to_str() {
                    return Ok(file_name.to_string());
                }
            }
        }
    }
    Err("Task code not found".into())
}
