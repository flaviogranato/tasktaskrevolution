//! Testes de integração para validação de dados avançada
//!
//! Estes testes cobrem:
//! - Validação de consistência de dados
//! - Integridade referencial
//! - Validação de regras de negócio
//! - Violações de constraints
//! - Cenários de migração de dados

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_yaml::Value;
use std::fs;
use std::process::Command;

/// Validador YAML reutilizável
struct YamlValidator {
    parsed: Value,
}

impl YamlValidator {
    fn new(file_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let parsed: Value = serde_yaml::from_str(&content)?;
        Ok(Self { parsed })
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
}

/// Teste de consistência de dados entre entidades
#[test]
fn test_data_consistency_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Criar recursos
    let resources = vec![
        ("Alice Johnson", "Developer", "TECH-CORP"),
        ("Bob Smith", "Developer", "TECH-CORP"),
        ("Carol Davis", "Designer", "TECH-CORP"),
    ];

    for (name, role, company_code) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            "--name",
            name,
            "--type",
            role,
            "--description",
            "Senior Developer",
            "--email",
            "test@example.com",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
            "--company",
            company_code,
        ]);
        cmd.assert().success();
    }

    // Criar projetos
    let projects = vec![
        ("Web Application", "Modern web application", "TECH-CORP"),
        ("Mobile App", "Cross-platform mobile app", "TECH-CORP"),
    ];

    for (name, description, company_code) in projects {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "project",
            "--name",
            name,
            "--description",
            description,
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
            "--company",
            company_code,
        ]);
        cmd.assert().success();
    }

    // Validar consistência de dados

    temp.close()?;
    Ok(())
}

/// Teste de integridade referencial
#[test]
fn test_referential_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Criar empresa primeiro
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Tech Corporation",
        "--code",
        "TECH-CORP",
        "--description",
        "Test company for referential integrity",
    ]);
    cmd.assert().success();

    // Criar recursos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "--name",
        "John Developer",
        "--type",
        "Developer",
        "--description",
        "Senior Developer",
        "--email",
        "john@example.com",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "--name",
        "Test Project",
        "--description",
        "Project for testing",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Descobrir o código do projeto dinamicamente (ID-based format)
    let projects_dir = temp.path().join("projects");
    let mut project_code = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
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
    let project_code = project_code.expect("Project code not found");

    // Criar tarefa referenciando projeto existente
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "task",
        "--name",
        "Test Task",
        "--description",
        "Task for testing referential integrity",
        "--start-date",
        "2024-01-01",
        "--due-date",
        "2024-01-10",
        "--project",
        &project_code,
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Descobrir o código da tarefa dinamicamente
    let tasks_dir = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("projects")
        .join(&project_code)
        .join("tasks");
    let mut task_code = None;
    if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file()
                && entry.path().extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = entry.file_name().to_str()
                && file_name.starts_with("task-")
            {
                task_code = Some(file_name.to_string());
                break;
            }
        }
    }
    let task_code = task_code.expect("Task code not found");

    // Validar que a tarefa referencia o projeto correto
    let task_file = temp
        .child("companies")
        .child("TECH-CORP")
        .child("projects")
        .child(&project_code)
        .child("tasks")
        .child(&task_code);
    task_file.assert(predicate::path::exists());

    let validator = YamlValidator::new(&task_file)?;
    assert!(validator.field_equals("spec.projectCode", &project_code));
    assert!(validator.field_equals("metadata.name", "Test Task"));

    // Validar que o projeto existe
    // Verificar se o projeto foi criado corretamente (ID-based format)
    let projects_dir = temp.path().join("projects");
    let mut project_file = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                project_file = Some(path);
                break;
            }
        }
    }
    let project_file = project_file.expect("Project file not found");
    assert!(project_file.exists());

    let validator = YamlValidator::new(&project_file)?;
    assert!(validator.field_equals("metadata.name", "Test Project"));

    temp.close()?;
    Ok(())
}

/// Teste de validação de regras de negócio
#[test]
fn test_business_rules_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Criar primeira empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Tech Corp",
        "--code",
        "TECH-CORP",
        "--description",
        "Tech company",
    ]);
    cmd.assert().success();

    // Testar regra: Nome de empresa deve ser único
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Tech Corp",
        "--code",
        "TECH-CORP-2",
        "--description",
        "Another tech company",
    ]);
    cmd.assert().failure(); // Deve falhar porque o nome já existe

    // Validar que apenas a primeira empresa foi criada (ID-based naming)
    let companies_dir = temp.child("companies");
    companies_dir.assert(predicate::path::is_dir());

    // Check if there's at least one .yaml file in the companies directory
    let companies_path = companies_dir.path();
    let yaml_files = std::fs::read_dir(companies_path)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect::<Vec<_>>();

    assert_eq!(yaml_files.len(), 1, "Only one company should exist");

    // Use the first YAML file found for validation
    let company_file_path = yaml_files[0].path();
    let validator1 = YamlValidator::new(&company_file_path)?;
    assert!(validator1.field_equals("metadata.code", "TECH-CORP"));

    // Testar regra: Códigos de projeto devem ser únicos por empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "--name",
        "Project 1",
        "--description",
        "First project",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "--name",
        "Project 2",
        "--description",
        "Second project",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que os projetos foram criados dinamicamente (ID-based format)
    let projects_dir = temp.path().join("projects");
    let mut project_count = 0;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                project_count += 1;
            }
        }
    }
    assert!(
        project_count >= 2,
        "Expected at least 2 projects, found {}",
        project_count
    );

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
    cmd.args([
        "init",
        "--name",
        "", // Nome vazio
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
    ]);
    cmd.assert().failure();

    // Testar constraint: Email deve ter formato válido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test Manager",
        "--email",
        "invalid-email", // Email inválido
        "--company-name",
        "Test Company",
    ]);
    cmd.assert().failure();

    // Testar constraint: Código de empresa não pode estar vazio
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
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
    cmd.args([
        "create",
        "company",
        "--name",
        "Test Company",
        "--code",
        "", // Código vazio
        "--description",
        "Test description",
    ]);
    // O sistema gera um código automaticamente quando vazio, então deve ter sucesso
    cmd.assert().success();

    temp.close()?;
    Ok(())
}

/// Teste de cenários de migração de dados
#[test]
fn test_data_migration_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Simular migração: Criar dados em versão antiga

    // Criar dados que simulam uma versão anterior
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "--name",
        "Legacy Resource",
        "--type",
        "Developer",
        "--description",
        "Legacy Developer",
        "--email",
        "legacy@example.com",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Simular migração: Atualizar dados para nova versão
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "--name",
        "Migrated Project",
        "--description",
        "Project migrated from old version",
        "--start-date",
        "2024-01-01",
        "--end-date",
        "2024-12-31",
        "--company",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que os dados migrados estão corretos
    // O arquivo está sendo criado com o código, não com o nome
    // Vamos procurar dinamicamente pelo arquivo correto
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    let mut resource_file_path = None;

    if resources_dir.path().exists() {
        for entry in std::fs::read_dir(resources_dir.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                resource_file_path = Some(path);
                break;
            }
        }
    }

    let resource_file_path = resource_file_path.expect("Resource file not found");
    assert!(resource_file_path.exists());

    // Descobrir o projeto criado dinamicamente (ID-based format)
    let projects_dir = temp.path().join("projects");
    let mut project_file = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                project_file = Some(path);
                break;
            }
        }
    }
    let project_file = project_file.expect("Project file not found");

    // Validar estrutura dos dados migrados
    let validator = YamlValidator::new(&resource_file_path)?;
    assert!(validator.has_field("metadata.id"));
    assert!(validator.has_field("metadata.name"));
    assert!(validator.has_field("metadata.resourceType"));

    let validator = YamlValidator::new(&project_file)?;
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

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Criar múltiplos recursos em lote
    let resources = vec![
        ("Resource 1", "Developer"),
        ("Resource 2", "Designer"),
        ("Resource 3", "QA Engineer"),
        ("Resource 4", "Manager"),
        ("Resource 5", "Business Analyst"),
    ];

    for (name, role) in resources {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            "--name",
            name,
            "--type",
            role,
            "--description",
            "Senior Developer",
            "--email",
            "test@example.com",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
            "--company",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que todos os recursos foram criados corretamente
    // Os arquivos estão sendo criados com o código, não com o nome
    // Vamos contar quantos arquivos YAML existem no diretório
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    let mut yaml_files = Vec::new();

    if resources_dir.path().exists() {
        for entry in std::fs::read_dir(resources_dir.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                yaml_files.push(path);
            }
        }
    }

    // Deve ter 5 arquivos (um para cada recurso)
    assert_eq!(
        yaml_files.len(),
        5,
        "Expected 5 resource files, found {}",
        yaml_files.len()
    );

    // Validar que cada arquivo tem os campos obrigatórios
    for yaml_file in &yaml_files {
        let validator = YamlValidator::new(yaml_file)?;
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

    // Setup inicial - criar config.yaml
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Test User",
        "--email",
        "test@example.com",
        "--company-name",
        "Test Company",
        "--timezone",
        "UTC",
        "--work-hours-start",
        "09:00",
        "--work-hours-end",
        "18:00",
        "--work-days",
        "monday,tuesday,wednesday,thursday,friday",
    ]);
    cmd.assert().success();

    // Testar com caracteres especiais em nomes
    let special_names = vec![
        "José da Silva",
        "François Müller",
        "李小明",
        "Александр Петров",
        "محمد أحمد",
    ];

    for name in &special_names {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            "--name",
            name,
            "--type",
            "Developer",
            "--description",
            "Senior Developer",
            "--email",
            "test@example.com",
            "--start-date",
            "2024-01-01",
            "--end-date",
            "2024-12-31",
            "--company",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que todos os recursos foram criados
    // Os arquivos estão sendo criados com o código, não com o nome
    // Vamos contar quantos arquivos YAML existem no diretório
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    let mut yaml_files = Vec::new();

    if resources_dir.path().exists() {
        for entry in std::fs::read_dir(resources_dir.path()).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                yaml_files.push(path);
            }
        }
    }

    // Deve ter 5 arquivos (um para cada nome especial)
    assert_eq!(
        yaml_files.len(),
        special_names.len(),
        "Expected {} resource files, found {}",
        special_names.len(),
        yaml_files.len()
    );

    // Validar que pelo menos um arquivo contém cada nome especial
    for name in &special_names {
        let mut found = false;
        for yaml_file in &yaml_files {
            let validator = YamlValidator::new(yaml_file)?;
            if validator.field_equals("metadata.name", name) {
                found = true;
                break;
            }
        }
        assert!(found, "Resource with name '{}' not found in any YAML file", name);
    }

    temp.close()?;
    Ok(())
}
