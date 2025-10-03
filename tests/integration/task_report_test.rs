use assert_fs::TempDir;
use std::process::Command;

#[test]
fn test_task_report_generates_correct_records() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize the repository
    let ttr_binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join("ttr");
    
    let output = Command::new(&ttr_binary)
        .args(&["init"])
        .args(&["--name", "Test Company"])
        .args(&["--email", "test@example.com"])
        .args(&["--timezone", "America/Sao_Paulo"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Init failed: {}", String::from_utf8_lossy(&output.stderr));

    // Create a company
    let output = Command::new(&ttr_binary)
        .args(&["create", "company"])
        .args(&["--code", "TECH-001"])
        .args(&["--name", "Tech Corp"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create company failed: {}", String::from_utf8_lossy(&output.stderr));

    // Create a project
    let output = Command::new(&ttr_binary)
        .args(&["create", "project"])
        .args(&["--company", "TECH-001"])
        .args(&["--code", "WEB-APP"])
        .args(&["--name", "Web Application"])
        .args(&["--start-date", "2024-01-01"])
        .args(&["--end-date", "2024-12-31"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create project failed: {}", String::from_utf8_lossy(&output.stderr));

    // Create tasks
    let output = Command::new(&ttr_binary)
        .args(&["create", "task"])
        .args(&["--company", "TECH-001"])
        .args(&["--project", "WEB-APP"])
        .args(&["--code", "TASK-LOGIN"])
        .args(&["--name", "Login Task"])
        .args(&["--description", "Implement user login"])
        .args(&["--start-date", "2024-01-01"])
        .args(&["--due-date", "2024-01-15"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create task 1 failed: {}", String::from_utf8_lossy(&output.stderr));

    let output = Command::new(&ttr_binary)
        .args(&["create", "task"])
        .args(&["--company", "TECH-001"])
        .args(&["--project", "WEB-APP"])
        .args(&["--code", "TASK-API"])
        .args(&["--name", "API Task"])
        .args(&["--description", "Implement API endpoints"])
        .args(&["--start-date", "2024-01-16"])
        .args(&["--due-date", "2024-01-30"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create task 2 failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify tasks exist
    let output = Command::new(&ttr_binary)
        .args(&["list", "tasks"])
        .args(&["--company", "TECH-001"])
        .args(&["--project", "WEB-APP"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "List tasks failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TASK-LOGIN"));
    assert!(stdout.contains("TASK-API"));

    // Generate task report
    let output = Command::new(&ttr_binary)
        .args(&["report", "generate"])
        .args(&["--type", "task"])
        .args(&["--format", "csv"])
        .args(&["--project", "WEB-APP"])
        .args(&["--company", "TECH-001"])
        .args(&["--output", "tasks.csv"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Generate report failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Total records: 2"), "Expected 2 records, got: {}", stdout);

    // Verify CSV file was created and contains data
    let csv_file = temp_path.join("tasks.csv");
    assert!(csv_file.exists(), "CSV file should be created");
    
    let csv_content = std::fs::read_to_string(&csv_file).unwrap();
    assert!(!csv_content.is_empty(), "CSV file should not be empty");
    assert!(csv_content.contains("TASK-LOGIN"), "CSV should contain TASK-LOGIN");
    assert!(csv_content.contains("TASK-API"), "CSV should contain TASK-API");
    
    // Verify CSV has proper headers
    assert!(csv_content.contains("assigned_resources,code,company_code"), "CSV should have proper headers");
}

#[test]
fn test_task_report_with_no_tasks_returns_zero_records() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Initialize the repository
    let ttr_binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join("ttr");
    
    let output = Command::new(&ttr_binary)
        .args(&["init"])
        .args(&["--name", "Test Company"])
        .args(&["--email", "test@example.com"])
        .args(&["--timezone", "America/Sao_Paulo"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Init failed: {}", String::from_utf8_lossy(&output.stderr));

    // Create a company but no projects or tasks
    let output = Command::new(&ttr_binary)
        .args(&["create", "company"])
        .args(&["--code", "EMPTY-001"])
        .args(&["--name", "Empty Corp"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create company failed: {}", String::from_utf8_lossy(&output.stderr));

    // Generate task report (should return 0 records)
    let output = Command::new(&ttr_binary)
        .args(&["report", "generate"])
        .args(&["--type", "task"])
        .args(&["--format", "csv"])
        .args(&["--company", "EMPTY-001"])
        .args(&["--output", "empty_tasks.csv"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Generate report failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Total records: 0"), "Expected 0 records, got: {}", stdout);

    // Verify CSV file was created but is empty (only headers)
    let csv_file = temp_path.join("empty_tasks.csv");
    assert!(csv_file.exists(), "CSV file should be created");
    
    let csv_content = std::fs::read_to_string(&csv_file).unwrap();
    // Should only contain headers, no data rows
    let lines: Vec<&str> = csv_content.lines().collect();
    assert!(lines.len() <= 1, "CSV should only contain headers when no tasks exist, got {} lines", lines.len());
}
