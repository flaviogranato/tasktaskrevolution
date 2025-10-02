use assert_fs::TempDir;
use chrono::NaiveDate;

use task_task_revolution::domain::project_management::{
    AnyProject, project::Project, repository::{ProjectRepository, ProjectRepositoryWithId}, project::ProjectStatus
};
use task_task_revolution::infrastructure::persistence::project_repository::FileProjectRepository;

/// Test fixtures for ProjectRepository tests
struct ProjectRepositoryTestFixture {
    temp_dir: TempDir,
    repository: FileProjectRepository,
}

impl ProjectRepositoryTestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileProjectRepository::with_base_path(temp_dir.path().to_path_buf());
        
        Self {
            temp_dir,
            repository,
        }
    }

    fn create_test_project(&self, code: &str, name: &str, company_code: &str) -> AnyProject {
        let project = Project::new(
            code.to_string(),
            name.to_string(),
            company_code.to_string(),
            "test@example.com".to_string(),
        ).unwrap();
        AnyProject::from(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find_by_code() {
        let fixture = ProjectRepositoryTestFixture::new();
        let project = fixture.create_test_project("PROJ-001", "Test Project", "COMP-001");

        // Save the project
        fixture.repository.save(project.clone()).unwrap();

        // Find by code
        let found = fixture.repository.find_by_code("PROJ-001").unwrap();
        assert!(found.is_some());
        let found_project = found.unwrap();
        assert_eq!(found_project.code(), "PROJ-001");
        assert_eq!(found_project.name(), "Test Project");
    }

    #[test]
    fn test_save_and_find_by_id() {
        let fixture = ProjectRepositoryTestFixture::new();
        let project = fixture.create_test_project("PROJ-002", "Test Project 2", "COMP-001");

        // Save the project
        fixture.repository.save(project.clone()).unwrap();

        // Find by ID
        let found = fixture.repository.find_by_id(project.id()).unwrap();
        assert!(found.is_some());
        let found_project = found.unwrap();
        assert_eq!(found_project.id(), project.id());
        assert_eq!(found_project.code(), "PROJ-002");
    }

    #[test]
    fn test_find_all() {
        let fixture = ProjectRepositoryTestFixture::new();
        
        // Create multiple projects
        let project1 = fixture.create_test_project("PROJ-001", "Test Project 1", "COMP-001");
        let project2 = fixture.create_test_project("PROJ-002", "Test Project 2", "COMP-001");

        // Save projects
        fixture.repository.save(project1).unwrap();
        fixture.repository.save(project2).unwrap();

        // Find all
        let all_projects = fixture.repository.find_all().unwrap();
        assert_eq!(all_projects.len(), 2);
        
        let codes: Vec<&str> = all_projects.iter().map(|p| p.code()).collect();
        assert!(codes.contains(&"PROJ-001"));
        assert!(codes.contains(&"PROJ-002"));
    }

    #[test]
    fn test_load_single_project() {
        let fixture = ProjectRepositoryTestFixture::new();
        let project = fixture.create_test_project("PROJ-003", "Single Project", "COMP-001");

        // Save the project
        fixture.repository.save(project).unwrap();

        // Load single project
        let loaded = fixture.repository.load().unwrap();
        assert_eq!(loaded.code(), "PROJ-003");
        assert_eq!(loaded.name(), "Single Project");
    }

    #[test]
    fn test_get_next_code() {
        let fixture = ProjectRepositoryTestFixture::new();

        // Get next code
        let next_code = fixture.repository.get_next_code().unwrap();
        assert!(!next_code.is_empty());
        assert!(next_code.starts_with("proj-"));
    }

    #[test]
    fn test_find_nonexistent_project() {
        let fixture = ProjectRepositoryTestFixture::new();

        // Try to find a project that doesn't exist
        let found = fixture.repository.find_by_code("NONEXISTENT").unwrap();
        assert!(found.is_none());

        let found_by_id = fixture.repository.find_by_id("nonexistent-id").unwrap();
        assert!(found_by_id.is_none());
    }

    #[test]
    fn test_project_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create first repository instance and save a project
        {
            let repository = FileProjectRepository::with_base_path(temp_path.to_path_buf());
            let project = Project::new(
                "PERSIST-001".to_string(),
                "Persistent Project".to_string(),
                "COMP-001".to_string(),
                "persist@example.com".to_string(),
            ).unwrap();
            let any_project = AnyProject::from(project);
            repository.save(any_project).unwrap();
        }

        // Create second repository instance and verify the project exists
        {
            let repository = FileProjectRepository::with_base_path(temp_path.to_path_buf());
            let found = repository.find_by_code("PERSIST-001").unwrap();
            assert!(found.is_some());
            let found_project = found.unwrap();
            assert_eq!(found_project.name(), "Persistent Project");
        }
    }

    #[test]
    fn test_project_with_dates() {
        let fixture = ProjectRepositoryTestFixture::new();
        let mut project = Project::new(
            "PROJ-DATES".to_string(),
            "Project with Dates".to_string(),
            "COMP-001".to_string(),
            "test@example.com".to_string(),
        ).unwrap();

        // Set dates
        project.start_date = Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        project.end_date = Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());

        let any_project = AnyProject::from(project);

        // Save the project
        fixture.repository.save(any_project.clone()).unwrap();

        // Find and verify dates
        let found = fixture.repository.find_by_code("PROJ-DATES").unwrap();
        assert!(found.is_some());
        let found_project = found.unwrap();
        assert_eq!(found_project.start_date(), Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(found_project.end_date(), Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()));
    }

    #[test]
    fn test_project_with_status() {
        let fixture = ProjectRepositoryTestFixture::new();
        let mut project = Project::new(
            "PROJ-STATUS".to_string(),
            "Project with Status".to_string(),
            "COMP-001".to_string(),
            "test@example.com".to_string(),
        ).unwrap();

        // Set status
        project.status = ProjectStatus::InProgress;

        let any_project = AnyProject::from(project);

        // Save the project
        fixture.repository.save(any_project.clone()).unwrap();

        // Find and verify status
        let found = fixture.repository.find_by_code("PROJ-STATUS").unwrap();
        assert!(found.is_some());
        let found_project = found.unwrap();
        assert_eq!(found_project.status(), ProjectStatus::InProgress);
    }
}
