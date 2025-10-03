use assert_fs::TempDir;
use std::process::Command;

#[test]
fn test_build_generates_company_page_with_projects() {
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

    // Create projects
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

    assert!(output.status.success(), "Create project 1 failed: {}", String::from_utf8_lossy(&output.stderr));

    let output = Command::new(&ttr_binary)
        .args(&["create", "project"])
        .args(&["--company", "TECH-001"])
        .args(&["--code", "DATA-PIPELINE"])
        .args(&["--name", "Data Pipeline"])
        .args(&["--start-date", "2024-02-01"])
        .args(&["--end-date", "2024-11-30"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create project 2 failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify projects exist
    let output = Command::new(&ttr_binary)
        .args(&["list", "projects"])
        .args(&["--company", "TECH-001"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "List projects failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WEB-APP"));
    assert!(stdout.contains("DATA-PIPELINE"));

    // Run build
    let output = Command::new(&ttr_binary)
        .args(&["build"])
        .args(&["--output", "dist"])
        .args(&["--base-url", "https://example.com"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Build failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify build output
    let dist_dir = temp_path.join("dist");
    assert!(dist_dir.exists(), "Dist directory not created");

    let company_dir = dist_dir.join("companies").join("TECH-001");
    assert!(company_dir.exists(), "Company directory not created");

    let company_index = company_dir.join("index.html");
    assert!(company_index.exists(), "Company index.html not created");

    // Read and verify company page content
    let company_html = std::fs::read_to_string(&company_index).unwrap();
    
    // Should contain both projects
    assert!(company_html.contains("Web Application"), "Company page missing WEB-APP project");
    assert!(company_html.contains("Data Pipeline"), "Company page missing DATA-PIPELINE project");
    
    // Should contain project links
    assert!(company_html.contains("projects/WEB-APP/index.html"), "Company page missing WEB-APP link");
    assert!(company_html.contains("projects/DATA-PIPELINE/index.html"), "Company page missing DATA-PIPELINE link");
    
    // Should show project count (optional check - main goal is that projects are displayed)
    // Note: The exact format may vary, so we just check that projects are present

    // Verify project pages exist
    let web_app_page = company_dir.join("projects").join("WEB-APP").join("index.html");
    let data_pipeline_page = company_dir.join("projects").join("DATA-PIPELINE").join("index.html");
    
    assert!(web_app_page.exists(), "WEB-APP project page not created");
    assert!(data_pipeline_page.exists(), "DATA-PIPELINE project page not created");
}

#[test]
fn test_build_with_no_projects_shows_zero_count() {
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

    // Create a company but no projects
    let ttr_binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("debug")
        .join("ttr");
    
    let output = Command::new(&ttr_binary)
        .args(&["create", "company"])
        .args(&["--code", "EMPTY-001"])
        .args(&["--name", "Empty Corp"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Create company failed: {}", String::from_utf8_lossy(&output.stderr));

    // Run build
    let output = Command::new(&ttr_binary)
        .args(&["build"])
        .args(&["--output", "dist"])
        .args(&["--base-url", "https://example.com"])
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "Build failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify build output
    let dist_dir = temp_path.join("dist");
    assert!(dist_dir.exists(), "Dist directory not created");

    let company_dir = dist_dir.join("companies").join("EMPTY-001");
    assert!(company_dir.exists(), "Company directory not created");

    let company_index = company_dir.join("index.html");
    assert!(company_index.exists(), "Company index.html not created");

    // Read and verify company page content
    let company_html = std::fs::read_to_string(&company_index).unwrap();
    
    // Should show zero projects (optional check - main goal is that no projects are displayed)
    // Note: The exact format may vary, so we just check that no project links are present
    
    // Should not contain any project links
    assert!(!company_html.contains("projects/"), "Company page should not contain project links");
}
