//! Testes End-to-End para TTR CLI
//! 
//! Este módulo contém testes que validam o funcionamento completo do CLI,
//! desde a criação de projetos até a geração de relatórios e exportação de dados.

pub mod utils;
pub mod scenarios;

pub use scenarios::project_lifecycle::{ProjectLifecycleTest, ProjectErrorTest};

/// Executa todos os testes e2e
pub fn run_all_e2e_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Iniciando execução de todos os testes E2E...");
    println!("{}", "=".repeat(60));
    
    // Testes de ciclo de vida do projeto
    ProjectLifecycleTest::run_all()?;
    println!();
    
    // Testes de tratamento de erros
    ProjectErrorTest::run_all()?;
    println!();
    
    // TODO: Adicionar outros cenários de teste
    // - ResourceManagementTest::run_all()?;
    // - ReportingWorkflowTest::run_all()?;
    // - DataConsistencyTest::run_all()?;
    
    println!("{}", "=".repeat(60));
    println!("🎉 Todos os testes E2E foram executados com sucesso!");
    
    Ok(())
}

/// Executa apenas os testes de ciclo de vida do projeto
pub fn run_project_lifecycle_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Executando testes de ciclo de vida do projeto...");
    ProjectLifecycleTest::run_all()
}

/// Executa apenas os testes de tratamento de erros
pub fn run_error_handling_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Executando testes de tratamento de erros...");
    ProjectErrorTest::run_all()
}

/// Executa testes de validação de arquivos
pub fn run_file_validation_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Executando testes de validação de arquivos...");
    
    use crate::utils::file_assertions::FileAssertions;
    use tempfile::TempDir;
    use std::fs;
    
    let temp_dir = TempDir::new()?;
    
    // Teste de arquivo YAML
    let yaml_file = temp_dir.path().join("test.yaml");
    fs::write(&yaml_file, "name: Test\nvalue: 42")?;
    
    FileAssertions::assert_file_exists(&yaml_file)?;
    FileAssertions::assert_valid_yaml(&yaml_file)?;
    FileAssertions::assert_yaml_contains_key(&yaml_file, "name")?;
    FileAssertions::assert_yaml_contains_value(&yaml_file, "name", "Test")?;
    
    // Teste de arquivo CSV
    let csv_file = temp_dir.path().join("test.csv");
    fs::write(&csv_file, "Name,Value\nTest,42\nAnother,100")?;
    
    FileAssertions::assert_file_exists(&csv_file)?;
    FileAssertions::assert_valid_csv(&csv_file)?;
    FileAssertions::assert_file_contains(&csv_file, "Test,42")?;
    
    // Teste de arquivo HTML
    let html_file = temp_dir.path().join("test.html");
    fs::write(&html_file, "<!DOCTYPE html><html><head><title>Test</title></head><body><h1>Hello</h1></body></html>")?;
    
    FileAssertions::assert_file_exists(&html_file)?;
    FileAssertions::assert_valid_html(&html_file)?;
    FileAssertions::assert_file_contains(&html_file, "Hello")?;
    
    println!("  ✅ Validação de arquivos funcionou!");
    Ok(())
}

/// Executa testes de integração CLI
pub fn run_cli_integration_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Executando testes de integração CLI...");
    
    use crate::utils::cli_runner::CliRunner;
    
    let runner = CliRunner::new()?;
    
    // Teste básico de criação de diretório temporário
    assert!(runner.temp_path().exists());
    assert!(runner.temp_path().is_dir());
    
    // Teste de caminhos de diretórios
    let ttr_path = runner.ttr_path();
    assert_eq!(ttr_path.file_name().unwrap(), ".ttr");
    
    let projects_path = runner.projects_path();
    assert!(projects_path.ends_with("projects"));
    
    let resources_path = runner.resources_path();
    assert!(resources_path.ends_with("resources"));
    
    let companies_path = runner.companies_path();
    assert!(companies_path.ends_with("companies"));
    
    println!("  ✅ Integração CLI funcionou!");
    Ok(())
}

/// Executa todos os testes de utilitários
pub fn run_utility_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛠️ Executando testes de utilitários...");
    
    run_file_validation_tests()?;
    run_cli_integration_tests()?;
    
    println!("  ✅ Todos os utilitários funcionaram!");
    Ok(())
}

/// Executa testes específicos baseados em argumentos
pub fn run_specific_tests(test_names: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Executando testes específicos: {:?}", test_names);
    
    for test_name in test_names {
        match *test_name {
            "project_lifecycle" => {
                run_project_lifecycle_tests()?;
            }
            "error_handling" => {
                run_error_handling_tests()?;
            }
            "file_validation" => {
                run_file_validation_tests()?;
            }
            "cli_integration" => {
                run_cli_integration_tests()?;
            }
            "utilities" => {
                run_utility_tests()?;
            }
            "all" => {
                run_all_e2e_tests()?;
            }
            _ => {
                println!("⚠️ Teste desconhecido: {}", test_name);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_utility_functions() {
        let result = run_utility_tests();
        assert!(result.is_ok(), "Utility tests failed: {:?}", result.err());
    }
    
    #[test]
    fn test_cli_integration() {
        let result = run_cli_integration_tests();
        assert!(result.is_ok(), "CLI integration tests failed: {:?}", result.err());
    }
    
    #[test]
    fn test_file_validation() {
        let result = run_file_validation_tests();
        assert!(result.is_ok(), "File validation tests failed: {:?}", result.err());
    }
    
    #[test]
    fn test_specific_tests() {
        let test_names = vec!["file_validation", "cli_integration"];
        let result = run_specific_tests(&test_names);
        assert!(result.is_ok(), "Specific tests failed: {:?}", result.err());
    }
}
