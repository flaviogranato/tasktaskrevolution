use crate::domain::{
    company_settings::repository::ConfigRepository,
    project_management::AnyProject,
    // task_management::repository::TaskRepository,
};
use crate::infrastructure::persistence::{
    config_repository::FileConfigRepository,
    project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository, // task_repository::FileTaskRepository,
};
use crate::interface::assets::{StaticAssets, TemplateAssets};
use crate::application::build_context::BuildContext;

use glob::glob;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

/// `BuildUseCase` is responsible for orchestrating the static site generation.
pub struct BuildUseCase {
    base_path: PathBuf,
    tera: Tera,
    output_dir: PathBuf,
    #[allow(dead_code)]
    context: BuildContext,
}

impl BuildUseCase {
    pub fn new(base_path: PathBuf, output_dir: &str) -> Result<Self, Box<dyn Error>> {
        // Detect build context
        let context = BuildContext::detect(&base_path)
            .map_err(|e| format!("Failed to detect build context: {}", e))?;
        
        println!("[INFO] Detected build context: {}", context.display_name());
        
        let mut tera = Tera::default();
        for filename in TemplateAssets::iter() {
            let file = TemplateAssets::get(filename.as_ref()).unwrap();
            let content = std::str::from_utf8(file.data.as_ref())?;
            tera.add_raw_template(filename.as_ref(), content)?;
        }

        Ok(Self {
            base_path,
            tera,
            output_dir: PathBuf::from(output_dir),
            context,
        })
    }

    /// Executes the build process.
    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        // 1. Clean and create the output directory.
        if self.output_dir.exists() {
            fs::remove_dir_all(&self.output_dir)?;
        }
        fs::create_dir_all(&self.output_dir)?;

        // 2. Copy all embedded static assets to the output directory.
        for filename in StaticAssets::iter() {
            let asset = StaticAssets::get(filename.as_ref()).unwrap();
            let dest_path = self.output_dir.join(filename.as_ref());
            fs::write(dest_path, asset.data)?;
        }

        // 3. Load global configuration.
        let config_repo = FileConfigRepository::with_base_path(self.base_path.clone());
        let (config, _) = config_repo.load()?;

        // 4. Find all projects and load their data.
        let mut all_projects_data = Vec::new();
        let project_manifest_pattern = self.base_path.join("companies/*/projects/*/project.yaml");
        for entry in glob(project_manifest_pattern.to_str().unwrap())? {
            let manifest_path = entry?;
            let project_path = manifest_path.parent().unwrap().to_path_buf();
            println!("[INFO] Loading project from: {}", project_path.display());

            let project_repo = FileProjectRepository::with_base_path(self.base_path.clone());
            let resource_repo = FileResourceRepository::new(self.base_path.clone());

            let project = project_repo.load_from_path(&project_path)?;
            
            // Extract company and project codes from the path
            let path_components: Vec<_> = project_path.components().collect();
            let company_code = path_components[path_components.len() - 3].as_os_str().to_str().unwrap();
            let project_code = project.code();
            
            // Load resources using the new hierarchical method
            let resources = resource_repo.find_all_by_project(company_code, project_code)?;
            let tasks: Vec<_> = project.tasks().values().cloned().collect();

            let project = if project.timezone().is_none() {
                // Clone the project and update its timezone
                let mut project_clone = project.clone();
                let AnyProject::Project(ref mut p) = project_clone;
                p.settings.timezone = Some(config.default_timezone.clone());
                project_clone
            } else {
                project
            };

            all_projects_data.push((project, tasks, resources));
        }

        // 5. Render the global index page with all projects.
        println!("[INFO] Generating global index page...");
        let mut context = Context::new();

        let project_values: Vec<_> = all_projects_data
            .iter()
            .map(|(project, tasks, resources)| {
                let mut project_map = tera::Map::new();
                project_map.insert("code".to_string(), tera::Value::String(project.code().to_string()));
                project_map.insert("name".to_string(), tera::Value::String(project.name().to_string()));
                project_map.insert(
                    "description".to_string(),
                    tera::Value::String(
                        project
                            .description()
                            .map_or("No description available.".to_string(), |d| d.to_string()),
                    ),
                );
                project_map.insert("status".to_string(), tera::Value::String(project.status().to_string()));
                project_map.insert(
                    "start_date".to_string(),
                    project
                        .start_date()
                        .map_or(tera::Value::Null, |d| tera::Value::String(d.to_string())),
                );

                let mut map = tera::Map::new();
                map.insert("project".to_string(), tera::Value::Object(project_map));
                map.insert("tasks".to_string(), tera::to_value(tasks).unwrap());
                map.insert("resources".to_string(), tera::to_value(resources).unwrap());
                tera::Value::Object(map)
            })
            .collect();

        context.insert("projects", &project_values);
        context.insert("relative_path_prefix", "");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

        // Create a dummy project for the base template header, which expects a `project` object.
        let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
            .code("TTR_DASHBOARD".to_string())
            .name("Projects Dashboard".to_string())
            .company_code("TTR".to_string())
            .created_by("system".to_string())
            .end_date(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .build()
            .unwrap()
            .into();
        context.insert("project", &dummy_project);

        let index_html = match self.tera.render("index.html", &context) {
            Ok(html) => html,
            Err(e) => {
                eprintln!("Template render error: {:?}", e);
                return Err(format!("Template error: {}", e).into());
            }
        };
        fs::write(self.output_dir.join("index.html"), index_html)?;
        println!("✅ Global index page generated successfully.");

        // 6. Render a detail page for each project.
        let projects_base_dir = self.output_dir.join("projects");
        fs::create_dir_all(&projects_base_dir)?;

        for (project, tasks, resources) in &all_projects_data {
            let project_code = project.code();
            let project_name = project.name();
            println!("[INFO] Generating page for project: {project_name} ({project_code})");

            let project_output_dir = projects_base_dir.join(project_code);
            fs::create_dir_all(&project_output_dir)?;

            let mut context = Context::new();
            // Create a simplified project object for the template
            let mut project_map = tera::Map::new();
            project_map.insert("code".to_string(), tera::Value::String(project.code().to_string()));
            project_map.insert("name".to_string(), tera::Value::String(project.name().to_string()));
            project_map.insert(
                "description".to_string(),
                tera::Value::String(
                    project
                        .description()
                        .map_or("No description available.".to_string(), |d| d.to_string()),
                ),
            );
            project_map.insert("status".to_string(), tera::Value::String(project.status().to_string()));
            project_map.insert(
                "start_date".to_string(),
                project
                    .start_date()
                    .map_or(tera::Value::Null, |d| tera::Value::String(d.to_string())),
            );

            context.insert("project", &tera::Value::Object(project_map));
            context.insert("tasks", tasks);
            context.insert("resources", resources);
            context.insert("relative_path_prefix", "../../");
            context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

            // Render project detail page (e.g., project.html)
            let project_html = match self.tera.render("project.html", &context) {
                Ok(html) => html,
                Err(e) => {
                    eprintln!("Template render error for project.html: {:?}", e);
                    return Err(format!("Template error: {}", e).into());
                }
            };
            let project_page_path = project_output_dir.join("index.html");
            fs::write(project_page_path, project_html)?;

            println!("✅ Project '{project_name}' page generated successfully.");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn setup_test_environment() -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();

        // Create config.yaml
        let config_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  createdAt: "2024-01-01T00:00:00Z"
spec:
  managerName: "Test Manager"
  managerEmail: "manager@test.com"
  defaultTimezone: "America/Sao_Paulo"
"#;
        let mut config_file = File::create(root.join("config.yaml")).unwrap();
        writeln!(config_file, "{config_content}").unwrap();

        // Create project subdirectory and project.yaml
        let project_dir = root.join("my-project");
        fs::create_dir(&project_dir).unwrap();
        let project_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-1"
  name: "My Test Project"
  description: "A description for the test project."
spec:
  status: "InProgress"
  startDate: "2024-08-01"
  endDate: "2024-09-30"
"#;
        let mut project_file = File::create(project_dir.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content}").unwrap();

        // Create tasks subdirectory
        let tasks_dir = project_dir.join("tasks");
        fs::create_dir(&tasks_dir).unwrap();

        // Create a test task file
        let task_content = r#"
api_version: v1
kind: Task
metadata:
  id: "01901dea-3e4b-7698-b323-95232d306587"
  code: "TSK-01"
  name: "Design the API"
  description: "A test task for the build process."
spec:
  projectCode: "proj-1"
  assignee: "dev-01"
  status: "Planned"
  priority: "Medium"
  estimatedStartDate: "2024-08-05"
  estimatedEndDate: "2024-08-10"
  dependencies: []
  tags: []
  effort:
    estimatedHours: 8.0
  acceptanceCriteria: []
  comments: []
"#;
        let mut task_file = File::create(tasks_dir.join("task1.yaml")).unwrap();
        writeln!(task_file, "{task_content}").unwrap();

        // Create resources subdirectory
        fs::create_dir(project_dir.join("resources")).unwrap();

        // Create a test resource file
        let resource_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  code: "dev-01"
  name: "Developer One"
  resourceType: "Human"
spec:
  email: "dev1@example.com"
  timeOffBalance: 0
"#;
        let mut resource_file = File::create(project_dir.join("resources").join("dev1.yaml")).unwrap();
        writeln!(resource_file, "{resource_content}").unwrap();

        // Create a second project, this one WITHOUT dates, to replicate the bug.
        let project_dir_2 = root.join("project-no-dates");
        fs::create_dir(&project_dir_2).unwrap();
        let project_content_2 = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-2"
  name: "Project Without Dates"
spec:
  status: "Planned"
"#;
        let mut project_file_2 = File::create(project_dir_2.join("project.yaml")).unwrap();
        writeln!(project_file_2, "{project_content_2}").unwrap();
        fs::create_dir(project_dir_2.join("tasks")).unwrap();
        fs::create_dir(project_dir_2.join("resources")).unwrap();

        // Persist the temporary directory for inspection after the test.
        let _ = temp_dir.keep();
        root
    }

    #[test]
    fn test_build_use_case_finds_files_and_builds() {
        // 1. Setup temporary directory with config and project files.
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // 2. Create and execute the use case.
        let use_case = BuildUseCase::new(temp_root, output_dir.to_str().unwrap()).unwrap();
        let result = use_case.execute();
        if let Err(e) = &result {
            // Provide more context on failure.
            eprintln!("BuildUseCase::execute failed: {e}");
        }
        assert!(result.is_ok());

        // 3. Assert that the global index.html was created correctly.
        let global_index_path = output_dir.join("index.html");
        assert!(global_index_path.exists(), "Global index.html was not created");
        let global_index_content = fs::read_to_string(global_index_path).unwrap();
        // Check the title of the global index page, which is composed by the base template.
        // This ensures the dummy project context is correctly passed and rendered.
        let title_content = global_index_content
            .split_once("<title>")
            .and_then(|(_, after_title_tag)| after_title_tag.split_once("</title>"))
            .map(|(content, _)| content)
            .unwrap_or("")
            .trim();
        assert!(
            title_content.contains("Projects Dashboard"),
            "The rendered title content ('{title_content}') did not contain 'Projects Dashboard'."
        );
        assert!(
            global_index_content.contains("My Test Project"),
            "Global index.html should list the test project"
        );

        // 4. Assert that the project-specific detail page was created correctly.
        let project_page_path = output_dir.join("projects").join("proj-1").join("index.html");
        assert!(project_page_path.exists(), "Project detail page was not created");
        let project_page_content = fs::read_to_string(project_page_path).unwrap();
        assert!(
            project_page_content.contains("My Test Project"),
            "Project page should contain project name"
        );
        assert!(
            project_page_content.contains("A description for the test project."),
            "Project page should contain project description"
        );
        assert!(
            project_page_content.contains("Developer One"),
            "Project page should list the test resource"
        );
        assert!(
            project_page_content.contains("Design the API"),
            "Project page should list the test task"
        );
    }

    #[test]
    fn test_build_use_case_with_existing_output_directory() {
        // Test that the use case can handle existing output directory
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // Create the output directory beforehand
        fs::create_dir_all(&output_dir).unwrap();

        let use_case = BuildUseCase::new(temp_root, output_dir.to_str().unwrap()).unwrap();
        let result = use_case.execute();
        assert!(result.is_ok());

        // Verify files were still created
        let global_index_path = output_dir.join("index.html");
        assert!(global_index_path.exists());
    }

    #[test]
    fn test_build_use_case_with_different_project_states() {
        // Test projects with different states to cover Completed, Cancelled, and InProgress
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // Create additional projects with different states
        let project_dir_completed = temp_root.join("project-completed");
        fs::create_dir(&project_dir_completed).unwrap();
        let project_content_completed = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-completed"
  name: "Completed Project"
spec:
  status: "Completed"
  startDate: "2024-01-01"
  endDate: "2024-02-01"
"#;
        let mut project_file = File::create(project_dir_completed.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content_completed}").unwrap();
        fs::create_dir(project_dir_completed.join("tasks")).unwrap();
        fs::create_dir(project_dir_completed.join("resources")).unwrap();

        let project_dir_cancelled = temp_root.join("project-cancelled");
        fs::create_dir(&project_dir_cancelled).unwrap();
        let project_content_cancelled = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-cancelled"
  name: "Cancelled Project"
spec:
  status: "Cancelled"
  startDate: "2024-01-01"
  endDate: "2024-02-01"
"#;
        let mut project_file = File::create(project_dir_cancelled.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content_cancelled}").unwrap();
        fs::create_dir(project_dir_cancelled.join("tasks")).unwrap();
        fs::create_dir(project_dir_cancelled.join("resources")).unwrap();

        let project_dir_in_progress = temp_root.join("project-in-progress");
        fs::create_dir(&project_dir_in_progress).unwrap();
        let project_content_in_progress = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-in-progress"
  name: "In Progress Project"
spec:
  status: "InProgress"
  startDate: "2024-01-01"
  endDate: "2024-12-31"
"#;
        let mut project_file = File::create(project_dir_in_progress.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content_in_progress}").unwrap();
        fs::create_dir(project_dir_in_progress.join("tasks")).unwrap();
        fs::create_dir(project_dir_in_progress.join("resources")).unwrap();

        let use_case = BuildUseCase::new(temp_root, output_dir.to_str().unwrap()).unwrap();
        let result = use_case.execute();
        assert!(result.is_ok());

        // Verify all project pages were created
        let completed_page = output_dir.join("projects").join("proj-completed").join("index.html");
        let cancelled_page = output_dir.join("projects").join("proj-cancelled").join("index.html");
        let in_progress_page = output_dir.join("projects").join("proj-in-progress").join("index.html");

        assert!(completed_page.exists());
        assert!(cancelled_page.exists());
        assert!(in_progress_page.exists());
    }

    #[test]
    fn test_build_use_case_with_projects_having_timezone() {
        // Test projects that already have timezone defined
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // Create a project with timezone already defined
        let project_dir_with_tz = temp_root.join("project-with-timezone");
        fs::create_dir(&project_dir_with_tz).unwrap();
        let project_content_with_tz = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-with-tz"
  name: "Project With Timezone"
spec:
  status: "Planned"
  startDate: "2024-01-01"
  endDate: "2024-12-31"
  timezone: "Europe/London"
"#;
        let mut project_file = File::create(project_dir_with_tz.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content_with_tz}").unwrap();
        fs::create_dir(project_dir_with_tz.join("tasks")).unwrap();
        fs::create_dir(project_dir_with_tz.join("resources")).unwrap();

        let use_case = BuildUseCase::new(temp_root, output_dir.to_str().unwrap()).unwrap();
        let result = use_case.execute();
        assert!(result.is_ok());

        // Verify the project page was created
        let project_page = output_dir.join("projects").join("proj-with-tz").join("index.html");
        assert!(project_page.exists());
    }
}
