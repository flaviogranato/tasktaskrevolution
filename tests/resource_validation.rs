//! Integration tests for resource validation functionality

use assert_fs::TempDir;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_resource_conflict_detection() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    // Initialize workspace
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["create", "company", "TECH-001", "Tech Corp"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "project", "WEB-APP", "Web Application",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "resource", "DEV-001", "John Doe",
        "--company", "TECH-001",
        "--role", "Developer"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create first task with resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-001", "Implement authentication",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--due-date", "2025-01-15",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Try to create second task with same resource in overlapping period
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-002", "Implement database",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-10",
        "--due-date", "2025-01-25",
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
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["create", "company", "TECH-001", "Tech Corp"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "project", "WEB-APP", "Web Application",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "resource", "DEV-001", "John Doe",
        "--company", "TECH-001",
        "--role", "Developer"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create task on a weekend (should work but might have warnings)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-001", "Weekend task",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-04", // Saturday
        "--due-date", "2025-01-05",   // Sunday
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
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["create", "company", "TECH-001", "Tech Corp"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "project", "WEB-APP", "Web Application",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create multiple resources
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "resource", "DEV-001", "John Doe",
        "--company", "TECH-001",
        "--role", "Developer"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "resource", "DEV-002", "Jane Smith",
        "--company", "TECH-001",
        "--role", "Developer"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create task with multiple resources
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-001", "Team task",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--due-date", "2025-01-15",
        "--assigned-resources", "DEV-001", "DEV-002"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create another task with overlapping resources
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-002", "Conflicting task",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-10",
        "--due-date", "2025-01-25",
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
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["workspace", "init"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a company
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args(["create", "company", "TECH-001", "Tech Corp"])
        .current_dir(base_path);
    cmd.assert().success();

    // Create a project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "project", "WEB-APP", "Web Application",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--end-date", "2025-12-31"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create a resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "resource", "DEV-001", "John Doe",
        "--company", "TECH-001",
        "--role", "Developer"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create first task
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-001", "First task",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-01-01",
        "--due-date", "2025-01-15",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    // Create second task with non-overlapping dates
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.args([
        "create", "task", "TASK-002", "Second task",
        "--project", "WEB-APP",
        "--company", "TECH-001",
        "--start-date", "2025-02-01",
        "--due-date", "2025-02-15",
        "--assigned-resources", "DEV-001"
    ])
    .current_dir(base_path);
    cmd.assert().success();

    Ok(())
}
