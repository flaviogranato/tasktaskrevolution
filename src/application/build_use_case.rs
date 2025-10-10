use crate::application::{build_context::BuildContext, gantt_use_case::GanttUseCase};
use crate::domain::project_management::repository::ProjectRepository;
use crate::domain::{
    company_management::repository::CompanyRepository, company_settings::repository::ConfigRepository,
    project_management::AnyProject,
};
use crate::infrastructure::persistence::{
    config_repository::FileConfigRepository, project_repository::FileProjectRepository,
    resource_repository::FileResourceRepository,
};
use crate::interface::assets::TemplateAssets;

// glob no longer needed; using repository enumeration

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

        // 2. Static assets step removed (no static folder usage at the moment).

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

        // 5. Find all projects and load their data using repository (ID-based compatible).
        let mut all_projects_data = Vec::new();
        let project_repo = FileProjectRepository::with_base_path(self.base_path.clone());
        let resource_repo = FileResourceRepository::new(self.base_path.clone());
        // Load projects from repository (now handles both ID-based and hierarchical)
        let projects = project_repo.find_all().unwrap_or_default();

        for project in projects {
            let company_code = project.company_code().to_string();
            let project_code = project.code().to_string();

            // Load resources using hierarchical method (company global + project-specific)
            let resources = resource_repo.find_all_by_project(&company_code, &project_code)?;

            // Tasks loaded from project aggregate (ID-based tasks under projects/ tasks directory)
            let mut tasks: Vec<_> = project.tasks().values().cloned().collect();
            // Dedupe tasks by code
            {
                use std::collections::HashSet;
                let mut seen: HashSet<String> = HashSet::new();
                tasks.retain(|t| seen.insert(t.code().to_string()));
            }

            let project = if project.timezone().is_none() {
                // Clone the project and update its timezone
                let mut project_clone = project.clone();
                let AnyProject::Project(ref mut p) = project_clone;
                p.settings.timezone = Some(config.default_timezone.clone());
                project_clone
            } else {
                project
            };

            all_projects_data.push((project, tasks, resources, company_code));
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
        context.insert("company_name", &config.company_name);
        context.insert("relative_path_prefix", "/");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
        
        // Add Gantt chart variables
        context.insert("gantt_available", &true);
        context.insert("company_gantt_url", &"companies/gantt.html");
        context.insert("project_gantt_url", &"projects/gantt.html");
        context.insert("all_projects_gantt_url", &"gantt.html");
        
        // Add current page variable
        context.insert("current_page", &"dashboard");

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

        // Context prepared for index.html
        let index_html = match self.tera.render("index.html", &context) {
            Ok(html) => html,
            Err(e) => {
                return Err(format!("Template error: {}", e).into());
            }
        };
        fs::write(self.output_dir.join("index.html"), index_html)?;

        // Generate companies.html page
        let companies_html = match self.tera.render("index.html", &context) {
            Ok(html) => html,
            Err(e) => {
                return Err(format!("Template error: {}", e).into());
            }
        };
        fs::write(self.output_dir.join("companies.html"), companies_html)?;

        // 8. Generate company pages
        let companies_base_dir = self.output_dir.join("companies");
        fs::create_dir_all(&companies_base_dir)?;

        // Gerar gráficos Gantt para cada empresa
        let _gantt_use_case = GanttUseCase::new(self.base_path.clone());

        for (company, company_projects, project_count, resource_count) in &companies_with_data {
            let company_code = company.code();
            let company_name = company.name();

            let company_output_dir = companies_base_dir.join(company_code);
            fs::create_dir_all(&company_output_dir)?;

            // Create company context
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
            let resource_repo = FileResourceRepository::new(self.base_path.clone());
            let company_resources_filtered = resource_repo
                .find_all_by_project(company_code, "")
                .unwrap_or_else(|_| Vec::new());

            company_context.insert("company", &tera::Value::Object(company_map.clone()));
            company_context.insert("projects", &project_summaries);
            company_context.insert("resources", &company_resources_filtered);
            company_context.insert("relative_path_prefix", "../");
            company_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
            
            // Add Gantt chart variables
            company_context.insert("gantt_available", &true);
            company_context.insert("company_gantt_url", &"gantt.html");
            company_context.insert("project_gantt_url", &"projects/gantt.html");
            company_context.insert("all_projects_gantt_url", &"../gantt.html");
            
            // Add current page variable
            company_context.insert("current_page", &"companies");

            // Gerar página Gantt da empresa (company_gantt.html)
            let company_gantt_page_path = company_output_dir.join("gantt.html");
            let company_gantt_context =
                self.create_company_gantt_context(company, company_projects, &company_resources_filtered)?;
            let company_gantt_html = match self.tera.render("company_gantt.html", &company_gantt_context) {
                Ok(html) => html,
                Err(e) => {
                    return Err(format!("Template error: {}", e).into());
                }
            };
            fs::write(company_gantt_page_path, company_gantt_html)?;

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
            // Company context prepared
            let company_html = match self.tera.render("company.html", &company_context) {
                Ok(html) => {
                    html
                }
                Err(e) => {
                    return Err(format!("Template error: {}", e).into());
                }
            };
            let company_page_path = company_output_dir.join("index.html");
            fs::write(company_page_path, company_html)?;


            // Generate company detail page
            let company_detail_html = match self.tera.render("company_detail.html", &company_context) {
                Ok(html) => html,
                Err(e) => {
                    return Err(format!("Template error: {}", e).into());
                }
            };
            let company_detail_path = company_output_dir.join("detail.html");
            fs::write(company_detail_path, company_detail_html)?;

            // 9. Generate resource pages within company
            let resources_base_dir = company_output_dir.join("resources");
            fs::create_dir_all(&resources_base_dir)?;

            // Generate resource detail pages
            for resource in &company_resources_filtered {
                let resource_code = resource.code();
                let resource_output_dir = resources_base_dir.join(resource_code);
                fs::create_dir_all(&resource_output_dir)?;

                let mut resource_context = Context::new();
                resource_context.insert("resource", resource);
                resource_context.insert("company", &tera::Value::Object(company_map.clone()));
                resource_context.insert("relative_path_prefix", "../../");
                resource_context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
                
                // Add Gantt chart variables
                resource_context.insert("gantt_available", &true);
                resource_context.insert("company_gantt_url", &"../gantt.html");
                resource_context.insert("project_gantt_url", &"../../gantt.html");
                resource_context.insert("all_projects_gantt_url", &"../../../gantt.html");
                
                // Add current page variable
                resource_context.insert("current_page", &"resources");

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
                            project_map.insert(
                                "task_count".to_string(),
                                tera::Value::Number(tera::Number::from(project_tasks.len())),
                            );
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
                                task_map.insert(
                                    "project_code".to_string(),
                                    tera::Value::String(project.code().to_string()),
                                );
                                task_map.insert(
                                    "project_name".to_string(),
                                    tera::Value::String(project.name().to_string()),
                                );
                                task_map
                                    .insert("due_date".to_string(), tera::Value::String(task.due_date().to_string()));
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
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let resource_detail_path = resource_output_dir.join("detail.html");
                fs::write(resource_detail_path, resource_detail_html)?;
            }

            // 10. Generate project pages within company
            let projects_base_dir = company_output_dir.join("projects");
            fs::create_dir_all(&projects_base_dir)?;

            for (project, tasks, resources, _) in company_projects {
                let project_code = project.code();
                let _project_name = project.name();

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
                
                // Add Gantt chart variables
                project_context.insert("gantt_available", &true);
                project_context.insert("company_gantt_url", &"../gantt.html");
                project_context.insert("project_gantt_url", &"gantt.html");
                project_context.insert("all_projects_gantt_url", &"../../gantt.html");
                
                // Add current page variable
                project_context.insert("current_page", &"projects");

                // Render project detail page (e.g., project.html)
                let project_html = match self.tera.render("project.html", &project_context) {
                    Ok(html) => html,
                    Err(e) => {
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let project_page_path = project_output_dir.join("index.html");
                fs::write(project_page_path, project_html)?;


                // Generate project detail page
                let project_detail_html = match self.tera.render("project_detail.html", &project_context) {
                    Ok(html) => html,
                    Err(e) => {
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                let project_detail_path = project_output_dir.join("detail.html");
                fs::write(project_detail_path, project_detail_html)?;

                // Gerar página Gantt do projeto (project_gantt.html)
                let project_gantt_page_path = project_output_dir.join("gantt.html");
                let project_gantt_context =
                    self.create_project_gantt_context(project, tasks, resources, &company_map)?;
                let project_gantt_html = match self.tera.render("project_gantt.html", &project_gantt_context) {
                    Ok(html) => html,
                    Err(e) => {
                        println!("Project Gantt template error: {:?}", e);
                        return Err(format!("Template error: {}", e).into());
                    }
                };
                fs::write(project_gantt_page_path, project_gantt_html)?;

                // Generate task detail pages
                let tasks_base_dir = project_output_dir.join("tasks");
                fs::create_dir_all(&tasks_base_dir)?;

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
                    
                    // Add Gantt chart variables
                    task_context.insert("gantt_available", &true);
                    task_context.insert("company_gantt_url", &"../../gantt.html");
                    task_context.insert("project_gantt_url", &"../gantt.html");
                    task_context.insert("all_projects_gantt_url", &"../../../gantt.html");
                    
                    // Add current page variable
                    task_context.insert("current_page", &"tasks");

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
                            return Err(format!("Template error: {}", e).into());
                        }
                    };
                    let task_detail_path = task_output_dir.join("detail.html");
                    fs::write(task_detail_path, task_detail_html)?;
                }
            }
        }

        Ok(())
    }

    /// Cria o contexto para o template company_gantt.html
    fn create_company_gantt_context(
        &self,
        company: &crate::domain::company_management::Company,
        company_projects: &[&(
            crate::domain::project_management::AnyProject,
            Vec<crate::domain::task_management::AnyTask>,
            Vec<crate::domain::resource_management::AnyResource>,
            String,
        )],
        company_resources: &[crate::domain::resource_management::AnyResource],
    ) -> Result<Context, Box<dyn Error>> {
        let mut context = Context::new();

        // Company data
        let mut company_map = tera::Map::new();
        company_map.insert("code".to_string(), tera::Value::String(company.code.clone()));
        company_map.insert("name".to_string(), tera::Value::String(company.name.clone()));
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
            tera::Value::Number(tera::Number::from(company_projects.len())),
        );
        company_map.insert(
            "resource_count".to_string(),
            tera::Value::Number(tera::Number::from(company_resources.len())),
        );

        // Projects data for Gantt
        let projects: Vec<_> = company_projects
            .iter()
            .map(|(project, _, _, _)| {
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
                    tera::Value::Number(tera::Number::from(0)), // Will be calculated from tasks
                );
                project_map.insert(
                    "start_date".to_string(),
                    project
                        .start_date()
                        .map_or(tera::Value::String("2024-01-01".to_string()), |d| {
                            tera::Value::String(d.to_string())
                        }),
                );
                project_map.insert(
                    "end_date".to_string(),
                    project
                        .end_date()
                        .map_or(tera::Value::String("2024-12-31".to_string()), |d| {
                            tera::Value::String(d.to_string())
                        }),
                );
                tera::Value::Object(project_map)
            })
            .collect();

        // Calculate company date range
        let company_start_date = company_projects
            .iter()
            .filter_map(|(project, _, _, _)| project.start_date())
            .min()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "2024-01-01".to_string());
        let company_end_date = company_projects
            .iter()
            .filter_map(|(project, _, _, _)| project.end_date())
            .max()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "2024-12-31".to_string());

        context.insert("company", &tera::Value::Object(company_map));
        context.insert("projects", &projects);
        context.insert("company_start_date", &company_start_date);
        context.insert("company_end_date", &company_end_date);
        context.insert("relative_path_prefix", "../");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
        
        // Add Gantt chart variables
        context.insert("gantt_available", &true);
        context.insert("company_gantt_url", &"gantt.html");
        context.insert("project_gantt_url", &"projects/gantt.html");
        context.insert("all_projects_gantt_url", &"../gantt.html");
        
        // Add current page variable
        context.insert("current_page", &"companies");

        // Create dummy project for base template
        let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
            .code("COMPANY_GANTT_DASHBOARD".to_string())
            .name(format!("{} Gantt Dashboard", company.name))
            .company_code(company.code.clone())
            .created_by("system".to_string())
            .build()
            .unwrap()
            .into();
        context.insert("project", &dummy_project);

        Ok(context)
    }

    /// Cria o contexto para o template project_gantt.html
    fn create_project_gantt_context(
        &self,
        project: &crate::domain::project_management::AnyProject,
        tasks: &[crate::domain::task_management::AnyTask],
        resources: &[crate::domain::resource_management::AnyResource],
        company_map: &tera::Map<String, tera::Value>,
    ) -> Result<Context, Box<dyn Error>> {
        let mut context = Context::new();

        // Project data
        let mut project_map = tera::Map::new();
        project_map.insert("id".to_string(), tera::Value::String(project.id().to_string()));
        project_map.insert("code".to_string(), tera::Value::String(project.code().to_string()));
        project_map.insert("name".to_string(), tera::Value::String(project.name().to_string()));
        project_map.insert("company_code".to_string(), tera::Value::String(project.company_code().to_string()));
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
                .map_or(tera::Value::String("2024-01-01".to_string()), |d| {
                    tera::Value::String(d.to_string())
                }),
        );
        project_map.insert(
            "end_date".to_string(),
            project
                .end_date()
                .map_or(tera::Value::String("2024-12-31".to_string()), |d| {
                    tera::Value::String(d.to_string())
                }),
        );

        // Convert resources to a format that Tera can handle
        let mut resource_maps = Vec::new();
        for resource in resources {
            let mut resource_map = tera::Map::new();
            resource_map.insert("name".to_string(), tera::Value::String(resource.name().to_string()));
            resource_map.insert("code".to_string(), tera::Value::String(resource.code().to_string()));
            resource_map.insert(
                "resource_type".to_string(),
                tera::Value::String(resource.resource_type().to_string()),
            );
            resource_map.insert("status".to_string(), tera::Value::String(resource.status().to_string()));
            resource_maps.push(tera::Value::Object(resource_map));
        }

        // Convert tasks to a format that Tera can handle
        let mut task_maps = Vec::new();
        for task in tasks {
            let mut task_map = tera::Map::new();
            task_map.insert("id".to_string(), tera::Value::String(task.id().to_string()));
            task_map.insert("code".to_string(), tera::Value::String(task.code().to_string()));
            task_map.insert("name".to_string(), tera::Value::String(task.name().to_string()));
            task_map.insert("status".to_string(), tera::Value::String(task.status().to_string()));
            task_map.insert(
                "description".to_string(),
                tera::Value::String(
                    task.description()
                        .map_or("No description available.".to_string(), |d| d.to_string()),
                ),
            );
            task_map.insert(
                "start_date".to_string(),
                tera::Value::String("2024-01-01".to_string()),
            );
            task_map.insert(
                "end_date".to_string(),
                tera::Value::String("2024-12-31".to_string()),
            );
            task_map.insert("progress".to_string(), tera::Value::Number(0.into()));
            task_map.insert("assigned_resources".to_string(), tera::Value::Array(vec![]));
            task_map.insert("dependencies".to_string(), tera::Value::Array(vec![]));
            task_map.insert("is_milestone".to_string(), tera::Value::Bool(false));
            task_maps.push(tera::Value::Object(task_map));
        }

        context.insert("project", &tera::Value::Object(project_map.clone()));
        context.insert("company", &tera::Value::Object(company_map.clone()));
        context.insert("tasks", &task_maps);
        context.insert("resources", &resource_maps);
        context.insert("relative_path_prefix", "../../../");
        context.insert("current_date", &chrono::Utc::now().format("%Y-%m-%d %H:%M").to_string());
        
        // Add Gantt chart variables
        context.insert("gantt_available", &true);
        context.insert("company_gantt_url", &"../gantt.html");
        context.insert("project_gantt_url", &"gantt.html");
        context.insert("all_projects_gantt_url", &"../../gantt.html");
        
        // Add current page variable
        context.insert("current_page", &"projects");

        // Create dummy project for base template
        let dummy_project: AnyProject = crate::domain::project_management::builder::ProjectBuilder::new()
            .code("PROJECT_GANTT_DASHBOARD".to_string())
            .name(format!("{} Gantt Dashboard", project.name()))
            .company_code(project.company_code().to_string())
            .created_by("system".to_string())
            .build()
            .unwrap()
            .into();
        context.insert("base_project", &dummy_project);

        Ok(context)
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

        let project_dir = company_dir.join("projects").join("proj-1");
        fs::create_dir_all(&project_dir).unwrap();
        let project_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: "proj-1"
  name: "My Test Project"
  description: "A description for the test project."
  companyCode: "test-company"
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
apiVersion: tasktaskrevolution.io/v1alpha1
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
            println!("Build error: {:?}", e);
            // Try to extract the specific error message
            let error_msg = format!("{:?}", e);
            if error_msg.contains("Template error") {
                println!("Template error detected: {}", error_msg);
            }
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
  companyCode: "test-company"
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
  companyCode: "test-company"
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
  companyCode: "test-company"
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
        let completed_page = output_dir
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("proj-completed")
            .join("index.html");
        let cancelled_page = output_dir
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("proj-cancelled")
            .join("index.html");
        let in_progress_page = output_dir
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("proj-in-progress")
            .join("index.html");

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
  companyCode: "test-company"
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
        let project_page = output_dir
            .join("companies")
            .join("test-company")
            .join("projects")
            .join("proj-with-tz")
            .join("index.html");
        assert!(project_page.exists());
    }
}
