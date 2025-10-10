use task_task_revolution::application::search::{SearchExecutor, SearchResultFormatter, SearchFilter, SearchFilterBuilder};
use task_task_revolution::domain::shared::search_engine::{SearchOptions, SearchQuery, FileType};
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_search_engine_basic_search() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create test files
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project\nstatus: active\ndescription: A test project").unwrap();

    let task_file = temp_dir.path().join("tasks").join("test.yaml");
    fs::create_dir_all(task_file.parent().unwrap()).unwrap();
    fs::write(&task_file, "name: Test Task\nstatus: planned\ndescription: A test task").unwrap();

    let options = SearchOptions::default();
    let results = executor.search("test", options).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.file_path.ends_with("projects/test.yaml")));
    assert!(results.iter().any(|r| r.file_path.ends_with("tasks/test.yaml")));
}

#[test]
fn test_search_engine_case_sensitive() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a Test with capital T").unwrap();

    let case_sensitive_options = SearchOptions {
        case_sensitive: true,
        ..Default::default()
    };

    let results = executor.search("Test", case_sensitive_options).unwrap();
    assert_eq!(results.len(), 1);

    let case_insensitive_options = SearchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    let results = executor.search("test", case_insensitive_options).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_engine_whole_word() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test case for testing").unwrap();

    let whole_word_options = SearchOptions {
        whole_word: true,
        ..Default::default()
    };

    let results = executor.search("test", whole_word_options).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].matches.len() >= 1);
}

#[test]
fn test_search_engine_regex() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Project-001, Project-002, Project-003").unwrap();

    let regex_options = SearchOptions {
        regex: true,
        ..Default::default()
    };

    let results = executor.search(r"Project-\d{3}", regex_options).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].matches.len(), 3);
}

#[test]
fn test_search_engine_by_entity_type() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create project file
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project\nstatus: active").unwrap();

    // Create task file
    let task_file = temp_dir.path().join("tasks").join("test.yaml");
    fs::create_dir_all(task_file.parent().unwrap()).unwrap();
    fs::write(&task_file, "name: Test Task\nstatus: planned").unwrap();

    let options = SearchOptions::default();

    // Search only projects
    let project_results = executor.search_by_entity_type(
        task_task_revolution::application::search::search_executor::EntityType::Project,
        "test",
        options.clone(),
    ).unwrap();
    assert_eq!(project_results.len(), 1);
    assert!(project_results[0].file_path.ends_with("projects/test.yaml"));

    // Search only tasks
    let task_results = executor.search_by_entity_type(
        task_task_revolution::application::search::search_executor::EntityType::Task,
        "test",
        options,
    ).unwrap();
    assert_eq!(task_results.len(), 1);
    assert!(task_results[0].file_path.ends_with("tasks/test.yaml"));
}

#[test]
fn test_search_engine_metadata_search() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.yaml");
    fs::write(&test_file, "---\nname: Test Project\nstatus: active\n---\nContent here").unwrap();

    let metadata_options = SearchOptions {
        include_metadata: true,
        include_content: false,
        ..Default::default()
    };

    let results = executor.search("active", metadata_options).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_engine_content_search() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is content with test keyword").unwrap();

    let content_options = SearchOptions {
        include_metadata: false,
        include_content: true,
        ..Default::default()
    };

    let results = executor.search("test", content_options).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_search_engine_max_results() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create multiple test files
    for i in 1..=5 {
        let test_file = temp_dir.path().join(format!("test{}.txt", i));
        fs::write(&test_file, format!("This is test file {}", i)).unwrap();
    }

    let limited_options = SearchOptions {
        max_results: Some(3),
        ..Default::default()
    };

    let results = executor.search("test", limited_options).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_search_result_formatter_table() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let formatted = SearchResultFormatter::format_table(&results);

    assert!(formatted.contains("Search Results"));
    assert!(formatted.contains("test.txt"));
}

#[test]
fn test_search_result_formatter_json() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let formatted = SearchResultFormatter::format_json(&results).unwrap();

    assert!(formatted.contains("test.txt"));
    assert!(formatted.contains("file_path"));
}

#[test]
fn test_search_result_formatter_csv() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let formatted = SearchResultFormatter::format_csv(&results);

    assert!(formatted.contains("file_path,file_type,score"));
    assert!(formatted.contains("test.txt"));
}

#[test]
fn test_search_result_formatter_highlighted() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let formatted = SearchResultFormatter::format_highlighted(&results, "test");

    assert!(formatted.contains(">>>test<<<"));
}

#[test]
fn test_search_filter_by_file_type() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create project file
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project").unwrap();

    // Create task file
    let task_file = temp_dir.path().join("tasks").join("test.yaml");
    fs::create_dir_all(task_file.parent().unwrap()).unwrap();
    fs::write(&task_file, "name: Test Task").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let filter = SearchFilter::new().file_types(vec![FileType::Project]);
    let filtered = filter.apply(&results);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].file_type, FileType::Project);
}

#[test]
fn test_search_filter_by_score() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This is a test file with test keyword").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let filter = SearchFilter::new().min_score(1.0);
    let filtered = filter.apply(&results);

    assert!(filtered.iter().all(|r| r.score >= 1.0));
}

#[test]
fn test_search_filter_by_path_pattern() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create files in different directories
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project").unwrap();

    let task_file = temp_dir.path().join("tasks").join("test.yaml");
    fs::create_dir_all(task_file.parent().unwrap()).unwrap();
    fs::write(&task_file, "name: Test Task").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let filter = SearchFilter::new().include_path("projects");
    let filtered = filter.apply(&results);

    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].file_path.to_string_lossy().contains("projects"));
}

#[test]
fn test_search_filter_builder() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create project file
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project").unwrap();

    // Create config file
    let config_file = temp_dir.path().join("config.yaml");
    fs::write(&config_file, "name: Test Config").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    let filter = SearchFilterBuilder::new()
        .projects_only()
        .exclude_config()
        .build();
    let filtered = filter.apply(&results);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].file_type, FileType::Project);
}

#[test]
fn test_search_engine_context_lines() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Line 1\nLine 2\nLine 3 with test\nLine 4\nLine 5").unwrap();

    let context_options = SearchOptions {
        context_lines: 1,
        ..Default::default()
    };

    let results = executor.search("test", context_options).unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].matches.is_empty());
    
    let match_result = &results[0].matches[0];
    assert!(match_result.context_before.is_some());
    assert!(match_result.context_after.is_some());
}

#[test]
fn test_search_engine_empty_results() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "This file has no matches").unwrap();

    let results = executor.search("nonexistent", SearchOptions::default());
    assert!(results.is_err());
}

#[test]
fn test_search_engine_file_type_detection() {
    let temp_dir = tempdir().unwrap();
    let executor = SearchExecutor::new(temp_dir.path().to_path_buf());

    // Create different file types
    let project_file = temp_dir.path().join("projects").join("test.yaml");
    fs::create_dir_all(project_file.parent().unwrap()).unwrap();
    fs::write(&project_file, "name: Test Project").unwrap();

    let task_file = temp_dir.path().join("tasks").join("test.yaml");
    fs::create_dir_all(task_file.parent().unwrap()).unwrap();
    fs::write(&task_file, "name: Test Task").unwrap();

    let config_file = temp_dir.path().join("config.yaml");
    fs::write(&config_file, "name: Test Config").unwrap();

    let results = executor.search("test", SearchOptions::default()).unwrap();
    
    let project_results: Vec<_> = results.iter().filter(|r| r.file_type == FileType::Project).collect();
    let task_results: Vec<_> = results.iter().filter(|r| r.file_type == FileType::Task).collect();
    let config_results: Vec<_> = results.iter().filter(|r| r.file_type == FileType::Config).collect();

    assert_eq!(project_results.len(), 1);
    assert_eq!(task_results.len(), 1);
    assert_eq!(config_results.len(), 1);
}
