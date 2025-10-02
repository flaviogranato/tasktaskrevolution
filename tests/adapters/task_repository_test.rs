use assert_fs::TempDir;
use chrono::NaiveDate;

use task_task_revolution::domain::task_management::{
    AnyTask, task::Task, repository::TaskRepository, category::Category, priority::Priority, state::Planned
};
use task_task_revolution::infrastructure::persistence::task_repository::FileTaskRepository;

/// Test fixtures for TaskRepository tests
struct TaskRepositoryTestFixture {
    temp_dir: TempDir,
    repository: FileTaskRepository,
}

impl TaskRepositoryTestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileTaskRepository::new(temp_dir.path());
        
        Self {
            temp_dir,
            repository,
        }
    }

    fn create_test_task(&self, code: &str, name: &str, project_code: &str) -> AnyTask {
        let task = Task {
            id: uuid7::uuid7(),
            project_code: project_code.to_string(),
            code: code.to_string(),
            name: name.to_string(),
            description: Some("Test task description".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["RES-001".to_string()],
            priority: Priority::Medium,
            category: Category::Development,
        };
        AnyTask::Planned(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find_by_code() {
        let fixture = TaskRepositoryTestFixture::new();
        let task = fixture.create_test_task("TASK-001", "Test Task", "PROJ-001");

        // Save the task
        let saved_task = fixture.repository.save(task).unwrap();

        // Find by code
        let found = fixture.repository.find_by_code("TASK-001").unwrap();
        assert!(found.is_some());
        let found_task = found.unwrap();
        assert_eq!(found_task.code(), "TASK-001");
        assert_eq!(found_task.name(), "Test Task");
    }

    #[test]
    fn test_find_all() {
        let fixture = TaskRepositoryTestFixture::new();
        
        // Create multiple tasks
        let task1 = fixture.create_test_task("TASK-001", "Test Task 1", "PROJ-001");
        let task2 = fixture.create_test_task("TASK-002", "Test Task 2", "PROJ-001");

        // Save tasks
        fixture.repository.save(task1).unwrap();
        fixture.repository.save(task2).unwrap();

        // Find all
        let all_tasks = fixture.repository.find_all().unwrap();
        assert_eq!(all_tasks.len(), 2);
        
        let codes: Vec<&str> = all_tasks.iter().map(|t| t.code()).collect();
        assert!(codes.contains(&"TASK-001"));
        assert!(codes.contains(&"TASK-002"));
    }

    #[test]
    fn test_find_by_project() {
        let fixture = TaskRepositoryTestFixture::new();
        let task = fixture.create_test_task("TASK-003", "Project Task", "PROJ-001");

        // Save the task
        fixture.repository.save(task).unwrap();

        // Find by project
        let found = fixture.repository.find_by_project("PROJ-001").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code(), "TASK-003");
    }

    #[test]
    fn test_find_all_by_project() {
        let fixture = TaskRepositoryTestFixture::new();
        let task = fixture.create_test_task("TASK-004", "Company Project Task", "PROJ-001");

        // Save the task in hierarchy
        fixture.repository.save_in_hierarchy(task, "COMP-001", "PROJ-001").unwrap();

        // Find all by project within company
        let found = fixture.repository.find_all_by_project("COMP-001", "PROJ-001").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].code(), "TASK-004");
    }

    #[test]
    fn test_save_in_hierarchy() {
        let fixture = TaskRepositoryTestFixture::new();
        let task = fixture.create_test_task("TASK-005", "Hierarchy Task", "PROJ-001");

        // Save in hierarchy
        let saved_task = fixture.repository.save_in_hierarchy(
            task,
            "COMP-001",
            "PROJ-001",
        ).unwrap();

        // Verify the task was saved
        let found = fixture.repository.find_by_code("TASK-005").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_get_next_code() {
        let fixture = TaskRepositoryTestFixture::new();

        // Get next code for a project
        let next_code = fixture.repository.get_next_code("PROJ-001").unwrap();
        assert!(!next_code.is_empty());
        assert!(next_code.starts_with("PROJ-001-"));
    }

    #[test]
    fn test_find_nonexistent_task() {
        let fixture = TaskRepositoryTestFixture::new();

        // Try to find a task that doesn't exist
        let found = fixture.repository.find_by_code("NONEXISTENT").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_task_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create first repository instance and save a task
        {
            let repository = FileTaskRepository::new(temp_path);
            let task = Task {
                id: uuid7::uuid7(),
                project_code: "PERSIST-PROJ".to_string(),
                code: "PERSIST-001".to_string(),
                name: "Persistent Task".to_string(),
                description: Some("Persistent task description".to_string()),
                state: Planned,
                start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                actual_end_date: None,
                dependencies: vec![],
                assigned_resources: vec![],
                priority: Priority::Medium,
                category: Category::Development,
            };
            let any_task = AnyTask::Planned(task);
            repository.save(any_task).unwrap();
        }

        // Create second repository instance and verify the task exists
        {
            let repository = FileTaskRepository::new(temp_path);
            let found = repository.find_by_code("PERSIST-001").unwrap();
            assert!(found.is_some());
            let found_task = found.unwrap();
            assert_eq!(found_task.name(), "Persistent Task");
        }
    }

    #[test]
    fn test_task_with_dependencies() {
        let fixture = TaskRepositoryTestFixture::new();
        let mut task = Task {
            id: uuid7::uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-DEPS".to_string(),
            name: "Task with Dependencies".to_string(),
            description: Some("Task with dependencies".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: vec!["TASK-001".to_string(), "TASK-002".to_string()],
            assigned_resources: vec!["RES-001".to_string()],
            priority: Priority::High,
            category: Category::Testing,
        };
        let any_task = AnyTask::Planned(task);

        // Save the task
        fixture.repository.save(any_task).unwrap();

        // Find and verify dependencies
        let found = fixture.repository.find_by_code("TASK-DEPS").unwrap();
        assert!(found.is_some());
        let found_task = found.unwrap();
        assert_eq!(found_task.dependencies().len(), 2);
        assert!(found_task.dependencies().contains(&"TASK-001".to_string()));
        assert!(found_task.dependencies().contains(&"TASK-002".to_string()));
    }

    #[test]
    fn test_task_with_dates() {
        let fixture = TaskRepositoryTestFixture::new();
        let mut task = Task {
            id: uuid7::uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-DATES".to_string(),
            name: "Task with Dates".to_string(),
            description: Some("Task with specific dates".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 2, 28).unwrap(),
            actual_end_date: Some(NaiveDate::from_ymd_opt(2024, 2, 25).unwrap()),
            dependencies: vec![],
            assigned_resources: vec![],
            priority: Priority::Low,
            category: Category::Documentation,
        };
        let any_task = AnyTask::Planned(task);

        // Save the task
        fixture.repository.save(any_task).unwrap();

        // Find and verify dates
        let found = fixture.repository.find_by_code("TASK-DATES").unwrap();
        assert!(found.is_some());
        let found_task = found.unwrap();
        assert_eq!(*found_task.start_date(), NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(*found_task.due_date(), NaiveDate::from_ymd_opt(2024, 2, 28).unwrap());
        // Note: actual_end_date is not available in the current API
        // This test would need to be updated based on the actual API
    }

    #[test]
    fn test_task_with_assigned_resources() {
        let fixture = TaskRepositoryTestFixture::new();
        let mut task = Task {
            id: uuid7::uuid7(),
            project_code: "PROJ-001".to_string(),
            code: "TASK-RESOURCES".to_string(),
            name: "Task with Resources".to_string(),
            description: Some("Task with assigned resources".to_string()),
            state: Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            actual_end_date: None,
            dependencies: vec![],
            assigned_resources: vec!["RES-001".to_string(), "RES-002".to_string()],
            priority: Priority::Critical,
            category: Category::Development,
        };
        let any_task = AnyTask::Planned(task);

        // Save the task
        fixture.repository.save(any_task).unwrap();

        // Find and verify assigned resources
        let found = fixture.repository.find_by_code("TASK-RESOURCES").unwrap();
        assert!(found.is_some());
        let found_task = found.unwrap();
        assert_eq!(found_task.assigned_resources().len(), 2);
        assert!(found_task.assigned_resources().contains(&"RES-001".to_string()));
        assert!(found_task.assigned_resources().contains(&"RES-002".to_string()));
    }
}
