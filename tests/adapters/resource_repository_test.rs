use assert_fs::TempDir;
use chrono::Local;

use task_task_revolution::domain::resource_management::{
    AnyResource, resource::Resource, repository::{ResourceRepository, ResourceRepositoryWithId}, resource::ResourceScope, state::Available
};
use task_task_revolution::infrastructure::persistence::resource_repository::FileResourceRepository;

/// Test fixtures for ResourceRepository tests
struct ResourceRepositoryTestFixture {
    #[allow(dead_code)]
    temp_dir: TempDir,
    repository: FileResourceRepository,
}

impl ResourceRepositoryTestFixture {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let repository = FileResourceRepository::new(temp_dir.path());
        
        Self {
            temp_dir,
            repository,
        }
    }

    fn create_test_resource(&self, code: &str, name: &str, resource_type: &str) -> AnyResource {
        let resource = Resource::<Available>::new(
            code.to_string(),
            name.to_string(),
            Some("test@example.com".to_string()),
            resource_type.to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            40,
        );
        AnyResource::Available(resource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_find_by_code() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-001", "Test Resource", "Developer");

        // Save the resource
        let _saved_resource = fixture.repository.save(resource).unwrap();

        // Find by code
        let found = fixture.repository.find_by_code("RES-001").unwrap();
        assert!(found.is_some());
        let found_resource = found.unwrap();
        assert_eq!(found_resource.code(), "RES-001");
        assert_eq!(found_resource.name(), "Test Resource");
    }

    #[test]
    fn test_save_and_find_by_id() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-002", "Test Resource 2", "Manager");

        // Save the resource
        let saved_resource = fixture.repository.save(resource).unwrap();

        // Find by ID
        let found = fixture.repository.find_by_id(&saved_resource.id().to_string()).unwrap();
        assert!(found.is_some());
        let found_resource = found.unwrap();
        assert_eq!(found_resource.id(), saved_resource.id());
        assert_eq!(found_resource.code(), "RES-002");
    }

    #[test]
    fn test_find_all() {
        let fixture = ResourceRepositoryTestFixture::new();
        
        // Create multiple resources
        let resource1 = fixture.create_test_resource("RES-001", "Test Resource 1", "Developer");
        let resource2 = fixture.create_test_resource("RES-002", "Test Resource 2", "Manager");

        // Save resources
        fixture.repository.save(resource1).unwrap();
        fixture.repository.save(resource2).unwrap();

        // Find all
        let all_resources = fixture.repository.find_all().unwrap();
        assert_eq!(all_resources.len(), 2);
        
        let codes: Vec<&str> = all_resources.iter().map(|r| r.code()).collect();
        assert!(codes.contains(&"RES-001"));
        assert!(codes.contains(&"RES-002"));
    }

    #[test]
    fn test_find_by_company() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-003", "Company Resource", "Developer");

        // Save the resource
        fixture.repository.save(resource).unwrap();

        // Find by company (assuming default company)
        let _found = fixture.repository.find_by_company("COMP-001").unwrap();
        // Note: This test might need adjustment based on actual company association logic
        // found.len() is always >= 0, so this assertion is redundant
    }

    #[test]
    fn test_find_all_with_context() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-004", "Context Resource", "Developer");

        // Save the resource
        fixture.repository.save(resource).unwrap();

        // Find all with context
        let contexts = fixture.repository.find_all_with_context().unwrap();
        assert!(!contexts.is_empty());
        
        let (found_resource, company_code, _project_codes) = &contexts[0];
        assert_eq!(found_resource.code(), "RES-004");
        assert!(!company_code.is_empty());
        // project_codes might be empty for company-scoped resources
    }

    #[test]
    fn test_save_in_hierarchy() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-005", "Hierarchy Resource", "Developer");

        // Save in hierarchy
        let _saved_resource = fixture.repository.save_in_hierarchy(
            resource,
            "COMP-001",
            Some("PROJ-001"),
        ).unwrap();

        // Verify the resource was saved
        let found = fixture.repository.find_by_code("RES-005").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_save_time_off() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-006", "Time Off Resource", "Developer");

        // Save the resource first
        fixture.repository.save(resource).unwrap();

        // Save time off
        let updated_resource = fixture.repository.save_time_off(
            "Time Off Resource",
            8,
            "2024-01-15",
            Some("Sick leave".to_string()),
        ).unwrap();

        // Verify the resource was updated
        assert_eq!(updated_resource.name(), "Time Off Resource");
    }

    #[test]
    fn test_save_vacation() {
        let fixture = ResourceRepositoryTestFixture::new();
        let resource = fixture.create_test_resource("RES-007", "Vacation Resource", "Developer");

        // Save the resource first
        fixture.repository.save(resource).unwrap();

        // Save vacation
        let updated_resource = fixture.repository.save_vacation(
            "Vacation Resource",
            "2024-07-01",
            "2024-07-15",
            false,
            None,
        ).unwrap();

        // Verify the resource was updated
        assert_eq!(updated_resource.name(), "Vacation Resource");
    }

    #[test]
    fn test_check_layoff_period() {
        let fixture = ResourceRepositoryTestFixture::new();
        let start_date = Local::now().date_naive();
        let end_date = start_date + chrono::Duration::days(7);

        // Check layoff period (this will depend on the actual implementation)
        let _is_layoff = fixture.repository.check_if_layoff_period(
            &start_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap(),
            &end_date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local).unwrap(),
        );

        // This is a boolean check, so we just verify it returns a boolean
        // We just verify that the function returns a boolean value
        // Note: This test just verifies the function doesn't panic
    }

    #[test]
    fn test_get_next_code() {
        let fixture = ResourceRepositoryTestFixture::new();

        // Get next code for different resource types
        let dev_code = fixture.repository.get_next_code("Developer").unwrap();
        let mgr_code = fixture.repository.get_next_code("Manager").unwrap();

        assert!(!dev_code.is_empty());
        assert!(!mgr_code.is_empty());
        assert_ne!(dev_code, mgr_code);
    }

    #[test]
    fn test_find_nonexistent_resource() {
        let fixture = ResourceRepositoryTestFixture::new();

        // Try to find a resource that doesn't exist
        let found = fixture.repository.find_by_code("NONEXISTENT").unwrap();
        assert!(found.is_none());

        let found_by_id = fixture.repository.find_by_id("nonexistent-id").unwrap();
        assert!(found_by_id.is_none());
    }

    #[test]
    fn test_resource_persistence_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create first repository instance and save a resource
        {
            let repository = FileResourceRepository::new(temp_path);
            let resource = Resource::<Available>::new(
                "PERSIST-001".to_string(),
                "Persistent Resource".to_string(),
                Some("persist@example.com".to_string()),
                "Developer".to_string(),
                ResourceScope::Company,
                None,
                None,
                None,
                None,
                40,
            );
            let any_resource = AnyResource::Available(resource);
            repository.save(any_resource).unwrap();
        }

        // Create second repository instance and verify the resource exists
        {
            let repository = FileResourceRepository::new(temp_path);
            let found = repository.find_by_code("PERSIST-001").unwrap();
            assert!(found.is_some());
            let found_resource = found.unwrap();
            assert_eq!(found_resource.name(), "Persistent Resource");
        }
    }

    #[test]
    fn test_resource_with_different_scopes() {
        let fixture = ResourceRepositoryTestFixture::new();
        
        // Test company-scoped resource
        let company_resource = Resource::<Available>::new(
            "COMP-RES-001".to_string(),
            "Company Resource".to_string(),
            Some("company@example.com".to_string()),
            "Developer".to_string(),
            ResourceScope::Company,
            None,
            None,
            None,
            None,
            40,
        );
        let any_company_resource = AnyResource::Available(company_resource);
        fixture.repository.save(any_company_resource).unwrap();

        // Test project-scoped resource
        let project_resource = Resource::<Available>::new(
            "PROJ-RES-001".to_string(),
            "Project Resource".to_string(),
            Some("project@example.com".to_string()),
            "Developer".to_string(),
            ResourceScope::Project,
            None,
            None,
            None,
            None,
            40,
        );
        let any_project_resource = AnyResource::Available(project_resource);
        fixture.repository.save(any_project_resource).unwrap();

        // Verify both resources exist
        let found_company = fixture.repository.find_by_code("COMP-RES-001").unwrap();
        let found_project = fixture.repository.find_by_code("PROJ-RES-001").unwrap();
        
        assert!(found_company.is_some());
        assert!(found_project.is_some());
    }
}
