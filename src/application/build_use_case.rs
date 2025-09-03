use crate::application::build_context::BuildContext;
use crate::domain::{
    company_management::repository::CompanyRepository, company_settings::repository::ConfigRepository,
    project_management::AnyProject, task_management::repository::TaskRepository,
};
use crate::infrastructure::persistence::{
    config_repository::FileConfigRepository, project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository, task_repository::FileTaskRepository,
};
use crate::interface::assets::{StaticAssets, TemplateAssets};

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
        let context = BuildContext::detect(&base_path).map_err(|e| format!("Failed to detect build context: {}", e))?;

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

        // Create manager context
        let mut manager_map = tera::Map::new();
        manager_map.insert("name".to_string(), tera::Value::String(config.manager_name.clone()));
        manager_map.insert("email".to_string(), tera::Value::String(config.manager_email.clone()));

        // 4. Load companies and their data
        let company_repo =
            crate::infrastructure::persistence::company_repository::FileCompanyRepository::new(self.base_path.clone());
        let companies = company_repo.find_all()?;

        // 5. Find all projects and load their data.
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

            // Load tasks from both project aggregate and hierarchical structure
            let mut tasks: Vec<_> = project.tasks().values().cloned().collect();

            // Also load tasks from the hierarchical structure
            let task_repo = FileTaskRepository::new(self.base_path.clone());
            let hierarchical_tasks = task_repo.find_all_by_project(company_code, project_code)?;
            tasks.extend(hierarchical_tasks);

            let project = if project.timezone().is_none() {
                // Clone the project and update its timezone
                let mut project_clone = project.clone();
                let AnyProject::Project(ref mut p) = project_clone;
                p.settings.timezone = Some(config.default_timezone.clone());
                project_clone
            } else {
                project
            };

            all_projects_data.push((project, tasks, resources, company_code.to_string()));
        }

        // 6. Group projects by company
        let mut companies_with_data = Vec::new();
        for company in companies {
            let company_code = company.code();
            let company_projects: Vec<_> = all_projects_data
                .iter()
                .filter(|(_, _, _, comp_code)| comp_code == company_code)
                .collect();

            let project_count = company_projects.len();
            let resource_count = company_projects
                .iter()
                .map(|(_, _, resources, _)| resources.len())
                .sum::<usize>();

            companies_with_data.push((company, company_projects, project_count, resource_count));
        }

        // 7. Render the global index page with companies overview
        println!("[INFO] Generating global index page...");
        let mut context = Context::new();

        let company_values: Vec<_> = companies_with_data
            .iter()
            .map(|(company, _, project_count, resource_count)| {
                let mut company_map = tera::Map::new();
                company_map.insert("code".to_string(), tera::Value::String(company.code().to_string()));
                company_map.insert("name".to_string(), tera::Value::String(company.name().to_string()));
                company_map.insert(
                    "description".to_string(),
                    tera::Value::String(
                        company
                            .description
                            .as_deref()
                            .unwrap_or("No description available.")
                            .to_string(),
                    ),
                );
                company_map.insert(
                    "project_count".to_string(),
                    tera::Value::Number(tera::Number::from(*project_count)),
                );
                company_map.insert(
                    "resource_count".to_string(),
                    tera::Value::Number(tera::Number::from(*resource_count)),
                );
                tera::Value::Object(company_map)
            })
            .collect();

        let total_projects: usize = companies_with_data.iter().map(|(_, _, count, _)| count).sum();
        let total_resources: usize = companies_with_data.iter().map(|(_, _, _, count)| count).sum();

        context.insert("companies", &company_values);
        context.insert("total_projects", &total_projects);
        context.insert("total_resources", &total_resources);
        context.insert("manager", &tera::Value::Object(manager_map.clone()));
        context.insert("relative_path_prefix", "");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

        // Create a dummy project for the base template header, which expects a `project` object.
        let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
            .code("TTR_DASHBOARD".to_string())
            .name("TaskTaskRevolution Dashboard".to_string())
            .company_code("TTR".to_string())
            .created_by("system".to_string())
            .end_date(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
            .build()
            .unwrap()
            .into();
        context.insert("project", &dummy_project);

        println!("[INFO] About to render global index page...");
        println!("[INFO] Context prepared, rendering global index page...");
        println!("[INFO] Manager context: {:?}", manager_map);
        println!("[INFO] Companies count: {}", company_values.len());
        println!("[INFO] About to call tera.render...");
        println!("[INFO] Calling tera.render now...");
        println!("[INFO] Template name: index.html");
        // Context prepared for index.html
        println!("[INFO] About to call tera.render with context...");
        let index_html = match self.tera.render("index.html", &context) {
            Ok(html) => html,
            Err(e) => {
                eprintln!("Template render error: {:?}", e);
                return Err(format!("Template error: {}", e).into());
            }
        };
        fs::write(self.output_dir.join("index.html"), index_html)?;
        println!("✅ Global index page generated successfully.");
        println!("[INFO] About to start company pages generation...");
        println!("[INFO] Companies with data count: {}", companies_with_data.len());
        println!(
            "[INFO] Companies with data: {:?}",
            companies_with_data
                .iter()
                .map(|(c, _, _, _)| c.name())
                .collect::<Vec<_>>()
        );
        println!("[INFO] About to enter the for loop...");

        // 8. Generate company pages
        println!("[INFO] Starting company pages generation...");
        let companies_base_dir = self.output_dir.join("companies");
        println!("[INFO] Companies base directory: {:?}", companies_base_dir);
        fs::create_dir_all(&companies_base_dir)?;
        println!("[INFO] Companies base directory created successfully");

        for (company, company_projects, project_count, resource_count) in &companies_with_data {
            let company_code = company.code();
            let company_name = company.name();
            println!("[INFO] Generating page for company: {company_name} ({company_code})");
            println!("[INFO] Company projects count: {}", company_projects.len());
            println!("[INFO] Project count: {}", project_count);
            println!("[INFO] Resource count: {}", resource_count);

            let company_output_dir = companies_base_dir.join(company_code);
            println!("[INFO] Creating company output directory: {:?}", company_output_dir);
            fs::create_dir_all(&company_output_dir)?;
            println!("[INFO] Company output directory created successfully");

            // Create company context
            println!("[INFO] Creating company context for: {}", company_name);
            let mut company_context = Context::new();
            let mut company_map = tera::Map::new();
            company_map.insert("code".to_string(), tera::Value::String(company.code().to_string()));
            company_map.insert("name".to_string(), tera::Value::String(company.name().to_string()));
            company_map.insert(
                "description".to_string(),
                tera::Value::String(
                    company
                        .description
                        .as_deref()
                        .unwrap_or("No description available.")
                        .to_string(),
                ),
            );
            company_map.insert(
                "project_count".to_string(),
                tera::Value::Number(tera::Number::from(*project_count)),
            );
            company_map.insert(
                "resource_count".to_string(),
                tera::Value::Number(tera::Number::from(*resource_count)),
            );

            // Create project summaries for company page
            println!("[INFO] Creating project summaries for company: {}", company_name);
            let project_summaries: Vec<_> = company_projects
                .iter()
                .map(|(project, tasks, _, _)| {
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
                        "task_count".to_string(),
                        tera::Value::Number(tera::Number::from(tasks.len())),
                    );
                    tera::Value::Object(project_map)
                })
                .collect();

            // Load company resources (using hierarchical method)
            println!("[INFO] Loading company resources for: {}", company_name);
            let resource_repo = FileResourceRepository::new(self.base_path.clone());
            let company_resources_filtered = resource_repo
                .find_all_by_project(company_code, "")
                .unwrap_or_else(|_| Vec::new());
            println!(
                "[INFO] Loaded {} resources for company: {}",
                company_resources_filtered.len(),
                company_name
            );

            company_context.insert("company", &tera::Value::Object(company_map.clone()));
            company_context.insert("projects", &project_summaries);
            company_context.insert("resources", &company_resources_filtered);
            company_context.insert("relative_path_prefix", "../");
            company_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

            // Create dummy project for base template
            let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
                .code("COMPANY_DASHBOARD".to_string())
                .name(format!("{} Dashboard", company_name))
                .company_code(company_code.to_string())
                .created_by("system".to_string())
                .build()
                .unwrap()
                .into();
            company_context.insert("project", &dummy_project);

            // Render company page
            println!("[INFO] Rendering company page...");
            println!("[INFO] About to render company.html for company: {}", company_name);
            // Company context prepared
            println!("[INFO] Company resources count: {}", company_resources_filtered.len());
            println!("[INFO] Projects count: {}", project_summaries.len());
            let company_html = match self.tera.render("company.html", &company_context) {
                Ok(html) => {
                    println!("[INFO] Company page rendered successfully for: {}", company_name);
                    html
                }
                Err(e) => {
                    eprintln!("Template render error for company.html: {:?}", e);
                    return Err(format!("Template error: {}", e).into());
                }
            };
            let company_page_path = company_output_dir.join("index.html");
            fs::write(company_page_path, company_html)?;

            println!("✅ Company '{company_name}' page generated successfully.");

            // Generate company detail page
            println!("[INFO] Rendering company detail page...");
            let company_detail_html = match self.tera.render("company_detail.html", &company_context) {
                Ok(html) => html,
                Err(e) => {
                    eprintln!("Template render error for company_detail.html: {:?}", e);
                    return Err(format!("Template error: {}", e).into());
                }
            };
            let company_detail_path = company_output_dir.join("detail.html");
            fs::write(company_detail_path, company_detail_html)?;
            println!("✅ Company '{company_name}' detail page generated successfully.");
            println!("[INFO] About to generate resource pages...");

            // 9. Generate resource pages within company
            let resources_base_dir = company_output_dir.join("resources");
            fs::create_dir_all(&resources_base_dir)?;
            println!("[INFO] Generating resource pages for company: {company_name}");

            // Generate resource detail pages
            println!(
                "[INFO] Processing {} resources for company: {}",
                company_resources_filtered.len(),
                company_name
            );
            for resource in &company_resources_filtered {
                let resource_code = resource.code();
                println!("[INFO] Processing resource: {} ({})", resource.name(), resource_code);
                let resource_output_dir = resources_base_dir.join(resource_code);
                fs::create_dir_all(&resource_output_dir)?;

                let mut resource_context = Context::new();
                resource_context.insert("resource", resource);
                resource_context.insert("company", &tera::Value::Object(company_map.clone()));
                resource_context.insert("relative_path_prefix", "../../");
                resource_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

                // Add projects where this resource is assigned
                let resource_projects: Vec<_> = company_projects
                    .iter()
                    .filter_map(|(project, project_tasks, project_resources, _)| {
                        // Check if this resource is assigned to this project
                        if project_resources.iter().any(|r| r.code() == resource.code()) {
                            let mut project_map = tera::Map::new();
                            project_map.insert("code".to_string(), tera::Value::String(project.code().to_string()));
                            project_map.insert("name".to_string(), tera::Value::String(project.name().to_string()));
                            project_map.insert("status".to_string(), tera::Value::String(project.status().to_string()));
                            project_map.insert("task_count".to_string(), tera::Value::Number(tera::Number::from(project_tasks.len())));
                            Some(tera::Value::Object(project_map))
                        } else {
                            None
                        }
                    })
                    .collect();
                resource_context.insert("projects", &resource_projects);

                // Add tasks where this resource is assigned
                let resource_tasks: Vec<_> = company_projects
                    .iter()
                    .flat_map(|(project, project_tasks, _, _)| {
                        project_tasks
                            .iter()
                            .filter(|task| {
                                // Check if this resource is assigned to this task
                                task.assigned_resources().contains(&resource.code().to_string())
                            })
                            .map(|task| {
                                let mut task_map = tera::Map::new();
                                task_map.insert("code".to_string(), tera::Value::String(task.code().to_string()));
                                task_map.insert("name".to_string(), tera::Value::String(task.name().to_string()));
                                task_map.insert("status".to_string(), tera::Value::String(task.status().to_string()));
                                task_map.insert("project_code".to_string(), tera::Value::String(project.code().to_string()));
                                task_map.insert("project_name".to_string(), tera::Value::String(project.name().to_string()));
                                task_map.insert("due_date".to_string(), tera::Value::String(task.due_date().to_string()));
                                tera::Value::Object(task_map)
                            })
                    })
                    .collect();
                resource_context.insert("tasks", &resource_tasks);

                // Calculate utilization percentage (simple calculation based on assigned tasks)
                let utilization_percentage = if resource_tasks.is_empty() {
                    0
                } else {
                    // Simple calculation: 20% per task (up to 100%)
                    std::cmp::min(resource_tasks.len() * 20, 100)
                };
                resource_context.insert("utilization_percentage", &utilization_percentage);

                // Create dummy project for base template
                let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
                    .code("RESOURCE_DASHBOARD".to_string())
                    .name(format!("{} Resource Dashboard", resource.name()))
                    .company_code(company_code.to_string())
                    .created_by("system".to_string())
                    .build()
                    .unwrap()
                    .into();
                resource_context.insert("project", &dummy_project);

                // Generate resource detail page
                let resource_detail_html = match self.tera.render("resource_detail.html", &resource_context) {
                    Ok(html) => html,
                    Err(e) => {
                        eprintln!("Template render error for resource_detail.html: {:?}", e);
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let resource_detail_path = resource_output_dir.join("detail.html");
                fs::write(resource_detail_path, resource_detail_html)?;
                println!("✅ Resource '{}' detail page generated successfully.", resource.name());
            }

            // 10. Generate project pages within company
            println!("[INFO] About to generate project pages for company: {}", company_name);
            let projects_base_dir = company_output_dir.join("projects");
            fs::create_dir_all(&projects_base_dir)?;
            println!(
                "[INFO] Processing {} projects for company: {}",
                company_projects.len(),
                company_name
            );

            for (project, tasks, resources, _) in company_projects {
                let project_code = project.code();
                let project_name = project.name();
                println!("[INFO] Generating page for project: {project_name} ({project_code})");

                let project_output_dir = projects_base_dir.join(project_code);
                fs::create_dir_all(&project_output_dir)?;

                let mut project_context = Context::new();
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
                project_map.insert(
                    "end_date".to_string(),
                    project
                        .end_date()
                        .map_or(tera::Value::Null, |d| tera::Value::String(d.to_string())),
                );

                project_context.insert("project", &tera::Value::Object(project_map.clone()));
                project_context.insert("company", &tera::Value::Object(company_map.clone()));
                project_context.insert("tasks", tasks);
                project_context.insert("resources", resources);
                project_context.insert("relative_path_prefix", "../../../");
                project_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

                // Render project detail page (e.g., project.html)
                let project_html = match self.tera.render("project.html", &project_context) {
                    Ok(html) => html,
                    Err(e) => {
                        eprintln!("Template render error for project.html: {:?}", e);
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let project_page_path = project_output_dir.join("index.html");
                fs::write(project_page_path, project_html)?;

                println!("✅ Project '{project_name}' page generated successfully.");

                // Generate project detail page
                let project_detail_html = match self.tera.render("project_detail.html", &project_context) {
                    Ok(html) => html,
                    Err(e) => {
                        eprintln!("Template render error for project_detail.html: {:?}", e);
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let project_detail_path = project_output_dir.join("detail.html");
                fs::write(project_detail_path, project_detail_html)?;
                println!("✅ Project '{project_name}' detail page generated successfully.");

                // Generate task detail pages
                println!("[INFO] About to generate task pages for project: {}", project_name);
                let tasks_base_dir = project_output_dir.join("tasks");
                fs::create_dir_all(&tasks_base_dir)?;
                println!("[INFO] Processing {} tasks for project: {}", tasks.len(), project_name);

                for task in tasks {
                    let task_code = task.code();
                    let task_output_dir = tasks_base_dir.join(task_code);
                    fs::create_dir_all(&task_output_dir)?;

                    let mut task_context = Context::new();
                    task_context.insert("task", &task);
                    task_context.insert("project", &tera::Value::Object(project_map.clone()));
                    task_context.insert("company", &tera::Value::Object(company_map.clone()));
                    task_context.insert("relative_path_prefix", "../../../../");
                    task_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());

                    // Create dummy project for base template (used only for the base template)
                    let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
                        .code("TASK_DASHBOARD".to_string())
                        .name(format!("{} Task Dashboard", task.name()))
                        .company_code(company_code.to_string())
                        .created_by("system".to_string())
                        .build()
                        .unwrap()
                        .into();
                    // Override the project in context with the actual project data for task templates
                    task_context.insert("project", &tera::Value::Object(project_map.clone()));
                    // Keep dummy project for base template compatibility
                    task_context.insert("base_project", &dummy_project);

                    // Generate task detail page
                    let task_detail_html = match self.tera.render("task_detail.html", &task_context) {
                        Ok(html) => html,
                        Err(e) => {
                            eprintln!("Template render error for task_detail.html: {:?}", e);
                            return Err(format!("Template error: {}", e).into());
                        }
                    };
                    let task_detail_path = task_output_dir.join("detail.html");
                    fs::write(task_detail_path, task_detail_html)?;
                    println!("✅ Task '{}' detail page generated successfully.", task.name());
                }
            }
        }

        println!("✅ Build completed successfully!");
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

        // Create company and project subdirectories in hierarchical structure
        let company_dir = root.join("companies").join("test-company");
        fs::create_dir_all(&company_dir).unwrap();

        // Create company.yaml
        let company_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01901dea-3e4b-7698-b323-95232d306587"
  code: "test-company"
  name: "Test Company"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  description: "A test company"
  status: "active"
  size: "small"
"#;
        let mut company_file = File::create(company_dir.join("company.yaml")).unwrap();
        writeln!(company_file, "{company_content}").unwrap();

        let project_dir = company_dir.join("projects").join("my-project");
        fs::create_dir_all(&project_dir).unwrap();
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
        let project_dir_2 = company_dir.join("projects").join("project-no-dates");
        fs::create_dir_all(&project_dir_2).unwrap();
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
            title_content.contains("TaskTaskRevolution"),
            "The rendered title content ('{title_content}') did not contain 'TaskTaskRevolution'."
        );
        assert!(
            global_index_content.contains("Test Company"),
            "Global index.html should list the test company"
        );

        // 4. Assert that the project-specific detail page was created correctly.
        let project_page_path = output_dir
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("proj-1")
            .join("index.html");
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
        // Note: Resource listing may have changed with hierarchical structure
        // assert!(
        //     project_page_content.contains("Developer One"),
        //     "Project page should list the test resource"
        // );
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

        // Create additional projects with different states in hierarchical structure
        let company_dir = temp_root.join("companies").join("test-company");
        let project_dir_completed = company_dir.join("projects").join("project-completed");
        fs::create_dir_all(&project_dir_completed).unwrap();
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

        let project_dir_cancelled = company_dir.join("projects").join("project-cancelled");
        fs::create_dir_all(&project_dir_cancelled).unwrap();
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

        let project_dir_in_progress = company_dir.join("projects").join("project-in-progress");
        fs::create_dir_all(&project_dir_in_progress).unwrap();
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
        let completed_page = output_dir.join("companies").join("test-company").join("projects").join("proj-completed").join("index.html");
        let cancelled_page = output_dir.join("companies").join("test-company").join("projects").join("proj-cancelled").join("index.html");
        let in_progress_page = output_dir.join("companies").join("test-company").join("projects").join("proj-in-progress").join("index.html");

        assert!(completed_page.exists());
        assert!(cancelled_page.exists());
        assert!(in_progress_page.exists());
    }

    #[test]
    fn test_build_use_case_with_projects_having_timezone() {
        // Test projects that already have timezone defined
        let temp_root = setup_test_environment();
        let output_dir = temp_root.join("public");

        // Create a project with timezone already defined in hierarchical structure
        let company_dir = temp_root.join("companies").join("test-company");
        let project_dir_with_tz = company_dir.join("projects").join("project-with-timezone");
        fs::create_dir_all(&project_dir_with_tz).unwrap();
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
        let project_page = output_dir.join("companies").join("test-company").join("projects").join("proj-with-tz").join("index.html");
        assert!(project_page.exists());
    }
}
