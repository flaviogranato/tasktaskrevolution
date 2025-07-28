use crate::domain::{
    project_management::repository::ProjectRepository, resource_management::repository::ResourceRepository,
    task_management::repository::TaskRepository,
};
use crate::interface::assets::{StaticAssets, TemplateAssets};

use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

/// `SiteContext` holds all the data that will be passed to the Tera templates.
/// It needs to derive `Serialize` for Tera to be able to use it.
#[derive(Serialize)]
struct SiteContext {
    project: crate::domain::project_management::project::Project,
    tasks: Vec<crate::domain::task_management::task::Task>,
    resources: Vec<crate::domain::resource_management::resource::Resource>,
}

/// `BuildUseCase` is responsible for orchestrating the static site generation.
pub struct BuildUseCase<P, R, T>
where
    P: ProjectRepository,
    R: ResourceRepository,
    T: TaskRepository,
{
    project_repo: P,
    resource_repo: R,
    task_repo: T,
    tera: Tera,
    output_dir: PathBuf,
}

impl<P, R, T> BuildUseCase<P, R, T>
where
    P: ProjectRepository,
    R: ResourceRepository,
    T: TaskRepository,
{
    pub fn new(project_repo: P, resource_repo: R, task_repo: T, output_dir: &str) -> Result<Self, Box<dyn Error>> {
        let mut tera = Tera::default();
        for filename in TemplateAssets::iter() {
            let file = TemplateAssets::get(filename.as_ref()).unwrap();
            let content = std::str::from_utf8(file.data.as_ref())?;
            tera.add_raw_template(filename.as_ref(), content)?;
        }

        Ok(Self {
            project_repo,
            resource_repo,
            task_repo,
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

        // 3. Load all necessary data from the repositories.
        let project = self.project_repo.load(Path::new("."))?;
        let tasks = self.task_repo.find_all()?;
        let resources = self.resource_repo.find_all()?;

        // 4. Create the context for Tera.
        let site_data = SiteContext {
            project,
            tasks,
            resources,
        };
        let context = Context::from_serialize(&site_data)?;

        // 5. Render each template and write it to the output directory.
        let templates_to_render: Vec<_> = self
            .tera
            .get_template_names()
            .filter(|t| !t.starts_with('_') && *t != "base.html")
            .collect();

        if templates_to_render.is_empty() {
            return Err(
                "No templates were found embedded in the binary. Ensure 'templates/' directory is not empty.".into(),
            );
        }

        for tmpl_name in &templates_to_render {
            let rendered_page = self.tera.render(tmpl_name, &context)?;
            fs::write(self.output_dir.join(tmpl_name), rendered_page)?;
        }

        println!("✅ Site generated successfully in '{}'", self.output_dir.display());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        project_management::project::{Project, ProjectStatus},
        resource_management::resource::Resource,
        shared::errors::DomainError,
        task_management::task::{Task, TaskStatus},
    };
    use chrono::{DateTime, Local, NaiveDate};
    use tempfile::tempdir;

    // --- Mocks for repositories ---
    struct MockProjectRepository {
        project: Project,
    }
    impl ProjectRepository for MockProjectRepository {
        fn save(&self, _p: Project) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn load(&self, _p: &Path) -> Result<Project, DomainError> {
            Ok(self.project.clone())
        }
    }

    struct MockResourceRepository {
        resources: Vec<Resource>,
    }
    impl ResourceRepository for MockResourceRepository {
        fn save(&self, _r: Resource) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<Resource>, DomainError> {
            Ok(self.resources.clone())
        }
        fn save_time_off(
            &self,
            _r: String,
            _h: u32,
            _d: String,
            _desc: Option<String>,
        ) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn save_vacation(
            &self,
            _r: String,
            _s: String,
            _e: String,
            _i: bool,
            _c: Option<u32>,
        ) -> Result<Resource, DomainError> {
            unimplemented!()
        }
        fn check_if_layoff_period(&self, _s: &DateTime<Local>, _e: &DateTime<Local>) -> bool {
            unimplemented!()
        }
    }

    struct MockTaskRepository {
        tasks: Vec<Task>,
    }
    impl TaskRepository for MockTaskRepository {
        fn save(&self, _t: Task) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn load(&self, _p: &Path) -> Result<Task, DomainError> {
            unimplemented!()
        }
        fn find_by_code(&self, _c: &str) -> Result<Option<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_id(&self, _i: &str) -> Result<Option<Task>, DomainError> {
            unimplemented!()
        }
        fn find_all(&self) -> Result<Vec<Task>, DomainError> {
            Ok(self.tasks.clone())
        }
        fn delete(&self, _i: &str) -> Result<(), DomainError> {
            unimplemented!()
        }
        fn update_status(&self, _c: &str, _s: TaskStatus) -> Result<Task, DomainError> {
            unimplemented!()
        }
        fn find_by_assignee(&self, _a: &str) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_status(&self, _s: &TaskStatus) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
        fn find_by_date_range(&self, _s: NaiveDate, _e: NaiveDate) -> Result<Vec<Task>, DomainError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_build_use_case_with_embedded_assets() {
        // 1. Setup temporary directory for output.
        let temp_root = tempdir().unwrap();
        let output_dir = temp_root.path().join("public");

        // 2. Setup mock data and repositories.
        let project = Project::new(
            None,
            "My Embedded Site".to_string(),
            None,
            None,
            None,
            ProjectStatus::InProgress,
            None,
        );
        let tasks = vec![Task {
            id: "1".to_string(),
            code: "T1".to_string(),
            name: "First Task".to_string(),
            description: None,
            status: TaskStatus::Planned,
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            actual_end_date: None,
            assigned_resources: vec![],
        }];

        let mock_project_repo = MockProjectRepository { project };
        let mock_task_repo = MockTaskRepository { tasks };
        let mock_resource_repo = MockResourceRepository { resources: vec![] };

        // 3. Create and execute the use case.
        let use_case = BuildUseCase::new(
            mock_project_repo,
            mock_resource_repo,
            mock_task_repo,
            output_dir.to_str().unwrap(),
        )
        .unwrap();

        let result = use_case.execute();
        assert!(result.is_ok());

        // 4. Assert that output files were created correctly from embedded assets.
        let index_path = output_dir.join("index.html");
        assert!(index_path.exists());
        let index_content = fs::read_to_string(index_path).unwrap();
        assert!(index_content.contains("My Embedded Site"));

        let css_path = output_dir.join("style.css");
        assert!(css_path.exists());
        let css_content = fs::read_to_string(css_path).unwrap();
        assert!(css_content.contains("body {"));
    }
}
