//! Testes de integração para validação de dados avançada
//! 
//! Estes testes cobrem:
//! - Validação de consistência de dados
//! - Integridade referencial
//! - Validação de regras de negócio
//! - Violações de constraints
//! - Cenários de migração de dados

use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;
use serde_yaml::Value;
use std::fs;

/// Validador YAML reutilizável
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
    
    fn get_field_value(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;
        
        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        
        if let Some(str_value) = current.as_str() {
            Some(str_value.to_string())
        } else {
            None
        }
    }
}

/// Teste de consistência de dados entre entidades
#[test]
fn test_data_consistency_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Criar recursos
    let resources = vec![
        ("Alice Johnson", "Senior Developer", "TECH-CORP"),
        ("Bob Smith", "Frontend Developer", "TECH-CORP"),
        ("Carol Davis", "UI/UX Designer", "TECH-CORP")
    ];
    
    for (name, role, company_code) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            name, role,
            "--company-code", company_code
        ]);
        cmd.assert().success();
    }
    
    // Criar projetos
    let projects = vec![
        ("Web Application", "Modern web application", "TECH-CORP"),
        ("Mobile App", "Cross-platform mobile app", "TECH-CORP")
    ];
    
    for (name, description, company_code) in projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "project",
            name, description,
            "--company-code", company_code
        ]);
        cmd.assert().success();
    }
    
    // Validar consistência de dados
    validate_data_consistency(&temp)?;
    
    temp.close()?;
    Ok(())
}

/// Teste de integridade referencial
#[test]
fn test_referential_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Criar recursos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "John Developer", "Senior Developer",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Test Project", "Project for testing",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Criar tarefa referenciando projeto existente
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "Test Task",
        "--description", "Task for testing referential integrity",
        "--start-date", "2024-01-01",
        "--due-date", "2024-01-10",
        "--project-code", "proj-1",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Validar que a tarefa referencia o projeto correto
    let task_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("tasks").child("task-1.yaml");
    task_file.assert(predicate::path::exists());
    
    let validator = YamlValidator::new(task_file.path())?;
    assert!(validator.field_equals("spec.projectCode", "proj-1"));
    assert!(validator.field_equals("metadata.name", "Test Task"));
    
    // Validar que o projeto existe
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    project_file.assert(predicate::path::exists());
    
    let validator = YamlValidator::new(project_file.path())?;
    assert!(validator.field_equals("metadata.name", "Test Project"));
    
    temp.close()?;
    Ok(())
}

/// Teste de validação de regras de negócio
#[test]
fn test_business_rules_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Testar regra: Nome de empresa deve ser único
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP-2",
        "--description", "Another tech company"
    ]);
    cmd.assert().success();
    
    // Validar que ambas as empresas foram criadas com códigos únicos
    let company1_file = temp.child("companies").child("TECH-CORP").child("company.yaml");
    let company2_file = temp.child("companies").child("TECH-CORP-2").child("company.yaml");
    
    company1_file.assert(predicate::path::exists());
    company2_file.assert(predicate::path::exists());
    
    let validator1 = YamlValidator::new(company1_file.path())?;
    let validator2 = YamlValidator::new(company2_file.path())?;
    
    assert!(validator1.field_equals("metadata.code", "TECH-CORP"));
    assert!(validator2.field_equals("metadata.code", "TECH-CORP-2"));
    
    // Testar regra: Códigos de projeto devem ser únicos por empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Project 1", "First project",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Project 2", "Second project",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Validar que ambos os projetos foram criados
    let project1_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    let project2_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-2").child("project.yaml");
    
    project1_file.assert(predicate::path::exists());
    project2_file.assert(predicate::path::exists());
    
    temp.close()?;
    Ok(())
}

/// Teste de violações de constraints
#[test]
fn test_constraint_violations() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar constraint: Nome não pode estar vazio
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "",  // Nome vazio
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    // Testar constraint: Email deve ter formato válido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "invalid-email",  // Email inválido
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    // Testar constraint: Código de empresa não pode estar vazio
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
        "--name", "Test Company",
        "--code", "",  // Código vazio
        "--description", "Test description"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de cenários de migração de dados
#[test]
fn test_data_migration_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Simular migração: Criar dados em versão antiga
    setup_test_environment(&temp)?;
    
    // Criar dados que simulam uma versão anterior
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "Legacy Resource", "Legacy Developer",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Simular migração: Atualizar dados para nova versão
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Migrated Project", "Project migrated from old version",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().success();
    
    // Validar que os dados migrados estão corretos
    let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child("legacy_resource.yaml");
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    
    resource_file.assert(predicate::path::exists());
    project_file.assert(predicate::path::exists());
    
    // Validar estrutura dos dados migrados
    let validator = YamlValidator::new(resource_file.path())?;
    assert!(validator.has_field("metadata.id"));
    assert!(validator.has_field("metadata.name"));
    assert!(validator.has_field("metadata.resourceType"));
    
    let validator = YamlValidator::new(project_file.path())?;
    assert!(validator.has_field("metadata.id"));
    assert!(validator.has_field("metadata.name"));
    assert!(validator.has_field("metadata.description"));
    
    temp.close()?;
    Ok(())
}

/// Teste de validação de dados em lote
#[test]
fn test_batch_data_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Criar múltiplos recursos em lote
    let resources = vec![
        ("Resource 1", "Developer"),
        ("Resource 2", "Designer"),
        ("Resource 3", "Tester"),
        ("Resource 4", "Manager"),
        ("Resource 5", "Analyst")
    ];
    
    for (name, role) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            name, role,
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Validar que todos os recursos foram criados corretamente
    for i in 1..=5 {
        let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child(&format!("resource_{}.yaml", i));
        resource_file.assert(predicate::path::exists());
        
        let validator = YamlValidator::new(resource_file.path())?;
        assert!(validator.field_not_empty("metadata.id"));
        assert!(validator.field_not_empty("metadata.name"));
        assert!(validator.field_not_empty("metadata.resourceType"));
    }
    
    // Validar integridade geral do sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();
    
    temp.close()?;
    Ok(())
}

/// Teste de validação de dados com caracteres especiais
#[test]
fn test_special_characters_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Testar com caracteres especiais em nomes
    let special_names = vec![
        "José da Silva",
        "François Müller",
        "李小明",
        "Александр Петров",
        "محمد أحمد"
    ];
    
    for name in special_names {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            name, "Developer",
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Validar que todos os recursos foram criados
    for (i, name) in special_names.iter().enumerate() {
        let resource_file = temp.child("companies").child("TECH-CORP").child("resources").child(&format!("resource_{}.yaml", i + 1));
        resource_file.assert(predicate::path::exists());
        
        let validator = YamlValidator::new(resource_file.path())?;
        assert!(validator.field_equals("metadata.name", name));
    }
    
    temp.close()?;
    Ok(())
}

// Funções auxiliares

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

fn validate_data_consistency(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    // Validar que todos os recursos pertencem à empresa correta
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    resources_dir.assert(predicate::path::is_dir());
    
    // Validar que todos os projetos pertencem à empresa correta
    let projects_dir = temp.child("companies").child("TECH-CORP").child("projects");
    projects_dir.assert(predicate::path::is_dir());
    
    // Validar que as datas são consistentes
    let config_file = temp.child("config.yaml");
    let validator = YamlValidator::new(config_file.path())?;
    assert!(validator.field_not_empty("metadata.createdAt"));
    
    Ok(())
}
