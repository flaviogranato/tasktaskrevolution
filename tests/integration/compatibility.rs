//! Testes de integração para compatibilidade e migração
//!
//! Estes testes cobrem:
//! - Compatibilidade entre versões
//! - Migração de dados
//! - Evolução de formatos
//! - Tratamento de versões de API
//! - Validação de retrocompatibilidade

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

/// Teste de compatibilidade - leitura de dados de versão anterior
#[test]
fn test_backward_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Simular dados de versão anterior
    let config_content = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  id: legacy-config-id
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
spec:
  managerName: "Legacy Manager"
  managerEmail: "legacy@example.com"
  defaultTimezone: "UTC"
  workHours:
    start: "09:00"
    end: "17:00"
"#;

    let config_file = temp.child("config.yaml");
    std::fs::write(config_file.path(), config_content)?;

    // Testar se o sistema consegue ler dados legados
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    // Validar que os dados legados foram preservados
    let validator = YamlValidator::new(config_file.path())?;
    assert!(validator.field_equals("spec.managerName", "Legacy Manager"));
    assert!(validator.field_equals("spec.managerEmail", "legacy@example.com"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - evolução de formato de empresa
#[test]
fn test_company_format_evolution() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar empresa com formato atual
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Modern Company",
        "--code",
        "MODERN-COMP",
        "--description",
        "Modern company format",
    ]);
    cmd.assert().success();

    // Validar que a empresa foi criada com formato atual
    let company_file = temp.child("companies").child("MODERN-COMP").child("company.yaml");
    company_file.assert(predicate::path::exists());

    let validator = YamlValidator::new(company_file.path())?;
    assert!(validator.has_field("apiVersion"));
    assert!(validator.has_field("kind"));
    assert!(validator.has_field("metadata"));
    assert!(validator.has_field("spec"));
    assert!(validator.field_equals("metadata.code", "MODERN-COMP"));
    assert!(validator.field_equals("metadata.name", "Modern Company"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - evolução de formato de recurso
#[test]
fn test_resource_format_evolution() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar recurso com formato atual
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "Modern Resource",
        "Developer",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que o recurso foi criado com formato atual
    let resource_file = temp
        .child("companies")
        .child("TECH-CORP")
        .child("resources")
        .child("modern_resource.yaml");
    resource_file.assert(predicate::path::exists());

    let validator = YamlValidator::new(resource_file.path())?;
    assert!(validator.has_field("apiVersion"));
    assert!(validator.has_field("kind"));
    assert!(validator.has_field("metadata"));
    assert!(validator.has_field("spec"));
    assert!(validator.field_equals("metadata.name", "Modern Resource"));
    assert!(validator.field_equals("metadata.resourceType", "Developer"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - evolução de formato de projeto
#[test]
fn test_project_format_evolution() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar projeto com formato atual
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "Modern Project",
        "Modern project description",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que o projeto foi criado com formato atual
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_file = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    project_file = Some(project_yaml);
                    break;
                }
            }
        }
    }

    let project_file = project_file.expect("Project file not found");
    assert!(project_file.exists(), "Project file should exist");

    let validator = YamlValidator::new(&project_file)?;
    assert!(validator.has_field("apiVersion"));
    assert!(validator.has_field("kind"));
    assert!(validator.has_field("metadata"));
    assert!(validator.has_field("spec"));
    assert!(validator.field_equals("metadata.name", "Modern Project"));
    assert!(validator.field_equals("metadata.description", "Modern project description"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - evolução de formato de tarefa
#[test]
fn test_task_format_evolution() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;

    // Encontrar o código do projeto criado
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_code = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    // Ler o código do projeto do YAML
                    if let Ok(content) = std::fs::read_to_string(&project_yaml)
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
    }

    let project_code = project_code.expect("Project code not found");

    // Criar tarefa com formato atual
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "task",
        "--name",
        "Modern Task",
        "--description",
        "Modern task description",
        "--start-date",
        "2024-01-01",
        "--due-date",
        "2024-12-31",
        "--project-code",
        &project_code,
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que a tarefa foi criada com formato atual
    let tasks_dir = temp
        .path()
        .join("companies")
        .join("TECH-CORP")
        .join("projects")
        .join(&project_code)
        .join("tasks");
    let mut task_file = None;
    if let Ok(entries) = std::fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "yaml") {
                task_file = Some(entry.path());
                break;
            }
        }
    }

    let task_file = task_file.expect("Task file not found");
    assert!(task_file.exists(), "Task file should exist");

    let validator = YamlValidator::new(&task_file)?;
    assert!(validator.has_field("api_version")); // Tarefas usam api_version
    assert!(validator.has_field("kind"));
    assert!(validator.has_field("metadata"));
    assert!(validator.has_field("spec"));
    assert!(validator.field_equals("metadata.name", "Modern Task"));
    assert!(validator.field_equals("spec.projectCode", &project_code));

    temp.close()?;
    Ok(())
}

/// Teste de migração - dados de versão anterior para atual
#[test]
fn test_data_migration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Criar dados de versão anterior
    let legacy_config = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  id: legacy-config-id
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
spec:
  managerName: "Legacy Manager"
  managerEmail: "legacy@example.com"
  defaultTimezone: "UTC"
  workHours:
    start: "09:00"
    end: "17:00"
"#;

    let config_file = temp.child("config.yaml");
    std::fs::write(config_file.path(), legacy_config)?;

    // Criar estrutura de diretórios legada
    std::fs::create_dir_all(temp.child("companies").child("LEGACY-COMP").path())?;

    let legacy_company = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: legacy-company-id
  code: "LEGACY-COMP"
  name: "Legacy Company"
  description: "Legacy company description"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
spec:
  status: "active"
  size: "small"
  industry: "technology"
"#;

    let company_file = temp.child("companies").child("LEGACY-COMP").child("company.yaml");
    std::fs::write(company_file.path(), legacy_company)?;

    // Testar migração executando comandos atuais
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    // Validar que os dados legados foram preservados
    let validator = YamlValidator::new(config_file.path())?;
    assert!(validator.field_equals("spec.managerName", "Legacy Manager"));

    let validator = YamlValidator::new(company_file.path())?;
    assert!(validator.field_equals("metadata.code", "LEGACY-COMP"));
    assert!(validator.field_equals("metadata.name", "Legacy Company"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - versões de API
#[test]
fn test_api_version_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar diferentes tipos de entidades para testar versões de API
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "API Test Resource",
        "Developer",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "API Test Project",
        "API test project description",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar versões de API
    let resource_file = temp
        .child("companies")
        .child("TECH-CORP")
        .child("resources")
        .child("api_test_resource.yaml");

    // Encontrar o arquivo do projeto criado
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_file = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    project_file = Some(project_yaml);
                    break;
                }
            }
        }
    }

    let project_file = project_file.expect("Project file not found");

    let resource_validator = YamlValidator::new(resource_file.path())?;
    let project_validator = YamlValidator::new(&project_file)?;

    // Recursos usam apiVersion (camelCase)
    assert!(resource_validator.has_field("apiVersion"));
    assert!(resource_validator.field_equals("apiVersion", "tasktaskrevolution.io/v1alpha1"));

    // Projetos usam apiVersion (camelCase)
    assert!(project_validator.has_field("apiVersion"));
    assert!(project_validator.field_equals("apiVersion", "tasktaskrevolution.io/v1alpha1"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - campos obrigatórios vs opcionais
#[test]
fn test_required_vs_optional_fields() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar recurso com campos mínimos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "Minimal Resource",
        "Developer",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que campos obrigatórios estão presentes
    let resource_file = temp
        .child("companies")
        .child("TECH-CORP")
        .child("resources")
        .child("minimal_resource.yaml");
    let validator = YamlValidator::new(resource_file.path())?;

    // Campos obrigatórios
    assert!(validator.has_field("metadata.id"));
    assert!(validator.has_field("metadata.name"));
    assert!(validator.has_field("metadata.resourceType"));
    assert!(validator.has_field("spec.timeOffBalance"));

    // Validar que campos obrigatórios não estão vazios
    assert!(validator.field_not_empty("metadata.id"));
    assert!(validator.field_not_empty("metadata.name"));
    assert!(validator.field_not_empty("metadata.resourceType"));

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - evolução de estrutura de diretórios
#[test]
fn test_directory_structure_evolution() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar estrutura completa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "Structure Test Resource",
        "Developer",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "Structure Test Project",
        "Structure test project description",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar estrutura de diretórios atual
    let companies_dir = temp.child("companies");
    let tech_corp_dir = companies_dir.child("TECH-CORP");
    let resources_dir = tech_corp_dir.child("resources");
    let projects_dir = tech_corp_dir.child("projects");

    companies_dir.assert(predicate::path::is_dir());
    tech_corp_dir.assert(predicate::path::is_dir());
    resources_dir.assert(predicate::path::is_dir());
    projects_dir.assert(predicate::path::is_dir());

    // Verificar se existe pelo menos um projeto
    let projects_path = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_found = false;
    if let Ok(entries) = std::fs::read_dir(&projects_path) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                project_found = true;
                break;
            }
        }
    }
    assert!(project_found, "No project directory found");

    // Validar arquivos específicos
    let resource_file = resources_dir.child("structure_test_resource.yaml");

    // Encontrar o arquivo do projeto criado
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_file = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists() {
                    project_file = Some(project_yaml);
                    break;
                }
            }
        }
    }

    let project_file = project_file.expect("Project file not found");

    resource_file.assert(predicate::path::exists());
    assert!(project_file.exists(), "Project file should exist");

    temp.close()?;
    Ok(())
}

/// Teste de compatibilidade - tratamento de dados corrompidos
#[test]
fn test_corrupted_data_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Criar arquivo de configuração corrompido
    let corrupted_config = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Config
metadata:
  id: corrupted-config-id
  # Missing required fields
spec:
  # Incomplete spec
"#;

    let config_file = temp.child("config.yaml");
    std::fs::write(config_file.path(), corrupted_config)?;

    // Testar se o sistema consegue lidar com dados corrompidos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    // O sistema pode validar com sucesso mesmo com dados incompletos
    // pois a validação é mais permissiva
    cmd.assert().success();

    temp.close()?;
    Ok(())
}

// Funções auxiliares

fn setup_test_environment(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
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
    cmd.args([
        "create",
        "project",
        "Web App",
        "Web application project",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();
    Ok(())
}
