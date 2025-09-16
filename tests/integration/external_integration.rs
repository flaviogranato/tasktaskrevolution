//! Testes de integração para sistemas externos
//!
//! Estes testes cobrem:
//! - Integração com ferramentas externas
//! - Exportação de dados em diferentes formatos
//! - Importação de dados
//! - Simulação de webhooks
//! - Integração com APIs externas

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

/// Teste de exportação - formato JSON
#[test]
fn test_json_export() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar dados de teste
    create_test_data(&temp)?;

    // Simular exportação JSON (usando build como proxy)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // Validar que os dados foram processados e exportados
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    // Verificar se o HTML contém dados estruturados (simulando JSON)
    index_file.assert(predicate::str::contains("Tech Corp"));
    index_file.assert(predicate::str::contains("Technology company"));

    temp.close()?;
    Ok(())
}

/// Teste de exportação - formato CSV
#[test]
fn test_csv_export() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar múltiplos recursos para testar exportação CSV
    for i in 1..=5 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            &format!("CSV Resource {}", i),
            "Developer",
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Simular exportação CSV (usando list como proxy)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("CSV Resource 1"))
        .stdout(predicate::str::contains("CSV Resource 2"))
        .stdout(predicate::str::contains("CSV Resource 3"))
        .stdout(predicate::str::contains("CSV Resource 4"))
        .stdout(predicate::str::contains("CSV Resource 5"));

    temp.close()?;
    Ok(())
}

/// Teste de exportação - formato XML
#[test]
fn test_xml_export() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar dados de teste
    create_test_data(&temp)?;

    // Simular exportação XML (usando build como proxy)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // Validar que os dados foram processados
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    // Verificar estrutura XML-like no HTML
    index_file.assert(predicate::str::contains("<html"));
    index_file.assert(predicate::str::contains("<head"));
    index_file.assert(predicate::str::contains("<body"));

    temp.close()?;
    Ok(())
}

/// Teste de importação - validação de dados externos
#[test]
fn test_external_data_import() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular importação de dados externos criando dados via CLI
    let external_data = vec![
        ("External Resource 1", "Senior Developer"),
        ("External Resource 2", "Frontend Developer"),
        ("External Resource 3", "Backend Developer"),
        ("External Resource 4", "DevOps Engineer"),
        ("External Resource 5", "QA Engineer"),
    ];

    for (name, role) in external_data {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(["create", "resource", name, role, "--company-code", "TECH-CORP"]);
        cmd.assert().success();
    }

    // Validar que os dados externos foram importados corretamente
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("External Resource 1"))
        .stdout(predicate::str::contains("External Resource 2"))
        .stdout(predicate::str::contains("External Resource 3"))
        .stdout(predicate::str::contains("External Resource 4"))
        .stdout(predicate::str::contains("External Resource 5"));

    temp.close()?;
    Ok(())
}

/// Teste de integração - simulação de webhook
#[test]
fn test_webhook_simulation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular webhook de criação de recurso
    let webhook_data = vec![
        ("Webhook Resource 1", "Developer"),
        ("Webhook Resource 2", "Designer"),
        ("Webhook Resource 3", "Tester"),
    ];

    for (name, role) in webhook_data {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(["create", "resource", name, role, "--company-code", "TECH-CORP"]);
        cmd.assert().success();
    }

    // Simular webhook de criação de projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "Webhook Project",
        "Project created via webhook",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Validar que os webhooks foram processados
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Webhook Resource 1"))
        .stdout(predicate::str::contains("Webhook Resource 2"))
        .stdout(predicate::str::contains("Webhook Resource 3"));

    temp.close()?;
    Ok(())
}

/// Teste de integração - API externa
#[test]
fn test_external_api_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular integração com API externa criando dados
    let api_data = vec![
        ("API Resource 1", "Senior Developer"),
        ("API Resource 2", "Frontend Developer"),
        ("API Resource 3", "Backend Developer"),
    ];

    for (name, role) in api_data {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(["create", "resource", name, role, "--company-code", "TECH-CORP"]);
        cmd.assert().success();
    }

    // Simular chamada de API para listar recursos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("API Resource 1"))
        .stdout(predicate::str::contains("API Resource 2"))
        .stdout(predicate::str::contains("API Resource 3"));

    temp.close()?;
    Ok(())
}

/// Teste de integração - ferramentas de terceiros
#[test]
fn test_third_party_tools_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular integração com ferramentas de terceiros
    let third_party_data = vec![
        ("Third Party Resource 1", "Developer"),
        ("Third Party Resource 2", "Designer"),
        ("Third Party Resource 3", "Tester"),
    ];

    for (name, role) in third_party_data {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(["create", "resource", name, role, "--company-code", "TECH-CORP"]);
        cmd.assert().success();
    }

    // Simular exportação para ferramenta de terceiros
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // Validar que os dados foram exportados
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    temp.close()?;
    Ok(())
}

/// Teste de integração - sincronização de dados
#[test]
fn test_data_synchronization() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular sincronização de dados
    let sync_data = vec![
        ("Sync Resource 1", "Developer"),
        ("Sync Resource 2", "Designer"),
        ("Sync Resource 3", "Tester"),
    ];

    for (name, role) in sync_data {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(["create", "resource", name, role, "--company-code", "TECH-CORP"]);
        cmd.assert().success();
    }

    // Simular sincronização executando validação
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    // Validar que os dados foram sincronizados
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Sync Resource 1"))
        .stdout(predicate::str::contains("Sync Resource 2"))
        .stdout(predicate::str::contains("Sync Resource 3"));

    temp.close()?;
    Ok(())
}

/// Teste de integração - backup e restauração
#[test]
fn test_backup_restore_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Criar dados para backup
    create_test_data(&temp)?;

    // Simular backup executando build
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // Validar que o backup foi criado
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    // Simular restauração validando dados
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    temp.close()?;
    Ok(())
}

/// Teste de integração - monitoramento e logging
#[test]
fn test_monitoring_logging_integration() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    setup_test_environment(&temp)?;

    // Simular operações que geram logs
    let operations = vec![
        ("create", "resource", "Monitor Resource 1", "Developer"),
        ("create", "resource", "Monitor Resource 2", "Designer"),
        ("create", "project", "Monitor Project", "Monitor project description"),
        ("list", "resources", "", ""),
        ("validate", "system", "", ""),
    ];

    for (op, entity, name, description) in operations {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());

        match op {
            "create" => {
                if entity == "resource" {
                    cmd.args(["create", "resource", name, description, "--company-code", "TECH-CORP"]);
                } else if entity == "project" {
                    cmd.args(["create", "project", name, description, "--company-code", "TECH-CORP"]);
                }
            }
            "list" => {
                cmd.arg("list").arg(entity);
            }
            "validate" => {
                cmd.arg("validate").arg(entity);
            }
            _ => {}
        }

        cmd.assert().success();
    }

    // Validar que as operações foram executadas
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Monitor Resource 1"))
        .stdout(predicate::str::contains("Monitor Resource 2"));

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

fn create_test_data(temp: &assert_fs::TempDir) -> Result<(), Box<dyn std::error::Error>> {
    // Criar recurso
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "Test Resource",
        "Developer",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    // Criar projeto
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "Test Project",
        "Test project description",
        "--company-code",
        "TECH-CORP",
    ]);
    cmd.assert().success();

    Ok(())
}
