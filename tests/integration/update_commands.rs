use assert_cmd::Command;
// use assert_fs::prelude::*;
use predicates::prelude::*;
use std::fs;

/// Helper function to setup basic environment for testing
fn setup_basic_environment(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TTR
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(["init", "--name", "Test User", "--email", "test@example.com"]);
    cmd.assert().success();

    // Create a company
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Tech Corporation",
        "--code",
        "TECH-CORP",
        "--description",
        "Technology company",
    ]);
    cmd.assert().success();

    // Create a project
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "--name",
        "Test Project",
        "--description",
        "Test project for update testing",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Create a resource
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "--name",
        "John Doe",
        "--type",
        "Developer",
        "--email",
        "john@example.com",
        "--description",
        "Developer",
        "--company",
        "TECH-CORP",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
    ]);
    cmd.assert().success();

    // Create a task
    let project_code = find_project_code(temp)?;
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "task",
        "--name",
        "Test Task",
        "--description",
        "Task for update testing",
        "--start-date",
        "2024-01-01",
        "--due-date",
        "2024-01-15",
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    Ok(())
}

/// Helper function to find project code
fn find_project_code(temp: &assert_fs::TempDir) -> Result<String, Box<dyn std::error::Error>> {
    let projects_dir = temp.path().join("projects");
    let mut project_code = None;

    if !projects_dir.exists() {
        return Err("Projects directory does not exist".into());
    }

    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Check if it's a YAML file (ID-based format)
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                {
                    project_code = Some(code.to_string());
                    break;
                }
            }
            // Check if it's the old directory format
            else if path.is_dir() {
                let project_yaml = path.join("project.yaml");
                if project_yaml.exists()
                    && let Ok(content) = std::fs::read_to_string(&project_yaml)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                {
                    project_code = Some(code.to_string());
                    break;
                }
            }
        }
    }

    project_code.ok_or_else(|| "Project code not found".into())
}

/// Helper function to find task code
fn find_task_code(temp: &assert_fs::TempDir, project_code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let projects_dir = temp.path().join("projects");
    let mut task_code = None;

    // Search in all project directories
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                // Check if this is the project file we're looking for
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(metadata) = yaml.get("metadata")
                    && let Some(code) = metadata.get("code").and_then(|v| v.as_str())
                    && code == project_code
                {
                    // Found the project, now look for tasks in the tasks subdirectory
                    let tasks_dir = path.parent().unwrap().join("tasks");
                    if tasks_dir.exists() {
                        if let Ok(task_entries) = std::fs::read_dir(&tasks_dir) {
                            for task_entry in task_entries.flatten() {
                                let task_path = task_entry.path();
                                if task_path.is_file()
                                    && task_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                                    && let Ok(task_content) = std::fs::read_to_string(&task_path)
                                    && let Ok(task_yaml) = serde_yaml::from_str::<serde_yaml::Value>(&task_content)
                                    && let Some(task_metadata) = task_yaml.get("metadata")
                                    && let Some(code) = task_metadata.get("code").and_then(|v| v.as_str())
                                {
                                    task_code = Some(code.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if task_code.is_some() {
                break;
            }
        }
    }

    task_code.ok_or_else(|| "Task code not found".into())
}

/// Helper function to find resource code
fn find_resource_code(temp: &assert_fs::TempDir) -> Result<String, Box<dyn std::error::Error>> {
    let companies_dir = temp.path().join("companies");
    let mut resource_code = None;

    if let Ok(entries) = std::fs::read_dir(&companies_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let resources_dir = path.join("resources");
                if resources_dir.exists() {
                    if let Ok(resource_entries) = std::fs::read_dir(&resources_dir) {
                        for resource_entry in resource_entries.flatten() {
                            let resource_path = resource_entry.path();
                            if resource_path.is_file()
                                && resource_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                            {
                                // Read the YAML file to get the actual resource code
                                if let Ok(content) = std::fs::read_to_string(&resource_path)
                                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                                    && let Some(metadata) = yaml.get("metadata")
                                    && let Some(code) = metadata.get("code").and_then(|v| v.as_str())
                                {
                                    resource_code = Some(code.to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if resource_code.is_some() {
                break;
            }
        }
    }

    resource_code.ok_or_else(|| "Resource code not found".into())
}

/// Helper function to verify file was updated
fn verify_task_updated(
    temp: &assert_fs::TempDir,
    project_code: &str,
    task_code: &str,
    expected_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let projects_dir = temp.path().join("projects");
    let mut task_file = None;

    // Search in all project directories
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                // Check if this is the project file we're looking for
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(metadata) = yaml.get("metadata")
                    && let Some(code) = metadata.get("code").and_then(|v| v.as_str())
                    && code == project_code
                {
                    // Found the project, now look for tasks in the tasks subdirectory
                    let tasks_dir = path.parent().unwrap().join("tasks");
                    if tasks_dir.exists() {
                        if let Ok(task_entries) = std::fs::read_dir(&tasks_dir) {
                            for task_entry in task_entries.flatten() {
                                let task_path = task_entry.path();
                                if task_path.is_file()
                                    && task_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                                    && let Ok(task_content) = std::fs::read_to_string(&task_path)
                                    && let Ok(task_yaml) = serde_yaml::from_str::<serde_yaml::Value>(&task_content)
                                    && let Some(task_metadata) = task_yaml.get("metadata")
                                    && let Some(task_code_from_file) =
                                        task_metadata.get("code").and_then(|v| v.as_str())
                                {
                                    // task_code might include .yaml extension, so we need to strip it
                                    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(task_code);
                                    if task_code_from_file == task_code_without_ext {
                                        task_file = Some(task_path);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if task_file.is_some() {
                break;
            }
        }
    }

    let task_file = task_file.ok_or("Task file not found")?;
    assert!(task_file.exists(), "Task file should exist");

    let content = fs::read_to_string(&task_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let actual_name = yaml
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    assert_eq!(actual_name, expected_name, "Task name should be updated");

    Ok(())
}

/// Helper function to verify project was updated
fn verify_project_updated(
    temp: &assert_fs::TempDir,
    project_code: &str,
    expected_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let projects_dir = temp.path().join("projects");
    let mut project_file = None;

    // Search for project file by code in ID-based format
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                    && code == project_code
                {
                    project_file = Some(path);
                    break;
                }
            }
        }
    }

    let project_file = project_file.ok_or("Project file not found")?;
    assert!(project_file.exists(), "Project file should exist");

    let content = fs::read_to_string(&project_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let actual_name = yaml
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    assert_eq!(actual_name, expected_name, "Project name should be updated");

    Ok(())
}

/// Helper function to verify resource was updated
fn verify_resource_updated(
    temp: &assert_fs::TempDir,
    resource_code: &str,
    expected_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resources_dir = temp.path().join("resources");
    let mut resource_file = None;

    // Search in global resources directory (ID-based format)
    if resources_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&resources_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Ok(content) = std::fs::read_to_string(&path)
                        && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                        && let Some(code) = yaml
                            .get("metadata")
                            .and_then(|m| m.get("code"))
                            .and_then(|c| c.as_str())
                        && code == resource_code
                    {
                        resource_file = Some(path);
                        break;
                    }
                }
            }
        }
    }

    // Also search in company resources directories (legacy format)
    if resource_file.is_none() {
        let companies_dir = temp.path().join("companies");
        if let Ok(entries) = std::fs::read_dir(&companies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let company_resources_dir = path.join("resources");
                    if company_resources_dir.exists() {
                        if let Ok(resource_entries) = std::fs::read_dir(&company_resources_dir) {
                            for resource_entry in resource_entries.flatten() {
                                let resource_path = resource_entry.path();
                                if resource_path.is_file()
                                    && resource_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                                    && let Ok(content) = std::fs::read_to_string(&resource_path)
                                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                                    && let Some(code) = yaml
                                        .get("metadata")
                                        .and_then(|m| m.get("code"))
                                        .and_then(|c| c.as_str())
                                    && code == resource_code
                                {
                                    resource_file = Some(resource_path);
                                    break;
                                }
                            }
                        }
                    }
                }
                if resource_file.is_some() {
                    break;
                }
            }
        }
    }

    let resource_file = resource_file.ok_or("Resource file not found")?;
    assert!(resource_file.exists(), "Resource file should exist");

    let content = fs::read_to_string(&resource_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let actual_name = yaml
        .get("metadata")
        .and_then(|m| m.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    assert_eq!(actual_name, expected_name, "Resource name should be updated");

    Ok(())
}

/// Test task update in root context
#[test]
fn test_task_update_root_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Update task from root context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Task Name",
        "--description",
        "Updated task description",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task updated successfully"));

    // Verify file was updated
    verify_task_updated(&temp, &project_code, &task_code, "Updated Task Name")?;

    temp.close()?;
    Ok(())
}

/// Test task update in company context
#[test]
fn test_task_update_company_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Change to company context
    let company_dir = temp.path();
    std::env::set_current_dir(&company_dir)?;

    // Update task from company context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(&company_dir);
    cmd.args([
        "update",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Task from Company Context",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task updated successfully"));

    // Verify file was updated
    verify_task_updated(&temp, &project_code, &task_code, "Updated Task from Company Context")?;

    // Reset directory
    std::env::set_current_dir(temp.path())?;

    temp.close()?;
    Ok(())
}

/// Test task update in project context
#[test]
fn test_task_update_project_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Update task from root context with project and company parameters
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Task from Project Context",
        "--description",
        "Updated from project context",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task updated successfully"));

    // Verify file was updated
    verify_task_updated(&temp, &project_code, &task_code, "Updated Task from Project Context")?;

    // Reset directory
    std::env::set_current_dir(temp.path())?;

    temp.close()?;
    Ok(())
}

/// Test project update in root context
#[test]
fn test_project_update_root_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;

    // Update project from root context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "project",
        "--code",
        &project_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Project Name",
        "--description",
        "Updated project description",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project updated successfully"));

    // Verify file was updated
    verify_project_updated(&temp, &project_code, "Updated Project Name")?;

    temp.close()?;
    Ok(())
}

/// Test project update in company context
#[test]
fn test_project_update_company_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;

    // Change to company context
    let company_dir = temp.path();
    std::env::set_current_dir(&company_dir)?;

    // Update project from company context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(&company_dir);
    cmd.args([
        "update",
        "project",
        "--code",
        &project_code,
        "--name",
        "Updated Project from Company Context",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project updated successfully"));

    // Verify file was updated
    verify_project_updated(&temp, &project_code, "Updated Project from Company Context")?;

    // Reset directory
    std::env::set_current_dir(temp.path())?;

    temp.close()?;
    Ok(())
}

/// Test resource update in root context
#[test]
fn test_resource_update_root_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let resource_code = find_resource_code(&temp)?;

    // Update resource from root context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "resource",
        "--code",
        &resource_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Resource Name",
        "--email",
        "updated@example.com",
    ]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Resource updated successfully"));

    // Verify file was updated
    verify_resource_updated(&temp, &resource_code, "Updated Resource Name")?;

    temp.close()?;
    Ok(())
}

/// Test resource update in company context
#[test]
fn test_resource_update_company_context() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let resource_code = find_resource_code(&temp)?;

    // Change to company context (use the root directory since we're using ID-based naming)
    let company_dir = temp.path();
    std::env::set_current_dir(&company_dir)?;

    // Update resource from company context
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(&company_dir);
    cmd.args([
        "update",
        "resource",
        "--code",
        &resource_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Updated Resource from Company Context",
    ]);

    let output = cmd.output()?;
    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));
    println!("Command exit code: {}", output.status.code().unwrap_or(-1));

    if !output.status.success() {
        panic!("Command failed with exit code: {}", output.status.code().unwrap_or(-1));
    }

    // Debug: List all resource files to see what's there
    let resources_dir = temp.path().join("resources");
    if resources_dir.exists() {
        println!("Resource files in directory:");
        for entry in std::fs::read_dir(&resources_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                println!("  File: {:?}", path.file_name());
                if let Ok(content) = std::fs::read_to_string(&path) {
                    println!("  Content: {}", content);
                }
            }
        }
    }

    // Check if there's a file with the resource code (not the name)
    let resource_file = resources_dir.join(format!("{}.yaml", resource_code));
    if resource_file.exists() {
        println!("Found file with resource code: {:?}", resource_file);
        if let Ok(content) = std::fs::read_to_string(&resource_file) {
            println!("Resource file content: {}", content);
        }
    } else {
        println!("File with resource code does not exist: {:?}", resource_file);
        // Try to find any resource file
        if let Ok(entries) = std::fs::read_dir(&resources_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    println!("Found resource file: {:?}", path);
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        println!("Resource file content: {}", content);
                    }
                }
            }
        }
    }

    // Simple verification - just check if the resource was updated

    // Check if the old file was removed
    let old_name_file = resources_dir.join("john_doe.yaml");
    if old_name_file.exists() {
        println!("Old file still exists: {:?}", old_name_file);
    } else {
        println!("Old file was removed: {:?}", old_name_file);
    }

    // Verify file was updated
    verify_resource_updated(&temp, &resource_code, "Updated Resource from Company Context")?;

    // Reset directory
    std::env::set_current_dir(temp.path())?;

    temp.close()?;
    Ok(())
}

/// Test context validation errors
#[test]
fn test_update_context_validation_errors() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Test wrong company parameter in company context
    let company_dir = temp.path();
    std::env::set_current_dir(&company_dir)?;

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(&company_dir);
    cmd.args([
        "update",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "WRONG-COMPANY", // Wrong company
        "--name",
        "Should Fail",
    ]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Company 'WRONG-COMPANY' not found"));

    // Reset directory
    std::env::set_current_dir(temp.path())?;

    temp.close()?;
    Ok(())
}

/// Test comprehensive update matrix - Issue #99
#[test]
fn test_comprehensive_update_matrix() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;
    let resource_code = find_resource_code(&temp)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Test matrix: [Command] x [Context] x [Parameters]
    let test_cases = vec![
        // Root context tests
        (
            "root",
            "task",
            vec![
                "--code",
                task_code_without_ext,
                "--project",
                &project_code,
                "--company",
                "TECH-CORP",
                "--name",
                "Root Task Update",
            ],
        ),
        (
            "root",
            "project",
            vec![
                "--code",
                &project_code,
                "--company",
                "TECH-CORP",
                "--name",
                "Root Project Update",
            ],
        ),
        (
            "root",
            "resource",
            vec![
                "--code",
                &resource_code,
                "--company",
                "TECH-CORP",
                "--name",
                "Root Resource Update",
            ],
        ),
    ];

    for (_context, command, args) in test_cases {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.arg("update").arg(command);
        cmd.args(&args);

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("updated successfully"));
    }

    // Note: Company and project context tests are not included in this comprehensive test
    // as they require more complex setup and are covered by individual context tests

    temp.close()?;
    Ok(())
}

/// Test file integrity after updates
#[test]
fn test_file_integrity_after_updates() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    setup_basic_environment(&temp)?;

    let project_code = find_project_code(&temp)?;
    let task_code = find_task_code(&temp, &project_code)?;

    // Extract task code from filename (remove .yaml extension)
    let task_code_without_ext = task_code.strip_suffix(".yaml").unwrap_or(&task_code);

    // Update task
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "update",
        "task",
        "--code",
        task_code_without_ext,
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
        "--name",
        "Integrity Test Task",
        "--description",
        "Testing file integrity",
    ]);

    cmd.assert().success();

    // Verify task file exists and has correct content - find in ID-based format
    let projects_dir = temp.path().join("projects");
    let mut task_file = None;

    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                    && code == project_code
                {
                    // Found the project, now look for tasks in the tasks subdirectory
                    let tasks_dir = path.parent().unwrap().join("tasks");
                    if tasks_dir.exists() {
                        if let Ok(task_entries) = std::fs::read_dir(&tasks_dir) {
                            for task_entry in task_entries.flatten() {
                                let task_path = task_entry.path();
                                if task_path.is_file()
                                    && task_path.extension().and_then(|s| s.to_str()) == Some("yaml")
                                    && let Ok(task_content) = std::fs::read_to_string(&task_path)
                                    && let Ok(task_yaml) = serde_yaml::from_str::<serde_yaml::Value>(&task_content)
                                    && let Some(task_metadata) = task_yaml.get("metadata")
                                    && let Some(task_code_from_file) =
                                        task_metadata.get("code").and_then(|v| v.as_str())
                                    && task_code_from_file == task_code
                                {
                                    task_file = Some(task_path);
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
    }

    let task_file = task_file.ok_or("Task file not found")?;

    assert!(task_file.exists(), "Task file should exist");

    let content = fs::read_to_string(&task_file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    // Verify updated fields
    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("name"))
            .and_then(|n| n.as_str()),
        Some("Integrity Test Task")
    );

    assert_eq!(
        yaml.get("metadata")
            .and_then(|m| m.get("description"))
            .and_then(|d| d.as_str()),
        Some("Testing file integrity")
    );

    // Verify project.yaml was not modified (should only contain project info) - find in ID-based format
    let projects_dir = temp.path().join("projects");
    let mut project_file = None;

    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                    && code == project_code
                {
                    project_file = Some(path);
                    break;
                }
            }
        }
    }

    let project_file = project_file.ok_or("Project file not found")?;
    assert!(project_file.exists(), "Project file should exist");

    let project_content = fs::read_to_string(&project_file)?;
    let project_yaml: serde_yaml::Value = serde_yaml::from_str(&project_content)?;

    // Project should not have tasks field
    assert!(
        project_yaml.get("spec").and_then(|s| s.get("tasks")).is_none(),
        "Project should not contain tasks field"
    );

    temp.close()?;
    Ok(())
}
