//! Configuração para testes funcionais
//! 
//! Este módulo contém configurações e utilitários compartilhados
//! para os testes funcionais do TTR CLI.

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Configuração dos testes funcionais
pub struct TestConfig {
    pub ttr_binary: String,
    pub test_timeout: u64,
    pub verbose: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            ttr_binary: "target/debug/ttr".to_string(),
            test_timeout: 30, // 30 segundos
            verbose: false,
        }
    }
}

impl TestConfig {
    /// Cria uma nova configuração de teste
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Define o caminho do binário TTR
    pub fn with_binary(mut self, binary_path: String) -> Self {
        self.ttr_binary = binary_path;
        self
    }
    
    /// Define o timeout dos testes
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.test_timeout = timeout;
        self
    }
    
    /// Habilita output verboso
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// Verifica se o binário TTR existe
    pub fn validate_binary(&self) -> Result<(), String> {
        if !Path::new(&self.ttr_binary).exists() {
            Err(format!("TTR binary not found at: {}", self.ttr_binary))
        } else {
            Ok(())
        }
    }
    
    /// Executa o comando TTR com a configuração atual
    pub fn run_command(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        if self.verbose {
            println!("Running: {} {}", self.ttr_binary, args.join(" "));
        }
        
        let output = Command::new(&self.ttr_binary)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            let stderr = String::from_utf8(output.stderr)?;
            Err(format!("Command failed: {}\nStderr: {}", 
                args.join(" "), stderr).into())
        }
    }
}

/// Builder para criar cenários de teste
pub struct TestScenarioBuilder {
    config: TestConfig,
    temp_dir: TempDir,
}

impl TestScenarioBuilder {
    /// Cria um novo builder de cenário
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = TestConfig::new();
        config.validate_binary()?;
        
        let temp_dir = TempDir::new()?;
        
        Ok(Self {
            config,
            temp_dir,
        })
    }
    
    /// Configura o cenário com dados básicos
    pub fn with_basic_data(self) -> Result<Self, Box<dyn std::error::Error>> {
        // Inicializar TTR
        self.config.run_command(&[
            "init", 
            "--name", "Test Manager", 
            "--email", "test@example.com", 
            "--company-name", "Test Company"
        ])?;
        
        // Criar empresa
        self.config.run_command(&[
            "create", "company",
            "--name", "Tech Corp",
            "--code", "TECH-CORP",
            "--description", "Technology company"
        ])?;
        
        // Criar recurso
        self.config.run_command(&[
            "create", "resource",
            "John Doe", "Developer",
            "--company-code", "TECH-CORP"
        ])?;
        
        // Criar projeto
        self.config.run_command(&[
            "create", "project",
            "Web App", "Web application project",
            "--company-code", "TECH-CORP"
        ])?;
        
        Ok(self)
    }
    
    /// Adiciona tarefas ao cenário
    pub fn with_tasks(self, tasks: &[(&str, &str, &str)]) -> Result<Self, Box<dyn std::error::Error>> {
        for (name, start_date, due_date) in tasks {
            self.config.run_command(&[
                "create", "task",
                "--name", name,
                "--description", &format!("Task: {}", name),
                "--start-date", start_date,
                "--due-date", due_date,
                "--project-code", "web-app",
                "--company-code", "TECH-CORP"
            ])?;
        }
        
        Ok(self)
    }
    
    /// Gera HTML no cenário
    pub fn with_html_generation(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.config.run_command(&["build"])?;
        Ok(self)
    }
    
    /// Retorna o diretório temporário do cenário
    pub fn temp_dir(&self) -> &TempDir {
        &self.temp_dir
    }
    
    /// Retorna a configuração do cenário
    pub fn config(&self) -> &TestConfig {
        &self.config
    }
}

/// Utilitários para validação de testes
pub struct TestValidator;

impl TestValidator {
    /// Valida se um diretório contém arquivos esperados
    pub fn validate_directory_structure(dir: &Path, expected_files: &[&str]) -> Result<(), String> {
        if !dir.exists() {
            return Err(format!("Directory does not exist: {:?}", dir));
        }
        
        for expected_file in expected_files {
            let file_path = dir.join(expected_file);
            if !file_path.exists() {
                return Err(format!("Expected file not found: {:?}", file_path));
            }
        }
        
        Ok(())
    }
    
    /// Valida se um arquivo contém texto específico
    pub fn validate_file_contains(file: &Path, expected_text: &str) -> Result<(), String> {
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file));
        }
        
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if content.contains(expected_text) {
            Ok(())
        } else {
            Err(format!("File does not contain expected text: '{}'", expected_text))
        }
    }
    
    /// Valida se um arquivo YAML é válido
    pub fn validate_yaml_file(file: &Path) -> Result<(), String> {
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file));
        }
        
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .map_err(|e| format!("Invalid YAML in file {:?}: {}", file, e))?;
        
        Ok(())
    }
    
    /// Valida se um arquivo HTML é válido
    pub fn validate_html_file(file: &Path) -> Result<(), String> {
        if !file.exists() {
            return Err(format!("File does not exist: {:?}", file));
        }
        
        let content = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if !content.contains("<html") && !content.contains("<!DOCTYPE") {
            return Err("File does not appear to be valid HTML".to_string());
        }
        
        Ok(())
    }
}

/// Macros para facilitar a criação de testes
#[macro_export]
macro_rules! test_scenario {
    ($name:ident, $builder:expr) => {
        #[test]
        fn $name() {
            let scenario = $builder.expect("Failed to create test scenario");
            // O teste específico será implementado aqui
        }
    };
}

#[macro_export]
macro_rules! assert_cli_success {
    ($result:expr, $message:expr) => {
        match $result {
            Ok(_) => {},
            Err(e) => panic!("{}: {}", $message, e),
        }
    };
}

#[macro_export]
macro_rules! assert_cli_failure {
    ($result:expr, $message:expr) => {
        match $result {
            Ok(_) => panic!("Expected failure but got success: {}", $message),
            Err(_) => {},
        }
    };
}
