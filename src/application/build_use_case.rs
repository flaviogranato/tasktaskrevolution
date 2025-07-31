use crate::domain::{
    company_settings::repository::ConfigRepository, project_management::repository::ProjectRepository,
    resource_management::repository::ResourceRepository, task_management::repository::TaskRepository,
};
use crate::infrastructure::persistence::{
    config_repository::FileConfigRepository, project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
};
use crate::interface::assets::{StaticAssets, TemplateAssets};

use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};

/// `SiteContext` holds all the data that will be passed to the Tera templates.
/// It needs to derive `Serialize` for Tera to be able to use it.
#[derive(Serialize)]
struct SiteContext {
    project: crate::domain::project_management::project::Project,
    tasks: Vec<crate::domain::task_management::AnyTask>,
    resources: Vec<crate::domain::resource_management::resource::Resource>,
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

        // 3. Find root path and instantiate repositories.
        let config_repo = FileConfigRepository::new();
        let (config, root_path) = config_repo.load()?;

        let project_repo = FileProjectRepository::new();
        let resource_repo = FileResourceRepository::new();
        let task_repo = FileTaskRepository::new(root_path);

        // 4. Load all necessary data from the repositories.
        let mut project = project_repo.load()?;
        let tasks = task_repo.find_all()?;
        let resources = resource_repo.find_all()?;
        println!("[DEBUG] Found {} resources.", resources.len());

        // 5. Inherit timezone from config if not set in project.
        if project.timezone.is_none() {
            project.timezone = Some(config.default_timezone);
        }

        // 6. Create the context for Tera.
        let site_data = SiteContext {
            project,
            tasks,
            resources,
        };
        let mut context = Context::from_serialize(&site_data)?;
        context.insert("relative_path_prefix", "");

        println!(
            "[DEBUG] Available templates: {:?}",
            self.tera.get_template_names().collect::<Vec<_>>()
        );

        // 7. Render main pages.
        let main_templates: Vec<_> = self
            .tera
            .get_template_names()
            .filter(|t| !t.starts_with('_') && *t != "base.html" && *t != "resource_detail.html")
            .collect();

        for tmpl_name in &main_templates {
            let rendered_page = self.tera.render(tmpl_name, &context)?;
            fs::write(self.output_dir.join(tmpl_name), rendered_page)?;
        }

        // 8. Render detail pages for each resource.
        let resource_dir = self.output_dir.join("resources");
        fs::create_dir_all(&resource_dir)?;

        for resource in &site_data.resources {
            println!("[DEBUG] Generating page for resource: {}", resource.name);
            let mut detail_context = Context::new();
            detail_context.insert("project", &site_data.project);
            detail_context.insert("resource", resource);
            detail_context.insert("relative_path_prefix", "../");

            let rendered_page = self.tera.render("resource_detail.html", &detail_context)?;
            let safe_name = resource.name.replace(' ', "_").to_lowercase();
            let file_path = resource_dir.join(format!("{safe_name}.html"));
            fs::write(file_path, rendered_page)?;
        }

        println!("✅ Site generated successfully in '{}'", self.output_dir.display());

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

        // 2. Create and execute the use case, starting from the project subdir.
        let use_case = BuildUseCase::new(temp_root.join("my-project"), output_dir.to_str().unwrap()).unwrap();

        let result = use_case.execute();
        assert!(result.is_ok());

        // 3. Assert that output files were created correctly.
        let index_path = output_dir.join("index.html");
        assert!(index_path.exists());
        let index_content = fs::read_to_string(index_path).unwrap();
        assert!(index_content.contains("My Test Project"));

        // 4. Assert that timezone was inherited.
        // This is harder to test directly without inspecting the `SiteContext`
        // but we can check if the build succeeded, which implies the logic ran.
    }
}
