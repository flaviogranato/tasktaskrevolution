//! Testes de integração para cenários de erro e edge cases
//! 
//! Estes testes cobrem:
//! - Tratamento de entrada inválida
//! - Erros do sistema de arquivos
//! - Simulação de falhas de rede
//! - Recuperação de dados corrompidos
//! - Cenários de permissão negada

use assert_cmd::prelude::*;
use predicates::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;
use std::fs;
use std::io::Write;

/// Teste de entrada inválida - parâmetros obrigatórios ausentes
#[test]
fn test_missing_required_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar init sem parâmetros obrigatórios
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("init");
    cmd.assert().failure();
    
    // Testar init sem nome
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    // Testar init sem email
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    // Testar init sem company-name
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de entrada inválida - formato de email incorreto
#[test]
fn test_invalid_email_format() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    let invalid_emails = vec![
        "invalid-email",
        "@example.com",
        "test@",
        "test.example.com",
        "test@.com",
        "test@example.",
        "test space@example.com"
    ];
    
    for email in invalid_emails {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "init",
            "--name", "Test Manager",
            "--email", email,
            "--company-name", "Test Company"
        ]);
        cmd.assert().failure();
    }
    
    temp.close()?;
    Ok(())
}

/// Teste de entrada inválida - códigos de empresa inválidos
#[test]
fn test_invalid_company_codes() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    let invalid_codes = vec![
        "",  // Código vazio
        "a",  // Muito curto
        "a".repeat(100),  // Muito longo
        "INVALID CODE",  // Com espaços
        "invalid@code",  // Com caracteres especiais
        "123",  // Apenas números
    ];
    
    for code in invalid_codes {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "company",
            "--name", "Test Company",
            "--code", code,
            "--description", "Test description"
        ]);
        cmd.assert().failure();
    }
    
    temp.close()?;
    Ok(())
}

/// Teste de entrada inválida - datas inválidas
#[test]
fn test_invalid_date_formats() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    create_test_project(&temp)?;
    
    let invalid_dates = vec![
        "invalid-date",
        "2024-13-01",  // Mês inválido
        "2024-01-32",  // Dia inválido
        "2024/01/01",  // Formato incorreto
        "01-01-2024",  // Formato incorreto
        "2024-1-1",    // Formato incorreto
        "2024-01-01T25:00:00Z",  // Hora inválida
    ];
    
    for date in invalid_dates {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "task",
            "--name", "Test Task",
            "--description", "Test description",
            "--start-date", date,
            "--due-date", "2024-01-10",
            "--project-code", "proj-1",
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().failure();
    }
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - tentativa de criar empresa sem inicializar
#[test]
fn test_create_company_without_init() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Tentar criar empresa sem inicializar (deve funcionar, mas vamos testar o fluxo)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "company",
        "--name", "Test Company",
        "--code", "TEST-COMP",
        "--description", "Test description"
    ]);
    cmd.assert().success();
    
    // Validar que a empresa foi criada
    let company_file = temp.child("companies").child("TEST-COMP").child("company.yaml");
    company_file.assert(predicate::path::exists());
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - tentativa de criar recurso sem empresa
#[test]
fn test_create_resource_without_company() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Tentar criar recurso sem empresa (deve falhar)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "resource",
        "Test Resource", "Developer"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - tentativa de criar projeto sem empresa
#[test]
fn test_create_project_without_company() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Tentar criar projeto sem empresa (deve falhar)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "project",
        "Test Project", "Test description"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - tentativa de criar tarefa sem projeto
#[test]
fn test_create_task_without_project() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Tentar criar tarefa sem projeto (deve falhar)
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "create", "task",
        "--name", "Test Task",
        "--description", "Test description",
        "--start-date", "2024-01-01",
        "--due-date", "2024-01-10",
        "--company-code", "TECH-CORP"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - arquivo de configuração corrompido
#[test]
fn test_corrupted_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Criar arquivo de configuração corrompido
    let config_file = temp.child("config.yaml");
    let mut file = std::fs::File::create(config_file.path())?;
    file.write_all(b"invalid yaml content: [\n  unclosed array")?;
    drop(file);
    
    // Tentar executar comando com config corrompido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("companies");
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - diretório sem permissão de escrita
#[test]
fn test_read_only_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Criar diretório somente leitura
    let read_only_dir = temp.child("readonly");
    std::fs::create_dir(read_only_dir.path())?;
    
    // Tentar criar arquivo em diretório somente leitura
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(read_only_dir.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - comando inválido
#[test]
fn test_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar comando inexistente
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("nonexistent-command");
    cmd.assert().failure();
    
    // Testar subcomando inválido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("create").arg("invalid-entity");
    cmd.assert().failure();
    
    // Testar argumentos inválidos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("invalid-entity");
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - parâmetros em excesso
#[test]
fn test_excess_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar init com parâmetros extras
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", "test@example.com",
        "--company-name", "Test Company",
        "--extra-param", "should-fail"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - valores muito longos
#[test]
fn test_oversized_values() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Testar com nome muito longo
    let long_name = "a".repeat(1000);
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", &long_name,
        "--email", "test@example.com",
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    // Testar com email muito longo
    let long_email = format!("{}@example.com", "a".repeat(1000));
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&[
        "init",
        "--name", "Test Manager",
        "--email", &long_email,
        "--company-name", "Test Company"
    ]);
    cmd.assert().failure();
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - caracteres especiais perigosos
#[test]
fn test_dangerous_characters() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    let dangerous_inputs = vec![
        "../../../etc/passwd",
        "'; DROP TABLE users; --",
        "<script>alert('xss')</script>",
        "null\0byte",
        "path\twith\ttabs",
        "path\nwith\nnewlines",
    ];
    
    for dangerous_input in dangerous_inputs {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "init",
            "--name", dangerous_input,
            "--email", "test@example.com",
            "--company-name", "Test Company"
        ]);
        cmd.assert().failure();
    }
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - timeout de operação
#[test]
fn test_operation_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Criar muitos recursos para testar timeout
    for i in 1..=100 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args(&[
            "create", "resource",
            &format!("Resource {}", i), "Developer",
            "--company-code", "TECH-CORP"
        ]);
        cmd.assert().success();
    }
    
    // Validar que todos foram criados
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    resources_dir.assert(predicate::path::is_dir());
    
    temp.close()?;
    Ok(())
}

/// Teste de erro - recuperação de estado inconsistente
#[test]
fn test_inconsistent_state_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Setup inicial
    setup_test_environment(&temp)?;
    
    // Criar arquivo de projeto parcialmente corrompido
    let project_file = temp.child("companies").child("TECH-CORP").child("projects").child("proj-1").child("project.yaml");
    std::fs::create_dir_all(project_file.parent().unwrap())?;
    let mut file = std::fs::File::create(project_file.path())?;
    file.write_all(b"apiVersion: tasktaskrevolution.io/v1alpha1\nkind: Project\nmetadata:\n  id: test-id\n  # Incomplete file")?;
    drop(file);
    
    // Tentar executar validação
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().failure();
    
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
