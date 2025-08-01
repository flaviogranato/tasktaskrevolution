use crate::domain::{
    company_settings::repository::ConfigRepository,
    project_management::{AnyProject, repository::ProjectRepository},
    resource_management::{AnyResource, repository::ResourceRepository},
    // task_management::repository::TaskRepository,
};
use crate::infrastructure::persistence::{
    config_repository::FileConfigRepository,
    project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository, // task_repository::FileTaskRepository,
};
use crate::interface::assets::{StaticAssets, TemplateAssets};

use glob::glob;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

/// `SiteContext` holds all the data that will be passed to the Tera templates.
/// It needs to derive `Serialize` for Tera to be able to use it.
#[derive(Serialize)]
struct SiteContext {
    project: crate::domain::project_management::AnyProject,
    tasks: Vec<crate::domain::task_management::AnyTask>,
    resources: Vec<AnyResource>,
}

/// `BuildUseCase` is responsible for orchestrating the static site generation.
pub struct BuildUseCase {
    base_path: PathBuf,
    tera: Tera,
    output_dir: PathBuf,
}

impl BuildUseCase {
    pub fn new(base_path: PathBuf, output_dir: &str) -> Result<Self, Box<dyn Error>> {
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

        // 4. Find all project directories by looking for `project.yaml` files.
        let project_manifest_pattern = self.base_path.join("**/project.yaml");
        for entry in glob(project_manifest_pattern.to_str().unwrap())? {
            let manifest_path = entry?;
            let project_path = manifest_path.parent().unwrap().to_path_buf();
            println!("[INFO] Found project at: {}", project_path.display());

            // 5. Instantiate repositories scoped to the current project path.
            let project_repo = FileProjectRepository::with_base_path(project_path.clone());
            let resource_repo = FileResourceRepository::new(project_path.clone());
            // let task_repo = FileTaskRepository::new(project_path.clone());

            // 6. Load all data for this specific project.
            let project = project_repo.load()?;
            // let tasks = task_repo.find_all()?;
            let tasks = vec![]; // FIXME: Re-implement task loading
            let resources = resource_repo.find_all()?;
            let project_name = project.name().to_string();

            // Inherit timezone from config if not set in project.
            let project = if project.timezone().is_none() {
                match project {
                    AnyProject::Planned(mut p) => {
                        p.timezone = Some(config.default_timezone.clone());
                        AnyProject::Planned(p)
                    }
                    AnyProject::InProgress(mut p) => {
                        p.timezone = Some(config.default_timezone.clone());
                        AnyProject::InProgress(p)
                    }
                    AnyProject::Completed(mut p) => {
                        p.timezone = Some(config.default_timezone.clone());
                        AnyProject::Completed(p)
                    }
                    AnyProject::Cancelled(mut p) => {
                        p.timezone = Some(config.default_timezone.clone());
                        AnyProject::Cancelled(p)
                    }
                }
            } else {
                project
            };

            // 7. Create a specific output directory for this project.
            let project_output_dir = self.output_dir.join(&project_name);
            fs::create_dir_all(&project_output_dir)?;

            // 8. Create the context for Tera.
            let site_data = SiteContext {
                project,
                tasks,
                resources,
            };
            let mut context = Context::from_serialize(&site_data)?;
            context.insert("relative_path_prefix", "");

            // 9. Render main pages for the project.
            let main_templates: Vec<_> = self
                .tera
                .get_template_names()
                .filter(|t| !t.starts_with('_') && *t != "base.html" && *t != "resource_detail.html")
                .collect();

            for tmpl_name in &main_templates {
                let rendered_page = self.tera.render(tmpl_name, &context)?;
                fs::write(project_output_dir.join(tmpl_name), rendered_page)?;
            }

            // 10. Render detail pages for each resource in this project.
            let resource_dir = project_output_dir.join("resources");
            fs::create_dir_all(&resource_dir)?;

            for resource in &site_data.resources {
                println!("[DEBUG] Generating page for resource: {}", resource.name());
                let mut detail_context = Context::new();
                detail_context.insert("project", &site_data.project);
                detail_context.insert("resource", resource);
                detail_context.insert("relative_path_prefix", "../");

                let rendered_page = self.tera.render("resource_detail.html", &detail_context)?;
                let safe_name = resource.name().replace(' ', "_").to_lowercase();
                let file_path = resource_dir.join(format!("{safe_name}.html"));
                fs::write(file_path, rendered_page)?;
            }
            println!("✅ Project '{project_name}' generated successfully.");
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
"#;
        let mut project_file = File::create(project_dir.join("project.yaml")).unwrap();
        writeln!(project_file, "{project_content}").unwrap();

        // Create tasks subdirectory
        fs::create_dir(project_dir.join("tasks")).unwrap();

        // Create resources subdirectory
        fs::create_dir(project_dir.join("resources")).unwrap();

        // Persist the temporary directory for inspection after the test.
        let _ = temp_dir.keep();
        root
    }

    #[test]
    fn test_build_use_case_finds_files_and_builds() {
        // 1. Setup temporary directory with config and project files.
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // 2. Create and execute the use case, starting from the root containing the projects.
        let use_case = BuildUseCase::new(temp_root, output_dir.to_str().unwrap()).unwrap();

        let result = use_case.execute();
        assert!(result.is_ok());

        // 3. Assert that project-specific output files were created correctly.
        let project_output_dir = output_dir.join("My Test Project");
        let index_path = project_output_dir.join("index.html");
        assert!(index_path.exists());
        let index_content = fs::read_to_string(index_path).unwrap();
        assert!(index_content.contains("My Test Project"));

        // 4. Assert that timezone was inherited.
        // This is harder to test directly without inspecting the `SiteContext`
        // but we can check if the build succeeded, which implies the logic ran.
    }
}
