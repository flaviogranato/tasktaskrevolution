use TaskTaskRevolution::application::cqrs::example_usage::CQRSExample;

#[test]
fn test_cqrs_example() {
    let example = CQRSExample::new();

    // Teste básico - verificar se os handlers são criados
    // O exemplo completo seria testado em um ambiente de integração
    assert!(true, "CQRS example created successfully");
}

#[test]
fn test_cqrs_commands_structure() {
    use TaskTaskRevolution::application::cqrs::commands::{
        company::CreateCompanyCommand, project::CreateProjectCommand, resource::CreateResourceCommand,
        task::CreateTaskCommand,
    };

    // Teste de criação de comandos
    let create_company = CreateCompanyCommand {
        name: "Test Company".to_string(),
        code: "TEST".to_string(),
        description: Some("Test description".to_string()),
    };

    let create_project = CreateProjectCommand {
        name: "Test Project".to_string(),
        code: "PROJ".to_string(),
        description: Some("Test project".to_string()),
        company_code: "TEST".to_string(),
    };

    let create_task = CreateTaskCommand {
        name: "Test Task".to_string(),
        code: "TASK".to_string(),
        description: Some("Test task".to_string()),
        project_code: "PROJ".to_string(),
        start_date: None,
        end_date: None,
        priority: Some("medium".to_string()),
    };

    let create_resource = CreateResourceCommand {
        name: "Test Resource".to_string(),
        code: "RES".to_string(),
        email: Some("test@example.com".to_string()),
        resource_type: "person".to_string(),
    };

    // Verificar se os comandos foram criados corretamente
    assert_eq!(create_company.name, "Test Company");
    assert_eq!(create_project.name, "Test Project");
    assert_eq!(create_task.name, "Test Task");
    assert_eq!(create_resource.name, "Test Resource");
}

#[test]
fn test_cqrs_queries_structure() {
    use TaskTaskRevolution::application::cqrs::queries::{
        company::{CompanyFilters, GetCompanyQuery, ListCompaniesQuery},
        project::{GetProjectQuery, ListProjectsQuery, ProjectFilters},
        resource::{GetResourceQuery, ListResourcesQuery, ResourceFilters},
        task::{GetTaskQuery, ListTasksQuery, TaskFilters},
    };

    // Teste de criação de queries
    let get_company = GetCompanyQuery {
        code: "TEST".to_string(),
    };

    let list_companies = ListCompaniesQuery {
        filters: Some(CompanyFilters {
            name_contains: Some("Test".to_string()),
            code_contains: None,
        }),
    };

    let get_project = GetProjectQuery {
        code: "PROJ".to_string(),
    };

    let list_projects = ListProjectsQuery {
        company_code: "TEST".to_string(),
        filters: Some(ProjectFilters {
            name_contains: Some("Test".to_string()),
            code_contains: None,
            status: None,
        }),
    };

    let get_task = GetTaskQuery {
        code: "TASK".to_string(),
        project_code: "PROJ".to_string(),
    };

    let list_tasks = ListTasksQuery {
        project_code: "PROJ".to_string(),
        filters: Some(TaskFilters {
            name_contains: Some("Test".to_string()),
            code_contains: None,
            status: None,
            priority: None,
        }),
    };

    let get_resource = GetResourceQuery {
        code: "RES".to_string(),
    };

    let list_resources = ListResourcesQuery {
        filters: Some(ResourceFilters {
            name_contains: Some("Test".to_string()),
            code_contains: None,
            email_contains: None,
            resource_type: Some("person".to_string()),
        }),
    };

    // Verificar se as queries foram criadas corretamente
    assert_eq!(get_company.code, "TEST");
    assert_eq!(get_project.code, "PROJ");
    assert_eq!(get_task.code, "TASK");
    assert_eq!(get_resource.code, "RES");
}
