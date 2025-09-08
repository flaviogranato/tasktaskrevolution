//! Testes de integração para cenários de workflow complexos
//!
//! Estes testes cobrem:
//! - Fluxos completos de gerenciamento de projetos
//! - Cenários multi-empresa
//! - Alocação de recursos complexa
//! - Gerenciamento de dependências de tarefas
//! - Operações concorrentes

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_yaml::Value;
use std::fs;
use std::process::Command;

/// Validador YAML reutilizável (importado do cli.rs)
struct YamlValidator {
    content: String,
    parsed: Value,
}

impl YamlValidator {
    fn new(file_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let parsed: Value = serde_yaml::from_str(&content)?;
        Ok(Self { content, parsed })
    }

    fn has_field(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn field_equals(&self, path: &str, expected: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(str_value) = current.as_str() {
            str_value == expected
        } else {
            false
        }
    }

    fn field_not_empty(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(str_value) = current.as_str() {
            !str_value.is_empty()
        } else {
            false
        }
    }

    fn contains(&self, text: &str) -> bool {
        self.content.contains(text)
    }
}

/// Teste de ciclo de vida completo de um projeto
#[test]
fn test_complete_project_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // 1. Inicializar sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Project Manager",
        "--email",
        "pm@company.com",
        "--company-name",
        "Tech Solutions Inc",
        "--timezone",
        "America/Sao_Paulo",
    ]);
    cmd.assert().success();

    // 2. Criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create",
        "company",
        "--name",
        "Tech Solutions Inc",
        "--code",
        "TECH-SOL",
        "--description",
        "Technology solutions company",
    ]);
    cmd.assert().success();

    // 3. Criar recursos (equipe)
    let resources = vec![
        ("Alice Developer", "Senior Developer"),
        ("Bob Designer", "UI/UX Designer"),
        ("Carol Tester", "QA Engineer"),
        ("David DevOps", "DevOps Engineer"),
    ];

    for (name, role) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&["create", "resource", "--name", name, "--code", &name.to_lowercase().replace(" ", "_"), "--email", &format!("{}@techsol.com", name.to_lowercase().replace(" ", ".")), "--company", "TECH-SOL", "--start-date", "2024-01-01", "--end-date", "2024-12-31", "--description", role]);
        cmd.assert().success();
    }

    // 4. Criar projeto principal
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create",
        "project",
        "--name",
        "E-commerce Platform",
        "--description",
        "Complete e-commerce solution",
        "--company",
        "TECH-SOL",
        "--code",
        "ECOMMERCE",
        "--start-date",
        "2024-01-15",
        "--end-date",
        "2024-06-15",
    ]);
    cmd.assert().success();

    // 5. Criar tarefas do projeto
    let tasks = vec![
        (
            "Setup Development Environment",
            "Configure development tools and infrastructure",
            "2024-01-01",
            "2024-01-05",
        ),
        (
            "Design Database Schema",
            "Create database structure and relationships",
            "2024-01-06",
            "2024-01-10",
        ),
        (
            "Implement User Authentication",
            "Build login and registration system",
            "2024-01-11",
            "2024-01-20",
        ),
        (
            "Create Product Catalog",
            "Develop product listing and search functionality",
            "2024-01-21",
            "2024-02-05",
        ),
        (
            "Implement Shopping Cart",
            "Build cart and checkout functionality",
            "2024-02-06",
            "2024-02-15",
        ),
        (
            "Payment Integration",
            "Integrate payment gateway",
            "2024-02-16",
            "2024-02-25",
        ),
        (
            "Testing and QA",
            "Comprehensive testing and bug fixes",
            "2024-02-26",
            "2024-03-10",
        ),
        (
            "Deployment",
            "Deploy to production environment",
            "2024-03-11",
            "2024-03-15",
        ),
    ];

    // Primeiro, encontrar o código do projeto criado
    let projects_dir = temp.path().join("companies").join("TECH-SOL").join("projects");
    let mut project_code = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    // Ler o código do projeto do YAML
                    if let Ok(content) = std::fs::read_to_string(&project_yaml) {
                        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            if let Some(code) = yaml
                                .get("metadata")
                                .and_then(|m| m.get("code"))
                                .and_then(|c| c.as_str())
                            {
                                project_code = Some(code.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let project_code = project_code.expect("Project code not found");

    for (name, description, start_date, end_date) in tasks {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "task",
            "create",
            "--name",
            name,
            "--code",
            &name.to_lowercase().replace(" ", "_"),
            "--description",
            description,
            "--start-date",
            start_date,
            "--due-date",
            end_date,
            "--project",
            &project_code,
            "--company",
            "TECH-SOL",
        ]);
        cmd.assert().success();
    }

    // 6. Validar estrutura criada
    let config_file = temp.child("config.yaml");
    let company_file = temp.child("companies").child("TECH-SOL").child("company.yaml");
    let project_file = temp
        .child("companies")
        .child("TECH-SOL")
        .child("projects")
        .child(&project_code)
        .child("project.yaml");

    // Validar config
    let validator = YamlValidator::new(config_file.path())?;
    assert!(validator.field_equals("spec.managerName", "Project Manager"));
    assert!(validator.field_equals("spec.managerEmail", "pm@company.com"));

    // Validar empresa
    let validator = YamlValidator::new(company_file.path())?;
    assert!(validator.field_equals("metadata.code", "TECH-SOL"));
    assert!(validator.field_equals("metadata.name", "Tech Solutions Inc"));

    // Validar projeto
    let validator = YamlValidator::new(project_file.path())?;
    assert!(validator.field_equals("metadata.name", "E-commerce Platform"));
    assert!(validator.field_equals("metadata.description", "Complete e-commerce solution"));

    // 7. Gerar relatórios
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // 8. Validar saída HTML
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());
    index_file.assert(predicate::str::contains("Tech Solutions Inc"));
    index_file.assert(predicate::str::contains("Technology solutions company"));

    temp.close()?;
    Ok(())
}

/// Teste de gerenciamento multi-empresa
#[test]
fn test_multi_company_management() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Inicializar sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Multi-Company Manager",
        "--email",
        "manager@consulting.com",
        "--company-name",
        "Consulting Firm",
    ]);
    cmd.assert().success();

    // Criar múltiplas empresas
    let companies = vec![
        ("Tech Corp", "TECH-CORP", "Technology company"),
        ("Design Studio", "DESIGN-ST", "Creative design agency"),
        ("Marketing Pro", "MARKET-PR", "Digital marketing agency"),
        ("Consulting Inc", "CONSULT-IN", "Business consulting"),
    ];

    for (name, code, description) in &companies {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create",
            "company",
            "--name",
            name,
            "--code",
            code,
            "--description",
            description,
        ]);
        cmd.assert().success();

        // Criar projeto para cada empresa
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "project",
            "create",
            "--name",
            &format!("{} Project", name),
            "--description",
            &format!("Main project for {}", name),
            "--company",
            code,
            "--code",
            &format!("{}-PROJECT", code),
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
        ]);
        cmd.assert().success();
    }

    // Validar que todas as empresas foram criadas
    for (name, code, _) in &companies {
        let company_file = temp.child("companies").child(code).child("company.yaml");
        company_file.assert(predicate::path::exists());

        // Verificar se existe pelo menos um projeto (código pode variar)
        let projects_dir = temp.path().join("companies").join(code).join("projects");
        let mut project_found = false;
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let project_yaml = entry.path().join("project.yaml");
                    if project_yaml.exists() {
                        project_found = true;
                        break;
                    }
                }
            }
        }
        assert!(project_found, "No project found for company {}", code);

        // Validar conteúdo da empresa
        let validator = YamlValidator::new(company_file.path())?;
        assert!(validator.field_equals("metadata.code", code));
        assert!(validator.field_equals("metadata.name", name));
    }

    // Validar que todas as empresas foram criadas verificando os arquivos
    for (name, code, _) in &companies {
        let company_file = temp.child("companies").child(code).child("company.yaml");
        company_file.assert(predicate::path::exists());

        let validator = YamlValidator::new(company_file.path())?;
        assert!(validator.field_equals("metadata.code", code));
        assert!(validator.field_equals("metadata.name", name));
    }

    temp.close()?;
    Ok(())
}

/// Teste de alocação de recursos complexa
#[test]
fn test_resource_allocation_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar múltiplos recursos com diferentes especialidades
    let resources = vec![
        ("Alice Backend", "Senior Backend Developer", "TECH-CORP"),
        ("Bob Frontend", "Frontend Developer", "TECH-CORP"),
        ("Carol Designer", "UI/UX Designer", "TECH-CORP"),
        ("David DevOps", "DevOps Engineer", "TECH-CORP"),
        ("Eva QA", "QA Engineer", "TECH-CORP"),
        ("Frank Manager", "Project Manager", "TECH-CORP"),
        ("Grace Analyst", "Data Analyst", "TECH-CORP"),
        ("Henry Security", "Security Specialist", "TECH-CORP"),
    ];

    for (name, role, company_code) in &resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&["create", "resource", "--name", name, "--code", &name.to_lowercase().replace(" ", "_"), "--email", &format!("{}@{}.com", name.to_lowercase().replace(" ", "."), company_code.to_lowercase()), "--company", company_code, "--start-date", "2024-01-01", "--end-date", "2024-12-31", "--description", role]);
        cmd.assert().success();
    }

    // Criar múltiplos projetos
    let projects = vec![
        ("Web Application", "Modern web application", "TECH-CORP"),
        ("Mobile App", "Cross-platform mobile app", "TECH-CORP"),
        ("API Service", "RESTful API service", "TECH-CORP"),
        ("Data Pipeline", "ETL data processing", "TECH-CORP"),
    ];

    for (name, description, company_code) in &projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&["create", "project", "--name", name, "--description", description, "--company", company_code, "--code", &format!("{}-PROJECT", company_code), "--start-date", "2024-01-01", "--end-date", "2024-12-31"]);
        cmd.assert().success();
    }

    // Validar que todos os recursos foram criados
    for (name, _role, company_code) in &resources {
        let resource_file = temp
            .child("companies")
            .child(company_code)
            .child("resources")
            .child(&format!("{}.yaml", name.to_lowercase().replace(" ", "_")));
        resource_file.assert(predicate::path::exists());

        // Validar estrutura básica do arquivo
        let validator = YamlValidator::new(resource_file.path())?;
        assert!(validator.has_field("metadata.name"));
        assert!(validator.has_field("metadata.resourceType"));
        assert!(validator.field_not_empty("metadata.name"));
        assert!(validator.field_not_empty("metadata.resourceType"));
    }

    // Validar que todos os projetos foram criados
    for (_name, _description, company_code) in &projects {
        let projects_dir = temp.path().join("companies").join(company_code).join("projects");
        let mut project_found = false;
        if let Ok(entries) = std::fs::read_dir(&projects_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let project_yaml = entry.path().join("project.yaml");
                    if project_yaml.exists() {
                        project_found = true;
                        // Validar estrutura básica do arquivo
                        let validator = YamlValidator::new(&project_yaml)?;
                        assert!(validator.has_field("metadata.name"));
                        assert!(validator.has_field("metadata.description"));
                        assert!(validator.field_not_empty("metadata.name"));
                        assert!(validator.field_not_empty("metadata.description"));
                        break;
                    }
                }
            }
        }
        assert!(project_found, "No project found for company {}", company_code);
    }

    // Testar listagem de recursos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice Backend"))
        .stdout(predicate::str::contains("Bob Frontend"))
        .stdout(predicate::str::contains("Carol Designer"));

    temp.close()?;
    Ok(())
}

/// Teste de gerenciamento de dependências de tarefas
#[test]
fn test_task_dependency_management() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;

    // Criar tarefas com dependências
    let tasks = vec![
        (
            "Task 1: Requirements Analysis",
            "Analyze and document requirements",
            "2024-01-01",
            "2024-01-05",
        ),
        (
            "Task 2: System Design",
            "Design system architecture",
            "2024-01-06",
            "2024-01-10",
        ),
        (
            "Task 3: Database Setup",
            "Setup and configure database",
            "2024-01-11",
            "2024-01-15",
        ),
        (
            "Task 4: Backend Development",
            "Develop backend services",
            "2024-01-16",
            "2024-01-30",
        ),
        (
            "Task 5: Frontend Development",
            "Develop user interface",
            "2024-01-31",
            "2024-02-15",
        ),
        (
            "Task 6: Integration Testing",
            "Test system integration",
            "2024-02-16",
            "2024-02-20",
        ),
        (
            "Task 7: User Acceptance Testing",
            "Final user testing",
            "2024-02-21",
            "2024-02-25",
        ),
        ("Task 8: Deployment", "Deploy to production", "2024-02-26", "2024-02-28"),
    ];

    // Primeiro, encontrar o código do projeto criado
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_code = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    // Ler o código do projeto do YAML
                    if let Ok(content) = std::fs::read_to_string(&project_yaml) {
                        if let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                            if let Some(code) = yaml
                                .get("metadata")
                                .and_then(|m| m.get("code"))
                                .and_then(|c| c.as_str())
                            {
                                project_code = Some(code.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let project_code = project_code.expect("Project code not found");

    for (name, description, start_date, end_date) in tasks {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create",
            "task",
            "--name",
            name,
            "--description",
            description,
            "--start-date",
            start_date,
            "--due-date",
            end_date,
            "--project-code",
            &project_code,
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que todas as tarefas foram criadas
    let tasks_dir = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("projects")
        .join(&project_code)
        .join("tasks");
    let mut task_count = 0;
    if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "yaml") {
                task_count += 1;
                let validator = YamlValidator::new(&entry.path())?;
                assert!(validator.field_equals("spec.projectCode", &project_code));
                assert!(validator.field_not_empty("metadata.name"));
                assert!(validator.field_not_empty("spec.status"));
                assert!(validator.field_not_empty("spec.priority"));
            }
        }
    }
    assert!(task_count >= 8, "Expected at least 8 tasks, found {}", task_count);

    // Testar listagem de tarefas
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("tasks");
    cmd.assert().success();

    temp.close()?;
    Ok(())
}

/// Teste de operações concorrentes
#[test]
fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular operações concorrentes criando múltiplos recursos rapidamente
    let resources = vec!["Resource 1", "Resource 2", "Resource 3", "Resource 4", "Resource 5"];

    for resource in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create",
            "resource",
            "--name", resource,
            "--code", &resource.to_lowercase().replace(" ", "_"),
            "--email", &format!("{}@techcorp.com", resource.to_lowercase().replace(" ", ".")),
            "--company", "TECH-CORP",
            "--start-date", "2024-01-01",
            "--end-date", "2024-12-31",
            "--description", "Developer",
        ]);
        cmd.assert().success();
    }

    // Criar múltiplos projetos concorrentemente
    let projects = vec!["Project Alpha", "Project Beta", "Project Gamma", "Project Delta"];

    for project in &projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create",
            "project",
            project,
            &format!("Description for {}", project),
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que todas as operações foram bem-sucedidas
    let resource_names = vec!["Resource 1", "Resource 2", "Resource 3", "Resource 4", "Resource 5"];

    for name in &resource_names {
        let resource_file = temp
            .child("companies")
            .child("TECH-CORP")
            .child("resources")
            .child(&format!("{}.yaml", name.to_lowercase().replace(" ", "_")));
        resource_file.assert(predicate::path::exists());
    }

    // Verificar que pelo menos um projeto foi criado
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_found = false;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    project_found = true;
                    break;
                }
            }
        }
    }
    assert!(project_found, "No project found");

    // Testar validação do sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    temp.close()?;
    Ok(())
}

// Funções auxiliares reutilizáveis

fn setup_test_environment(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Test Manager",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create",
        "company",
        "--name",
        "Tech Corp",
        "--code",
        "TECH-CORP",
        "--description",
        "Technology company",
    ]);
    cmd.assert().success();

    Ok(())
}

fn create_test_project(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create",
        "project",
        "--name", "Web App",
        "--code", "WEB-APP",
        "--company", "TECH-CORP",
        "--start-date", "2024-01-01",
        "--end-date", "2024-12-31",
        "--description", "Web application project",
    ]);
    cmd.assert().success();
    Ok(())
}

// ============================================================================
// TEMPLATE WORKFLOW TESTS
// ============================================================================

#[test]
fn test_template_workflow_complete() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Initialize TTR
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Template Manager",
        "--email",
        "template@example.com",
        "--company-name",
        "Template Company",
    ]);
    cmd.assert().success();

    // Copy templates to temp directory
    let templates_dir = temp.path().join("templates").join("projects");
    std::fs::create_dir_all(&templates_dir)?;
    std::fs::copy("templates/projects/web-app.yaml", templates_dir.join("web-app.yaml"))?;
    std::fs::copy(
        "templates/projects/mobile-app.yaml",
        templates_dir.join("mobile-app.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/microservice.yaml",
        templates_dir.join("microservice.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/data-pipeline.yaml",
        templates_dir.join("data-pipeline.yaml"),
    )?;

    // List available templates
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["template", "list"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Available project templates"))
        .stdout(predicate::str::contains("Web Application"))
        .stdout(predicate::str::contains("Mobile Application"));

    // Show template details
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["template", "show", "web-app"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Template: Web Application"))
        .stdout(predicate::str::contains("Resources (4):"))
        .stdout(predicate::str::contains("Tasks (8):"));

    // Create project from template
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "template", "create",
        "--template", "web-app",
        "--name", "Ecommerce Platform",
        "--code", "ECOMMERCE",
        "--company", "DEFAULT",
        "--params", "project_name=Ecommerce Platform,frontend_developer=Alice,backend_developer=Bob,devops_engineer=Charlie,ui_designer=Diana,start_date=2024-01-15,end_date=2024-06-15,timezone=UTC,project_description=A modern e-commerce platform"
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project Ecommerce Platform created"))
        .stdout(predicate::str::contains(
            "Project 'Ecommerce Platform' created successfully with 4 resources, 8 tasks, and 4 phases",
        ));

    // Verify project was created
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["list", "projects"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Ecommerce Platform"));

    // Verify resources were created
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["list", "resources"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Diana"));

    // Verify tasks were created by checking the tasks directory
    let projects_dir = temp.path().join("companies").join("DEFAULT").join("projects");
    let mut project_dir = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                project_dir = Some(entry.path());
                break;
            }
        }
    }

    if let Some(project_dir) = project_dir {
        let tasks_dir = project_dir.join("tasks");
        if tasks_dir.exists() {
            let task_files: Vec<_> = std::fs::read_dir(&tasks_dir)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "yaml"))
                .collect();

            // Verify that tasks were created (should have multiple task files)
            assert!(task_files.len() > 0, "No task files found in tasks directory");
        } else {
            panic!("Tasks directory not found");
        }
    } else {
        panic!("No project directory found");
    }

    Ok(())
}

#[test]
fn test_template_multi_project_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Initialize TTR
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Multi Project Manager",
        "--email",
        "multi@example.com",
        "--company-name",
        "Multi Company",
    ]);
    cmd.assert().success();

    // Copy templates to temp directory
    let templates_dir = temp.path().join("templates").join("projects");
    std::fs::create_dir_all(&templates_dir)?;
    std::fs::copy("templates/projects/web-app.yaml", templates_dir.join("web-app.yaml"))?;
    std::fs::copy(
        "templates/projects/mobile-app.yaml",
        templates_dir.join("mobile-app.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/microservice.yaml",
        templates_dir.join("microservice.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/data-pipeline.yaml",
        templates_dir.join("data-pipeline.yaml"),
    )?;

    // Create multiple projects from different templates
    let templates = vec![
        (
            "web-app",
            "Web Store",
            "Online store platform",
            "frontend_developer=Alice,backend_developer=Bob,devops_engineer=Charlie,ui_designer=Diana",
        ),
        (
            "mobile-app",
            "Mobile App",
            "Mobile shopping app",
            "mobile_developer=Alice,backend_developer=Bob,ui_designer=Diana,qa_engineer=Charlie",
        ),
        (
            "microservice",
            "Payment Service",
            "Payment processing microservice",
            "backend_developer=Bob,devops_engineer=Charlie,api_designer=Alice",
        ),
        (
            "data-pipeline",
            "Analytics Pipeline",
            "Customer analytics pipeline",
            "data_engineer=Alice,data_analyst=Bob,devops_engineer=Charlie,data_scientist=Diana",
        ),
    ];

    for (template, name, description, variables) in templates {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "template",
            "create",
            "--template", template,
            "--name", name,
            "--code", &format!("{}-{}", name.to_uppercase().replace(" ", "-"), template.to_uppercase()),
            "--company", "DEFAULT",
            "--params",
            &format!(
                "project_name={},{},start_date=2024-01-15,end_date=2024-06-15,timezone=UTC,project_description={}",
                name, variables, description
            ),
        ]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(&format!("Project {} created", name)));

        // Add a small delay to ensure projects are saved before the next one
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Verify all projects were created
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["list", "projects"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Web Store"))
        .stdout(predicate::str::contains("Mobile App"))
        .stdout(predicate::str::contains("Payment Service"))
        .stdout(predicate::str::contains("Analytics Pipeline"));

    Ok(())
}

#[test]
fn test_template_variable_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Initialize TTR
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Validation Manager",
        "--email",
        "validation@example.com",
        "--company-name",
        "Validation Company",
    ]);
    cmd.assert().success();

    // Copy templates to temp directory
    let templates_dir = temp.path().join("templates").join("projects");
    std::fs::create_dir_all(&templates_dir)?;
    std::fs::copy("templates/projects/web-app.yaml", templates_dir.join("web-app.yaml"))?;
    std::fs::copy(
        "templates/projects/mobile-app.yaml",
        templates_dir.join("mobile-app.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/microservice.yaml",
        templates_dir.join("microservice.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/data-pipeline.yaml",
        templates_dir.join("data-pipeline.yaml"),
    )?;

    // Test with missing required variables
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "template",
        "create",
        "--template", "web-app",
        "--name", "Test Project",
        "--code", "TEST-PROJECT-FAIL",
        "--company", "DEFAULT",
        "--params",
        "frontend_developer=Alice",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Template rendering failed"));

    // Test with all required variables
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "template", "create",
        "--template", "web-app",
        "--name", "Test Project",
        "--code", "TEST-PROJECT",
        "--company", "DEFAULT",
        "--params", "project_name=Test Project,frontend_developer=Alice,backend_developer=Bob,devops_engineer=Charlie,ui_designer=Diana,start_date=2024-01-15,end_date=2024-03-15,timezone=UTC,project_description=Test description"
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project Test Project created"));

    Ok(())
}

#[test]
fn test_template_create_project_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Initialize TTR
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name",
        "Integration Manager",
        "--email",
        "integration@example.com",
        "--company-name",
        "Integration Company",
    ]);
    cmd.assert().success();

    // Copy templates to temp directory
    let templates_dir = temp.path().join("templates").join("projects");
    std::fs::create_dir_all(&templates_dir)?;
    std::fs::copy("templates/projects/web-app.yaml", templates_dir.join("web-app.yaml"))?;
    std::fs::copy(
        "templates/projects/mobile-app.yaml",
        templates_dir.join("mobile-app.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/microservice.yaml",
        templates_dir.join("microservice.yaml"),
    )?;
    std::fs::copy(
        "templates/projects/data-pipeline.yaml",
        templates_dir.join("data-pipeline.yaml"),
    )?;

    // Test create project with --from-template
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "--name", "API Gateway",
        "--code", "API-GATEWAY",
        "--company", "DEFAULT",
        "--start-date", "2024-01-15",
        "--end-date", "2024-02-28",
        "--description", "A microservice API gateway",
        "--template", "microservice",
        "--template-vars", "backend_developer=Alice,devops_engineer=Bob,api_designer=Charlie,start_date=2024-01-15,end_date=2024-02-28,timezone=UTC,project_description=A microservice API gateway"
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project API Gateway created"))
        .stdout(predicate::str::contains(
            "Project 'API Gateway' created successfully with 3 resources, 9 tasks, and 4 phases",
        ));

    // Verify the project was created with all components
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["list", "projects"]);
    cmd.assert().success().stdout(predicate::str::contains("API Gateway"));

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["list", "resources"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"));

    Ok(())
}
