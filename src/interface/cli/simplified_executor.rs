use crate::application::{
    company_management::{CreateCompanyArgs, CreateCompanyUseCase},
    create::{
        project::CreateProjectUseCase,
        resource::{CreateResourceUseCase, CreateResourceParams},
        task::{CreateTaskArgs, CreateTaskUseCase},
    },
    execution_context::ExecutionContext,
    list::{
        companies::ListCompaniesUseCase, projects::ListProjectsUseCase, resources::ListResourcesUseCase,
        tasks::ListTasksUseCase,
    },
    project::{
        cancel_project::CancelProjectUseCase,
        update_project::{UpdateProjectArgs, UpdateProjectUseCase},
    },
    resource::{
        deactivate_resource::DeactivateResourceUseCase,
        update_resource::{UpdateResourceArgs, UpdateResourceUseCase},
    },
    task::{
        delete_task::DeleteTaskUseCase,
        update_task::{UpdateTaskArgs, UpdateTaskUseCase},
    },
};
use crate::interface::cli::{
    commands::{CreateCommand, DeleteCommand, ListCommand, UpdateCommand},
    context_manager::ContextManager,
    table_formatter::TableFormatter,
    Cli,
};
use chrono::NaiveDate;

/// Simplified command executor that directly calls use cases
pub struct SimplifiedExecutor;

impl SimplifiedExecutor {
    /// Execute create commands
    pub fn execute_create(command: CreateCommand) -> Result<(), Box<dyn std::error::Error>> {
        let context_manager = ContextManager::new()?;
        if Cli::is_verbose() {
            println!("[INFO] Current context: {}", context_manager.context().display_name());
        }

        match command {
            CreateCommand::Company {
                name,
                code,
                description,
            } => {
                context_manager.validate_command("create", "company")?;

                // Only allow company creation from root context
                match context_manager.context() {
                    ExecutionContext::Root => {
                        let company_repo = context_manager.create_company_repository();
                        let use_case = CreateCompanyUseCase::new(company_repo);

                        let args = CreateCompanyArgs {
                            code,
                            name: name.clone(),
                            description,
                            tax_id: None,
                            address: None,
                            email: None,
                            phone: None,
                            website: None,
                            industry: None,
                            created_by: "CLI".to_string(),
                        };

                        match use_case.execute(args) {
                            Ok(company) => {
                                println!("✅ Company created successfully!");
                                println!("   Name: {}", company.name());
                                println!("   Code: {}", company.code());
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to create company: {}", e);
                                Err(Box::new(e))
                            }
                        }
                    }
                    _ => Err("Companies can only be created from root context".into()),
                }
            }
            CreateCommand::Project {
                name,
                code,
                company,
                description,
                start_date,
                end_date,
                template: _,
                template_vars: _,
            } => {
                context_manager.validate_command("create", "project")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let project_repo = context_manager.get_project_repository();
                let use_case = CreateProjectUseCase::new(project_repo);

                // Parse dates
                let start_date_parsed = start_date.parse::<chrono::NaiveDate>()
                    .map_err(|e| format!("Invalid start date format: {}", e))?;
                let end_date_parsed = end_date.parse::<chrono::NaiveDate>()
                    .map_err(|e| format!("Invalid end date format: {}", e))?;

                match use_case.execute(&name, description.as_deref(), company_code.clone(), code, Some(start_date_parsed), Some(end_date_parsed)) {
                    Ok(project) => {
                        println!("✅ Project created successfully!");
                        println!("   Name: {}", project.name());
                        println!("   Code: {}", project.code());
                        println!("   Company: {}", company_code);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create project: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            CreateCommand::Task {
                name,
                code,
                project,
                company,
                description: _,
                start_date,
                due_date,
                assigned_resources,
            } => {
                context_manager.validate_command("create", "task")?;

                let (project_code, company_code) = context_manager.resolve_project_codes(project, company)?;
                let project_repo = context_manager.get_project_repository();

                // Parse dates
                let start_date_parsed = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid start date format: {}", e))?;
                let due_date_parsed = NaiveDate::parse_from_str(&due_date, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid due date format: {}", e))?;

                // Parse assigned resources
                let assigned_resources_vec = if let Some(resources) = assigned_resources {
                    resources.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    Vec::new()
                };

                let args = CreateTaskArgs {
                    company_code: company_code.clone(),
                    project_code: project_code.clone(),
                    name: name.clone(),
                    code,
                    start_date: start_date_parsed,
                    due_date: due_date_parsed,
                    assigned_resources: assigned_resources_vec,
                };

                let task_repo = context_manager.create_task_repository();
                let use_case = CreateTaskUseCase::new(project_repo, task_repo);
                match use_case.execute(args) {
                    Ok(_) => {
                        println!("✅ Task created successfully!");
                        println!("   Name: {}", name);
                        println!("   Project: {}", project_code);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create task: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            CreateCommand::Resource {
                name,
                code,
                email,
                company,
                description,
                start_date,
                end_date,
            } => {
                context_manager.validate_command("create", "resource")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let resource_repo = context_manager.create_resource_repository();
                let use_case = CreateResourceUseCase::new(resource_repo);

                let resource_type = description.as_deref().unwrap_or("employee");
                
                // Parse dates if provided
                let start_date_parsed = if let Some(start_date_str) = start_date {
                    Some(chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d")
                        .map_err(|e| format!("Invalid start date format: {}", e))?)
                } else {
                    None
                };
                
                let end_date_parsed = if let Some(end_date_str) = end_date {
                    Some(chrono::NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d")
                        .map_err(|e| format!("Invalid end date format: {}", e))?)
                } else {
                    None
                };
                
                let params = CreateResourceParams {
                    name: name.clone(),
                    resource_type: resource_type.to_string(),
                    company_code: company_code.clone(),
                    project_code: None,
                    code,
                    email: Some(email),
                    start_date: start_date_parsed,
                    end_date: end_date_parsed,
                };
                match use_case.execute(params) {
                    Ok(_) => {
                        println!("✅ Resource created successfully!");
                        println!("   Name: {}", name);
                        println!("   Company: {}", company_code);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to create resource: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
        }
    }

    /// Execute list commands
    pub fn execute_list(command: ListCommand) -> Result<(), Box<dyn std::error::Error>> {
        let context_manager = ContextManager::new()?;
        
        // Determine context based on command parameters
        let display_context = match &command {
            ListCommand::Resources { company } => {
                if let Some(company_code) = company {
                    ExecutionContext::Company(company_code.clone())
                } else {
                    context_manager.context().clone()
                }
            }
            ListCommand::Projects { company } => {
                if let Some(company_code) = company {
                    ExecutionContext::Company(company_code.clone())
                } else {
                    context_manager.context().clone()
                }
            }
            ListCommand::Tasks { project, company } => {
                if let Some(project_code) = project {
                    if let Some(company_code) = company {
                        ExecutionContext::Project(company_code.clone(), project_code.clone())
                    } else {
                        ExecutionContext::Project("UNKNOWN".to_string(), project_code.clone())
                    }
                } else if let Some(company_code) = company {
                    ExecutionContext::Company(company_code.clone())
                } else {
                    context_manager.context().clone()
                }
            }
            _ => context_manager.context().clone(),
        };
        
        if Cli::is_verbose() {
            println!("[INFO] Current context: {}", display_context.display_name());
        }

        match command {
            ListCommand::Companies => {
                context_manager.validate_command("list", "companies")?;

                match context_manager.context() {
                    ExecutionContext::Root => {
                        let company_repo = context_manager.create_company_repository();
                        let use_case = ListCompaniesUseCase::new(company_repo);

                        match use_case.execute() {
                            Ok(companies) => {
                                if companies.is_empty() {
                                    println!("No companies found.");
                                } else {
                                    let mut table = TableFormatter::new(vec![
                                        "NAME".to_string(),
                                        "CODE".to_string(),
                                        "DESCRIPTION".to_string(),
                                        "STATUS".to_string(),
                                    ]);
                                    
                                    for company in companies {
                                    table.add_row(vec![
                                        company.name().to_string(),
                                        company.code().to_string(),
                                        company.description.as_deref().unwrap_or("-").to_string(),
                                        if company.is_active() { "Active".to_string() } else { "Inactive".to_string() },
                                    ]);
                                    }
                                    
                                    println!("{}", table);
                                }
                                Ok(())
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to list companies: {}", e);
                                Err(e.into())
                            }
                        }
                    }
                    _ => Err("Companies can only be listed from root context".into()),
                }
            }
            ListCommand::Projects { company } => {
                context_manager.validate_command("list", "projects")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let project_repo = context_manager.get_project_repository();
                let use_case = ListProjectsUseCase::new(project_repo);

                match use_case.execute() {
                    Ok(projects) => {
                        if company_code == "ALL" {
                            // Global listing - show all projects
                            if projects.is_empty() {
                                println!("No projects found.");
                            } else {
                                let mut table = TableFormatter::new(vec![
                                    "NAME".to_string(),
                                    "CODE".to_string(),
                                    "COMPANY".to_string(),
                                    "STATUS".to_string(),
                                ]);
                                
                                for project in projects {
                                    table.add_row(vec![
                                        project.name().to_string(),
                                        project.code().to_string(),
                                        project.company_code().to_string(),
                                        project.status().to_string(),
                                    ]);
                                }
                                
                                println!("{}", table);
                            }
                        } else {
                            // Company-specific listing
                            let filtered_projects: Vec<_> = projects
                                .into_iter()
                                .filter(|p| p.company_code() == company_code)
                                .collect();

                            if filtered_projects.is_empty() {
                                println!("No projects found for company '{}'.", company_code);
                            } else {
                                let mut table = TableFormatter::new(vec![
                                    "NAME".to_string(),
                                    "CODE".to_string(),
                                    "STATUS".to_string(),
                                ]);
                                
                                for project in filtered_projects {
                                    table.add_row(vec![
                                        project.name().to_string(),
                                        project.code().to_string(),
                                        project.status().to_string(),
                                    ]);
                                }
                                
                                println!("{}", table);
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to list projects: {}", e);
                        Err(e.into())
                    }
                }
            }
            ListCommand::Tasks { project, company } => {
                context_manager.validate_command("list", "tasks")?;

                if let Some(project_code) = project {
                    // List tasks for specific project
                    let (project_code, company_code) = context_manager.resolve_project_codes(Some(project_code), company)?;
                    let project_repo = context_manager.get_project_repository();
                    let use_case = ListTasksUseCase::new(project_repo);

                        match use_case.execute(&project_code, &company_code) {
                            Ok(tasks) => {
                                if tasks.is_empty() {
                                    println!("No tasks found for project '{}'.", project_code);
                                } else {
                                    let mut table = TableFormatter::new(vec![
                                        "NAME".to_string(),
                                        "CODE".to_string(),
                                        "STATUS".to_string(),
                                        "START DATE".to_string(),
                                        "DUE DATE".to_string(),
                                    ]);
                                    
                                    for task in tasks {
                                        table.add_row(vec![
                                            task.name().to_string(),
                                            task.code().to_string(),
                                            task.status().to_string(),
                                            task.start_date().format("%Y-%m-%d").to_string(),
                                            task.due_date().format("%Y-%m-%d").to_string(),
                                        ]);
                                    }
                                    
                                    println!("{}", table);
                                }
                                Ok(())
                            }
                        Err(e) => {
                            eprintln!("❌ Failed to list tasks: {}", e);
                            Err(e.into())
                        }
                    }
                } else {
                    // List tasks for all projects in company
                    let company_code = context_manager.resolve_company_code(company)?;
                    let project_repo = context_manager.get_project_repository();
                    let use_case = ListTasksUseCase::new(project_repo);

                    match use_case.execute_all_by_company(&company_code) {
                        Ok(tasks) => {
                            if tasks.is_empty() {
                                println!("No tasks found for company '{}'.", company_code);
                            } else {
                                let mut table = TableFormatter::new(vec![
                                    "NAME".to_string(),
                                    "CODE".to_string(),
                                    "PROJECT".to_string(),
                                    "STATUS".to_string(),
                                    "START DATE".to_string(),
                                    "DUE DATE".to_string(),
                                ]);
                                
                                for task in tasks {
                                    table.add_row(vec![
                                        task.name().to_string(),
                                        task.code().to_string(),
                                        task.project_code().to_string(),
                                        task.status().to_string(),
                                        task.start_date().format("%Y-%m-%d").to_string(),
                                        task.due_date().format("%Y-%m-%d").to_string(),
                                    ]);
                                }
                                
                                println!("{}", table);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to list tasks: {}", e);
                            Err(e.into())
                        }
                    }
                }
            }
            ListCommand::Resources { company } => {
                context_manager.validate_command("list", "resources")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let resource_repo = context_manager.create_resource_repository();
                let use_case = ListResourcesUseCase::new(resource_repo);

                match use_case.execute() {
                    Ok(resources) => {
                        if resources.is_empty() {
                            println!("No resources found for company '{}'.", company_code);
                        } else {
                            let mut table = TableFormatter::new(vec![
                                "NAME".to_string(),
                                "CODE".to_string(),
                                "EMAIL".to_string(),
                                "TYPE".to_string(),
                                "STATUS".to_string(),
                            ]);
                            
                            for resource in resources {
                                table.add_row(vec![
                                    resource.name().to_string(),
                                    resource.code().to_string(),
                                    resource.email().unwrap_or("-").to_string(),
                                    resource.resource_type().to_string(),
                                    resource.status().to_string(),
                                ]);
                            }
                            
                            println!("{}", table);
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to list resources: {}", e);
                        Err(e.into())
                    }
                }
            }
        }
    }

    /// Execute update commands
    pub fn execute_update(command: UpdateCommand) -> Result<(), Box<dyn std::error::Error>> {
        let context_manager = ContextManager::new()?;
        if Cli::is_verbose() {
            println!("[INFO] Current context: {}", context_manager.context().display_name());
        }

        match command {
            UpdateCommand::Project {
                code,
                company,
                name,
                description,
                start_date: _,
                end_date: _,
            } => {
                context_manager.validate_command("update", "project")?;

                let _company_code = context_manager.resolve_company_code(company)?;
                let project_repo = context_manager.get_project_repository();
                let use_case = UpdateProjectUseCase::new(project_repo);

                let args = UpdateProjectArgs { name, description };

                match use_case.execute(&code, args) {
                    Ok(_) => {
                        println!("✅ Project updated successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to update project: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            UpdateCommand::Task {
                code,
                project,
                company,
                name,
                description,
                start_date,
                due_date,
            } => {
                context_manager.validate_command("update", "task")?;

                let (project_code, _company_code) = context_manager.resolve_project_codes(project, company)?;
                let project_repo = context_manager.get_project_repository();
                let task_repo = context_manager.create_task_repository();
                let use_case = UpdateTaskUseCase::new(project_repo, task_repo);

                let start = start_date
                    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                    .transpose()
                    .map_err(|e| format!("Invalid start date format: {}", e))?;
                let due = due_date
                    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
                    .transpose()
                    .map_err(|e| format!("Invalid due date format: {}", e))?;

                let args = UpdateTaskArgs {
                    name,
                    description,
                    start_date: start,
                    due_date: due,
                };

                match use_case.execute(&project_code, &code, args) {
                    Ok(_) => {
                        println!("✅ Task updated successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to update task: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            UpdateCommand::Resource {
                code,
                company,
                name,
                email,
                description,
            } => {
                context_manager.validate_command("update", "resource")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let resource_repo = context_manager.create_resource_repository();
                let use_case = UpdateResourceUseCase::new(resource_repo);

                let args = UpdateResourceArgs {
                    name,
                    email,
                    resource_type: description,
                };

                match use_case.execute(&code, &company_code, args) {
                    Ok(_) => {
                        println!("✅ Resource updated successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to update resource: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
        }
    }

    /// Execute delete commands
    pub fn execute_delete(command: DeleteCommand) -> Result<(), Box<dyn std::error::Error>> {
        let context_manager = ContextManager::new()?;
        if Cli::is_verbose() {
            println!("[INFO] Current context: {}", context_manager.context().display_name());
        }

        match command {
            DeleteCommand::Project { code, company } => {
                context_manager.validate_command("delete", "project")?;

                let _company_code = context_manager.resolve_company_code(company)?;
                let project_repo = context_manager.get_project_repository();
                let use_case = CancelProjectUseCase::new(project_repo);

                match use_case.execute(&code) {
                    Ok(_) => {
                        println!("✅ Project cancelled successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to cancel project: {}", e);
                        Err(e.into())
                    }
                }
            }
            DeleteCommand::Task { code, project, company } => {
                context_manager.validate_command("delete", "task")?;

                let (project_code, _company_code) = context_manager.resolve_project_codes(project, company)?;
                let project_repo = context_manager.get_project_repository();
                let use_case = DeleteTaskUseCase::new(project_repo);

                match use_case.execute(&project_code, &code) {
                    Ok(_) => {
                        println!("✅ Task cancelled successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to cancel task: {}", e);
                        Err(e.into())
                    }
                }
            }
            DeleteCommand::Resource { code, company } => {
                context_manager.validate_command("delete", "resource")?;

                let company_code = context_manager.resolve_company_code(company)?;
                let resource_repo = context_manager.create_resource_repository();
                let use_case = DeactivateResourceUseCase::new(resource_repo);

                match use_case.execute(&code, &company_code) {
                    Ok(_) => {
                        println!("✅ Resource deactivated successfully!");
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to deactivate resource: {}", e);
                        Err(e.into())
                    }
                }
            }
        }
    }
}
