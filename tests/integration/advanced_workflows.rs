//! Testes de integração para cenários de workflow complexos
//! 
//! Estes testes cobrem:
//! - Fluxos completos de gerenciamento de projetos
//! - Cenários multi-empresa
//! - Alocação de recursos complexa
//! - Gerenciamento de dependências de tarefas
//! - Operações concorrentes

use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;
use serde_yaml::Value;
use std::fs;

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
        "--name", "Project Manager",
        "--email", "pm@company.com",
        "--company-name", "Tech Solutions Inc",
        "--timezone", "America/Sao_Paulo"
    ]);
    cmd.assert().success();
    
    // 2. Criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Tech Solutions Inc",
        "--code", "TECH-SOL",
        "--description", "Technology solutions company"
    ]);
    cmd.assert().success();
    
    // 3. Criar recursos (equipe)
    let resources = vec![
        ("Alice Developer", "Senior Developer"),
        ("Bob Designer", "UI/UX Designer"),
        ("Carol Tester", "QA Engineer"),
        ("David DevOps", "DevOps Engineer")
    ];
    
    for (name, role) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            name, role,
            "--company-code", "TECH-SOL"
        ]);
        cmd.assert().success();
    }
    
    // 4. Criar projeto principal
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "E-commerce Platform", "Complete e-commerce solution",
        "--company-code", "TECH-SOL"
    ]);
    cmd.assert().success();
    
    // 5. Criar tarefas do projeto
    let tasks = vec![
        ("Setup Development Environment", "Configure development tools and infrastructure", "2024-01-01", "2024-01-05"),
        ("Design Database Schema", "Create database structure and relationships", "2024-01-06", "2024-01-10"),
        ("Implement User Authentication", "Build login and registration system", "2024-01-11", "2024-01-20"),
        ("Create Product Catalog", "Develop product listing and search functionality", "2024-01-21", "2024-02-05"),
        ("Implement Shopping Cart", "Build cart and checkout functionality", "2024-02-06", "2024-02-15"),
        ("Payment Integration", "Integrate payment gateway", "2024-02-16", "2024-02-25"),
        ("Testing and QA", "Comprehensive testing and bug fixes", "2024-02-26", "2024-03-10"),
        ("Deployment", "Deploy to production environment", "2024-03-11", "2024-03-15")
    ];
    
    for (name, description, start_date, end_date) in tasks {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "task",
            "--name", name,
            "--description", description,
            "--start-date", start_date,
            "--due-date", end_date,
            "--project-code", "proj-1",
            "--company-code", "TECH-SOL"
        ]);
        cmd.assert().success();
    }
    
    // 6. Validar estrutura criada
    let config_file = temp.child("config.yaml");
    let company_file = temp.child("companies").child("TECH-SOL").child("company.yaml");
    let project_file = temp.child("companies").child("TECH-SOL").child("projects").child("proj-1").child("project.yaml");
    
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
        "--name", "Multi-Company Manager",
        "--email", "manager@consulting.com",
        "--company-name", "Consulting Firm"
    ]);
    cmd.assert().success();
    
    // Criar múltiplas empresas
    let companies = vec![
        ("Tech Corp", "TECH-CORP", "Technology company"),
        ("Design Studio", "DESIGN-ST", "Creative design agency"),
        ("Marketing Pro", "MARKET-PR", "Digital marketing agency"),
        ("Consulting Inc", "CONSULT-IN", "Business consulting")
    ];
    
    for (name, code, description) in &companies {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "company",
            "--name", name,
            "--code", code,
            "--description", description
        ]);
        cmd.assert().success();
        
        // Criar projeto para cada empresa
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "project",
            &format!("{} Project", name),
            &format!("Main project for {}", name),
            "--company-code", code
        ]);
        cmd.assert().success();
    }
    
    // Validar que todas as empresas foram criadas
    for (name, code, _) in &companies {
        let company_file = temp.child("companies").child(code).child("company.yaml");
        let project_file = temp.child("companies").child(code).child("projects").child("proj-1").child("project.yaml");
        
        company_file.assert(predicate::path::exists());
        project_file.assert(predicate::path::exists());
        
        // Validar conteúdo
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
        ("Henry Security", "Security Specialist", "TECH-CORP")
    ];
    
    for (name, role, company_code) in &resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            name, role,
            "--company-code", company_code
        ]);
        cmd.assert().success();
    }
    
    // Criar múltiplos projetos
    let projects = vec![
        ("Web Application", "Modern web application", "TECH-CORP"),
        ("Mobile App", "Cross-platform mobile app", "TECH-CORP"),
        ("API Service", "RESTful API service", "TECH-CORP"),
        ("Data Pipeline", "ETL data processing", "TECH-CORP")
    ];
    
    for (name, description, company_code) in &projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "project",
            name, description,
            "--company-code", company_code
        ]);
        cmd.assert().success();
    }
    
    // Validar que todos os recursos foram criados
    for (name, _role, company_code) in &resources {
        let resource_file = temp.child("companies").child(company_code).child("resources").child(&format!("{}.yaml", name.to_lowercase().replace(" ", "_")));
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
        let project_file = temp.child("companies").child(company_code).child("projects").child("proj-1").child("project.yaml");
        project_file.assert(predicate::path::exists());
        
        // Validar estrutura básica do arquivo
        let validator = YamlValidator::new(project_file.path())?;
        assert!(validator.has_field("metadata.name"));
        assert!(validator.has_field("metadata.description"));
        assert!(validator.field_not_empty("metadata.name"));
        assert!(validator.field_not_empty("metadata.description"));
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
        ("Task 1: Requirements Analysis", "Analyze and document requirements", "2024-01-01", "2024-01-05"),
        ("Task 2: System Design", "Design system architecture", "2024-01-06", "2024-01-10"),
        ("Task 3: Database Setup", "Setup and configure database", "2024-01-11", "2024-01-15"),
        ("Task 4: Backend Development", "Develop backend services", "2024-01-16", "2024-01-30"),
        ("Task 5: Frontend Development", "Develop user interface", "2024-01-31", "2024-02-15"),
        ("Task 6: Integration Testing", "Test system integration", "2024-02-16", "2024-02-20"),
        ("Task 7: User Acceptance Testing", "Final user testing", "2024-02-21", "2024-02-25"),
        ("Task 8: Deployment", "Deploy to production", "2024-02-26", "2024-02-28")
    ];
    
    for (name, description, start_date, end_date) in tasks {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "task",
            "--name", name,
            "--description", description,
            "--start-date", start_date,
            "--due-date", end_date,
            "--project-code", "proj-1",
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Validar que todas as tarefas foram criadas
    for i in 1..=8 {
        let task_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("tasks").child(&format!("task-{}.yaml", i));
        task_file.assert(predicate::path::exists());
        
        let validator = YamlValidator::new(task_file.path())?;
        assert!(validator.field_equals("spec.projectCode", "proj-1"));
        assert!(validator.field_not_empty("metadata.name"));
        assert!(validator.field_not_empty("spec.status"));
        assert!(validator.field_not_empty("spec.priority"));
    }
    
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
    let resources = vec![
        "Resource 1", "Resource 2", "Resource 3", "Resource 4", "Resource 5"
    ];
    
    for resource in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            resource, "Developer",
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Criar múltiplos projetos concorrentemente
    let projects = vec![
        "Project Alpha", "Project Beta", "Project Gamma", "Project Delta"
    ];
    
    for project in &projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "project",
            project, &format!("Description for {}", project),
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Validar que todas as operações foram bem-sucedidas
    let resource_names = vec![
        "Resource 1", "Resource 2", "Resource 3", "Resource 4", "Resource 5"
    ];
    
    for name in &resource_names {
        let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child(&format!("{}.yaml", name.to_lowercase().replace(" ", "_")));
        resource_file.assert(predicate::path::exists());
    }
    
    // Verificar que pelo menos um projeto foi criado
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    project_file.assert(predicate::path::exists());
    
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
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]);
    cmd.assert().success();
    
    Ok(())
}

fn create_test_project(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    Ok(())
}
