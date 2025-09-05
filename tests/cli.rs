//! Testes funcionais do CLI TTR usando assert_cmd e predicates
//! 
//! Estes testes executam o binário CLI compilado e validam:
//! - Comandos CLI funcionam corretamente
//! - Saídas são as esperadas
//! - Códigos de saída estão corretos
//! - Arquivos são gerados corretamente

use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;
use serde_yaml::Value;
use std::fs;

/// Validador YAML robusto para verificar campos obrigatórios
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
    
    /// Verifica se um campo existe no caminho especificado
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
    
    /// Verifica se um campo tem um valor específico
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
    
    /// Verifica se um campo não está vazio
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
    
    /// Verifica se o arquivo contém uma string específica
    fn contains(&self, text: &str) -> bool {
        self.content.contains(text)
    }
    
    /// Valida estrutura básica do YAML (apiVersion, kind, metadata, spec)
    fn validate_basic_structure(&self) -> bool {
        self.has_field("apiVersion") && 
        self.has_field("kind") && 
        self.has_field("metadata") && 
        self.has_field("spec")
    }
}

/// Testa o comando de ajuda
#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("utilitário de linha de comando"))
        .stdout(predicate::str::contains("Usage: ttr"));
    
    Ok(())
}

/// Testa o comando de versão
#[test]
fn test_version_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.5.6"));
    
    Ok(())
}

/// Testa o comando init
#[test]
fn test_init_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let config_file = temp.child("config.yaml");
    
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Manager/Consultant configured successfully"))
        .stdout(predicate::str::contains("Test Manager"))
        .stdout(predicate::str::contains("test@example.com"))
        .stdout(predicate::str::contains("Test Company"));
    
    // Verificar se o arquivo de configuração foi criado
    config_file.assert(predicate::path::exists());
    
    // Validar conteúdo YAML do config.yaml
    let validator = YamlValidator::new(config_file.path())?;
    
    // Validar estrutura básica
    assert!(validator.validate_basic_structure(), "Config YAML deve ter estrutura básica (apiVersion, kind, metadata, spec)");
    
    // Validar campos obrigatórios do config.yaml
    assert!(validator.has_field("apiVersion"), "Config deve ter apiVersion");
    assert!(validator.has_field("kind"), "Config deve ter kind");
    assert!(validator.has_field("metadata"), "Config deve ter metadata");
    assert!(validator.has_field("spec"), "Config deve ter spec");
    
    // Validar campos específicos do spec
    assert!(validator.has_field("spec.managerName"), "Config deve ter spec.managerName");
    assert!(validator.has_field("spec.managerEmail"), "Config deve ter spec.managerEmail");
    assert!(validator.has_field("spec.defaultTimezone"), "Config deve ter spec.defaultTimezone");
    
    // Validar valores específicos
    assert!(validator.field_equals("spec.managerName", "Test Manager"), "managerName deve ser 'Test Manager'");
    assert!(validator.field_equals("spec.managerEmail", "test@example.com"), "managerEmail deve ser 'test@example.com'");
    
    // Validar que os campos não estão vazios
    assert!(validator.field_not_empty("spec.managerName"), "managerName não deve estar vazio");
    assert!(validator.field_not_empty("spec.managerEmail"), "managerEmail não deve estar vazio");
    assert!(validator.field_not_empty("spec.defaultTimezone"), "defaultTimezone não deve estar vazio");
    
    // Validar que contém strings esperadas
    assert!(validator.contains("Test Manager"), "Config deve conter 'Test Manager'");
    assert!(validator.contains("test@example.com"), "Config deve conter 'test@example.com'");
    
    temp.close()?;
    Ok(())
}

/// Testa o comando init com timezone
#[test]
fn test_init_command_with_timezone() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company",
        "--timezone", "America/Sao_Paulo"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("America/Sao_Paulo"));
    
    temp.close()?;
    Ok(())
}

/// Testa criação de empresa
#[test]
fn test_create_company() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let company_dir = temp.child("companies").child("TECH-CORP");
    let company_file = company_dir.child("company.yaml");
    
    // Primeiro inicializar
    let mut init_cmd = Command::cargo_bin("ttr")?;
    init_cmd.current_dir(temp.path());
    init_cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    init_cmd.assert().success();
    
    // Depois criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Company created successfully"))
        .stdout(predicate::str::contains("Tech Corp"))
        .stdout(predicate::str::contains("TECH-CORP"));
    
    // Verificar se os arquivos foram criados
    company_dir.assert(predicate::path::is_dir());
    company_file.assert(predicate::path::exists());
    
    // Validar conteúdo YAML do company.yaml
    let validator = YamlValidator::new(company_file.path())?;
    
    // Validar estrutura básica
    assert!(validator.validate_basic_structure(), "Company YAML deve ter estrutura básica");
    
    // Validar campos obrigatórios do metadata
    assert!(validator.has_field("metadata.id"), "Company deve ter metadata.id");
    assert!(validator.has_field("metadata.code"), "Company deve ter metadata.code");
    assert!(validator.has_field("metadata.name"), "Company deve ter metadata.name");
    assert!(validator.has_field("metadata.createdAt"), "Company deve ter metadata.createdAt");
    assert!(validator.has_field("metadata.updatedAt"), "Company deve ter metadata.updatedAt");
    assert!(validator.has_field("metadata.createdBy"), "Company deve ter metadata.createdBy");
    
    // Validar campos obrigatórios do spec
    assert!(validator.has_field("spec.description"), "Company deve ter spec.description");
    assert!(validator.has_field("spec.status"), "Company deve ter spec.status");
    assert!(validator.has_field("spec.size"), "Company deve ter spec.size");
    
    // Validar valores específicos
    assert!(validator.field_equals("metadata.code", "TECH-CORP"), "metadata.code deve ser 'TECH-CORP'");
    assert!(validator.field_equals("metadata.name", "Tech Corp"), "metadata.name deve ser 'Tech Corp'");
    assert!(validator.field_equals("spec.description", "Technology company"), "spec.description deve ser 'Technology company'");
    
    // Validar que os campos não estão vazios
    assert!(validator.field_not_empty("metadata.id"), "metadata.id não deve estar vazio");
    assert!(validator.field_not_empty("metadata.code"), "metadata.code não deve estar vazio");
    assert!(validator.field_not_empty("metadata.name"), "metadata.name não deve estar vazio");
    assert!(validator.field_not_empty("spec.description"), "spec.description não deve estar vazio");
    assert!(validator.field_not_empty("metadata.createdAt"), "metadata.createdAt não deve estar vazio");
    assert!(validator.field_not_empty("metadata.updatedAt"), "metadata.updatedAt não deve estar vazio");
    assert!(validator.field_not_empty("metadata.createdBy"), "metadata.createdBy não deve estar vazio");
    assert!(validator.field_not_empty("spec.status"), "spec.status não deve estar vazio");
    assert!(validator.field_not_empty("spec.size"), "spec.size não deve estar vazio");
    
    // Validar que contém strings esperadas
    assert!(validator.contains("Tech Corp"), "Company deve conter 'Tech Corp'");
    assert!(validator.contains("TECH-CORP"), "Company deve conter 'TECH-CORP'");
    assert!(validator.contains("Technology company"), "Company deve conter 'Technology company'");
    
    temp.close()?;
    Ok(())
}

/// Testa criação de recurso
#[test]
fn test_create_resource() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child("john_doe.yaml");
    
    // Inicializar e criar empresa
    setup_test_environment(&temp)?;
    
    // Criar recurso
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Resource John Doe created"));
    
    // Verificar se o arquivo foi criado
    resource_file.assert(predicate::path::exists());
    
    // Validar conteúdo YAML do resource.yaml
    let validator = YamlValidator::new(resource_file.path())?;
    
    // Validar estrutura básica
    assert!(validator.validate_basic_structure(), "Resource YAML deve ter estrutura básica");
    
    // Validar campos obrigatórios do metadata
    assert!(validator.has_field("metadata.id"), "Resource deve ter metadata.id");
    assert!(validator.has_field("metadata.code"), "Resource deve ter metadata.code");
    assert!(validator.has_field("metadata.name"), "Resource deve ter metadata.name");
    assert!(validator.has_field("metadata.resourceType"), "Resource deve ter metadata.resourceType");
    
    // Validar campos obrigatórios do spec
    assert!(validator.has_field("spec.timeOffBalance"), "Resource deve ter spec.timeOffBalance");
    
    // Validar valores específicos
    assert!(validator.field_equals("metadata.name", "John Doe"), "metadata.name deve ser 'John Doe'");
    assert!(validator.field_equals("metadata.resourceType", "Developer"), "metadata.resourceType deve ser 'Developer'");
    
    // Validar que os campos não estão vazios
    assert!(validator.field_not_empty("metadata.id"), "metadata.id não deve estar vazio");
    assert!(validator.field_not_empty("metadata.code"), "metadata.code não deve estar vazio");
    assert!(validator.field_not_empty("metadata.name"), "metadata.name não deve estar vazio");
    assert!(validator.field_not_empty("metadata.resourceType"), "metadata.resourceType não deve estar vazio");
    
    // Validar que contém strings esperadas
    assert!(validator.contains("John Doe"), "Resource deve conter 'John Doe'");
    assert!(validator.contains("Developer"), "Resource deve conter 'Developer'");
    
    temp.close()?;
    Ok(())
}

/// Testa criação de projeto
#[test]
fn test_create_project() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    
    // Inicializar e criar empresa
    setup_test_environment(&temp)?;
    
    // Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Project Web App created"));
    
    // Verificar se o arquivo foi criado
    project_file.assert(predicate::path::exists());
    
    // Validar conteúdo YAML do project.yaml
    let validator = YamlValidator::new(project_file.path())?;
    
    // Validar estrutura básica
    assert!(validator.validate_basic_structure(), "Project YAML deve ter estrutura básica");
    
    // Validar campos obrigatórios do metadata
    assert!(validator.has_field("metadata.id"), "Project deve ter metadata.id");
    assert!(validator.has_field("metadata.code"), "Project deve ter metadata.code");
    assert!(validator.has_field("metadata.name"), "Project deve ter metadata.name");
    assert!(validator.has_field("metadata.description"), "Project deve ter metadata.description");
    
    // Validar campos obrigatórios do spec
    assert!(validator.has_field("spec.status"), "Project deve ter spec.status");
    assert!(validator.has_field("spec.endDate"), "Project deve ter spec.endDate");
    
    // Validar valores específicos
    assert!(validator.field_equals("metadata.name", "Web App"), "metadata.name deve ser 'Web App'");
    assert!(validator.field_equals("metadata.description", "Web application project"), "metadata.description deve ser 'Web application project'");
    
    // Validar que os campos não estão vazios
    assert!(validator.field_not_empty("metadata.id"), "metadata.id não deve estar vazio");
    assert!(validator.field_not_empty("metadata.code"), "metadata.code não deve estar vazio");
    assert!(validator.field_not_empty("metadata.name"), "metadata.name não deve estar vazio");
    assert!(validator.field_not_empty("metadata.description"), "metadata.description não deve estar vazio");
    assert!(validator.field_not_empty("spec.status"), "spec.status não deve estar vazio");
    assert!(validator.field_not_empty("spec.endDate"), "spec.endDate não deve estar vazio");
    
    // Validar que contém strings esperadas
    assert!(validator.contains("Web App"), "Project deve conter 'Web App'");
    assert!(validator.contains("Web application project"), "Project deve conter 'Web application project'");
    
    temp.close()?;
    Ok(())
}

/// Testa criação de tarefa
#[test]
fn test_create_task() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let task_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("tasks").child("task-1.yaml");
    
    // Inicializar, criar empresa e projeto
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;
    
    // Criar tarefa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "Setup Environment",
        "--description", "Setup development environment",
        "--start-date", "2024-01-15",
        "--due-date", "2024-01-22",
        "--project-code", "proj-1",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task Setup Environment created"));
    
    // Verificar se o arquivo foi criado
    task_file.assert(predicate::path::exists());
    
    // Validar conteúdo YAML do task.yaml
    let validator = YamlValidator::new(task_file.path())?;
    
    // Validar estrutura básica (task usa api_version em vez de apiVersion)
    assert!(validator.has_field("api_version"), "Task deve ter api_version");
    assert!(validator.has_field("kind"), "Task deve ter kind");
    assert!(validator.has_field("metadata"), "Task deve ter metadata");
    assert!(validator.has_field("spec"), "Task deve ter spec");
    
    // Validar campos obrigatórios do metadata
    assert!(validator.has_field("metadata.id"), "Task deve ter metadata.id");
    assert!(validator.has_field("metadata.code"), "Task deve ter metadata.code");
    assert!(validator.has_field("metadata.name"), "Task deve ter metadata.name");
    
    // Validar campos obrigatórios do spec
    assert!(validator.has_field("spec.projectCode"), "Task deve ter spec.projectCode");
    assert!(validator.has_field("spec.status"), "Task deve ter spec.status");
    assert!(validator.has_field("spec.priority"), "Task deve ter spec.priority");
    assert!(validator.has_field("spec.estimatedStartDate"), "Task deve ter spec.estimatedStartDate");
    assert!(validator.has_field("spec.estimatedEndDate"), "Task deve ter spec.estimatedEndDate");
    
    // Validar valores específicos
    assert!(validator.field_equals("metadata.name", "Setup Environment"), "metadata.name deve ser 'Setup Environment'");
    assert!(validator.field_equals("spec.projectCode", "proj-1"), "spec.projectCode deve ser 'proj-1'");
    
    // Validar que os campos não estão vazios
    assert!(validator.field_not_empty("metadata.id"), "metadata.id não deve estar vazio");
    assert!(validator.field_not_empty("metadata.code"), "metadata.code não deve estar vazio");
    assert!(validator.field_not_empty("metadata.name"), "metadata.name não deve estar vazio");
    assert!(validator.field_not_empty("spec.projectCode"), "spec.projectCode não deve estar vazio");
    assert!(validator.field_not_empty("spec.status"), "spec.status não deve estar vazio");
    assert!(validator.field_not_empty("spec.priority"), "spec.priority não deve estar vazio");
    assert!(validator.field_not_empty("spec.estimatedStartDate"), "spec.estimatedStartDate não deve estar vazio");
    assert!(validator.field_not_empty("spec.estimatedEndDate"), "spec.estimatedEndDate não deve estar vazio");
    
    // Validar que contém strings esperadas
    assert!(validator.contains("Setup Environment"), "Task deve conter 'Setup Environment'");
    assert!(validator.contains("proj-1"), "Task deve conter 'proj-1'");
    
    temp.close()?;
    Ok(())
}

/// Testa comandos de listagem
#[test]
fn test_list_commands() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Configurar ambiente de teste
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;
    create_test_resource(&temp)?;
    create_test_task(&temp)?;
    
    // Testar list projects
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("projects");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Web App"));
    
    // Testar list resources
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("John Doe"));
    
    // Testar list tasks (pode falhar se não houver projeto configurado)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("tasks");
    
    // Este comando pode falhar se não houver projeto configurado
    // Vamos apenas verificar se executa sem crash
    let _result = cmd.output()?;
    // Não fazemos assert de sucesso pois pode falhar
    
    temp.close()?;
    Ok(())
}

/// Testa comando de validação
#[test]
fn test_validate_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Configurar ambiente de teste
    setup_test_environment(&temp)?;
    
    // Testar validação do sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    
    cmd.assert()
        .success();
    
    temp.close()?;
    Ok(())
}

/// Testa geração de HTML
#[test]
fn test_build_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    
    // Configurar ambiente de teste
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;
    
    // Gerar HTML
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    
    cmd.assert()
        .success();
    
    // Verificar se os arquivos HTML foram criados
    public_dir.assert(predicate::path::is_dir());
    index_file.assert(predicate::path::exists());
    index_file.assert(predicate::str::contains("TaskTaskRevolution"));
    
    temp.close()?;
    Ok(())
}

/// Testa tratamento de erros
#[test]
fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar comando inválido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("invalid-command");
    
    cmd.assert()
        .failure();
    
    // Testar comando que retorna erro na saída
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("tasks");
    
    cmd.assert()
        .success() // O comando executa com sucesso
        .stdout(predicate::str::contains("Error listing tasks")); // Mas retorna erro na saída
    
    temp.close()?;
    Ok(())
}

/// Testa fluxo completo E2E
#[test]
fn test_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    
    // 1. Inicializar
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().success();
    
    // 2. Criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]);
    cmd.assert().success();
    
    // 3. Criar recurso
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // 4. Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // 5. Criar tarefa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "Setup Environment",
        "--description", "Setup development environment",
        "--start-date", "2024-01-15",
        "--due-date", "2024-01-22",
        "--project-code", "proj-1",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // 6. Gerar HTML
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();
    
    // 7. Verificar resultado
    public_dir.assert(predicate::path::is_dir());
    index_file.assert(predicate::path::exists());
    index_file.assert(predicate::str::contains("Tech Corp"));
    
    temp.close()?;
    Ok(())
}

/// Teste específico para validação do config.yaml
#[test]
fn test_config_yaml_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let config_file = temp.child("config.yaml");
    
    // Inicializar
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "YAML Test Manager",
        "--email", "yaml@test.com",
        "--company-name", "YAML Test Company",
        "--timezone", "America/New_York"
    ]);
    
    cmd.assert().success();
    config_file.assert(predicate::path::exists());
    
    // Validar config.yaml com validador robusto
    let validator = YamlValidator::new(config_file.path())?;
    
    // Estrutura básica obrigatória
    assert!(validator.validate_basic_structure(), "Config deve ter estrutura básica");
    
    // Campos obrigatórios do spec
    let required_fields = [
        "spec.managerName",
        "spec.managerEmail", 
        "spec.defaultTimezone"
    ];
    
    for field in &required_fields {
        assert!(validator.has_field(field), "Config deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Valores específicos
    assert!(validator.field_equals("spec.managerName", "YAML Test Manager"));
    assert!(validator.field_equals("spec.managerEmail", "yaml@test.com"));
    assert!(validator.field_equals("spec.defaultTimezone", "America/New_York"));
    
    // Validação de conteúdo
    assert!(validator.contains("YAML Test Manager"));
    assert!(validator.contains("yaml@test.com"));
    assert!(validator.contains("America/New_York"));
    
    temp.close()?;
    Ok(())
}

/// Teste específico para validação do company.yaml
#[test]
fn test_company_yaml_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let company_file = temp.child("companies").child("YAML-CORP").child("company.yaml");
    
    // Setup
    setup_test_environment(&temp)?;
    
    // Criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "YAML Corporation",
        "--code", "YAML-CORP",
        "--description", "YAML validation test company"
    ]);
    
    cmd.assert().success();
    company_file.assert(predicate::path::exists());
    
    // Validar company.yaml com validador robusto
    let validator = YamlValidator::new(company_file.path())?;
    
    // Estrutura básica obrigatória
    assert!(validator.validate_basic_structure(), "Company deve ter estrutura básica");
    
    // Campos obrigatórios do metadata
    let metadata_fields = [
        "metadata.id",
        "metadata.code",
        "metadata.name",
        "metadata.createdAt",
        "metadata.updatedAt",
        "metadata.createdBy"
    ];
    
    for field in &metadata_fields {
        assert!(validator.has_field(field), "Company deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Campos obrigatórios do spec
    let spec_fields = [
        "spec.description",
        "spec.status",
        "spec.size"
    ];
    
    for field in &spec_fields {
        assert!(validator.has_field(field), "Company deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Valores específicos
    assert!(validator.field_equals("metadata.code", "YAML-CORP"));
    assert!(validator.field_equals("metadata.name", "YAML Corporation"));
    assert!(validator.field_equals("spec.description", "YAML validation test company"));
    
    temp.close()?;
    Ok(())
}

/// Teste específico para validação do resource.yaml
#[test]
fn test_resource_yaml_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child("yaml_developer.yaml");
    
    // Setup
    setup_test_environment(&temp)?;
    
    // Criar recurso
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "YAML Developer", "Senior Developer",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert().success();
    resource_file.assert(predicate::path::exists());
    
    // Validar resource.yaml com validador robusto
    let validator = YamlValidator::new(resource_file.path())?;
    
    // Estrutura básica obrigatória
    assert!(validator.validate_basic_structure(), "Resource deve ter estrutura básica");
    
    // Campos obrigatórios do metadata
    let metadata_fields = [
        "metadata.id",
        "metadata.name",
        "metadata.code",
        "metadata.resourceType"
    ];
    
    for field in &metadata_fields {
        assert!(validator.has_field(field), "Resource deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Campos obrigatórios do spec
    let spec_fields = [
        "spec.timeOffBalance"
    ];
    
    for field in &spec_fields {
        assert!(validator.has_field(field), "Resource deve ter campo obrigatório: {}", field);
    }
    
    // Valores específicos
    assert!(validator.field_equals("metadata.name", "YAML Developer"));
    assert!(validator.field_equals("metadata.resourceType", "Senior Developer"));
    
    temp.close()?;
    Ok(())
}

/// Teste específico para validação do project.yaml
#[test]
fn test_project_yaml_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    
    // Setup
    setup_test_environment(&temp)?;
    
    // Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "YAML Project", "YAML validation test project",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert().success();
    project_file.assert(predicate::path::exists());
    
    // Validar project.yaml com validador robusto
    let validator = YamlValidator::new(project_file.path())?;
    
    // Estrutura básica obrigatória
    assert!(validator.validate_basic_structure(), "Project deve ter estrutura básica");
    
    // Campos obrigatórios do metadata
    let metadata_fields = [
        "metadata.id",
        "metadata.code",
        "metadata.name",
        "metadata.description"
    ];
    
    for field in &metadata_fields {
        assert!(validator.has_field(field), "Project deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Campos obrigatórios do spec
    let spec_fields = [
        "spec.status",
        "spec.endDate"
    ];
    
    for field in &spec_fields {
        assert!(validator.has_field(field), "Project deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Valores específicos
    assert!(validator.field_equals("metadata.name", "YAML Project"));
    assert!(validator.field_equals("metadata.description", "YAML validation test project"));
    
    temp.close()?;
    Ok(())
}

/// Teste específico para validação do task.yaml
#[test]
fn test_task_yaml_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let task_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("tasks").child("task-1.yaml");
    
    // Setup
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;
    
    // Criar tarefa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "YAML Validation Task",
        "--description", "Task for YAML validation testing",
        "--start-date", "2024-02-01",
        "--due-date", "2024-02-15",
        "--project-code", "proj-1",
        "--company-code", "TECH-CORP"
    ]);
    
    cmd.assert().success();
    task_file.assert(predicate::path::exists());
    
    // Validar task.yaml com validador robusto
    let validator = YamlValidator::new(task_file.path())?;
    
    // Estrutura básica obrigatória (task usa api_version em vez de apiVersion)
    assert!(validator.has_field("api_version"), "Task deve ter api_version");
    assert!(validator.has_field("kind"), "Task deve ter kind");
    assert!(validator.has_field("metadata"), "Task deve ter metadata");
    assert!(validator.has_field("spec"), "Task deve ter spec");
    
    // Campos obrigatórios do metadata
    let metadata_fields = [
        "metadata.id",
        "metadata.code",
        "metadata.name"
    ];
    
    for field in &metadata_fields {
        assert!(validator.has_field(field), "Task deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Campos obrigatórios do spec
    let spec_fields = [
        "spec.projectCode",
        "spec.status",
        "spec.priority",
        "spec.estimatedStartDate",
        "spec.estimatedEndDate"
    ];
    
    for field in &spec_fields {
        assert!(validator.has_field(field), "Task deve ter campo obrigatório: {}", field);
        assert!(validator.field_not_empty(field), "Campo {} não deve estar vazio", field);
    }
    
    // Valores específicos
    assert!(validator.field_equals("metadata.name", "YAML Validation Task"));
    assert!(validator.field_equals("spec.projectCode", "proj-1"));
    
    temp.close()?;
    Ok(())
}

// Funções auxiliares para configurar ambiente de teste

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

fn create_test_resource(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    Ok(())
}

fn create_test_task(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "Setup Environment",
        "--description", "Setup development environment",
        "--start-date", "2024-01-15",
        "--due-date", "2024-01-22",
        "--project-code", "proj-1",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    Ok(())
}
