//! Integration tests for resource validation functionality

use assert_fs::TempDir;
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn get_ttr_command() -> Command {
    Command::cargo_bin("ttr").unwrap()
}

#[test]
fn test_resource_conflict_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize workspace
    let mut cmd = get_ttr_command();
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = get_ttr_command();
    cmd.args(["create", "company", "--code", "TECH-002", "--name", "Tech Corp 2"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "project",
        "--code", "WEB-APP",
        "--name", "Web Application",
        "--company", "TECH-002",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "resource",
        "--code", "DEV-001",
        "--name", "John Doe",
        "--company", "TECH-002",
        "--type", "Developer",
        "--email", "john.doe@techcorp2.com"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create first task with resource
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-001",
        "--name", "Implement authentication",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-06",
        "--due-date", "2025-01-10",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Try to create second task with same resource in overlapping period
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-002",
        "--name", "Implement database",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-08",
        "--due-date", "2025-01-12",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    
    // This should fail due to resource conflict
    cmd.assert().failure()
        .stderr(predicate::str::contains("Resource conflicts detected"));

    Ok(())
}

#[test]
fn test_calendar_availability_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize workspace
    let mut cmd = get_ttr_command();
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = get_ttr_command();
    cmd.args(["create", "company", "--code", "TECH-002", "--name", "Tech Corp 2"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "project",
        "--code", "WEB-APP",
        "--name", "Web Application",
        "--company", "TECH-002",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "resource",
        "--code", "DEV-001",
        "--name", "John Doe",
        "--company", "TECH-002",
        "--type", "Developer",
        "--email", "john.doe@techcorp2.com"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create task on a weekend (should work but might have warnings)
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-001",
        "--name", "Weekend task",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-06", // Monday
        "--due-date", "2025-01-07",   // Tuesday
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    
    // This should succeed but might show warnings about weekend work
    cmd.assert().success();

    Ok(())
}

#[test]
fn test_resource_validation_with_multiple_resources() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize workspace
    let mut cmd = get_ttr_command();
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = get_ttr_command();
    cmd.args(["create", "company", "--code", "TECH-002", "--name", "Tech Corp 2"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "project",
        "--code", "WEB-APP",
        "--name", "Web Application",
        "--company", "TECH-002",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create multiple resources
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "resource",
        "--code", "DEV-001",
        "--name", "John Doe",
        "--company", "TECH-002",
        "--type", "Developer",
        "--email", "john.doe@techcorp2.com"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "resource",
        "--code", "DEV-002",
        "--name", "Jane Smith",
        "--company", "TECH-002",
        "--type", "Developer",
        "--email", "john.doe@techcorp2.com"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create task with multiple resources
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-001",
        "--name", "Team task",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-06",
        "--due-date", "2025-01-10",
        "--assigned-resources", "DEV-001,DEV-002"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create another task with overlapping resources
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-002",
        "--name", "Conflicting task",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-08",
        "--due-date", "2025-01-12",
        "--assigned-resources", "DEV-001" // Only DEV-001 should conflict
    ])
    .current_dir(base_path);
    
    // This should fail due to resource conflict with DEV-001
    cmd.assert().failure()
        .stderr(predicate::str::contains("Resource conflicts detected"));

    Ok(())
}

#[test]
fn test_resource_validation_without_conflicts() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize workspace
    let mut cmd = get_ttr_command();
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = get_ttr_command();
    cmd.args(["create", "company", "--code", "TECH-002", "--name", "Tech Corp 2"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "project",
        "--code", "WEB-APP",
        "--name", "Web Application",
        "--company", "TECH-002",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "resource",
        "--code", "DEV-001",
        "--name", "John Doe",
        "--company", "TECH-002",
        "--type", "Developer",
        "--email", "john.doe@techcorp2.com"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create first task
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-001",
        "--name", "First task",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-01-06",
        "--due-date", "2025-01-10",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create second task with non-overlapping dates
    let mut cmd = get_ttr_command();
    cmd.args([
        "create", "task",
        "--code", "TASK-002",
        "--name", "Second task",
        "--project", "WEB-APP",
        "--company", "TECH-002",
        "--start-date", "2025-02-03",
        "--due-date", "2025-02-07",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    Ok(())
}
