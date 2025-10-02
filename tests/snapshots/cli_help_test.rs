use std::process::Command;

#[test]
fn test_main_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Filter out compilation warnings and focus on the help output
    let help_output = if stdout.contains("Usage: ttr") {
        stdout
    } else {
        stderr
    };

    insta::assert_snapshot!(help_output);
}

#[test]
fn test_create_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "create", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let help_output = if stdout.contains("Create new entities") {
        stdout
    } else {
        stderr
    };

    insta::assert_snapshot!(help_output);
}

#[test]
fn test_create_project_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "create", "project", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let help_output = if stdout.contains("Create a new project") {
        stdout
    } else {
        stderr
    };

    insta::assert_snapshot!(help_output);
}

#[test]
fn test_list_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "list", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let help_output = if stdout.contains("List entities") {
        stdout
    } else {
        stderr
    };

    insta::assert_snapshot!(help_output);
}

#[test]
fn test_validate_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "validate", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let help_output = if stdout.contains("Validate system") {
        stdout
    } else {
        stderr
    };

    insta::assert_snapshot!(help_output);
}
