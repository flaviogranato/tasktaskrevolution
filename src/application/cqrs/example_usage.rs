use crate::application::cqrs::{
    commands::{
        company::{CreateCompanyCommand, UpdateCompanyCommand, DeleteCompanyCommand},
        project::{CreateProjectCommand, UpdateProjectCommand, CancelProjectCommand},
        task::{CreateTaskCommand, UpdateTaskCommand, AddTaskDependencyCommand},
        resource::{CreateResourceCommand, CreateTimeOffCommand},
    },
    queries::{
        company::{GetCompanyQuery, ListCompaniesQuery, CompanyFilters},
        project::{GetProjectQuery, ListProjectsQuery, ProjectFilters},
        task::{GetTaskQuery, ListTasksQuery, TaskFilters},
        resource::{GetResourceQuery, ListResourcesQuery, ResourceFilters},
    },
    handlers::{
        command_handlers::{
            CompanyCommandHandler, ProjectCommandHandler, TaskCommandHandler, ResourceCommandHandler,
        },
        query_handlers::{
            CompanyQueryHandler, ProjectQueryHandler, TaskQueryHandler, ResourceQueryHandler,
        },
    },
};

/// Exemplo de uso do CQRS
pub struct CQRSExample {
    company_command_handler: CompanyCommandHandler,
    project_command_handler: ProjectCommandHandler,
    task_command_handler: TaskCommandHandler,
    resource_command_handler: ResourceCommandHandler,
    company_query_handler: CompanyQueryHandler,
    project_query_handler: ProjectQueryHandler,
    task_query_handler: TaskQueryHandler,
    resource_query_handler: ResourceQueryHandler,
}

impl CQRSExample {
    pub fn new() -> Self {
        Self {
            company_command_handler: CompanyCommandHandler::new(),
            project_command_handler: ProjectCommandHandler::new(),
            task_command_handler: TaskCommandHandler::new(),
            resource_command_handler: ResourceCommandHandler::new(),
            company_query_handler: CompanyQueryHandler::new(),
            project_query_handler: ProjectQueryHandler::new(),
            task_query_handler: TaskQueryHandler::new(),
            resource_query_handler: ResourceQueryHandler::new(),
        }
    }

    /// Exemplo de fluxo completo usando CQRS
    pub fn run_example(&self) -> Result<(), String> {
        println!("🚀 Executando exemplo de CQRS...");

        // 1. Criar empresa
        let create_company_cmd = CreateCompanyCommand {
            name: "TechCorp".to_string(),
            code: "TECH".to_string(),
            description: Some("Empresa de tecnologia".to_string()),
        };
        
        let company = self.company_command_handler.handle_create_company(create_company_cmd)?;
        println!("✅ Empresa criada: {} ({})", company.name(), company.code());

        // 2. Criar projeto
        let create_project_cmd = CreateProjectCommand {
            name: "Projeto Alpha".to_string(),
            code: "ALPHA".to_string(),
            description: Some("Projeto de desenvolvimento".to_string()),
            company_code: "TECH".to_string(),
        };
        
        let project = self.project_command_handler.handle_create_project(create_project_cmd)?;
        println!("✅ Projeto criado: {} ({})", project.name(), project.code());

        // 3. Criar tarefa
        let create_task_cmd = CreateTaskCommand {
            name: "Implementar API".to_string(),
            code: "API-001".to_string(),
            description: Some("Desenvolver API REST".to_string()),
            project_code: "ALPHA".to_string(),
            start_date: None,
            end_date: None,
            priority: Some("high".to_string()),
        };
        
        let task = self.task_command_handler.handle_create_task(create_task_cmd)?;
        println!("✅ Tarefa criada: {} ({})", task.name(), task.code());

        // 4. Criar recurso
        let create_resource_cmd = CreateResourceCommand {
            name: "João Silva".to_string(),
            code: "DEV-001".to_string(),
            email: Some("joao@techcorp.com".to_string()),
            resource_type: "person".to_string(),
        };
        
        let resource = self.resource_command_handler.handle_create_resource(create_resource_cmd)?;
        println!("✅ Recurso criado: {} ({})", resource.name(), resource.code());

        // 5. Queries - Listar empresas
        let list_companies_query = ListCompaniesQuery {
            filters: Some(CompanyFilters {
                name_contains: Some("Tech".to_string()),
                code_contains: None,
            }),
        };
        
        let companies = self.company_query_handler.handle_list_companies(list_companies_query)?;
        println!("📋 Empresas encontradas: {}", companies.len());

        // 6. Queries - Listar projetos
        let list_projects_query = ListProjectsQuery {
            company_code: "TECH".to_string(),
            filters: Some(ProjectFilters {
                name_contains: Some("Alpha".to_string()),
                code_contains: None,
                status: None,
            }),
        };
        
        let projects = self.project_query_handler.handle_list_projects(list_projects_query)?;
        println!("📋 Projetos encontrados: {}", projects.len());

        // 7. Queries - Listar tarefas
        let list_tasks_query = ListTasksQuery {
            project_code: "ALPHA".to_string(),
            filters: Some(TaskFilters {
                name_contains: Some("API".to_string()),
                code_contains: None,
                status: None,
                priority: Some("high".to_string()),
            }),
        };
        
        let tasks = self.task_query_handler.handle_list_tasks(list_tasks_query)?;
        println!("📋 Tarefas encontradas: {}", tasks.len());

        // 8. Queries - Listar recursos
        let list_resources_query = ListResourcesQuery {
            filters: Some(ResourceFilters {
                name_contains: Some("João".to_string()),
                code_contains: None,
                email_contains: None,
                resource_type: Some("person".to_string()),
            }),
        };
        
        let resources = self.resource_query_handler.handle_list_resources(list_resources_query)?;
        println!("📋 Recursos encontrados: {}", resources.len());

        println!("🎉 Exemplo de CQRS executado com sucesso!");
        Ok(())
    }
}
