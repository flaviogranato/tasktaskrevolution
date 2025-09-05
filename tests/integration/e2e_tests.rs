//! Testes End-to-End do TTR CLI
//! 
//! Estes testes executam fluxos completos do CLI e validam:
//! - Fluxo completo de criação de dados
//! - Geração de HTML com navegação funcional
//! - Validação de integridade dos dados
//! - Testes de performance básicos

use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
use tempfile::TempDir;
use serde_yaml;

/// Runner para testes E2E
struct E2ETestRunner {
    temp_dir: TempDir,
    ttr_binary: String,
}

impl E2ETestRunner {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let ttr_binary = "target/debug/ttr".to_string();
        
        if !Path::new(&ttr_binary).exists() {
            return Err("TTR binary not found. Run 'cargo build' first.".into());
        }
        
        Ok(Self {
            temp_dir,
            ttr_binary,
        })
    }
    
    fn run_command(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new(&self.ttr_binary)
            .args(args)
            .current_dir(self.temp_dir.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(format!("Command failed: {}\nStderr: {}", 
                args.join(" "), stderr).into())
        }
    }
    
    fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    fn ttr_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join(".ttr")
    }
    
    fn public_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().join("public")
    }
}

/// Validador de dados
struct DataValidator;

impl DataValidator {
    fn validate_yaml_file(path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .map_err(|e| format!("Invalid YAML in file {:?}: {}", path, e))?;
        
        Ok(())
    }
    
    fn validate_html_file(path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path));
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if !content.contains("<html") && !content.contains("<!DOCTYPE") {
            return Err("File does not appear to be valid HTML".to_string());
        }
        
        Ok(())
    }
    
    fn count_yaml_files(dir: &Path) -> Result<usize, String> {
        if !dir.exists() {
            return Ok(0);
        }
        
        let mut count = 0;
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    fn count_html_files(dir: &Path) -> Result<usize, String> {
        if !dir.exists() {
            return Ok(0);
        }
        
        let mut count = 0;
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "html") {
                count += 1;
            } else if path.is_dir() {
                count += Self::count_html_files(&path)?;
            }
        }
        
        Ok(count)
    }
}

#[test]
fn test_complete_workflow() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // 1. Inicializar TTR
    println!("Step 1: Initializing TTR...");
    runner.run_command(&[
        "init", 
        "--name", "Test Manager", 
        "--email", "test@example.com", 
        "--company-name", "Test Company"
    ]).expect("Init should work");
    
    // 2. Criar empresa
    println!("Step 2: Creating company...");
    runner.run_command(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]).expect("Company creation should work");
    
    // 3. Criar recursos
    println!("Step 3: Creating resources...");
    runner.run_command(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]).expect("Resource creation should work");
    
    runner.run_command(&[
        "create", "resource",
        "Jane Smith", "Designer",
        "--company-code", "TECH-CORP"
    ]).expect("Resource creation should work");
    
    // 4. Criar projeto
    println!("Step 4: Creating project...");
    runner.run_command(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]).expect("Project creation should work");
    
    // 5. Criar tarefas
    println!("Step 5: Creating tasks...");
    runner.run_command(&[
        "create", "task",
        "--name", "Setup Environment",
        "--description", "Setup development environment",
        "--start-date", "2024-01-15",
        "--due-date", "2024-01-22",
        "--project-code", "web-app",
        "--company-code", "TECH-CORP"
    ]).expect("Task creation should work");
    
    runner.run_command(&[
        "create", "task",
        "--name", "Implement Features",
        "--description", "Implement core features",
        "--start-date", "2024-01-23",
        "--due-date", "2024-02-05",
        "--project-code", "web-app",
        "--company-code", "TECH-CORP"
    ]).expect("Task creation should work");
    
    // 6. Gerar HTML
    println!("Step 6: Generating HTML...");
    runner.run_command(&["build"]).expect("HTML generation should work");
    
    // 7. Validar estrutura de dados
    println!("Step 7: Validating data structure...");
    
    // Verificar se diretório .ttr existe
    assert!(runner.ttr_path().exists(), "TTR directory should exist");
    
    // Verificar se diretório public existe
    assert!(runner.public_path().exists(), "Public directory should exist");
    
    // Validar arquivos YAML
    let companies_dir = runner.ttr_path().join("companies");
    let resources_dir = runner.ttr_path().join("resources");
    let projects_dir = runner.ttr_path().join("projects");
    
    assert!(companies_dir.exists(), "Companies directory should exist");
    assert!(resources_dir.exists(), "Resources directory should exist");
    assert!(projects_dir.exists(), "Projects directory should exist");
    
    // Contar arquivos YAML
    let company_count = DataValidator::count_yaml_files(&companies_dir)
        .expect("Should count YAML files in companies");
    let resource_count = DataValidator::count_yaml_files(&resources_dir)
        .expect("Should count YAML files in resources");
    let project_count = DataValidator::count_yaml_files(&projects_dir)
        .expect("Should count YAML files in projects");
    
    assert!(company_count > 0, "Should have at least one company YAML file");
    assert!(resource_count > 0, "Should have at least one resource YAML file");
    assert!(project_count > 0, "Should have at least one project YAML file");
    
    // Validar arquivos HTML
    let html_count = DataValidator::count_html_files(&runner.public_path())
        .expect("Should count HTML files in public");
    assert!(html_count > 0, "Should have at least one HTML file");
    
    // Validar arquivos específicos
    let company_file = companies_dir.join("TECH-CORP.yaml");
    let resource_file = resources_dir.join("john-doe.yaml");
    let project_file = projects_dir.join("web-app.yaml");
    
    DataValidator::validate_yaml_file(&company_file)
        .expect("Company YAML should be valid");
    DataValidator::validate_yaml_file(&resource_file)
        .expect("Resource YAML should be valid");
    DataValidator::validate_yaml_file(&project_file)
        .expect("Project YAML should be valid");
    
    // Validar HTML principal
    let index_file = runner.public_path().join("index.html");
    DataValidator::validate_html_file(&index_file)
        .expect("Index HTML should be valid");
    
    println!("✅ Complete workflow test passed!");
}

#[test]
fn test_list_commands_workflow() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // Inicializar e criar dados
    runner.run_command(&[
        "init", 
        "--name", "Test Manager", 
        "--email", "test@example.com", 
        "--company-name", "Test Company"
    ]).expect("Init should work");
    
    runner.run_command(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]).expect("Company creation should work");
    
    runner.run_command(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]).expect("Resource creation should work");
    
    runner.run_command(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]).expect("Project creation should work");
    
    // Testar comandos de listagem
    let companies_output = runner.run_command(&["list", "companies"])
        .expect("List companies should work");
    assert!(companies_output.contains("Tech Corp"), "Should list created company");
    
    let resources_output = runner.run_command(&["list", "resources"])
        .expect("List resources should work");
    assert!(resources_output.contains("John Doe"), "Should list created resource");
    
    let projects_output = runner.run_command(&["list", "projects"])
        .expect("List projects should work");
    assert!(projects_output.contains("Web App"), "Should list created project");
    
    println!("✅ List commands workflow test passed!");
}

#[test]
fn test_validation_workflow() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // Inicializar e criar dados
    runner.run_command(&[
        "init", 
        "--name", "Test Manager", 
        "--email", "test@example.com", 
        "--company-name", "Test Company"
    ]).expect("Init should work");
    
    runner.run_command(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]).expect("Company creation should work");
    
    // Testar comando de validação
    let validation_output = runner.run_command(&["validate"])
        .expect("Validation command should work");
    
    // A validação deve retornar sucesso ou informações sobre o estado
    assert!(!validation_output.is_empty(), "Validation should return output");
    
    println!("✅ Validation workflow test passed!");
}

#[test]
fn test_error_handling_workflow() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // Testar comando inválido
    let result = runner.run_command(&["invalid-command"]);
    assert!(result.is_err(), "Invalid command should fail");
    
    // Testar criação de empresa sem inicializar
    let result = runner.run_command(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP"
    ]);
    assert!(result.is_err(), "Company creation without init should fail");
    
    println!("✅ Error handling workflow test passed!");
}

#[test]
fn test_html_navigation_workflow() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // Criar dados completos
    runner.run_command(&[
        "init", 
        "--name", "Test Manager", 
        "--email", "test@example.com", 
        "--company-name", "Test Company"
    ]).expect("Init should work");
    
    runner.run_command(&[
        "create", "company",
        "--name", "Tech Corp",
        "--code", "TECH-CORP",
        "--description", "Technology company"
    ]).expect("Company creation should work");
    
    runner.run_command(&[
        "create", "resource",
        "John Doe", "Developer",
        "--company-code", "TECH-CORP"
    ]).expect("Resource creation should work");
    
    runner.run_command(&[
        "create", "project",
        "Web App", "Web application project",
        "--company-code", "TECH-CORP"
    ]).expect("Project creation should work");
    
    // Gerar HTML
    runner.run_command(&["build"]).expect("HTML generation should work");
    
    // Validar arquivos HTML principais
    let index_file = runner.public_path().join("index.html");
    let companies_file = runner.public_path().join("companies.html");
    
    DataValidator::validate_html_file(&index_file)
        .expect("Index HTML should be valid");
    
    if companies_file.exists() {
        DataValidator::validate_html_file(&companies_file)
            .expect("Companies HTML should be valid");
    }
    
    // Verificar se HTML contém conteúdo esperado
    let index_content = fs::read_to_string(&index_file)
        .expect("Should read index HTML");
    
    assert!(index_content.contains("TaskTaskRevolution"), "Should contain title");
    assert!(index_content.contains("Tech Corp"), "Should contain company name");
    
    println!("✅ HTML navigation workflow test passed!");
}
